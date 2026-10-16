package screenreader

import "testing"

func TestCursorNavigationBoundaries(t *testing.T) {
	empty := NewCursor(0)
	if _, ok := empty.Current(); ok {
		t.Fatal("Current must fail before focus is established")
	}
	if _, ok := empty.First(); ok {
		t.Fatal("empty list must not focus an item")
	}

	c := NewCursor(2)
	if _, ok := c.Current(); ok {
		t.Fatal("Current must fail before First")
	}
	if got, ok := c.First(); !ok || got != 0 {
		t.Fatal("First should select index 0")
	}
	if got, ok := c.Current(); !ok || got != 0 {
		t.Fatal("Current should return index 0 after First")
	}
	if got, ok := c.Next(); !ok || got != 1 {
		t.Fatal("Next should select index 1")
	}
	if got, ok := c.Current(); !ok || got != 1 {
		t.Fatal("Current should return index 1 after Next")
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
