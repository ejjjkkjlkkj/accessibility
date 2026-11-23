// Desktop and Settings for the graine OS. Built only on the native accessibility core.
import Foundation

// MARK: - Settings store (saved in etc/settings.txt)

final class SettingsStore {
    private let path: String
    private(set) var values: [String: String] = [:]

    /// Defaults are safe and accessible: text on, speech off, confirmations on.
    static let defaults: [String: String] = [
        "voice": "off",
        "braille": "on",
        "confirm-destructive": "on",
        "explorer-start": "first-item",
    ]

    init(path: String) {
        self.path = path
        values = SettingsStore.defaults
        load()
    }

    func get(_ key: String) -> String { values[key] ?? SettingsStore.defaults[key] ?? "" }

    func set(_ key: String, _ value: String) {
        values[key] = value
        save()
    }

    func toggle(_ key: String) -> String {
        let next = get(key) == "on" ? "off" : "on"
        set(key, next)
        return next
    }

    private func load() {
        guard let text = try? String(contentsOfFile: path, encoding: .utf8) else { return }
        for raw in text.components(separatedBy: .newlines) {
            let line = raw.trimmingCharacters(in: .whitespaces)
            guard !line.isEmpty, !line.hasPrefix("#"), let eq = line.firstIndex(of: "=") else { continue }
            let k = line[..<eq].trimmingCharacters(in: .whitespaces)
            let v = line[line.index(after: eq)...].trimmingCharacters(in: .whitespaces)
            values[k] = v
        }
    }

    private func save() {
        let dir = (path as NSString).deletingLastPathComponent
        try? FileManager.default.createDirectory(atPath: dir, withIntermediateDirectories: true)
        let body = values.keys.sorted().map { "\($0) = \(values[$0]!)" }.joined(separator: "\n") + "\n"
        try? body.write(toFile: path, atomically: true, encoding: .utf8)
    }
}

// MARK: - Settings objects

/// One setting shown as an accessible toggle. Enter flips it.
final class SettingItem: AccessibleObject {
    let key: String
    let label: String
    private let store: SettingsStore
    private let onChange: (String, String) -> Void

    init(key: String, label: String, store: SettingsStore, onChange: @escaping (String, String) -> Void) {
        self.key = key
        self.label = label
        self.store = store
        self.onChange = onChange
    }

    var name: String { "\(label): \(store.get(key))" }
    var role: Role { .item }
    var state: Set<State> { [.enabled] }
    var identity: String { "setting:" + key }
    var actions: [Action] { [Action(key: "enter", label: "switch", destructive: false)] }

    func perform(_ action: Action) -> String {
        let now = store.toggle(key)
        onChange(key, now)
        return "\(label) is now \(now)."
    }
}

// MARK: - Containers

/// A container of objects. Its children are fixed when it is created.
final class Container: AccessibleObject {
    let name: String
    let role: Role
    let identity: String
    let kids: [AccessibleObject]

    init(name: String, role: Role, identity: String, children: [AccessibleObject]) {
        self.name = name
        self.role = role
        self.identity = identity
        self.kids = children
    }

    var state: Set<State> { [.enabled] }
    var children: [AccessibleObject] { kids }
    var actions: [Action] { [Action(key: "enter", label: "open", destructive: false)] }
    func perform(_ action: Action) -> String { "opened \(name)." }
}

/// A shortcut on the desktop that opens another surface (Files, Settings, ...).
final class Shortcut: AccessibleObject {
    let name: String
    let target: AccessibleObject

    init(name: String, target: AccessibleObject) {
        self.name = name
        self.target = target
    }

    var role: Role { .button }
    var state: Set<State> { [.enabled] }
    var identity: String { "shortcut:" + name }
    var actions: [Action] { [Action(key: "enter", label: "open", destructive: false)] }
    func perform(_ action: Action) -> String { "opening \(name)." }
}

// MARK: - Desktop

/// The desktop: shortcuts to the surfaces of the system. Starts on the first shortcut.
final class DesktopSurface {
    let bus: OutputBus
    let focus: FocusManager
    let store: SettingsStore
    let settingsRoot: Container
    let desktop: Container

    /// Builds the desktop from the root folder. `filesSurface` is the explorer's first object.
    init(rootPath: String, bus: OutputBus, filesSurface: AccessibleObject,
         confirm: @escaping (String) -> Bool) {
        self.bus = bus
        self.store = SettingsStore(path: rootPath + "/etc/settings.txt")
        self.focus = FocusManager(bus: bus, confirm: confirm)

        // Settings: each toggle is an accessible item. A change takes effect on the bus at once and is saved.
        var items: [AccessibleObject] = []
        items.append(SettingItem(key: "voice", label: "Voice output", store: store) { _, v in
            DesktopSurface.applyChannel(bus, name: "speech", on: v == "on") { SpeechOutput() }
        })
        items.append(SettingItem(key: "braille", label: "Braille output", store: store) { _, v in
            DesktopSurface.applyChannel(bus, name: "braille", on: v == "on") { BrailleOutput() }
        })
        items.append(SettingItem(key: "confirm-destructive", label: "Confirm before deleting", store: store) { _, _ in })
        self.settingsRoot = Container(name: "Settings", role: .window, identity: "settings", children: items)

        // Apply the saved settings at startup, so the chosen channels are active from the first message.
        DesktopSurface.applyChannel(bus, name: "speech", on: store.get("voice") == "on") { SpeechOutput() }
        DesktopSurface.applyChannel(bus, name: "braille", on: store.get("braille") == "on") { BrailleOutput() }

        // Desktop: shortcuts to Files and Settings.
        self.desktop = Container(name: "Desktop", role: .window, identity: "desktop", children: [
            Shortcut(name: "Files", target: filesSurface),
            Shortcut(name: "Settings", target: settingsRoot),
        ])
    }

    /// Attaches or detaches one output channel on the bus.
    static func applyChannel(_ bus: OutputBus, name: String, on: Bool, make: () -> OutputChannel) {
        if on {
            if bus.channel(named: name) == nil { bus.attach(make()) }
        } else {
            bus.detach(named: name)
        }
    }

    /// Opens the desktop and focuses its first shortcut.
    func start() {
        focus.enter(desktop)
        if let first = desktop.children.first { focus.enter(first) }
    }

    /// Keyboard: tab, shift-tab, enter (open a shortcut or toggle a setting), escape (back).
    func press(_ key: String) {
        switch key {
        case "tab": focus.next()
        case "shift-tab": focus.previous()
        case "escape": focus.back()
        case "enter":
            guard let current = focus.current else { return }
            if let shortcut = current as? Shortcut {
                focus.enter(shortcut.target)
                if let first = shortcut.target.children.first { focus.enter(first) }
            } else {
                focus.activate(actionKey: "enter")
            }
        case "ctrl-period":
            bus.cancelAll()
            bus.emit("Stopped.", priority: .alert)
        default:
            bus.emit("No command \(key). Keys: tab, shift-tab, enter, escape, ctrl-period, quit.", priority: .alert)
        }
    }
}
