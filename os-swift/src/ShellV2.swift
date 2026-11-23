// Shell graine v2 : pipelines d'objets typés, corbeille réversible, annulation (undo).
import Foundation

// MARK: - Objets typés

struct Entry {
    let name: String
    let path: String        // chemin shell, ex. "/etc/README.txt"
    let isDir: Bool
    let size: Int
    let modified: Date

    var kind: String { isDir ? "dossier" : "fichier" }

    /// Valeur d'un champ pour les filtres et tris.
    func field(_ f: String) -> String? {
        switch f {
        case "nom": return name
        case "chemin": return path
        case "type": return kind
        case "taille": return String(size)
        default: return nil
        }
    }
}

/// Valeur produite par une étape du pipeline.
enum Value {
    case entries([Entry])
    case text(String)
    case empty
}

// MARK: - Journal d'annulation

/// Une action réversible.
struct Undoable {
    let description: String
    let revert: () throws -> Void
}

final class UndoStack {
    private(set) var items: [Undoable] = []
    func push(_ u: Undoable) { items.append(u) }
    func pop() -> Undoable? { items.popLast() }
}

// MARK: - Shell

final class ShellV2 {
    let rootPath: String
    private(set) var cwd = "/"
    let undo = UndoStack()
    private let trashDir = ".corbeille"

    init(rootPath: String) throws {
        self.rootPath = rootPath
        try FileManager.default.createDirectory(atPath: rootPath + "/" + trashDir, withIntermediateDirectories: true)
    }

    var prompt: String { "graine\(cwd)> " }

    // Chemins
    func resolve(_ path: String) -> String {
        var parts = path.hasPrefix("/") ? [] : cwd.split(separator: "/").map(String.init)
        for c in path.split(separator: "/").map(String.init) {
            if c == "." { continue }
            if c == ".." { if !parts.isEmpty { parts.removeLast() }; continue }
            parts.append(c)
        }
        return rootPath + "/" + parts.joined(separator: "/")
    }

    func shellPath(of real: String) -> String {
        let rel = real.hasPrefix(rootPath + "/") ? String(real.dropFirst(rootPath.count + 1)) : ""
        return "/" + rel
    }

    func changeDirectory(_ path: String) throws {
        let real = resolve(path)
        var isDir: ObjCBool = false
        guard FileManager.default.fileExists(atPath: real, isDirectory: &isDir), isDir.boolValue else {
            throw CommandError("« \(path) » n'est pas un dossier. Essayez « ls » pour voir le contenu.")
        }
        cwd = shellPath(of: real)
    }

    // Exécution d'une ligne avec pipelines : "ls | où taille > 10 | trier taille"
    func execute(_ line: String) -> String {
        let stages = line.split(separator: "|").map { $0.trimmingCharacters(in: .whitespaces) }
        guard let first = stages.first, !first.isEmpty else { return "" }
        do {
            var value: Value = .empty
            for (i, stage) in stages.enumerated() {
                let words = stage.split(separator: " ").map(String.init)
                guard let verb = words.first else { throw CommandError("étape vide dans le pipeline") }
                value = try runVerb(verb, Array(words.dropFirst()), input: i == 0 ? .empty : value)
            }
            return render(value)
        } catch let e as CommandError {
            return "Erreur : \(e.message)"
        } catch {
            return "Erreur : \(error.localizedDescription)"
        }
    }

    private func runVerb(_ verb: String, _ args: [String], input: Value) throws -> Value {
        switch verb {
        case "aide", "help":
            return .text(helpText)
        case "où", "ou", "where":
            guard case .entries(let es) = input, args.count == 3 else {
                throw CommandError("usage : où <champ> <opérateur> <valeur>, par exemple « où taille > 10 »")
            }
            return .entries(try filter(es, field: args[0], op: args[1], value: args[2]))
        case "trier", "sort":
            guard case .entries(let es) = input, let f = args.first else {
                throw CommandError("usage : trier <champ>, par exemple « trier taille »")
            }
            let sorted = es.sorted { ($0.field(f) ?? "") < ($1.field(f) ?? "") }
            return .entries(sorted)
        case "compter", "count":
            guard case .entries(let es) = input else { throw CommandError("compter attend une liste") }
            return .text("\(es.count) élément(s)")
        case "ls":
            return .entries(try list(args.first ?? "."))
        case "pwd":
            return .text(cwd)
        case "cd":
            try changeDirectory(args.first ?? "/")
            return .empty
        case "cat", "lire":
            guard let p = args.first else { throw CommandError("usage : lire <fichier>") }
            guard let t = try? String(contentsOfFile: resolve(p), encoding: .utf8) else {
                throw CommandError("« \(p) » est introuvable ou n'est pas un fichier texte.")
            }
            return .text(t)
        case "mkdir", "creer-dossier":
            guard let p = args.first else { throw CommandError("usage : creer-dossier <nom>") }
            let real = resolve(p)
            try FileManager.default.createDirectory(atPath: real, withIntermediateDirectories: false)
            undo.push(Undoable(description: "créer le dossier \(p)") {
                try FileManager.default.removeItem(atPath: real)
            })
            return .text("dossier créé : \(p)")
        case "touch", "creer-fichier":
            guard let p = args.first else { throw CommandError("usage : creer-fichier <nom>") }
            let real = resolve(p)
            guard !FileManager.default.fileExists(atPath: real) else {
                throw CommandError("« \(p) » existe déjà.")
            }
            FileManager.default.createFile(atPath: real, contents: Data())
            undo.push(Undoable(description: "créer le fichier \(p)") {
                try FileManager.default.removeItem(atPath: real)
            })
            return .text("fichier créé : \(p)")
        case "supprimer", "rm":
            guard let p = args.first else { throw CommandError("usage : supprimer <chemin>") }
            return .text(try moveToTrash(p))
        case "annuler", "undo":
            guard let action = undo.pop() else { throw CommandError("rien à annuler.") }
            try action.revert()
            return .text("annulé : \(action.description)")
        default:
            throw CommandError("commande inconnue « \(verb) ». Tapez « aide ».")
        }
    }

