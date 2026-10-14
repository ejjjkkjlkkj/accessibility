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
