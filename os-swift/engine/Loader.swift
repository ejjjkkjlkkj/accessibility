// Chargement d'un modèle depuis un fichier safetensors aux noms Hugging Face (famille Qwen3).
import Foundation

enum LoadError: Error {
    case missing(String)
}

extension Model {
    static func loadSafetensors(path: String, config c: ModelConfig, quantize: Bool = false) throws -> Model {
        let st = try SafetensorsFile(path: path)

        func matrix(_ name: String, _ rows: Int, _ cols: Int) throws -> Matrix {
            guard let v = st.floats(name) else { throw LoadError.missing(name) }
            let m = Matrix(rows: rows, cols: cols, data: v)
            return quantize ? m.quantized() : m
        }
        func vector(_ name: String) throws -> [Float] {
            guard let v = st.floats(name) else { throw LoadError.missing(name) }
            return v
        }

        let H = c.hidden, hd = c.headDim, nH = c.heads, nKV = c.kvHeads, I = c.intermediate
        let rope = RotaryEmbedding(headDim: hd, theta: c.ropeTheta)

        // L'embedding reste en float : c'est une table de recherche, pas un produit matriciel
        guard let ev = st.floats("model.embed_tokens.weight") else { throw LoadError.missing("model.embed_tokens.weight") }
        let embedding = Matrix(rows: c.vocab, cols: H, data: ev)
        var layers: [DecoderLayer] = []
        for i in 0..<c.layers {
            let p = "model.layers.\(i)."
            let inputNorm = RMSNorm(weight: try vector(p + "input_layernorm.weight"), eps: c.eps)
            let attn = Attention(
                heads: nH, kvHeads: nKV, headDim: hd,
                wq: try matrix(p + "self_attn.q_proj.weight", nH * hd, H),
                wk: try matrix(p + "self_attn.k_proj.weight", nKV * hd, H),
                wv: try matrix(p + "self_attn.v_proj.weight", nKV * hd, H),
                wo: try matrix(p + "self_attn.o_proj.weight", H, nH * hd),
                qNorm: RMSNorm(weight: try vector(p + "self_attn.q_norm.weight"), eps: c.eps),
                kNorm: RMSNorm(weight: try vector(p + "self_attn.k_norm.weight"), eps: c.eps),
                rope: rope)
            let postNorm = RMSNorm(weight: try vector(p + "post_attention_layernorm.weight"), eps: c.eps)
            let mlp = SwiGLU(
                wGate: try matrix(p + "mlp.gate_proj.weight", I, H),
                wUp: try matrix(p + "mlp.up_proj.weight", I, H),
                wDown: try matrix(p + "mlp.down_proj.weight", H, I))
            layers.append(DecoderLayer(inputNorm: inputNorm, attention: attn, postNorm: postNorm, mlp: mlp))
        }
        let finalNorm = RMSNorm(weight: try vector("model.norm.weight"), eps: c.eps)
        // Tête liée à l'embedding si lm_head.weight est absent (cas de Qwen3-0.6B)
        let lmHead: Matrix = st.tensors["lm_head.weight"] != nil
            ? try matrix("lm_head.weight", c.vocab, H)
            : embedding
        return Model(config: c, embedding: embedding, layers: layers, finalNorm: finalNorm, lmHead: lmHead)
    }
}
