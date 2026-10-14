module accessibility_event;

enum Role : ubyte {
    unknown = 0,
    window = 1,
    button = 2,
    textField = 3,
    heading = 4
}

struct AccessibilityEvent {
    uint abiVersion = 1;
    Role role = Role.unknown;
    string name;
}

bool isValid(const ref AccessibilityEvent event) {
    return event.abiVersion == 1 &&
        event.role >= Role.unknown &&
        event.role <= Role.heading &&
        event.name.length <= 256;
}

unittest {
    auto valid = AccessibilityEvent(1, Role.button, "Save");
    assert(isValid(valid));
    auto invalid = AccessibilityEvent(2, Role.button, "Save");
    assert(!isValid(invalid));
    auto tooLongName = new char[257];
    tooLongName[] = 'x';
    auto tooLong = AccessibilityEvent(1, Role.button, cast(string) tooLongName);
    assert(!isValid(tooLong));
}
