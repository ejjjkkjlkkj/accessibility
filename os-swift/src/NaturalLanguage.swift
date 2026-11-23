// Interprète une phrase française et la traduit en commande du shell graine v2.
import Foundation

/// Résultat de l'interprétation : une commande du shell, ou une demande de précision.
enum Interpretation: Equatable {
    case command(String)
    case unclear(String)
}

/// Un interpréteur transforme une phrase en commande. Le modèle de langage local viendra
/// implémenter ce même protocole.
protocol NaturalLanguageInterpreter {
    func interpret(_ sentence: String) -> Interpretation
}

/// Interpréteur à règles : reconnaît des formulations françaises courantes.
final class RuleBasedInterpreter: NaturalLanguageInterpreter {

    private var originalWords: [String] = []

    func interpret(_ sentence: String) -> Interpretation {
        originalWords = sentence.split(separator: " ").map(String.init)
        let s = normalize(sentence)
        if s.isEmpty { return .unclear("Dites-moi ce que vous voulez faire.") }

        // Annulation
        if matches(s, any: ["annule", "annuler", "reviens en arriere", "defais", "undo"]) {
            return .command("annuler")
        }
        // Aide
        if matches(s, any: ["aide", "que peux tu faire", "que sais tu faire"]) {
            return .command("aide")
        }
        // Où suis-je
        if matches(s, any: ["ou suis je", "dans quel dossier", "repertoire actuel"]) {
            return .command("pwd")
        }
        // Aller dans un dossier
        if let target = capture(s, after: ["va dans ", "ouvre ", "entre dans ", "aller dans ", "go to ", "cd "]) {
            return .command("cd \(target)")
        }
        // Lire un fichier
        if let target = capture(s, after: ["lis ", "montre moi le contenu de ", "affiche ", "ouvre le fichier ", "read ", "show content of "]) {
            return .command("lire \(target)")
        }
        // Supprimer (vers la corbeille)
        if let target = capture(s, after: ["supprime ", "efface ", "jette ", "mets a la corbeille ", "delete ", "remove "]) {
            return .command("supprimer \(target)")
        }
        // Créer
        if let target = capture(s, after: ["cree un dossier ", "nouveau dossier "]) {
            return .command("creer-dossier \(target)")
        }
        if let target = capture(s, after: ["cree un fichier ", "nouveau fichier "]) {
            return .command("creer-fichier \(target)")
        }
        // Compter
        if matches(s, any: ["combien", "compte"]) {
            return .command("ls | compter")
        }
        // Lister avec filtres
        if matches(s, any: ["montre", "liste", "affiche", "qu'est ce qu'il y a", "quoi", "list", "show"]) || s.hasPrefix("ls") {
            var command = "ls"
            if matches(s, any: ["dossiers", "dossier", "folders"]) && !matches(s, any: ["fichiers", "files"]) {
                command += " | où type = dossier"
            } else if matches(s, any: ["fichiers", "files"]) {
                command += " | où type = fichier"
            }
            if let word = capture(s, after: ["contenant ", "qui contiennent ", "avec "]) {
                command += " | où nom contient \(word)"
            }
            if matches(s, any: ["plus gros", "les plus lourds", "trie par taille", "par taille"]) {
                command += " | trier taille"
            }
            return .command(command)
        }
        return .unclear("Je n'ai pas compris « \(sentence) ». Essayez : « montre-moi les fichiers », « va dans etc », « annule ».")
    }

    // MARK: Outils

    /// Minuscules, sans accents, espaces normalisés.
    private func normalize(_ text: String) -> String {
        let folded = text.folding(options: [.diacriticInsensitive, .caseInsensitive], locale: Locale(identifier: "fr_FR"))
        return folded
            .replacingOccurrences(of: "’", with: "'")
            .split(whereSeparator: { $0 == " " || $0 == "\t" })
            .joined(separator: " ")
            .trimmingCharacters(in: .whitespaces)
    }

    private func matches(_ s: String, any phrases: [String]) -> Bool {
        phrases.contains { s.contains($0) }
    }

    /// Retourne le mot (ou chemin) qui suit l'un des préfixes, en gardant la casse d'origine.
    /// Les mots normalisés et les mots d'origine ont la même position : on lit l'original à cet index.
    private func capture(_ s: String, after prefixes: [String]) -> String? {
        let normWords = s.split(separator: " ").map(String.init)
        for p in prefixes {
            let pWords = p.split(separator: " ").map(String.init)
            guard !pWords.isEmpty else { continue }
            var i = 0
            while i + pWords.count < normWords.count {
                if Array(normWords[i..<(i + pWords.count)]) == pWords {
                    let idx = i + pWords.count
                    let w = originalWords[idx].trimmingCharacters(in: CharacterSet(charactersIn: "?!.,"))
                    if !w.isEmpty { return w }
                }
                i += 1
            }
        }
        return nil
    }
}

/// Relie l'interpréteur au shell : la phrase est traduite puis exécutée.
final class NaturalShell {
    let shell: ShellV2
    let interpreter: NaturalLanguageInterpreter

    init(shell: ShellV2, interpreter: NaturalLanguageInterpreter = RuleBasedInterpreter()) {
        self.shell = shell
        self.interpreter = interpreter
    }

    /// Retourne le texte à afficher : la commande comprise et son résultat.
    func say(_ sentence: String) -> String {
        switch interpreter.interpret(sentence) {
        case .unclear(let message):
            return message
        case .command(let line):
            if line.hasPrefix("cd ") {
                do {
                    try shell.changeDirectory(String(line.dropFirst(3)))
                    return "→ \(shell.cwd)"
                } catch {
                    return "Erreur : \(error)"
                }
            }
            return "(\(line))\n" + shell.execute(line)
        }
    }
}
