// Échantillonnage des tokens : glouton, température et top-k, avec un générateur seedé reproductible.
import Foundation

/// Générateur pseudo-aléatoire (SplitMix64), reproductible à partir d'une graine.
struct SeededRNG: RandomNumberGenerator {
    private var state: UInt64
    init(seed: UInt64) { state = seed }
    mutating func next() -> UInt64 {
        state &+= 0x9E3779B97F4A7C15
        var z = state
        z = (z ^ (z >> 30)) &* 0xBF58476D1CE4E5B9
        z = (z ^ (z >> 27)) &* 0x94D049BB133111EB
        return z ^ (z >> 31)
    }
}

struct SamplingOptions {
    var temperature: Float = 0      // 0 = glouton
    var topK: Int = 0               // 0 = pas de limitation
    var seed: UInt64 = 42
}

final class Sampler {
    private(set) var options: SamplingOptions
    private var rng: SeededRNG

    init(options: SamplingOptions) {
        self.options = options
        self.rng = SeededRNG(seed: options.seed)
    }

    /// Choisit un token à partir des logits.
    func next(from logits: [Float]) -> Int {
        if options.temperature <= 0 {
            return logits.indices.max(by: { logits[$0] < logits[$1] })!
        }
        // Top-k : garde les k meilleurs logits
        var candidates = Array(logits.indices)
        if options.topK > 0 && options.topK < logits.count {
            candidates.sort { logits[$0] > logits[$1] }
            candidates = Array(candidates.prefix(options.topK))
        }
        // Softmax à température
        let scaled = candidates.map { logits[$0] / options.temperature }
        let m = scaled.max()!
        let exps = scaled.map { exp($0 - m) }
        let total = exps.reduce(0, +)
        // Tirage selon la distribution
        let r = Float.random(in: 0..<1, using: &rng) * total
        var acc: Float = 0
        for (i, e) in exps.enumerated() {
            acc += e
            if r < acc { return candidates[i] }
        }
        return candidates[candidates.count - 1]
    }
}
