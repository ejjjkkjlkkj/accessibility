// Package screenreader contains platform-independent screen-reader navigation logic.
// It does not access an operating system, display, keyboard, or speech engine.
package screenreader

type Cursor struct { count uint32; current uint32; hasItem bool }
func NewCursor(count uint32) Cursor { return Cursor{count: count} }
func (c *Cursor) First() (uint32, bool) {
 if c.count == 0 { c.hasItem = false; return 0, false }
 c.current, c.hasItem = 0, true; return 0, true
}
func (c *Cursor) Next() (uint32, bool) {
 if c.count == 0 || !c.hasItem || c.current+1 >= c.count { return 0, false }
 c.current++; return c.current, true
}
func (c *Cursor) Previous() (uint32, bool) {
 if c.count == 0 || !c.hasItem || c.current == 0 { return 0, false }
 c.current--; return c.current, true
}
func (c *Cursor) Current() (uint32, bool) {
 if !c.hasItem || c.current >= c.count { return 0, false }
 return c.current, true
}
