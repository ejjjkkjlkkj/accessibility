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

pub const STATE_FOCUSED: u64 = 1 << 0;
pub const STATE_FOCUSABLE: u64 = 1 << 1;
pub const STATE_ENABLED: u64 = 1 << 2;
pub const STATE_DISABLED: u64 = 1 << 3;
pub const STATE_KNOWN_MASK: u64 = 0x0F;

pub fn stateFlagsAreValid(flags: u64) bool {
    if ((flags & ~STATE_KNOWN_MASK) != 0) return false;
    if ((flags & STATE_ENABLED) != 0 and (flags & STATE_DISABLED) != 0) return false;
    if ((flags & STATE_FOCUSED) != 0 and
        ((flags & STATE_FOCUSABLE) == 0 or
         (flags & STATE_ENABLED) == 0 or
         (flags & STATE_DISABLED) != 0)) return false;
    return true;
}

pub const AccessibilityEvent = struct {
    abi_version: u32 = 1,
    role: Role = .unknown,
    name: []const u8 = "",
    state_flags: u64 = 0,

    pub fn isValid(self: AccessibilityEvent) bool {
        return self.abi_version == 1 and
            self.name.len <= 4096 and
            stateFlagsAreValid(self.state_flags);
    }
};

test "canonical role codes are stable" {
    try std.testing.expectEqual(@as(u8, 5), @intFromEnum(Role.button));
    try std.testing.expectEqual(@as(u8, 17), @intFromEnum(Role.menu_item));
}

test "valid event accepted" {
    const event = AccessibilityEvent{ .role = .button, .name = "Save" };
    try std.testing.expect(event.isValid());
}

test "semantic state flags reject contradictions and reserved bits" {
    try std.testing.expect(stateFlagsAreValid(STATE_FOCUSABLE | STATE_ENABLED | STATE_FOCUSED));
    try std.testing.expect(!stateFlagsAreValid(STATE_FOCUSED));
    try std.testing.expect(!stateFlagsAreValid(STATE_ENABLED | STATE_DISABLED));
    try std.testing.expect(!stateFlagsAreValid(@as(u64, 1) << 63));
}

test "invalid ABI and oversized names rejected" {
    const bad_abi = AccessibilityEvent{ .abi_version = 2, .role = .button };
    try std.testing.expect(!bad_abi.isValid());
    const long_name = [_]u8{'x'} ** 4097;
    const bad_name = AccessibilityEvent{ .role = .button, .name = &long_name };
    try std.testing.expect(!bad_name.isValid());
}
