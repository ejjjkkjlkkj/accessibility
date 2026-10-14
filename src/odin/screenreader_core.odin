package screenreader_core

import "core:strings"

AccessibleNode :: struct {
    role: string,
    name: string,
    value: string,
    focused: bool,
    disabled: bool,
}

announce :: proc(node: AccessibleNode) -> string {
    if !node.focused do return ""
    role := node.role
    if role == "" do role = "élément"
    out := strings.clone(role)
    if node.name != "" {
        out = strings.concatenate({out, ": ", node.name})
    }
    if node.value != "" {
        out = strings.concatenate({out, ", ", node.value})
    }
    if node.disabled {
        out = strings.concatenate({out, ", désactivé"})
    }
    return out
}

Event_Kind :: enum { Focus, Value_Changed, State_Changed }
Event :: struct {
    kind: Event_Kind,
    node: AccessibleNode,
}
Event_Queue :: struct {
    items: [16]Event,
    head: int,
    count: int,
}
push :: proc(q: ^Event_Queue, event: Event) -> bool {
    if q.count >= len(q.items) do return false
    index := (q.head + q.count) % len(q.items)
    q.items[index] = event
    q.count += 1
    return true
}
pop :: proc(q: ^Event_Queue) -> (Event, bool) {
    if q.count == 0 do return {}, false
    event := q.items[q.head]
    q.head = (q.head + 1) % len(q.items)
    q.count -= 1
    return event, true
}
