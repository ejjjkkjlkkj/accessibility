/// Pure navigation state. Platform adapters provide the semantic item count.
public struct ScreenReaderCursor: Sendable {
 public let count: UInt32
 private(set) public var current: UInt32 = 0
 private(set) public var hasItem = false
 public init(count: UInt32) { self.count = count }
 public mutating func first() -> UInt32? {
  guard count > 0 else { hasItem = false; return nil }
  current = 0; hasItem = true; return current
 }
 public mutating func next() -> UInt32? {
  guard hasItem, current < count - 1 else { return nil }
  current += 1; return current
 }
 public mutating func previous() -> UInt32? {
  guard hasItem, current > 0 else { return nil }
  current -= 1; return current
 }
}
