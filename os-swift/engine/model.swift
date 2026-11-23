// Notre modèle, en Swift orienté objet.
// Décodeur dense en pré-norme : RMSNorm, QK-norm, RoPE moitié-coupée, GQA, SwiGLU, sans biais.
import Foundation

// MARK: - Configuration

struct ModelConfig {
    let vocab: Int
    let hidden: Int
    let layers: Int
    let heads: Int
    let kvHeads: Int
    let headDim: Int
    let intermediate: Int
    let ropeTheta: Float
    let eps: Float
}

// MARK: - Tenseurs

/// Matrice 2D en ligne-majeur (rows x cols).
struct Matrix {
    let rows: Int
    let cols: Int
    var data: [Float]
    /// Version quantifiée int8 (symétrique, une échelle par ligne), si demandée.
    var q8: [Int8]? = nil
    var scales: [Float] = []

    init(rows: Int, cols: Int, data: [Float]) {
        precondition(data.count == rows * cols, "taille de matrice incorrecte")
        self.rows = rows
        self.cols = cols
        self.data = data
    }

    /// Quantifie en int8 par ligne : w ≈ q * scale, scale = max|w_ligne| / 127.
    func quantized() -> Matrix {
        var m = self
        var q = [Int8](repeating: 0, count: rows * cols)
        var s = [Float](repeating: 0, count: rows)
        for r in 0..<rows {
            var mx: Float = 0
            for c in 0..<cols { mx = max(mx, abs(data[r * cols + c])) }
            let scale = mx > 0 ? mx / 127 : 1
            s[r] = scale
            for c in 0..<cols {
                let v = (data[r * cols + c] / scale).rounded()
                q[r * cols + c] = Int8(max(-127, min(127, v)))
            }
        }
        m.q8 = q
        m.scales = s
        m.data = []          // on libère la version float : c'est le gain mémoire
        return m
    }
}

/// Source de poids : lit les tenseurs dans un ordre fixe.
struct WeightReader {
    private let raw: Data
    private var offset = 0

    init(path: String) throws {
        raw = try Data(contentsOf: URL(fileURLWithPath: path))
    }

    mutating func vector(_ n: Int) -> [Float] {
        let bytes = raw.subdata(in: offset..<(offset + n * 4))
        offset += n * 4
        return bytes.withUnsafeBytes { Array($0.bindMemory(to: Float.self)) }
    }

    mutating func matrix(_ rows: Int, _ cols: Int) -> Matrix {
        Matrix(rows: rows, cols: cols, data: vector(rows * cols))
    }

    var isFinished: Bool { offset == raw.count }
}

// MARK: - Opérations de base

