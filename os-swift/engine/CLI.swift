// Chargement d'un dossier modèle au format Hugging Face (config.json, model.safetensors, tokenizer.json).
import Foundation

enum ModelFolderError: Error, CustomStringConvertible {
    case missingFile(String)
    case badConfig(String)

    var description: String {
        switch self {
        case .missingFile(let f): return "fichier manquant : \(f)"
        case .badConfig(let k): return "clé de configuration manquante : \(k)"
        }
    }
}

extension ModelConfig {
    /// Construit la configuration depuis un config.json Hugging Face (famille Llama/Qwen3).
    static func fromHuggingFace(path: String) throws -> ModelConfig {
        let data = try Data(contentsOf: URL(fileURLWithPath: path))
        guard let j = try JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            throw ModelFolderError.badConfig("racine")
        }
        func int(_ k: String) throws -> Int {
            guard let v = j[k] as? Int else { throw ModelFolderError.badConfig(k) }
            return v
        }
        let hidden = try int("hidden_size")
        let heads = try int("num_attention_heads")
        return ModelConfig(
            vocab: try int("vocab_size"),
            hidden: hidden,
            layers: try int("num_hidden_layers"),
            heads: heads,
            kvHeads: (j["num_key_value_heads"] as? Int) ?? heads,
            headDim: (j["head_dim"] as? Int) ?? hidden / heads,
            intermediate: try int("intermediate_size"),
            ropeTheta: Float((j["rope_theta"] as? Double) ?? 10000.0),
            eps: Float((j["rms_norm_eps"] as? Double) ?? 1e-6))
    }
}

/// Dossier modèle : charge tout ce qu'il faut pour générer du texte.
final class TextGenerator {
    let model: Model
    let tokenizer: BPETokenizer

    init(folder: String) throws {
        let fm = FileManager.default
        for f in ["config.json", "model.safetensors", "tokenizer.json"] {
            let p = folder + "/" + f
            if !fm.fileExists(atPath: p) { throw ModelFolderError.missingFile(p) }
        }
        let cfg = try ModelConfig.fromHuggingFace(path: folder + "/config.json")
        model = try Model.loadSafetensors(path: folder + "/model.safetensors", config: cfg)
        tokenizer = try BPETokenizer(tokenizerJSON: folder + "/tokenizer.json")
    }

    /// Génère une suite de texte à partir d'un prompt.
    func generate(prompt: String, maxNew: Int, options: SamplingOptions) -> String {
        let ids = tokenizer.encode(prompt)
        let cache = model.makeCache()
        let sampler = Sampler(options: options)
        var logits = model.forward(tokens: ids, startPos: 0, cache: cache)
        var out: [Int] = []
        var pos = ids.count
        for _ in 0..<maxNew {
            let next = sampler.next(from: logits)
            out.append(next)
            logits = model.forward(tokens: [next], startPos: pos, cache: cache)
            pos += 1
        }
        return tokenizer.decode(out)
    }
}
