public enum AccessibilityRole: UInt8, Sendable {
    case unknown = 0
    case window = 1
    case button = 2
    case textField = 3
    case heading = 4
}

public struct AccessibilityEvent: Sendable {
    public let abiVersion: UInt32
    public let role: AccessibilityRole
    public let name: String

    public init(abiVersion: UInt32 = 1, role: AccessibilityRole, name: String) {
        self.abiVersion = abiVersion
        self.role = role
        self.name = name
    }

    public var isValid: Bool {
        abiVersion == 1 && role != .unknown && name.utf8.count <= 256
    }
}
