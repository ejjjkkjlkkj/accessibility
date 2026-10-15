const std = @import("std");
pub const Cursor = struct {
 count: u32, current: u32 = 0, has_item: bool = false,
 pub fn first(self: *Cursor) ?u32 {
  if (self.count == 0) { self.has_item = false; return null; }
  self.current = 0; self.has_item = true; return 0;
 }
 pub fn next(self: *Cursor) ?u32 {
  if (!self.has_item or self.current >= self.count - 1) return null;
  self.current += 1; return self.current;
 }
 pub fn previous(self: *Cursor) ?u32 {
  if (!self.has_item or self.current == 0) return null;
  self.current -= 1; return self.current;
 }
};
test "navigation is bounded and failure preserves state" {
 var c = Cursor{ .count = 2 };
 try std.testing.expectEqual(@as(?u32, null), c.next());
 try std.testing.expectEqual(@as(?u32, null), c.previous());
 try std.testing.expect(!c.has_item and c.current == 0);
 try std.testing.expectEqual(@as(?u32, 0), c.first());
 try std.testing.expectEqual(@as(?u32, 0), c.first());
 try std.testing.expectEqual(@as(?u32, 1), c.next());
 try std.testing.expectEqual(@as(?u32, null), c.next());
 try std.testing.expect(c.has_item and c.current == 1);
 try std.testing.expectEqual(@as(?u32, 0), c.previous());
 try std.testing.expectEqual(@as(?u32, null), c.previous());
 try std.testing.expect(c.has_item and c.current == 0);
}
