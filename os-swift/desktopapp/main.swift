// Desktop for the graine OS. Usage: desktop [root-folder]
import Foundation

let rootArg = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "rootfs"
let rootPath = URL(fileURLWithPath: rootArg).standardizedFileURL.path
let bus = OutputBus()
bus.attach(TextOutput())

let filesRoot = FileObject(path: rootPath, trashRoot: rootPath + "/.trash", isRoot: true)
let desktop = DesktopSurface(rootPath: rootPath, bus: bus, filesSurface: filesRoot) { q in
    bus.emit(q, priority: .alert)
    return (readLine() ?? "").lowercased().hasPrefix("y")
}
desktop.start()
print("keys: tab, shift-tab, enter, escape, ctrl-r, ctrl-period, quit")
while let key = readLine() {
    let k = key.trimmingCharacters(in: .whitespaces).lowercased()
    if k == "quit" { break }
    desktop.press(k.isEmpty ? "enter" : k)
}
