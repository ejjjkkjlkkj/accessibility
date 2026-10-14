const std = @import("std");

pub const Node = struct {
    role: []const u8 = "",
    name: []const u8 = "",
    value: []const u8 = "",
    focused: bool = false,
    disabled: bool = false,
};

/// Caller owns the returned allocation. Platform APIs and speech are not wired yet.
pub fn announce(allocator: std.mem.Allocator, node: Node) ![]u8 {
    if (!node.focused) return try allocator.dupe(u8, "");
    const role = if (node.role.len == 0) "élément" else node.role;
    if (node.name.len > 0 and node.value.len > 0) {
        if (node.disabled) return try std.fmt.allocPrint(allocator, "{s}: {s}, {s}, désactivé", .{ role, node.name, node.value });
        return try std.fmt.allocPrint(allocator, "{s}: {s}, {s}", .{ role, node.name, node.value });
    } else if (node.name.len > 0) {
        if (node.disabled) return try std.fmt.allocPrint(allocator, "{s}: {s}, désactivé", .{ role, node.name });
        return try std.fmt.allocPrint(allocator, "{s}: {s}", .{ role, node.name });
    } else if (node.value.len > 0) {
        if (node.disabled) return try std.fmt.allocPrint(allocator, "{s}, {s}, désactivé", .{ role, node.value });
        return try std.fmt.allocPrint(allocator, "{s}, {s}", .{ role, node.value });
    }
    if (node.disabled) return try std.fmt.allocPrint(allocator, "{s}, désactivé", .{role});
    return try allocator.dupe(u8, role);
}