enum Ops {
    /// y = x @ W^T, x : [T, in], W : [out, in].
    static func linear(_ x: [Float], tokens T: Int, weight W: Matrix) -> [Float] {
        if let q8 = W.q8 { return linearQ8(x, tokens: T, q8: q8, scales: W.scales, rows: W.rows, cols: W.cols) }
        let inDim = W.cols, outDim = W.rows
        var y = [Float](repeating: 0, count: T * outDim)
        let vecCount = inDim / 8
        x.withUnsafeBufferPointer { xp in
            W.data.withUnsafeBufferPointer { wp in
                y.withUnsafeMutableBufferPointer { yp in
                    let xBase = xp.baseAddress!, wBase = wp.baseAddress!, yBase = yp.baseAddress!
                    // Parallélisme sur les lignes de sortie (chaque thread écrit des colonnes distinctes)
                    // Blocs de lignes de sortie : moins de surcoût de parallélisme que une tâche par ligne
                    let block = 64
                    let blocks = (outDim + block - 1) / block
                    DispatchQueue.concurrentPerform(iterations: blocks) { b in
                        let oStart = b * block
                        let oEnd = min(outDim, oStart + block)
                        for o in oStart..<oEnd {
                            let wRow = wBase + o * inDim
                            for t in 0..<T {
                                let xRow = xBase + t * inDim
                                var acc0 = SIMD8<Float>(repeating: 0)
                                var acc1 = SIMD8<Float>(repeating: 0)
                                var v = 0
                                while v + 1 < vecCount {
                                    let w0 = UnsafeRawPointer(wRow + v * 8).loadUnaligned(as: SIMD8<Float>.self)
                                    let x0 = UnsafeRawPointer(xRow + v * 8).loadUnaligned(as: SIMD8<Float>.self)
                                    let w1 = UnsafeRawPointer(wRow + (v + 1) * 8).loadUnaligned(as: SIMD8<Float>.self)
                                    let x1 = UnsafeRawPointer(xRow + (v + 1) * 8).loadUnaligned(as: SIMD8<Float>.self)
                                    acc0 += w0 * x0
                                    acc1 += w1 * x1
                                    v += 2
                                }
                                if v < vecCount {
                                    let w0 = UnsafeRawPointer(wRow + v * 8).loadUnaligned(as: SIMD8<Float>.self)
                                    let x0 = UnsafeRawPointer(xRow + v * 8).loadUnaligned(as: SIMD8<Float>.self)
                                    acc0 += w0 * x0
                                }
                                var s = (acc0 + acc1).sum()
                                for i in (vecCount * 8)..<inDim { s += wRow[i] * xRow[i] }
                                yBase[t * outDim + o] = s
                            }
                        }
                    }
                }
            }
        }
        return y
    }

    /// Produit avec poids int8 : y[t,o] = scale[o] * sum_i x[t,i] * q[o,i]. Lit 4 fois moins de mémoire.
    static func linearQ8(_ x: [Float], tokens T: Int, q8: [Int8], scales: [Float], rows outDim: Int, cols inDim: Int) -> [Float] {
        var y = [Float](repeating: 0, count: T * outDim)
        let vecCount = inDim / 8
        x.withUnsafeBufferPointer { xp in
            q8.withUnsafeBufferPointer { qp in
                y.withUnsafeMutableBufferPointer { yp in
                    let xBase = xp.baseAddress!, qBase = qp.baseAddress!, yBase = yp.baseAddress!
                    let block = 64
                    let blocks = (outDim + block - 1) / block
                    DispatchQueue.concurrentPerform(iterations: blocks) { b in
                        let oStart = b * block
                        let oEnd = min(outDim, oStart + block)
                        for o in oStart..<oEnd {
                            let qRow = qBase + o * inDim
                            let sc = scales[o]
                            if T == 1 {
                                // Génération : un seul token, produit scalaire simple (le plus rapide ici)
                                var acc = SIMD8<Float>(repeating: 0)
                                for v in 0..<vecCount {
                                    let wi = UnsafeRawPointer(qRow + v * 8).loadUnaligned(as: SIMD8<Int8>.self)
                                    let xv = UnsafeRawPointer(xBase + v * 8).loadUnaligned(as: SIMD8<Float>.self)
                                    acc += SIMD8<Float>(wi) * xv
                                }
                                var s = acc.sum()
                                for i in (vecCount * 8)..<inDim { s += Float(qRow[i]) * xBase[i] }
                                yBase[o] = s * sc
                                continue
                            }
                            // Préremplissage : jusqu'à 4 tokens à la fois, chaque poids décodé une seule fois
                            var t0 = 0
                            while t0 < T {
                                let n = min(4, T - t0)
                                var a0 = SIMD8<Float>(repeating: 0), a1 = a0, a2 = a0, a3 = a0
                                let x0 = xBase + t0 * inDim
                                for v in 0..<vecCount {
                                    let wf = SIMD8<Float>(UnsafeRawPointer(qRow + v * 8).loadUnaligned(as: SIMD8<Int8>.self))
                                    let off = v * 8
                                    a0 += wf * UnsafeRawPointer(x0 + off).loadUnaligned(as: SIMD8<Float>.self)
                                    if n > 1 { a1 += wf * UnsafeRawPointer(x0 + inDim + off).loadUnaligned(as: SIMD8<Float>.self) }
                                    if n > 2 { a2 += wf * UnsafeRawPointer(x0 + 2 * inDim + off).loadUnaligned(as: SIMD8<Float>.self) }
                                    if n > 3 { a3 += wf * UnsafeRawPointer(x0 + 3 * inDim + off).loadUnaligned(as: SIMD8<Float>.self) }
                                }
                                for k in 0..<n {
                                    var s = (k == 0 ? a0 : k == 1 ? a1 : k == 2 ? a2 : a3).sum()
                                    let xRow = x0 + k * inDim
                                    for i in (vecCount * 8)..<inDim { s += Float(qRow[i]) * xRow[i] }
                                    yBase[(t0 + k) * outDim + o] = s * sc
                                }
                                t0 += n
                            }
                        }
                    }
                }
            }
        }
        return y
    }

