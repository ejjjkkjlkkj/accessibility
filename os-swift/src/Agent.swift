// Agent de la graine : interface par défaut du système.
// Boucle : comprendre -> planifier -> vérifier les autorisations -> agir -> observer -> mémoriser.
import Foundation

/// Un souvenir : ce qui a été demandé, ce qui a été fait, le résultat.
struct Memory {
    let time: Date
    let request: String
    let result: String
}

/// Résultat d'une demande à l'agent.
struct AgentReply {
    let text: String
    let steps: Int
}

final class Agent {
    let intent: IntentShell
    private(set) var memory: [Memory] = []
    private var pending: Plan? = nil
    private let logPath: String?

    init(shell: ShellV2, logPath: String? = nil) {
        self.intent = IntentShell(natural: NaturalShell(shell: shell))
        self.logPath = logPath
    }

    /// Une demande peut contenir plusieurs étapes : « liste les fichiers puis annule ».
    func handle(_ request: String) -> AgentReply {
        // Confirmation d'une action destructive en attente
        if let p = pending, ["oui", "ok", "confirme"].contains(request.lowercased()) {
            pending = nil
            let result = intent.execute(p, confirmed: true)
            remember(request: "confirmation : \(p.changes)", result: result)
            return AgentReply(text: result, steps: 1)
        }
        pending = nil
        let steps = splitSteps(request)
        var outputs: [String] = []
        for step in steps {
            let result = runOne(step)
            outputs.append(result)
            remember(request: step, result: result)
        }
        return AgentReply(text: outputs.joined(separator: "\n"), steps: steps.count)
    }

    /// Une étape : planifier, puis exécuter si l'autorisation le permet.
    private func runOne(_ step: String) -> String {
        do {
            let plan = try intent.plan(for: step)
            let header = intent.describe(plan)
            if plan.needsConfirmation {
                // L'agent ne supprime jamais sans confirmation explicite de l'utilisateur.
                pending = plan
                return header + "\nEn attente de confirmation : répondez « oui »."
            }
            return header + "\n" + intent.execute(plan, confirmed: true)
        } catch {
            return "\(error)"
        }
    }

    /// Découpe « A puis B », « A et ensuite B ».
    private func splitSteps(_ request: String) -> [String] {
        var text = " " + request + " "
        for sep in [" puis ", " ensuite ", " et après "] {
            text = text.replacingOccurrences(of: sep, with: "\u{1}")
        }
        return text.split(separator: "\u{1}")
            .map { $0.trimmingCharacters(in: .whitespaces) }
            .filter { !$0.isEmpty }
    }

    private func remember(request: String, result: String) {
        let m = Memory(time: Date(), request: request, result: result)
        memory.append(m)
        guard let path = logPath else { return }
        let stamp = ISO8601DateFormatter().string(from: m.time)
        let entry = "[\(stamp)] \(request) => \(result.replacingOccurrences(of: "\n", with: " / "))\n"
        let existing = (try? String(contentsOfFile: path, encoding: .utf8)) ?? ""
        try? (existing + entry).write(toFile: path, atomically: true, encoding: .utf8)
    }

    /// Réponse à « que s'est-il passé ? » : les derniers souvenirs.
    func recall(last n: Int = 5) -> String {
        if memory.isEmpty { return "Aucune action mémorisée pour l'instant." }
        return memory.suffix(n).map { "- \($0.request)" }.joined(separator: "\n")
    }
}
