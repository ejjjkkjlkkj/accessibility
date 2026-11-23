// Shell de notre système : navigation et gestion de fichiers dans la racine "/".
import Foundation

/// Une commande du shell.
protocol ShellCommand {
    var name: String { get }
    var help: String { get }
    /// Exécute la commande. Retourne le texte à afficher.
    func run(args: [String], shell: Shell) throws -> String
}

/// Contexte du shell : racine réelle et répertoire courant (chemin relatif à la racine).
final class Shell {
    let rootPath: String
    private(set) var cwd: String = "/"
    private var commands: [String: ShellCommand] = [:]

    init(rootPath: String) {
        self.rootPath = rootPath
        for c in Shell.builtins { commands[c.name] = c }
    }

    /// Traduit un chemin du shell ("/etc/x", "etc/x", "..") en chemin réel, sans sortir de la racine.
    func resolve(_ path: String) throws -> String {
        var parts = (path.hasPrefix("/") ? [] : cwd.split(separator: "/").map(String.init))
        for comp in path.split(separator: "/").map(String.init) {
            switch comp {
            case ".": continue
            case "..": if !parts.isEmpty { parts.removeLast() }
            default: parts.append(comp)
            }
        }
        return rootPath + "/" + parts.joined(separator: "/")
    }

    func shellPath(of real: String) -> String {
        "/" + (real.hasPrefix(rootPath + "/") ? String(real.dropFirst(rootPath.count + 1)) : "")
    }

    func changeDirectory(to path: String) throws {
        let real = try resolve(path)
        var isDir: ObjCBool = false
        guard FileManager.default.fileExists(atPath: real, isDirectory: &isDir), isDir.boolValue else {
            throw ShellError.notADirectory(path)
        }
        cwd = shellPath(of: real)
    }

    /// Exécute une ligne saisie. Retourne la sortie (vide si rien à afficher).
    func execute(line: String) -> String {
        let words = line.split(separator: " ").map(String.init)
        guard let name = words.first else { return "" }
        guard let cmd = commands[name] else { return "commande inconnue : \(name) (tapez help)" }
        do {
            return try cmd.run(args: Array(words.dropFirst()), shell: self)
        } catch {
            return "\(name) : \(error)"
        }
    }

    var prompt: String { "graine:\(cwd)$ " }

    static let builtins: [ShellCommand] = [HelpCmd(), PwdCmd(), LsCmd(), CdCmd(), CatCmd(),
                                           MkdirCmd(), TouchCmd(), RmCmd(), ExitCmd()]
}

enum ShellError: Error, CustomStringConvertible {
    case notADirectory(String)
    case notFound(String)
    case usage(String)

    var description: String {
        switch self {
        case .notADirectory(let p): return "\(p) n'est pas un répertoire"
        case .notFound(let p): return "\(p) introuvable"
        case .usage(let u): return "usage : \(u)"
        }
    }
}

// MARK: - Commandes

struct HelpCmd: ShellCommand {
    let name = "help"
    let help = "liste les commandes"
    func run(args: [String], shell: Shell) throws -> String {
        Shell.builtins.map { "\($0.name.padding(toLength: 8, withPad: " ", startingAt: 0)) \($0.help)" }.joined(separator: "\n")
    }
}

struct PwdCmd: ShellCommand {
    let name = "pwd"
    let help = "affiche le répertoire courant"
    func run(args: [String], shell: Shell) throws -> String { shell.cwd }
}

struct LsCmd: ShellCommand {
    let name = "ls"
    let help = "liste le contenu d'un répertoire"
    func run(args: [String], shell: Shell) throws -> String {
        let real = try shell.resolve(args.first ?? ".")
        let items = try FileManager.default.contentsOfDirectory(atPath: real).sorted()
        return items.map { name -> String in
            var isDir: ObjCBool = false
            FileManager.default.fileExists(atPath: real + "/" + name, isDirectory: &isDir)
            return isDir.boolValue ? name + "/" : name
        }.joined(separator: "  ")
    }
}

struct CdCmd: ShellCommand {
    let name = "cd"
    let help = "change de répertoire"
    func run(args: [String], shell: Shell) throws -> String {
        try shell.changeDirectory(to: args.first ?? "/")
        return ""
    }
}

struct CatCmd: ShellCommand {
    let name = "cat"
    let help = "affiche un fichier texte"
    func run(args: [String], shell: Shell) throws -> String {
        guard let p = args.first else { throw ShellError.usage("cat <fichier>") }
        let real = try shell.resolve(p)
        guard let text = try? String(contentsOfFile: real, encoding: .utf8) else { throw ShellError.notFound(p) }
        return text
    }
}

struct MkdirCmd: ShellCommand {
    let name = "mkdir"
    let help = "crée un répertoire"
    func run(args: [String], shell: Shell) throws -> String {
        guard let p = args.first else { throw ShellError.usage("mkdir <dossier>") }
        try FileManager.default.createDirectory(atPath: try shell.resolve(p), withIntermediateDirectories: false)
        return ""
    }
}

struct TouchCmd: ShellCommand {
    let name = "touch"
    let help = "crée un fichier vide"
    func run(args: [String], shell: Shell) throws -> String {
        guard let p = args.first else { throw ShellError.usage("touch <fichier>") }
        let real = try shell.resolve(p)
        if !FileManager.default.fileExists(atPath: real) {
            FileManager.default.createFile(atPath: real, contents: Data())
        }
        return ""
    }
}

struct RmCmd: ShellCommand {
    let name = "rm"
    let help = "supprime un fichier ou un répertoire"
    func run(args: [String], shell: Shell) throws -> String {
        guard let p = args.first else { throw ShellError.usage("rm <chemin>") }
        let real = try shell.resolve(p)
        // Protection : on ne supprime pas la racine elle-même
        guard real != shell.rootPath else { throw ShellError.usage("impossible de supprimer la racine") }
        try FileManager.default.removeItem(atPath: real)
        return ""
    }
}

struct ExitCmd: ShellCommand {
    let name = "exit"
    let help = "quitte le shell"
    func run(args: [String], shell: Shell) throws -> String { "__EXIT__" }
}
