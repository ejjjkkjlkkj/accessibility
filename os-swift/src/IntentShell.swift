// Shell d'intentions : phrase -> intention structurée -> plan visible -> autorisation -> exécution réversible.
import Foundation

/// Ce que l'utilisateur veut, indépendamment de la syntaxe.
struct Intent: Equatable {
    enum Action: String { case read, write, destroy, undo, inspect }
    let action: Action
    let target: String?        // chemin ou mot-clé, s'il existe
}

/// Permissions accordées à la session.
struct CapabilityPolicy {
    var canWrite: Bool
    var canDestroy: Bool

    static let readOnly = CapabilityPolicy(canWrite: false, canDestroy: false)
    static let trusted = CapabilityPolicy(canWrite: true, canDestroy: true)
}

/// Plan : commandes à exécuter, et ce qu'elles changent. Affiché avant exécution.
struct Plan {
    let intent: Intent
    let commands: [String]
    let changes: String
    let needsConfirmation: Bool
}

enum PlanError: Error, CustomStringConvertible {
    case denied(String)
    case unknown(String)

    var description: String {
        switch self {
        case .denied(let m): return "refusé : \(m)"
        case .unknown(let m): return "je ne sais pas faire ceci : \(m)"
        }
    }
}

/// Traduit une commande du shell v2 en intention, pour pouvoir la contrôler.
enum IntentParser {
    static func parse(commandLine: String) -> Intent? {
        let words = commandLine.split(separator: " ").map(String.init)
        guard let verb = words.first else { return nil }
        let target = words.dropFirst().first
        switch verb {
        case "ls", "pwd", "lire", "aide", "compter", "cd": return Intent(action: verb == "lire" ? .read : .inspect, target: target)
        case "creer-dossier", "creer-fichier": return Intent(action: .write, target: target)
        case "supprimer": return Intent(action: .destroy, target: target)
        case "annuler": return Intent(action: .undo, target: nil)
        default: return nil
        }
    }
}

/// Construit le plan à partir d'une phrase, applique la politique, puis exécute.
final class IntentShell {
    let natural: NaturalShell
    var policy: CapabilityPolicy

    init(natural: NaturalShell, policy: CapabilityPolicy = .readOnly) {
        self.natural = natural
        self.policy = policy
    }

    /// Étape 1 : phrase -> plan. Aucune action n'est exécutée ici.
    func plan(for sentence: String) throws -> Plan {
        let interp = natural.interpreter.interpret(sentence)
        guard case .command(let line) = interp else {
            if case .unclear(let m) = interp { throw PlanError.unknown(m) }
            throw PlanError.unknown(sentence)
        }
        // Le pipeline entier est une seule commande : on ne coupe pas sur les "|".
        let commands = [line]
        let firstStage = line.components(separatedBy: "|")[0].trimmingCharacters(in: .whitespaces)
        guard let intent = IntentParser.parse(commandLine: firstStage) else {
            throw PlanError.unknown(line)
        }
        switch intent.action {
        case .write where !policy.canWrite:
            throw PlanError.denied("écriture non autorisée. Accordez-la avec « autoriser ecriture ».")
        case .destroy where !policy.canDestroy:
            throw PlanError.denied("suppression non autorisée. Accordez-la avec « autoriser suppression ».")
        default: break
        }
        let destructive = intent.action == .destroy
        let changes: String
        switch intent.action {
        case .read, .inspect: changes = "aucune modification"
        case .write: changes = "crée « \(intent.target ?? "?") »"
        case .destroy: changes = "déplace « \(intent.target ?? "?") » dans la corbeille (annulable)"
        case .undo: changes = "annule la dernière action"
        }
        return Plan(intent: intent, commands: commands, changes: changes, needsConfirmation: destructive)
    }

    /// Étape 2 : exécution d'un plan validé. `confirmed` doit être vrai si le plan est destructif.
    func execute(_ plan: Plan, confirmed: Bool) -> String {
        if plan.needsConfirmation && !confirmed {
            return "Confirmation requise : \(plan.changes). Répondez « oui » pour continuer."
        }
        return plan.commands.map { natural.shell.execute($0) }.joined(separator: "\n")
    }

    /// Affiche le plan avant exécution.
    func describe(_ plan: Plan) -> String {
        "Plan : \(plan.commands.joined(separator: " | "))\nEffet : \(plan.changes)"
    }

    /// Autorisations explicites de la session.
    func grant(_ what: String) -> String {
        switch what {
        case "ecriture": policy.canWrite = true; return "écriture autorisée pour cette session."
        case "suppression": policy.canDestroy = true; return "suppression autorisée pour cette session."
        case "lecture-seule": policy = .readOnly; return "retour en lecture seule."
        default: return "autorisation inconnue : \(what). Valeurs : ecriture, suppression, lecture-seule."
        }
    }
}
