package screenreader

import "testing"

func TestCursorNavigationBoundaries(t *testing.T) {
	empty := NewCursor(0)
	if _, ok := empty.First(); ok {
		t.Fatal("empty list must not focus an item")
	}
	if _, ok := empty.Current(); ok {
		t.Fatal("empty list unexpectedly has a current item")
	}
	c := NewCursor(2)
	if _, ok := c.Next(); ok {
		t.Fatal("Next before First must fail")
	}
	if _, ok := c.Previous(); ok {
		t.Fatal("Previous before First must fail")
	}
	if _, ok := c.Current(); ok {
		t.Fatal("failed navigation must not create a current item")
	}
	if got, ok := c.First(); !ok || got != 0 {
		t.Fatal("First should select index 0")
	}
	if got, ok := c.First(); !ok || got != 0 {
		t.Fatal("repeated First should reset to index 0")
	}
	if got, ok := c.Next(); !ok || got != 1 {
		t.Fatal("Next should select index 1")
	}
	if _, ok := c.Next(); ok {
		t.Fatal("Next must stop at end; no implicit wrap")
	}
	if got, ok := c.Current(); !ok || got != 1 {
		t.Fatal("failed Next changed current item")
	}
	if got, ok := c.Previous(); !ok || got != 0 {
		t.Fatal("Previous should return to index 0")
	}
	if _, ok := c.Previous(); ok {
		t.Fatal("Previous must stop at beginning")
	}
	if got, ok := c.Current(); !ok || got != 0 {
		t.Fatal("failed Previous changed current item")
	}
}
