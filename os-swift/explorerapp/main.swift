// Explorer for the graine OS. Usage: explorer [root-folder]
import Foundation

let rootArg = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "rootfs"
let rootPath = URL(fileURLWithPath: rootArg).standardizedFileURL.path
let bus = OutputBus()
bus.attach(TextOutput())
bus.attach(BrailleOutput())

// Ask yes/no on the keyboard: y to confirm, anything else refuses.
let explorer = Explorer(rootPath: rootPath, bus: bus, confirm: { question in
    bus.emit(question, priority: .alert)
    return (readLine() ?? "").lowercased().hasPrefix("y")
})
explorer.start()
print("keys: tab, shift-tab, enter, escape, delete, ctrl-r, ctrl-period, quit")
while let key = readLine() {
    let k = key.trimmingCharacters(in: .whitespaces).lowercased()
    if k == "quit" { break }
    explorer.press(k.isEmpty ? "enter" : k)
}
