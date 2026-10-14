/// Portable formatting core. No OS accessibility API or speech backend is connected yet.
public struct FocusNode {
    public var role: String
    public var name: String
    public var value: String
    public var focused: Bool
    public var disabled: Bool

    public init(role: String = "", name: String = "", value: String = "",
                focused: Bool = false, disabled: Bool = false) {
        self.role = role
        self.name = name
        self.value = value
        self.focused = focused
        self.disabled = disabled
    }
}

public enum ScreenReaderCore {
    public static func announce(_ node: FocusNode) -> String {
        guard node.focused else { return "" }
        var parts: [String] = [node.role.isEmpty ? "élément" : node.role]
        if !node.name.isEmpty { parts.append(node.name) }
        if !node.value.isEmpty { parts.append(node.value) }
        var result = parts.joined(separator: ": ")
        if !node.value.isEmpty && !node.name.isEmpty {
            result = "\(parts[0]): \(node.name), \(node.value)"
        }
        if node.disabled { result += ", désactivé" }
        return result
    }
}
