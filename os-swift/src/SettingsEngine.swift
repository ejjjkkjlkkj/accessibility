// Settings engine for the graine OS. Built on the native accessibility core.
// Rules: every setting explains itself, is searchable, is undoable, is resettable, and every change is logged.
import Foundation

enum SettingCategory: String, CaseIterable {
    case accessibility = "Accessibility"   // first, as a top-level category
    case display = "Display and input"
    case safety = "Safety"
}

/// A setting definition: what it is, what it does, what values it accepts.
struct SettingDef {
    let key: String
    let category: SettingCategory
    let label: String
    let explanation: String        // read aloud before the user changes it
    let values: [String]           // allowed values, first one is the default
    var defaultValue: String { values[0] }
}

/// One change, kept for undo and for the audit log.
struct SettingChange {
    let time: Date
    let key: String
    let from: String
    let to: String
}

enum SettingsError: Error, CustomStringConvertible {
    case unknownKey(String)
    case invalidValue(String, String)
    var description: String {
        switch self {
        case .unknownKey(let k): return "No setting called \(k). Try: search <word>."
        case .invalidValue(let k, let v): return "\(v) is not a valid value for \(k)."
        }
    }
}

final class SettingsEngine {
    /// Every setting the system knows. New settings are added here, nowhere else.
    static let catalog: [SettingDef] = [
        SettingDef(key: "voice", category: .accessibility, label: "Voice output",
                   explanation: "Reads every message aloud. Turning it off leaves text and braille.",
                   values: ["off", "on"]),
        SettingDef(key: "voice-speed", category: .accessibility, label: "Voice speed",
                   explanation: "How fast the voice speaks. Slow, normal or fast.",
                   values: ["normal", "slow", "fast"]),
        SettingDef(key: "braille", category: .accessibility, label: "Braille output",
                   explanation: "Shows every message as braille cells as well as text.",
                   values: ["on", "off"]),
        SettingDef(key: "announce-focus", category: .accessibility, label: "Announce focus moves",
                   explanation: "Says the name and actions of each object you move to.",
                   values: ["on", "off"]),
        SettingDef(key: "keys", category: .display, label: "Key style",
                   explanation: "Which keys move focus: tab and arrows, or tab only.",
                   values: ["tab", "tab-and-arrows"]),
        SettingDef(key: "confirm-destructive", category: .safety, label: "Confirm before deleting",
                   explanation: "Asks yes or no before anything is moved to the trash.",
                   values: ["on", "off"]),
    ]

    private let path: String
    private(set) var values: [String: String] = [:]
    private(set) var history: [SettingChange] = []     // newest last, used for undo
    private(set) var log: [SettingChange] = []         // everything ever changed, never trimmed

    init(path: String) {
        self.path = path
        for d in SettingsEngine.catalog { values[d.key] = d.defaultValue }
        load()
    }

    func definition(_ key: String) throws -> SettingDef {
        guard let d = SettingsEngine.catalog.first(where: { $0.key == key }) else {
            throw SettingsError.unknownKey(key)
        }
        return d
    }

    func get(_ key: String) -> String { values[key] ?? "" }

    /// Sets a value after checking it is allowed. Records the change for undo and the log.
    @discardableResult
    func set(_ key: String, _ value: String) throws -> SettingChange {
        let d = try definition(key)
        guard d.values.contains(value) else { throw SettingsError.invalidValue(key, value) }
        let change = SettingChange(time: Date(), key: key, from: get(key), to: value)
        values[key] = value
        history.append(change)
        log.append(change)
        save()
        return change
    }

    /// Cycles to the next allowed value (used by a single Enter key press).
    @discardableResult
    func cycle(_ key: String) throws -> SettingChange {
        let d = try definition(key)
        let i = d.values.firstIndex(of: get(key)) ?? 0
        return try set(key, d.values[(i + 1) % d.values.count])
    }

    /// Undoes the last change. Undo is itself logged, so nothing is hidden.
    func undo() -> SettingChange? {
        guard let last = history.popLast() else { return nil }
        values[last.key] = last.from
        let back = SettingChange(time: Date(), key: last.key, from: last.to, to: last.from)
        log.append(back)
        save()
        return back
    }

    /// Resets one category to defaults, or everything when category is nil.
    func reset(_ category: SettingCategory? = nil) {
        for d in SettingsEngine.catalog where category == nil || d.category == category {
            if get(d.key) != d.defaultValue {
                let change = SettingChange(time: Date(), key: d.key, from: get(d.key), to: d.defaultValue)
                values[d.key] = d.defaultValue
                log.append(change)
                history.append(change)
            }
        }
        save()
    }

    /// Finds settings whose label or explanation contains every word typed.
    func search(_ query: String) -> [SettingDef] {
        let words = query.lowercased().split(separator: " ").map(String.init)
        guard !words.isEmpty else { return SettingsEngine.catalog }
        return SettingsEngine.catalog.filter { d in
            let haystack = (d.label + " " + d.explanation + " " + d.key).lowercased()
            return words.allSatisfy { haystack.contains($0) }
        }
    }

    /// Settings of one category, in catalog order.
    func settings(in category: SettingCategory) -> [SettingDef] {
        SettingsEngine.catalog.filter { $0.category == category }
    }

    // MARK: Storage

    private func load() {
        guard let text = try? String(contentsOfFile: path, encoding: .utf8) else { return }
        for raw in text.components(separatedBy: .newlines) {
            let line = raw.trimmingCharacters(in: .whitespaces)
            guard !line.isEmpty, !line.hasPrefix("#"), let eq = line.firstIndex(of: "=") else { continue }
            let k = line[..<eq].trimmingCharacters(in: .whitespaces)
            let v = line[line.index(after: eq)...].trimmingCharacters(in: .whitespaces)
            if let d = try? definition(k), d.values.contains(v) { values[k] = v }
        }
    }

    private func save() {
        let dir = (path as NSString).deletingLastPathComponent
        try? FileManager.default.createDirectory(atPath: dir, withIntermediateDirectories: true)
        let body = SettingsEngine.catalog.map { "\($0.key) = \(get($0.key))" }.joined(separator: "\n") + "\n"
        try? body.write(toFile: path, atomically: true, encoding: .utf8)
    }
}