    // MARK: Implémentations

    private func list(_ path: String) throws -> [Entry] {
        let real = resolve(path)
        let names = try FileManager.default.contentsOfDirectory(atPath: real)
        return try names.filter { $0 != trashDir }.map { name -> Entry in
            let full = real + "/" + name
            let attrs = try FileManager.default.attributesOfItem(atPath: full)
            let isDir = (attrs[.type] as? FileAttributeType) == .typeDirectory
            return Entry(name: name,
                         path: shellPath(of: full),
                         isDir: isDir,
                         size: (attrs[.size] as? NSNumber)?.intValue ?? 0,
                         modified: attrs[.modificationDate] as? Date ?? Date())
        }.sorted { $0.name < $1.name }
    }

    private func filter(_ es: [Entry], field: String, op: String, value: String) throws -> [Entry] {
        guard ["nom", "chemin", "type", "taille"].contains(field) else {
            throw CommandError("champ inconnu « \(field) ». Champs : nom, chemin, type, taille.")
        }
        return try es.filter { e in
            guard let raw = e.field(field) else { return false }
            if field == "taille", let a = Int(raw), let b = Int(value) {
                switch op {
                case ">": return a > b
                case "<": return a < b
                case "=": return a == b
                default: throw CommandError("opérateur inconnu « \(op) ». Utilisez >, <, =.")
                }
            }
            switch op {
            case "=": return raw == value
            case "contient": return raw.contains(value)
            default: throw CommandError("opérateur « \(op) » non valable pour le champ « \(field) ». Utilisez = ou contient.")
            }
        }
    }

    /// Déplace dans la corbeille au lieu de supprimer : l'action est annulable.
    private func moveToTrash(_ p: String) throws -> String {
        let real = resolve(p)
        guard real != rootPath else { throw CommandError("la racine ne peut pas être supprimée.") }
        guard FileManager.default.fileExists(atPath: real) else {
            throw CommandError("« \(p) » est introuvable.")
        }
        let stamp = Int(Date().timeIntervalSince1970 * 1000)
        let base = (real as NSString).lastPathComponent
        let trashed = rootPath + "/" + trashDir + "/\(stamp)-\(base)"
        try FileManager.default.moveItem(atPath: real, toPath: trashed)
        undo.push(Undoable(description: "supprimer \(p) (récupéré depuis la corbeille)") {
            try FileManager.default.moveItem(atPath: trashed, toPath: real)
        })
        return "déplacé dans la corbeille : \(p)  (annulable avec « annuler »)"
    }

    // MARK: Affichage

    private func render(_ v: Value) -> String {
        switch v {
        case .empty: return ""
        case .text(let t): return t
        case .entries(let es):
            if es.isEmpty { return "(vide)" }
            let nameW = max(4, es.map { $0.name.count }.max() ?? 4)
            var lines = ["TYPE     TAILLE  NOM"]
            for e in es {
                let type = e.isDir ? "dossier " : "fichier "
                let size = e.isDir ? "-" : String(e.size)
                let name = e.isDir ? e.name + "/" : e.name
                lines.append(type + size.padding(toLength: 8, withPad: " ", startingAt: 0) + name)
                _ = nameW
            }
            return lines.joined(separator: "\n")
        }
    }

    let helpText = """
    Commandes (utilisez | pour enchaîner) :
      ls [dossier]                 liste les entrées (objets typés)
      où <champ> <op> <valeur>     filtre : où taille > 10, où nom contient log
      trier <champ>                trie : trier taille
      compter                      nombre d'éléments
      cd <dossier>, pwd            navigation
      lire <fichier>               affiche un fichier texte
      creer-dossier, creer-fichier création réversible
      supprimer <chemin>           va dans la corbeille (réversible)
      annuler                      annule la dernière action
      aide                         cette aide
    Exemple : ls | où type = fichier | trier taille
    """
}

struct CommandError: Error {
    let message: String
    init(_ m: String) { message = m }
}
