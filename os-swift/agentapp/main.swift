// Graine OS: manual mode by default, orchestrated sub-agents on demand. Usage: graine [root-folder]
import Foundation

let rootArg = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "rootfs"
let rootPath = URL(fileURLWithPath: rootArg).standardizedFileURL.path
let shell = try ShellV2(rootPath: rootPath)
try? FileManager.default.createDirectory(atPath: rootPath + "/var", withIntermediateDirectories: true)
let core = Agent(shell: shell, logPath: rootPath + "/var/agent.log")

let orchestrator = Orchestrator()
orchestrator.register(HistoryAgent(core: core))   // first: history questions go here
orchestrator.register(FileAgent(core: core))

var mode = "manual"
let announcer = Announcer()

print("Graine OS. Mode: \(mode). You are in control.")
print("Commands: mode agent | mode manual | agents | enable <name> | disable <name> | quit")

while true {
    print("\(mode)\(shell.prompt)", terminator: "")
    guard let raw = readLine() else { break }
    let line = raw.trimmingCharacters(in: .whitespaces)
    if line.isEmpty { continue }
    if line == "quit" { break }

    // Always available, in any mode
    if line == "mode agent" { mode = "agent"; print("Agent mode. Ask in plain English."); continue }
    if line == "mode manual" { mode = "manual"; print("Manual mode. Every shell command runs as typed."); continue }
    if line == "agents" { announcer.announce(orchestrator.describeAgents()); continue }
    if line == "voice on" { announcer.setVoice(true); announcer.announce("Voice on."); continue }
    if line == "voice off" { announcer.announce("Voice off."); announcer.setVoice(false); continue }
    if line.hasPrefix("enable ") || line.hasPrefix("disable ") {
        let on = line.hasPrefix("enable ")
        let name = String(line.split(separator: " ").last ?? "")
        if let a = orchestrator.agent(named: name) {
            a.enabled = on
            print("\(name) is now \(on ? "on" : "off").")
        } else {
            print("No agent named \(name). Try: agents")
        }
        continue
    }

    if mode == "manual" {
        let out = shell.execute(line)
        if !out.isEmpty { print(out) }
        continue
    }

    announcer.announce(orchestrator.route(line))
}
