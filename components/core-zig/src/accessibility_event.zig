const std = @import("std");

pub const Role = enum(u8) {
    unknown = 0,
    application = 1,
    window = 2,
    group = 3,
    text = 4,
    button = 5,
    checkbox = 6,
    radio = 7,
    textbox = 8,
    link = 9,
    heading = 10,
    list = 11,
    list_item = 12,
    table = 13,
    row = 14,
    cell = 15,
    menu = 16,
    menu_item = 17,
};

pub const AccessibilityEvent = struct {
    abi_version: u32 = 1,
    role: Role = .unknown,
    name: []const u8 = "",

    pub fn isValid(self: AccessibilityEvent) bool {
        return self.abi_version == 1 and
            self.role != .unknown and
            self.name.len <= 4096;
    }
};

test "canonical role codes are stable" {\n    try std.testing.expectEqual(@as(u8, 5), @intFromEnum(Role.button));\n    try std.testing.expectEqual(@as(u8, 17), @intFromEnum(Role.menu_item));\n}\n\ntest "valid event accepted" {
    const event = AccessibilityEvent{ .role = .button, .name = "Save" };
    try std.testing.expect(event.isValid());
}

test "invalid ABI and oversized names rejected" {
    const bad_abi = AccessibilityEvent{ .abi_version = 2, .role = .button };
    try std.testing.expect(!bad_abi.isValid());
    const long_name = [_]u8{'x'} ** 4097;
    const bad_name = AccessibilityEvent{ .role = .button, .name = &long_name };
    try std.testing.expect(!bad_name.isValid());
}
