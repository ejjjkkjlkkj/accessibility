public enum AccessibilityRole: UInt8, Sendable {
    case unknown = 0
    case application = 1
    case window = 2
    case group = 3
    case text = 4
    case button = 5
    case checkbox = 6
    case radio = 7
    case textBox = 8
    case link = 9
    case heading = 10
    case list = 11
    case listItem = 12
    case table = 13
    case row = 14
    case cell = 15
    case menu = 16
    case menuItem = 17
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
        abiVersion == 1 && role != .unknown && name.utf8.count <= 4096
    }
}
