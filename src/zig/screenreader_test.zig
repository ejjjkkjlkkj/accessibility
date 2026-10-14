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
    var first = q.pop().?;
    try std.testing.expectEqual(sr.EventKind.focus, first.kind);
    try std.testing.expectEqualStrings("Valider", first.nameText());
    try std.testing.expect(q.pop() == null);
    try std.testing.expectError(error.TextTooLong, sr.Event.fromNode(.focus, .{ .name = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx" }));
    for (0..sr.queue_capacity) |_| try q.push(.state_changed, .{});
    try std.testing.expectError(error.Full, q.push(.focus, .{}));
    for (0..sr.queue_capacity) |_| try std.testing.expect(q.pop() != null);
    try std.testing.expect(q.pop() == null);
    _ = &first;
}