    @inline(__always) static func silu(_ v: Float) -> Float { v / (1 + exp(-v)) }
}

// MARK: - RMSNorm

final class RMSNorm {
    let weight: [Float]
    let eps: Float

    init(weight: [Float], eps: Float) {
        self.weight = weight
        self.eps = eps
    }

    /// Normalise chaque ligne de taille `weight.count`.
    func apply(_ x: [Float], rows: Int) -> [Float] {
        let dim = weight.count
        var y = [Float](repeating: 0, count: rows * dim)
        for r in 0..<rows {
            var ss: Float = 0
            for i in 0..<dim { let v = x[r * dim + i]; ss += v * v }
            let inv = 1.0 / sqrt(ss / Float(dim) + eps)
            for i in 0..<dim { y[r * dim + i] = x[r * dim + i] * inv * weight[i] }
        }
        return y
    }
}

// MARK: - RoPE

struct RotaryEmbedding {
    let headDim: Int
    let theta: Float

    /// Convention moitié-coupée : paires (i, i + hd/2). Modifie x en place.
    func apply(_ x: inout [Float], tokens T: Int, heads: Int, offset: Int = 0) {
        let half = headDim / 2
        for t in 0..<T {
            for h in 0..<heads {
                let base = (t * heads + h) * headDim
                for i in 0..<half {
                    let angle = Float(offset + t) * pow(theta, -Float(2 * i) / Float(headDim))
                    let c = cos(angle), s = sin(angle)
                    let x1 = x[base + i], x2 = x[base + i + half]
                    x[base + i] = x1 * c - x2 * s
                    x[base + i + half] = x2 * c + x1 * s
                }
            }
        }
    }
}

// MARK: - Cache KV (une entrée par couche, positions empilées)

final class KVCache {
    let kvHeads: Int
    let headDim: Int
    private(set) var keys: [[Float]]    // par couche : [pos, kvHead, hd]
    private(set) var values: [[Float]]
    private(set) var length = 0          // nombre de positions déjà stockées

    init(layers: Int, kvHeads: Int, headDim: Int) {
        self.kvHeads = kvHeads
        self.headDim = headDim
        keys = Array(repeating: [], count: layers)
        values = Array(repeating: [], count: layers)
    }

    func append(layer: Int, k: [Float], v: [Float]) {
        keys[layer].append(contentsOf: k)
        values[layer].append(contentsOf: v)
    }

    /// Appelé une fois par passe, après toutes les couches.
    func advance(by n: Int) { length += n }
}

// MARK: - Attention (GQA + QK-norm)

final class Attention {
    let heads: Int
    let kvHeads: Int
    let headDim: Int
    let wq: Matrix, wk: Matrix, wv: Matrix, wo: Matrix
    let qNorm: RMSNorm, kNorm: RMSNorm
    let rope: RotaryEmbedding

    init(heads: Int, kvHeads: Int, headDim: Int, wq: Matrix, wk: Matrix, wv: Matrix, wo: Matrix,
         qNorm: RMSNorm, kNorm: RMSNorm, rope: RotaryEmbedding) {
        self.heads = heads; self.kvHeads = kvHeads; self.headDim = headDim
        self.wq = wq; self.wk = wk; self.wv = wv; self.wo = wo
        self.qNorm = qNorm; self.kNorm = kNorm; self.rope = rope
    }

