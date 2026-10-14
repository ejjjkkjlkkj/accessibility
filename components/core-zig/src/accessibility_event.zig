const std = @import("std");

pub const Role = enum(u8) {
    unknown = 0,
    window = 1,
    button = 2,
    text_field = 3,
    heading = 4,
};

pub const AccessibilityEvent = struct {
    abi_version: u32 = 1,
    role: Role = .unknown,
    name: []const u8 = "",

    pub fn isValid(self: AccessibilityEvent) bool {
        return self.abi_version == 1 and
            self.role != .unknown and
            self.name.len <= 256;
    }
};

test "valid event accepted" {
    const event = AccessibilityEvent{ .role = .button, .name = "Save" };
    try std.testing.expect(event.isValid());
}

test "invalid ABI and oversized names rejected" {
    const bad_abi = AccessibilityEvent{ .abi_version = 2, .role = .button };
    try std.testing.expect(!bad_abi.isValid());
    const long_name = [_]u8{'x'} ** 257;
    const bad_name = AccessibilityEvent{ .role = .button, .name = &long_name };
    try std.testing.expect(!bad_name.isValid());
}
