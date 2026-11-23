// Native accessibility core. Part of the OS, not an add-on. Complete v0.2.
import Foundation

// MARK: - Vocabulary

enum Role: String {
    case window, list, item, text, button, dialog, status, alert
    // Web content (added with the browser): links, headings, form fields, images, page regions.
    case link, heading, field, image, region
}

enum State: String {
    case enabled, focused, selected, busy, error
}

/// An action an object can perform, reachable from the keyboard.
struct Action {
    let key: String            // keyboard key that triggers it, e.g. "enter"
    let label: String          // what the user hears
    let destructive: Bool      // destructive actions need explicit confirmation
}

/// Every object the OS creates implements this. There is no object without a name and a role.
protocol AccessibleObject: AnyObject {
    var name: String { get }
    var role: Role { get }
    var state: Set<State> { get }
    var actions: [Action] { get }
    /// Objects that contain other objects (windows, lists). Leaves return [].
    var children: [AccessibleObject] { get }
    /// Stable identity for finding the same object again.
    var identity: String { get }
    /// Runs an action. Called only after confirmation when the action is destructive.
    func perform(_ action: Action) -> String
}

extension AccessibleObject {
    var children: [AccessibleObject] { [] }
    /// Identity used to find an object again (objects may be re-created from disk).
    var identity: String { "\(ObjectIdentifier(self as AnyObject))" }
}

/// Message priority. An alert interrupts speech; info waits its turn.
enum Priority: Int, Comparable {
    case info = 0, alert = 1
    static func < (a: Priority, b: Priority) -> Bool { a.rawValue < b.rawValue }
}

struct Message {
    let text: String
    let priority: Priority
}

// MARK: - Output channels

/// One output channel. Text is the baseline; speech and braille are added to the bus.
protocol OutputChannel: AnyObject {
    var name: String { get }
    func emit(_ message: Message)
    /// Stops anything in progress (speech). Text and braille have nothing to stop.
    func cancel()
}

extension OutputChannel {
    func cancel() {}
}

/// Text output to the terminal: readable by any screen reader or braille display.
final class TextOutput: OutputChannel {
    let name = "text"
    func emit(_ message: Message) { print(message.text) }
}

/// Braille output: the same text as Unicode braille cells (U+2800 block), for braille displays
/// and for reading in a terminal. Letters and digits are mapped; other characters pass through.
final class BrailleOutput: OutputChannel {
    let name = "braille"
    private var lastLine: String = ""
    private var lines: [String] = []

    /// Dot patterns for a-z (standard six-dot braille, Unicode U+2800 + bitmask).
    static let letters: [Character: UInt32] = [
        "a": 0x01, "b": 0x03, "c": 0x09, "d": 0x19, "e": 0x11, "f": 0x0B, "g": 0x1B, "h": 0x13,
        "i": 0x0A, "j": 0x1A, "k": 0x05, "l": 0x07, "m": 0x0D, "n": 0x1D, "o": 0x15, "p": 0x0F,
        "q": 0x1F, "r": 0x17, "s": 0x0E, "t": 0x1E, "u": 0x25, "v": 0x27, "w": 0x3A, "x": 0x2D,
        "y": 0x3D, "z": 0x35,
    ]

    static func render(_ text: String) -> String {
        var out = ""
        for ch in text.lowercased() {
            if ch == " " { out.append(" "); continue }
            if let bits = letters[ch], let scalar = UnicodeScalar(0x2800 + bits) {
                out.append(Character(scalar))
            } else {
                out.append(ch)
            }
        }
        return out
    }

    func emit(_ message: Message) {
        let line = BrailleOutput.render(message.text)
        lastLine = line
        lines.append(line)
    }

    /// The last rendered line, for a braille display driver to pick up.
    var current: String { lastLine }
}

/// Speech output through the system voice (Windows SAPI via PowerShell).
/// Each utterance is a separate process so it can be cancelled.
final class SpeechOutput: OutputChannel {
    let name = "speech"
    private var running: [Process] = []

    func emit(_ message: Message) {
        let safe = message.text.replacingOccurrences(of: "'", with: "''")
        let p = Process()
        p.executableURL = URL(fileURLWithPath: "C:/Windows/System32/WindowsPowerShell/v1.0/powershell.exe")
        p.arguments = ["-NoProfile", "-NonInteractive", "-Command",
                       "Add-Type -AssemblyName System.Speech; (New-Object System.Speech.Synthesis.SpeechSynthesizer).Speak('\(safe)')"]
        do {
            try p.run()
            running.append(p)
        } catch {
            // Speech failure never blocks text output.
        }
    }

    func cancel() {
        for p in running where p.isRunning { p.terminate() }
        running.removeAll()
    }
}

// MARK: - Bus

/// The only way to produce output. Everything goes through here.
final class OutputBus {
    private(set) var channels: [OutputChannel] = []
    private(set) var history: [Message] = []
    private let historyLimit = 200

    func attach(_ channel: OutputChannel) { channels.append(channel) }

