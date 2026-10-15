module accessibility_event;

import std.array : replicate;
import std.utf : validate, UTFException;

enum Role : ubyte {
    unknown = 0,
    application = 1,
    window = 2,
    group = 3,
    text = 4,
    button = 5,
    checkbox = 6,
    radio = 7,
    textbox = 8,
    link = 9,
    heading = 10,
    list = 11,
    listItem = 12,
    table = 13,
    row = 14,
    cell = 15,
    menu = 16,
    menuItem = 17
}

enum ulong STATE_FOCUSED = 1UL << 0;
enum ulong STATE_FOCUSABLE = 1UL << 1;
enum ulong STATE_ENABLED = 1UL << 2;
enum ulong STATE_DISABLED = 1UL << 3;
enum ulong STATE_KNOWN_MASK = 0x0FUL;

bool isValidStateFlags(ulong flags) {
    if ((flags & ~STATE_KNOWN_MASK) != 0) return false;
    if ((flags & STATE_ENABLED) != 0 && (flags & STATE_DISABLED) != 0) return false;
    if ((flags & STATE_FOCUSED) != 0 &&
        ((flags & STATE_FOCUSABLE) == 0 ||
         (flags & STATE_ENABLED) == 0 ||
         (flags & STATE_DISABLED) != 0)) return false;
    return true;
}

bool isValidName(string name) {
    if (name.length > 4096) return false;
    foreach (char byteValue; name) {
        if (byteValue == '\0') return false;
    }
    try {
        validate(name);
    } catch (UTFException) {
        return false;
    }
    return true;
}

struct AccessibilityEvent {
    uint abiVersion = 1;
    Role role = Role.unknown;
    string name;
    ulong stateFlags = 0;
}

bool isValid(const ref AccessibilityEvent event) {
    return event.abiVersion == 1 &&
        event.role <= Role.menuItem &&
        isValidName(event.name) &&
        isValidStateFlags(event.stateFlags);
}

unittest {
    assert(cast(ubyte)Role.button == 5);
    assert(cast(ubyte)Role.menuItem == 17);
    auto valid = AccessibilityEvent(1, Role.button, "Save");
    assert(isValid(valid));
    auto invalid = AccessibilityEvent(2, Role.button, "Save");
    assert(!isValid(invalid));
    auto tooLong = AccessibilityEvent(1, Role.button, cast(string) replicate("x", 4097));
    assert(!isValid(tooLong));
    auto focused = AccessibilityEvent(1, Role.button, "Save", STATE_FOCUSABLE | STATE_ENABLED | STATE_FOCUSED);
    assert(isValid(focused));
    auto badFlags = AccessibilityEvent(1, Role.button, "Save", STATE_ENABLED | STATE_DISABLED);
    assert(!isValid(badFlags));
    assert(!isValidStateFlags(1UL << 63));
    auto nulName = AccessibilityEvent(1, Role.button, "bad\\0name");
    assert(!isValid(nulName));
    auto invalidUtf8 = AccessibilityEvent(1, Role.button, "\\xC0\\xAF");
    assert(!isValid(invalidUtf8));
}
