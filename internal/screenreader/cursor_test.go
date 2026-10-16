package screenreader

import "testing"

func TestCursorNavigationBoundaries(t *testing.T) {
	empty := NewCursor(0)
	if _, ok := empty.First(); ok {
		t.Fatal("empty list must not focus an item")
	}

	c := NewCursor(2)
	if got, ok := c.First(); !ok || got != 0 {
		t.Fatal("First should select index 0")
	}
	if got, ok := c.Next(); !ok || got != 1 {
		t.Fatal("Next should select index 1")
	}
	if _, ok := c.Next(); ok {
		t.Fatal("Next must stop at end; no implicit wrap")
	}
	if got, ok := c.Previous(); !ok || got != 0 {
		t.Fatal("Previous should return to index 0")
	}
	if _, ok := c.Previous(); ok {
		t.Fatal("Previous must stop at beginning")
	}
}
