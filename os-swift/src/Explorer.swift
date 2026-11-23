// Explorer: the file shell of the graine OS. Built only on the native accessibility core.
// Equivalent in role to Explorer on Windows and Finder on macOS, but accessible by construction.
import Foundation

/// A file or folder on disk, presented as an accessible object.
final class FileObject: AccessibleObject {
    let path: String           // real path on disk
    let trashRoot: String      // where deleted items go
    let isRoot: Bool           // the system root cannot be moved or deleted

    init(path: String, trashRoot: String, isRoot: Bool = false) {
        self.path = path
        self.trashRoot = trashRoot
        self.isRoot = isRoot
    }

    var identity: String { path }

    var isFolder: Bool {
        var d: ObjCBool = false
        return FileManager.default.fileExists(atPath: path, isDirectory: &d) && d.boolValue
    }

    var name: String {
        let n = (path as NSString).lastPathComponent
        return isFolder ? n + " folder" : n
    }

    var role: Role { isFolder ? .list : .item }

    var state: Set<State> {
        FileManager.default.isReadableFile(atPath: path) ? [.enabled] : [.enabled, .error]
    }

    var actions: [Action] {
        let open = Action(key: "enter", label: isFolder ? "open" : "read", destructive: false)
        if isRoot { return [open] }
        return [open, Action(key: "delete", label: "move to trash", destructive: true)]
    }

    /// Folder children are read from disk each time, so the view is always current.
    /// The trash folder is hidden from the user.
    var children: [AccessibleObject] {
        guard isFolder, let names = try? FileManager.default.contentsOfDirectory(atPath: path) else { return [] }
        return names.sorted()
            .filter { !($0.hasPrefix(".")) }
            .map { FileObject(path: path + "/" + $0, trashRoot: trashRoot) }
    }

    func perform(_ action: Action) -> String {
        switch action.key {
        case "enter":
            if isFolder { return "opened \(name)" }
            guard let text = try? String(contentsOfFile: path, encoding: .utf8) else {
                return "\(name) is not a text file."
            }
            return text.isEmpty ? "\(name) is empty." : text
        case "delete":
            if isRoot { return "the root cannot be moved to trash." }
            let stamp = Int(Date().timeIntervalSince1970 * 1000)
            let base = (path as NSString).lastPathComponent
            let dest = trashRoot + "/\(stamp)-\(base)"
            do {
                try FileManager.default.createDirectory(atPath: trashRoot, withIntermediateDirectories: true)
                try FileManager.default.moveItem(atPath: path, toPath: dest)
                return "moved \(base) to trash."
            } catch {
                return "could not move \(base) to trash."
            }
        default:
            return "unknown action \(action.key)."
        }
    }
}

/// The explorer: navigation, opening and deleting, all through keys. Never prints directly.
final class Explorer {
    let bus: OutputBus
    let focus: FocusManager
    private let root: FileObject
    private let trashRoot: String

    init(rootPath: String, bus: OutputBus, confirm: @escaping (String) -> Bool) {
        self.bus = bus
        self.trashRoot = rootPath + "/.trash"
        self.root = FileObject(path: rootPath, trashRoot: trashRoot, isRoot: true)
        self.focus = FocusManager(bus: bus, confirm: confirm)
        focus.enter(root)
    }

    /// Opens the root and focuses its first child, so the user starts on a real item.
    func start() {
        if let first = root.children.first { focus.enter(first) }
    }

    /// Keyboard entry point. Core keys (tab, shift-tab, escape, ctrl-period) are handled first.
    func press(_ key: String) {
        switch key {
        case "tab":
            focus.next()
        case "shift-tab":
            focus.previous()
        case "escape":
            focus.back()
        case "enter":
            guard let obj = focus.current as? FileObject else { return }
            if obj.isFolder {
                focus.enter(obj)
                if let first = obj.children.first { focus.enter(first) }
            } else {
                focus.activate(actionKey: "enter")
            }
        case "delete":
            focus.activate(actionKey: "delete")
        case "ctrl-period":
            bus.cancelAll()
            bus.emit("Stopped.", priority: .alert)
        case "ctrl-r":
            for line in bus.review(last: 5) { bus.emit("review: \(line)") }
        default:
            bus.emit("No command \(key). Keys: tab, shift-tab, enter, escape, delete, ctrl-r, ctrl-period.", priority: .alert)
        }
    }
}
