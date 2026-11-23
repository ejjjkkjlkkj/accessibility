// Agent orchestration for the graine OS.
// An orchestrator routes each user step to a sub-agent. Every sub-agent can be
// enabled, disabled or bypassed by the user at any time. Nothing is forced.
import Foundation

/// The outcome of a sub-agent working on one step.
struct StepResult {
    let agent: String
    let text: String
}

/// A specialised worker. It only sees the steps routed to it.
protocol SubAgent: AnyObject {
    var name: String { get }
    var purpose: String { get }
    var enabled: Bool { get set }
    /// True if this sub-agent wants to handle the step.
    func accepts(_ step: String) -> Bool
    func handle(_ step: String) -> StepResult
}

/// Works on files through the agent core: listing, creating, trashing, undoing.
final class FileAgent: SubAgent {
    let name = "files"
    let purpose = "list, filter, create, move to trash and undo files"
    var enabled = true
    private let core: Agent

    init(core: Agent) { self.core = core }

    func accepts(_ step: String) -> Bool {
        let s = step.lowercased()
        return ["list", "show", "files", "folder", "create", "delete", "remove", "undo", "cd", "go to", "read"]
            .contains { s.contains($0) }
    }

    func handle(_ step: String) -> StepResult {
        StepResult(agent: name, text: core.handle(step).text)
    }
}

/// Reports on what has happened so far.
final class HistoryAgent: SubAgent {
    let name = "history"
    let purpose = "summarise past actions from the agent memory"
    var enabled = true
    private let core: Agent

    init(core: Agent) { self.core = core }

    func accepts(_ step: String) -> Bool {
        let s = step.lowercased()
        return s.contains("history") || s.contains("what happened") || s.contains("recent")
    }

    func handle(_ step: String) -> StepResult {
        StepResult(agent: name, text: core.recall(last: 10))
    }
}

/// Routes steps to sub-agents. Unhandled steps are returned to the user untouched.
final class Orchestrator {
    private(set) var agents: [SubAgent] = []

    func register(_ agent: SubAgent) { agents.append(agent) }

    func agent(named n: String) -> SubAgent? { agents.first { $0.name == n } }

    /// Splits a request into steps, routes each one, and collects the results.
    func route(_ request: String) -> String {
        var lines: [String] = []
        for step in split(request) {
            let owner = agents.first { $0.enabled && $0.accepts(step) }
            if let owner = owner {
                let r = owner.handle(step)
                lines.append("[\(r.agent)] \(r.text)")
            } else {
                lines.append("[unrouted] no enabled agent takes “\(step)”. Use manual mode or enable an agent.")
            }
        }
        return lines.joined(separator: "\n")
    }

    /// Splits “A then B”, “A and then B”, “A; B”.
    func split(_ request: String) -> [String] {
        var text = " " + request + " "
        for sep in [" then ", " and then ", ";"] {
            text = text.replacingOccurrences(of: sep, with: "\u{1}")
        }
        return text.split(separator: "\u{1}").map { $0.trimmingCharacters(in: .whitespaces) }.filter { !$0.isEmpty }
    }

    func describeAgents() -> String {
        agents.map { "\($0.enabled ? "on " : "off") \($0.name.padding(toLength: 8, withPad: " ", startingAt: 0)) \($0.purpose)" }
            .joined(separator: "\n")
    }
}
