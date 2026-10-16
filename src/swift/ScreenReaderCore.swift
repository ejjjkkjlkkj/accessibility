/// Portable formatting core. No OS accessibility API or speech backend is connected yet.
public struct FocusNode {
    public var role: String
    public var name: String
    public var value: String
    public var focused: Bool
    public var disabled: Bool

    public init(role: String = "", name: String = "", value: String = "",
                focused: Bool = false, disabled: Bool = false) {
        self.role = role; self.name = name; self.value = value
        self.focused = focused; self.disabled = disabled
    }
}

public enum EventKind { case focus, valueChanged, stateChanged }
public struct SemanticEvent {
    public let kind: EventKind
    public let node: FocusNode
}

/// Bounded FIFO snapshots. String values are copied as Swift value types.
public struct EventQueue {
    public static let capacity = 16
    public static let textLimit = 96
    private var items: [SemanticEvent] = []
    public init() {}
    public var count: Int { items.count }

    @discardableResult
    public mutating func push(_ kind: EventKind, node: FocusNode) -> Bool {
        guard items.count < Self.capacity,
              node.role.utf8.count < Self.textLimit,
              node.name.utf8.count < Self.textLimit,
              node.value.utf8.count < Self.textLimit else { return false }
        items.append(SemanticEvent(kind: kind, node: node))
        return true
    }

    public mutating func pop() -> SemanticEvent? {
        guard !items.isEmpty else { return nil }
        return items.removeFirst()
    }
}

public enum ScreenReaderCore {
    public static func announce(_ node: FocusNode) -> String {
        guard node.focused else { return "" }
        let role = node.role.isEmpty ? "élément" : node.role
        var result = role
        if !node.name.isEmpty { result += ": " + node.name }
        if !node.value.isEmpty { result += ", " + node.value }
        if node.disabled { result += ", désactivé" }
        return result
    }
}