    /// Attention incrémentale : `h` contient T nouveaux tokens placés à partir de `startPos`.
    /// Les clés/valeurs sont ajoutées au cache de la couche `layer`, puis on attend sur tout l'historique.
    func forward(_ h: [Float], tokens T: Int, startPos: Int, cache: KVCache, layer: Int) -> [Float] {
        let hd = headDim, nH = heads, nKV = kvHeads
        let group = nH / nKV

        var q = Ops.linear(h, tokens: T, weight: wq)
        var k = Ops.linear(h, tokens: T, weight: wk)
        let v = Ops.linear(h, tokens: T, weight: wv)

        q = qNorm.apply(q, rows: T * nH)
        k = kNorm.apply(k, rows: T * nKV)
        rope.apply(&q, tokens: T, heads: nH, offset: startPos)
        rope.apply(&k, tokens: T, heads: nKV, offset: startPos)

        cache.append(layer: layer, k: k, v: v)
        let K = cache.keys[layer], V = cache.values[layer]

        var out = [Float](repeating: 0, count: T * nH * hd)
        let scale = 1.0 / sqrt(Float(hd))
        for head in 0..<nH {
            let kvHead = head / group
            for t in 0..<T {
                let visible = startPos + t + 1          // positions 0...startPos+t
                var scores = [Float](repeating: 0, count: visible)
                var maxS: Float = -Float.infinity
                let qb = (t * nH + head) * hd
                for s in 0..<visible {
                    let kb = (s * nKV + kvHead) * hd
                    var dot: Float = 0
                    for i in 0..<hd { dot += q[qb + i] * K[kb + i] }
                    scores[s] = dot * scale
                    maxS = max(maxS, scores[s])
                }
                var denom: Float = 0
                for s in 0..<visible { scores[s] = exp(scores[s] - maxS); denom += scores[s] }
                let ob = (t * nH + head) * hd
                for s in 0..<visible {
                    let p = scores[s] / denom
                    let vb = (s * nKV + kvHead) * hd
                    for i in 0..<hd { out[ob + i] += p * V[vb + i] }
                }
            }
        }
        return Ops.linear(out, tokens: T, weight: wo)
    }
}

// MARK: - MLP SwiGLU

final class SwiGLU {
    let wGate: Matrix, wUp: Matrix, wDown: Matrix

    init(wGate: Matrix, wUp: Matrix, wDown: Matrix) {
        self.wGate = wGate; self.wUp = wUp; self.wDown = wDown
    }

    func forward(_ x: [Float], tokens T: Int) -> [Float] {
        let g = Ops.linear(x, tokens: T, weight: wGate)
        let u = Ops.linear(x, tokens: T, weight: wUp)
        var m = [Float](repeating: 0, count: g.count)
        for i in 0..<m.count { m[i] = Ops.silu(g[i]) * u[i] }
        return Ops.linear(m, tokens: T, weight: wDown)
    }
}

// MARK: - Couche décodeur

final class DecoderLayer {
    let inputNorm: RMSNorm
    let attention: Attention
    let postNorm: RMSNorm
    let mlp: SwiGLU

    init(inputNorm: RMSNorm, attention: Attention, postNorm: RMSNorm, mlp: SwiGLU) {
        self.inputNorm = inputNorm; self.attention = attention
        self.postNorm = postNorm; self.mlp = mlp
    }

    func forward(_ x: [Float], tokens T: Int, startPos: Int, cache: KVCache, layer: Int) -> [Float] {
        var r = x
        let a = attention.forward(inputNorm.apply(r, rows: T), tokens: T,
                                  startPos: startPos, cache: cache, layer: layer)
        for i in 0..<r.count { r[i] += a[i] }
        let m = mlp.forward(postNorm.apply(r, rows: T), tokens: T)
        for i in 0..<r.count { r[i] += m[i] }
        return r
    }
}

// MARK: - Modèle complet

