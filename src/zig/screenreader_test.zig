const std = @import("std");
const sr = @import("screenreader.zig");

test "focused node announcement" {
    const got = try sr.announce(std.testing.allocator, .{ .role = "bouton", .name = "Valider", .focused = true });
    defer std.testing.allocator.free(got);
    try std.testing.expectEqualStrings("bouton: Valider", got);
}
test "unfocused node is silent" {
    const got = try sr.announce(std.testing.allocator, .{ .role = "bouton", .name = "Valider", .focused = false });
    defer std.testing.allocator.free(got);
    try std.testing.expectEqualStrings("", got);
}
test "value and disabled state" {
    const got = try sr.announce(std.testing.allocator, .{ .role = "champ", .name = "Volume", .value = "80 %", .focused = true, .disabled = true });
    defer std.testing.allocator.free(got);
    try std.testing.expectEqualStrings("champ: Volume, 80 %, désactivé", got);
}
test "bounded event queue copies fields and preserves FIFO" {
    var q = sr.EventQueue{};
    try q.push(.focus, .{ .role = "bouton", .name = "Valider", .focused = true });
    const first = q.pop().?;
    try std.testing.expectEqual(sr.EventKind.focus, first.kind);
    try std.testing.expectEqualStrings("Valider", first.nameText());
    try std.testing.expect(q.pop() == null);
    try std.testing.expectError(error.TextTooLong, sr.Event.fromNode(.focus, .{ .name = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx" }));
    for (0..sr.queue_capacity) |_| try q.push(.state_changed, .{});
    try std.testing.expectError(error.Full, q.push(.focus, .{}));
    for (0..sr.queue_capacity) |_| try std.testing.expect(q.pop() != null);
    try std.testing.expect(q.pop() == null);
}

test "event queue ring wraparound preserves FIFO" {
    var q = sr.EventQueue{};
    const a = sr.Node{ .role = "bouton", .name = "A", .focused = true };
    const b = sr.Node{ .role = "bouton", .name = "B", .focused = true };
    for (0..12) |_| try q.push(.focus, a);
    for (0..8) |_| try std.testing.expectEqualStrings("A", q.pop().?.nameText());
    for (0..12) |_| try q.push(.focus, b);
    try std.testing.expectEqual(sr.queue_capacity, q.count);
    for (0..4) |_| try std.testing.expectEqualStrings("A", q.pop().?.nameText());
    for (0..12) |_| try std.testing.expectEqualStrings("B", q.pop().?.nameText());
    try std.testing.expect(q.pop() == null);
}

test "missing role and empty fields use generic label" {
    const got = try sr.announce(std.testing.allocator, .{ .value = "42", .focused = true });
    defer std.testing.allocator.free(got);
    try std.testing.expectEqualStrings("élément, 42", got);
    const empty = try sr.announce(std.testing.allocator, .{ .focused = true });
    defer std.testing.allocator.free(empty);
    try std.testing.expectEqualStrings("élément", empty);
}
