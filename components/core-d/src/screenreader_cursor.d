module screenreader_cursor;
struct ScreenReaderCursor {
 uint count; uint current; bool hasItem;
 this(uint count) { this.count = count; current = 0; hasItem = false; }
 bool first(out uint index) {
  if (count == 0) { hasItem = false; return false; }
  current = 0; hasItem = true; index = current; return true;
 }
 bool next(out uint index) {
  if (!hasItem || current >= count - 1) return false;
  ++current; index = current; return true;
 }
 bool previous(out uint index) {
  if (!hasItem || current == 0) return false;
  --current; index = current; return true;
 }
}
unittest {
 uint i;
 auto empty = ScreenReaderCursor(0);
 assert(!empty.first(i)); assert(!empty.next(i)); assert(!empty.previous(i));
 auto single = ScreenReaderCursor(1);
 assert(single.first(i) && i == 0); assert(!single.next(i)); assert(!single.previous(i));
 assert(single.first(i) && i == 0);
 auto c = ScreenReaderCursor(2);
 assert(c.first(i) && i == 0); assert(c.next(i) && i == 1);
 assert(!c.next(i)); assert(c.previous(i) && i == 0); assert(!c.previous(i));
 assert(c.first(i) && i == 0);
}
