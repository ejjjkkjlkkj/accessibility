package screenreader

const eventTextLimit = 96
const eventQueueLimit = 16

// Node is normalized focus metadata. Native accessibility API adapters are separate.
type Node struct {
	Role, Name, Value	string
	Focused, Disabled	bool
}

func Announce(n Node) string {
	if !n.Focused {
		return ""
	}
	role := n.Role
	if role == "" {
		role = "élément"
	}
	text := role
	if n.Name != "" {
		text += ": " + n.Name
	}
	if n.Value != "" {
		text += ", " + n.Value
	}
	if n.Disabled {
		text += ", désactivé"
	}
	return text
}

type EventKind uint8

const (
	FocusEvent EventKind = iota + 1
	ValueChangedEvent
	StateChangedEvent
)

type Event struct {
	Kind EventKind
	Role, Name, Value string
	Focused, Disabled bool
}

type EventQueue struct {
	items [eventQueueLimit]Event
	head, count int
}

func (q *EventQueue) Push(kind EventKind, n Node) bool {
	if q == nil || q.count == eventQueueLimit || kind < FocusEvent || kind > StateChangedEvent {
		return false
	}
	if len(n.Role) >= eventTextLimit || len(n.Name) >= eventTextLimit || len(n.Value) >= eventTextLimit {
		return false
	}
	tail := (q.head + q.count) % eventQueueLimit
	q.items[tail] = Event{kind, n.Role, n.Name, n.Value, n.Focused, n.Disabled}
	q.count++
	return true
}

func (q *EventQueue) Pop() (Event, bool) {
	if q == nil || q.count == 0 {
		return Event{}, false
	}
	event := q.items[q.head]
	q.items[q.head] = Event{}
	q.head = (q.head + 1) % eventQueueLimit
	q.count--
	return event, true
}

func (q *EventQueue) Len() int {
	if q == nil {
		return 0
	}
	return q.count
}
