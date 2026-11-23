// Native specialised model as the agent's interpreter.
// The model proposes a command; the system accepts it only if it matches the allowed grammar.
// Anything else falls back to the rule-based interpreter. The model can never run unlisted commands.
import Foundation

final class NativeModelInterpreter: NaturalLanguageInterpreter {
    private let generator: TextGenerator
    private let fallback: NaturalLanguageInterpreter
    private let maxNew = 24

    /// The verbs the shell accepts from the model. Anything else is rejected.
    static let allowedVerbs: [String] = ["ls", "cd", "pwd", "lire", "creer-dossier", "creer-fichier",
                                         "supprimer", "annuler", "aide", "compter"]

    init(folder: String, fallback: NaturalLanguageInterpreter = RuleBasedInterpreter()) throws {
        self.generator = try TextGenerator(folder: folder)
        self.fallback = fallback
    }

    func interpret(_ sentence: String) -> Interpretation {
        let prompt = "Request: \(sentence)\nCommand:"
        let proposed = generator.generate(prompt: prompt, maxNew: maxNew, options: SamplingOptions())
        let line = proposed.components(separatedBy: "\n").first?.trimmingCharacters(in: .whitespaces) ?? ""
        if NativeModelInterpreter.isAllowed(line) {
            return .command(line)
        }
        // The model's output was not a valid command: use the rules instead.
        return fallback.interpret(sentence)
    }

    /// True only for a command whose first word is an allowed verb and that has no unsafe characters.
    static func isAllowed(_ line: String) -> Bool {
        guard !line.isEmpty, line.count < 200 else { return false }
        let first = line.split(separator: " ").first.map(String.init) ?? ""
        guard allowedVerbs.contains(first) else { return false }
        // Only letters, digits, spaces and a few safe symbols: no shell metacharacters.
        let safe = CharacterSet.alphanumerics.union(CharacterSet(charactersIn: " _-./|=<>é"))
        return line.unicodeScalars.allSatisfy { safe.contains($0) }
    }
}
