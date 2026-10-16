module screenreader;

/// Normalized focus metadata; native platform adapters are not implemented yet.
struct Node {
    string role;
    string name;
    string value;
    bool focused;
    bool disabled;
}

string announce(const Node n) @safe {
    if (!n.focused) return "";
    string result = n.role.length ? n.role : "élément";
    if (n.name.length) result ~= ": " ~ n.name;
    if (n.value.length) result ~= ", " ~ n.value;
    if (n.disabled) result ~= ", désactivé";
    return result;
}

enum EventKind : ubyte { focus = 1, valueChanged = 2, stateChanged = 3 }
enum QueueCapacity = 16;
enum TextCapacity = 96;
struct Event {
    EventKind kind;
    char[TextCapacity] role;
    char[TextCapacity] name;
    char[TextCapacity] value;
    size_t roleLength, nameLength, valueLength;
    bool focused, disabled;
}
struct EventQueue {
    private Event[QueueCapacity] items;
    private size_t head, count;
    @property size_t length() const @safe { return count; }
    bool push(EventKind kind, const Node n) @safe {
        if (count == QueueCapacity || n.role.length >= TextCapacity ||
            n.name.length >= TextCapacity || n.value.length >= TextCapacity) return false;
        auto tail = (head + count) % QueueCapacity;
        auto e = Event.init;
        e.kind = kind;
        e.roleLength = n.role.length; e.nameLength = n.name.length; e.valueLength = n.value.length;
        foreach (i, c; n.role) e.role[i] = c;
        foreach (i, c; n.name) e.name[i] = c;
        foreach (i, c; n.value) e.value[i] = c;
        e.focused = n.focused; e.disabled = n.disabled;
        items[tail] = e; count++;
        return true;
    }
    bool pop(out Event e) @safe {
        if (count == 0) { e = Event.init; return false; }
        e = items[head]; head = (head + 1) % QueueCapacity; count--;
        return true;
    }
}
