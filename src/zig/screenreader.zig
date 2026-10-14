const std = @import("std");

pub const Node = struct {
    role: []const u8 = "",
    name: []const u8 = "",
    value: []const u8 = "",
    focused: bool = false,
    disabled: bool = false,
};

pub const EventKind = enum { focus, value_changed, state_changed };
pub const text_capacity = 96;
pub const queue_capacity = 16;

pub const Event = struct {
    kind: EventKind,
    role: [text_capacity]u8 = [_]u8{0} ** text_capacity,
    name: [text_capacity]u8 = [_]u8{0} ** text_capacity,
    value: [text_capacity]u8 = [_]u8{0} ** text_capacity,
    role_len: usize = 0,
    name_len: usize = 0,
    value_len: usize = 0,
    focused: bool,
    disabled: bool,

    fn copyField(dst: *[text_capacity]u8, src: []const u8) error{TextTooLong}!usize {
        if (src.len >= text_capacity) return error.TextTooLong;
        @memcpy(dst[0..src.len], src);
        return src.len;
    }

    pub fn fromNode(kind: EventKind, node: Node) error{TextTooLong}!Event {
        var e = Event{ .kind = kind, .focused = node.focused, .disabled = node.disabled };
        e.role_len = try copyField(&e.role, node.role);
        e.name_len = try copyField(&e.name, node.name);
        e.value_len = try copyField(&e.value, node.value);
        return e;
    }

    pub fn roleText(self: *const Event) []const u8 { return self.role[0..self.role_len]; }
    pub fn nameText(self: *const Event) []const u8 { return self.name[0..self.name_len]; }
    pub fn valueText(self: *const Event) []const u8 { return self.value[0..self.value_len]; }
};

pub const EventQueue = struct {
    items: [queue_capacity]Event = undefined,
    head: usize = 0,
    count: usize = 0,

    pub fn push(self: *EventQueue, kind: EventKind, node: Node) error{Full, TextTooLong}!void {
        if (self.count == queue_capacity) return error.Full;
        const event = try Event.fromNode(kind, node);
        const tail = (self.head + self.count) % queue_capacity;
        self.items[tail] = event;
        self.count += 1;
    }

    pub fn pop(self: *EventQueue) ?Event {
        if (self.count == 0) return null;
        const event = self.items[self.head];
        self.head = (self.head + 1) % queue_capacity;
        self.count -= 1;
        return event;
    }
};

/// Caller owns the returned allocation. Platform APIs and speech are not wired yet.
pub fn announce(allocator: std.mem.Allocator, node: Node) ![]u8 {
    if (!node.focused) return try allocator.dupe(u8, "");
    const role = if (node.role.len == 0) "élément" else node.role;
    var result: std.ArrayList(u8) = .empty;
    defer result.deinit(allocator);
    try result.appendSlice(allocator, role);
    if (node.name.len > 0) {
        try result.appendSlice(allocator, ": ");
        try result.appendSlice(allocator, node.name);
    }
    if (node.value.len > 0) {
        try result.appendSlice(allocator, ", ");
        try result.appendSlice(allocator, node.value);
    }
    if (node.disabled) try result.appendSlice(allocator, ", désactivé");
    return try allocator.dupe(u8, result.items);
}
