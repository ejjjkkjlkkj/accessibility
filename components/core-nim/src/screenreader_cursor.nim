type ScreenReaderCursor* = object
 count*: uint32
 current*: uint32
 hasItem*: bool
proc initCursor*(count: uint32): ScreenReaderCursor =
 ScreenReaderCursor(count: count, current: 0, hasItem: false)
proc first*(c: var ScreenReaderCursor): int64 =
 if c.count == 0: c.hasItem = false; return -1
 c.current = 0; c.hasItem = true; return 0
proc next*(c: var ScreenReaderCursor): int64 =
 if not c.hasItem or c.current >= c.count - 1: return -1
 inc c.current; return int64(c.current)
proc previous*(c: var ScreenReaderCursor): int64 =
 if not c.hasItem or c.current == 0: return -1
 dec c.current; return int64(c.current)
when isMainModule:
 var c = initCursor(2)
 doAssert c.first() == 0; doAssert c.next() == 1; doAssert c.next() == -1
 doAssert c.previous() == 0; doAssert c.previous() == -1
 var empty = initCursor(0); doAssert empty.first() == -1
