module accessibility_event;

import std.array : replicate;

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

struct AccessibilityEvent {
    uint abiVersion = 1;
    Role role = Role.unknown;
    string name;
}

bool isValid(const ref AccessibilityEvent event) {
    return event.abiVersion == 1 &&
        event.role >= Role.unknown &&
        event.role <= Role.menuItem &&
        event.name.length <= 256;
}

unittest {
    auto valid = AccessibilityEvent(1, Role.button, "Save");
    assert(isValid(valid));
    auto invalid = AccessibilityEvent(2, Role.button, "Save");
    assert(!isValid(invalid));
    auto tooLong = AccessibilityEvent(1, Role.button, cast(string) replicate("x", 257));
    assert(!isValid(tooLong));
}
