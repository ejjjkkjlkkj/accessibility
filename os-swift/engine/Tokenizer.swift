// Tokenizer BPE au niveau octet (style GPT-2 / Qwen), chargé depuis un tokenizer.json Hugging Face.
import Foundation

final class BPETokenizer {
    private let vocab: [String: Int]          // symbole -> identifiant
    private let idToToken: [Int: String]
    private let ranks: [String: Int]          // "a b" -> rang de fusion (plus petit = prioritaire)
    private let byteToChar: [UInt8: Character]
    private let charToByte: [Character: UInt8]
    private let preRegex: NSRegularExpression

    // Pré-découpage GPT-2 (la plupart des modèles Qwen/Llama utilisent une variante proche)
    static let pattern = #"'s|'t|'re|'ve|'m|'ll|'d| ?\p{L}+| ?\p{N}+| ?[^\s\p{L}\p{N}]+|\s+(?!\S)|\s+"#

    init(tokenizerJSON path: String) throws {
        let data = try Data(contentsOf: URL(fileURLWithPath: path))
        guard let root = try JSONSerialization.jsonObject(with: data) as? [String: Any],
              let model = root["model"] as? [String: Any],
              let vocabDict = model["vocab"] as? [String: Int],
              let mergesList = model["merges"] as? [Any] else {
            throw NSError(domain: "tokenizer", code: 1, userInfo: [NSLocalizedDescriptionKey: "tokenizer.json invalide"])
        }
        vocab = vocabDict
        var inv: [Int: String] = [:]
        for (k, v) in vocabDict { inv[v] = k }
        idToToken = inv

        var r: [String: Int] = [:]
        for (i, m) in mergesList.enumerated() {
            // Les merges sont des chaînes "a b" ou des paires [a, b] selon la version
            if let s = m as? String { r[s] = i }
            else if let pair = m as? [String], pair.count == 2 { r[pair[0] + " " + pair[1]] = i }
        }
        ranks = r

        // Table GPT-2 octet <-> caractère visible
        var bs: [Int] = Array(33...126) + Array(161...172) + Array(174...255)
        var cs = bs
        var n = 0
        for b in 0..<256 where !bs.contains(b) {
            bs.append(b)
            cs.append(256 + n)
            n += 1
        }
        var b2c: [UInt8: Character] = [:]
        var c2b: [Character: UInt8] = [:]
        for (b, c) in zip(bs, cs) {
            let ch = Character(UnicodeScalar(c)!)
            b2c[UInt8(b)] = ch
            c2b[ch] = UInt8(b)
        }
        byteToChar = b2c
        charToByte = c2b
        preRegex = try NSRegularExpression(pattern: BPETokenizer.pattern)
    }

    /// Fusionne les symboles d'un mot selon les rangs BPE.
    private func bpe(_ word: [String]) -> [String] {
        var symbols = word
        while symbols.count > 1 {
            var bestRank = Int.max
            var bestIdx = -1
            for i in 0..<(symbols.count - 1) {
                if let rank = ranks[symbols[i] + " " + symbols[i + 1]], rank < bestRank {
                    bestRank = rank
                    bestIdx = i
                }
            }
            if bestIdx < 0 { break }
            symbols[bestIdx] = symbols[bestIdx] + symbols[bestIdx + 1]
            symbols.remove(at: bestIdx + 1)
        }
        return symbols
    }

    func encode(_ text: String) -> [Int] {
        var ids: [Int] = []
        let ns = text as NSString
        let matches = preRegex.matches(in: text, range: NSRange(location: 0, length: ns.length))
        for m in matches {
            let piece = ns.substring(with: m.range)
            // Octets UTF-8 -> caractères visibles
            let word = piece.utf8.map { String(byteToChar[$0]!) }
            for sym in bpe(word) {
                if let id = vocab[sym] { ids.append(id) }
            }
        }
        return ids
    }

    func decode(_ ids: [Int]) -> String {
        var bytes: [UInt8] = []
        for id in ids {
            guard let tok = idToToken[id] else { continue }
            for ch in tok {
                if let b = charToByte[ch] { bytes.append(b) }
            }
        }
        return String(decoding: bytes, as: UTF8.self)
    }
}
