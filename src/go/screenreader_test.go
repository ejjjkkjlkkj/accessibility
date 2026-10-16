package screenreader

import "testing"

func TestAnnounceFocusedNode(t *testing.T) {
	got := Announce(Node{Role: "bouton", Name: "Valider", Focused: true})
	if got != "bouton: Valider" {
		t.Fatalf("got %q", got)
	}
}

func TestAnnounceUnfocusedNode(t *testing.T) {
	if got := Announce(Node{Role: "bouton", Name: "Valider"}); got != "" {
		t.Fatalf("unfocused node announced: %q", got)
	}
}

func TestAnnounceValueAndDisabled(t *testing.T) {
	got := Announce(Node{Role: "champ", Name: "Volume", Value: "80 %", Focused: true, Disabled: true})
	if got != "champ: Volume, 80 %, désactivé" {
		t.Fatalf("got %q", got)
	}
}

func TestQueueFIFOCapacityAndBounds(t *testing.T) {
	var q EventQueue
	first := Node{Role: "bouton", Name: "Valider", Focused: true}
	if !q.Push(FocusEvent, first) {
		t.Fatal("push failed")
	}
	first.Name = "modified"
	got, ok := q.Pop()
	if !ok || got.Name != "Valider" || got.Kind != FocusEvent {
		t.Fatalf("unexpected event: %#v", got)
	}
	long := Node{Name: string(make([]byte, eventTextLimit))}
	if q.Push(FocusEvent, long) {
		t.Fatal("oversized text accepted")
	}
	for i := 0; i < eventQueueLimit; i++ {
		if !q.Push(StateChangedEvent, Node{Name: "ok"}) {
			t.Fatalf("push %d failed", i)
		}
	}
	if q.Push(FocusEvent, first) {
		t.Fatal("queue overflow accepted")
	}
	for i := 0; i < eventQueueLimit; i++ {
		if _, ok := q.Pop(); !ok {
			t.Fatalf("pop %d failed", i)
		}
	}
	if _, ok := q.Pop(); ok {
		t.Fatal("empty queue returned event")
	}
	if q.Push(EventKind(99), first) {
		t.Fatal("invalid event kind accepted")
	}
}

func TestQueueRingWraparoundPreservesFIFO(t *testing.T) {
	var q EventQueue
	a := Node{Role: "bouton", Name: "A", Focused: true}
	b := Node{Role: "bouton", Name: "B", Focused: true}
	for i := 0; i < 12; i++ {
		if !q.Push(FocusEvent, a) {
			t.Fatal("initial push failed")
		}
	}
	for i := 0; i < 8; i++ {
		got, ok := q.Pop()
		if !ok || got.Name != "A" {
			t.Fatalf("initial pop %d: %#v, %v", i, got, ok)
		}
	}
	for i := 0; i < 12; i++ {
		if !q.Push(FocusEvent, b) {
			t.Fatalf("wrapped push %d failed", i)
		}
	}
	if q.Len() != eventQueueLimit {
		t.Fatalf("len=%d", q.Len())
	}
	for i := 0; i < 4; i++ {
		got, ok := q.Pop()
		if !ok || got.Name != "A" {
			t.Fatalf("remaining A %d: %#v, %v", i, got, ok)
		}
	}
	for i := 0; i < 12; i++ {
		got, ok := q.Pop()
		if !ok || got.Name != "B" {
			t.Fatalf("wrapped B %d: %#v, %v", i, got, ok)
		}
	}
	if _, ok := q.Pop(); ok {
		t.Fatal("queue should be empty")
	}
}

func TestAnnounceFallbackRoleAndEmptyName(t *testing.T) {
	got := Announce(Node{Value: "42", Focused: true})
	if got != "élément, 42" {
		t.Fatalf("got %q", got)
	}
}