    func detach(named n: String) { channels.removeAll { $0.name == n } }

    func channel(named n: String) -> OutputChannel? { channels.first { $0.name == n } }

    func emit(_ text: String, priority: Priority = .info) {
        let m = Message(text: text, priority: priority)
        history.append(m)
        if history.count > historyLimit { history.removeFirst(history.count - historyLimit) }
        // An alert cancels whatever is still being spoken, so it is heard at once.
        if priority == .alert {
            for c in channels { c.cancel() }
        }
        for c in channels { c.emit(m) }
    }

    /// Stop everything in progress on every channel.
    func cancelAll() {
        for c in channels { c.cancel() }
    }

    /// The last `n` messages, oldest first. Lets the user review what was said.
    func review(last n: Int = 10) -> [String] {
        history.suffix(n).map { $0.text }
    }
}

// MARK: - Focus

/// Holds the single focused object, moves between siblings, and announces every move.
final class FocusManager {
    private let bus: OutputBus
    private let confirm: (String) -> Bool
    private var stack: [AccessibleObject] = []

    /// `confirm` asks the user yes/no for destructive actions. Default is to refuse.
    init(bus: OutputBus, confirm: @escaping (String) -> Bool = { _ in false }) {
        self.bus = bus
        self.confirm = confirm
    }

    var current: AccessibleObject? { stack.last }

    var depth: Int { stack.count }

    /// Enter an object (opening a window or list). Its name and actions are announced.
    func enter(_ object: AccessibleObject) {
        stack.append(object)
        announce(object)
    }

    /// Return to the parent object.
    func back() {
        guard stack.count > 1 else {
            bus.emit("Already at the top.", priority: .alert)
            return
        }
        stack.removeLast()
        if let c = current { announce(c) }
    }

    /// Move to the next sibling inside the current container. Wraps around.
    func next() { move(by: 1) }

    /// Move to the previous sibling inside the current container. Wraps around.
    func previous() { move(by: -1) }

    private func move(by step: Int) {
        // The siblings of the focused object are the children of its parent in the stack.
        let siblings = stackSiblings()
        guard let me = current, !siblings.isEmpty,
              let index = siblings.firstIndex(where: { $0.identity == me.identity }) else {
            bus.emit("Nothing to move to.", priority: .alert)
            return
        }
        let target = siblings[(index + step + siblings.count) % siblings.count]
        stack[stack.count - 1] = target
        announce(target)
    }

    private func stackSiblings() -> [AccessibleObject] {
        guard stack.count >= 2 else { return [] }
        return Array(stack[stack.count - 2].children)
    }

    /// Run action number `index` of the focused object. Destructive actions need confirmation.
    func activate(actionKey key: String) {
        guard let obj = current else { return }
        guard let action = obj.actions.first(where: { $0.key == key }) else {
            bus.emit("No action \(key) on \(obj.name).", priority: .alert)
            return
        }
        if action.destructive {
            let question = "Confirm: \(action.label) \(obj.name)? Say yes or no."
            guard confirm(question) else {
                bus.emit("Cancelled: \(action.label) \(obj.name).", priority: .alert)
                return
            }
        }
        let result = obj.perform(action)
        if !result.isEmpty { bus.emit(result) }
    }

    /// Replace the focused object by another one in the same container (for example the next heading).
    func jump(to object: AccessibleObject) {
        guard !stack.isEmpty else { return }
        stack[stack.count - 1] = object
        announce(object)
    }

    private func announce(_ o: AccessibleObject) {
        let actions = o.actions.map { "\($0.label) (\($0.key))" }.joined(separator: ", ")
        var line = "\(o.role.rawValue): \(o.name)."
        if !actions.isEmpty { line += " Actions: \(actions)." }
        if o.state.contains(.error) { line += " Error." }
        bus.emit(line, priority: o.state.contains(.error) ? .alert : .info)
    }
}

// MARK: - Keys and interruption

/// Core keys, the same everywhere in the system.
enum CoreKey: String {
    case next = "tab"
    case previous = "shift-tab"
    case enter = "enter"
    case back = "escape"
    case stop = "ctrl-period"
    case review = "ctrl-r"
}

/// Dispatches a key to the focus manager or the bus. Stop is always handled first.
final class KeyRouter {
    private let focus: FocusManager
    private let bus: OutputBus

    init(focus: FocusManager, bus: OutputBus) {
        self.focus = focus
        self.bus = bus
    }

    func press(_ key: String) {
        switch CoreKey(rawValue: key) {
        case .stop:
            bus.cancelAll()
            bus.emit("Stopped.", priority: .alert)
        case .next: focus.next()
        case .previous: focus.previous()
        case .back: focus.back()
        case .review:
            for line in bus.review(last: 5) { print("review: \(line)") }
        case .enter:
            focus.activate(actionKey: "enter")
        case .none:
            // Keys that are not core keys go to the focused object's own actions.
            focus.activate(actionKey: key)
        }
    }
}
