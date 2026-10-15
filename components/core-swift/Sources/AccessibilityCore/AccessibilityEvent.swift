public enum AccessibilityRole: UInt8, Sendable {
    case unknown = 0
    case application = 1
    case window = 2
    case group = 3
    case text = 4
    case button = 5
    case checkbox = 6
    case radio = 7
    case textbox = 8
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

public enum AccessibilityState {
    public static let focused: UInt64 = 1 << 0
    public static let focusable: UInt64 = 1 << 1
    public static let enabled: UInt64 = 1 << 2
    public static let disabled: UInt64 = 1 << 3
    public static let knownMask: UInt64 = focused | focusable | enabled | disabled

    public static func isValid(_ flags: UInt64) -> Bool {
        if flags & ~knownMask != 0 { return false }
        let isFocused = flags & focused != 0
        let isFocusable = flags & focusable != 0
        let isEnabled = flags & enabled != 0
        let isDisabled = flags & disabled != 0
        if isEnabled && isDisabled { return false }
        return !isFocused || (isFocusable && isEnabled && !isDisabled)
    }
}

public struct AccessibilityEvent: Sendable {
    public let abiVersion: UInt32
    public let role: AccessibilityRole
    public let name: String
    public let stateFlags: UInt64

    public init(abiVersion: UInt32 = 1, role: AccessibilityRole, name: String, stateFlags: UInt64 = 0) {
        self.abiVersion = abiVersion
        self.role = role
        self.name = name
        self.stateFlags = stateFlags
    }

    public var isValid: Bool {
        abiVersion == 1 &&
            name.utf8.count <= 4096 &&
            !name.contains("\0") &&
            AccessibilityState.isValid(stateFlags)
    }
}