final class Model {
    let config: ModelConfig
    let embedding: Matrix
    let layers: [DecoderLayer]
    let finalNorm: RMSNorm
    let lmHead: Matrix

    init(config: ModelConfig, embedding: Matrix, layers: [DecoderLayer], finalNorm: RMSNorm, lmHead: Matrix) {
        self.config = config
        self.embedding = embedding
        self.layers = layers
        self.finalNorm = finalNorm
        self.lmHead = lmHead
    }

    /// Charge les poids depuis un fichier float32 écrit par make_ref.py (ordre fixe).
    static func load(config c: ModelConfig, path: String) throws -> Model {
        var r = try WeightReader(path: path)
        let H = c.hidden, hd = c.headDim, nH = c.heads, nKV = c.kvHeads, I = c.intermediate
        let rope = RotaryEmbedding(headDim: hd, theta: c.ropeTheta)

        let embedding = r.matrix(c.vocab, H)
        var layers: [DecoderLayer] = []
        for _ in 0..<c.layers {
            let inputNorm = RMSNorm(weight: r.vector(H), eps: c.eps)
            let wq = r.matrix(nH * hd, H)
            let wk = r.matrix(nKV * hd, H)
            let wv = r.matrix(nKV * hd, H)
            let qNorm = RMSNorm(weight: r.vector(hd), eps: c.eps)
            let kNorm = RMSNorm(weight: r.vector(hd), eps: c.eps)
            let wo = r.matrix(H, nH * hd)
            let attn = Attention(heads: nH, kvHeads: nKV, headDim: hd, wq: wq, wk: wk, wv: wv, wo: wo,
                                 qNorm: qNorm, kNorm: kNorm, rope: rope)
            let postNorm = RMSNorm(weight: r.vector(H), eps: c.eps)
            let mlp = SwiGLU(wGate: r.matrix(I, H), wUp: r.matrix(I, H), wDown: r.matrix(H, I))
            layers.append(DecoderLayer(inputNorm: inputNorm, attention: attn, postNorm: postNorm, mlp: mlp))
        }
        let finalNorm = RMSNorm(weight: r.vector(H), eps: c.eps)
        let lmHead = r.matrix(c.vocab, H)
        precondition(r.isFinished, "fichier de poids inattendu")
        return Model(config: c, embedding: embedding, layers: layers, finalNorm: finalNorm, lmHead: lmHead)
    }

    /// Passe avant avec cache KV. `tokens` sont placés à partir de `startPos`.
    /// Retourne les logits du dernier token fourni.
    func forward(tokens: [Int], startPos: Int, cache: KVCache) -> [Float] {
        let T = tokens.count, H = config.hidden
        var x = [Float](repeating: 0, count: T * H)
        for (t, tok) in tokens.enumerated() {
            for i in 0..<H { x[t * H + i] = embedding.data[tok * H + i] }
        }
        for (li, layer) in layers.enumerated() {
            x = layer.forward(x, tokens: T, startPos: startPos, cache: cache, layer: li)
        }
        cache.advance(by: T)
        let xf = finalNorm.apply(x, rows: T)
        let logits = Ops.linear(xf, tokens: T, weight: lmHead)
        let last = T - 1
        return Array(logits[(last * config.vocab)..<((last + 1) * config.vocab)])
    }

    func makeCache() -> KVCache {
        KVCache(layers: config.layers, kvHeads: config.kvHeads, headDim: config.headDim)
    }

    /// Génération gloutonne (argmax) : préremplit le prompt puis produit `maxNew` tokens.
    func generateGreedy(prompt: [Int], maxNew: Int) -> [Int] {
        let cache = makeCache()
        var logits = forward(tokens: prompt, startPos: 0, cache: cache)
        var out: [Int] = []
        var pos = prompt.count
        for _ in 0..<maxNew {
            let next = logits.indices.max(by: { logits[$0] < logits[$1] })!
            out.append(next)
            logits = forward(tokens: [next], startPos: pos, cache: cache)
            pos += 1
        }
        return out
    }
}
