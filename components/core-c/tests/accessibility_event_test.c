#include "accessibility_event.h"

#include <assert.h>
#include <string.h>

int main(void) {
    accessibility_event event;

    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_BUTTON, 42u, "Continue") == 0);
    assert(event.abi_version == ACCESSIBILITY_EVENT_ABI_VERSION);
    assert(event.role == ACCESSIBILITY_ROLE_BUTTON);
    assert(ACCESSIBILITY_ROLE_BUTTON == 5);
    assert(ACCESSIBILITY_ROLE_MENU_ITEM == 17);
    assert(event.source_id == 42u);
    assert(strcmp(event.name, "Continue") == 0);
    assert(event.state_flags == 0u);
    assert(accessibility_state_flags_valid(0u));
    assert(accessibility_state_flags_valid(ACCESSIBILITY_STATE_FOCUSABLE |
                                           ACCESSIBILITY_STATE_ENABLED |
                                           ACCESSIBILITY_STATE_FOCUSED));
    assert(!accessibility_state_flags_valid(ACCESSIBILITY_STATE_ENABLED |
                                            ACCESSIBILITY_STATE_DISABLED));
    assert(!accessibility_state_flags_valid(ACCESSIBILITY_STATE_FOCUSED));
    assert(!accessibility_state_flags_valid(UINT64_C(1) << 63));
    assert(accessibility_event_set_state_flags(&event,
        ACCESSIBILITY_STATE_FOCUSABLE | ACCESSIBILITY_STATE_ENABLED |
        ACCESSIBILITY_STATE_FOCUSED) == 0);
    assert(event.state_flags == (ACCESSIBILITY_STATE_FOCUSABLE |
                                 ACCESSIBILITY_STATE_ENABLED |
                                 ACCESSIBILITY_STATE_FOCUSED));
    uint64_t valid_before_failure = event.state_flags;
    assert(accessibility_event_set_state_flags(&event,
        ACCESSIBILITY_STATE_ENABLED | ACCESSIBILITY_STATE_DISABLED) == -2);
    assert(event.state_flags == valid_before_failure);
    assert(accessibility_event_set_state_flags(NULL, 0u) == -1);

    assert(accessibility_event_init(NULL, ACCESSIBILITY_ROLE_BUTTON, 1u, "x") != 0);
    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_BUTTON, 1u, NULL) != 0);
    assert(accessibility_event_init(&event, (accessibility_role)99, 1u, "x") != 0);
    char long_name[ACCESSIBILITY_NAME_CAPACITY + 1u];
    memset(long_name, 'x', sizeof(long_name));
    long_name[sizeof(long_name) - 1u] = '\0';
    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_BUTTON, 1u, long_name) == -3);

    char max_name[ACCESSIBILITY_NAME_CAPACITY];
    memset(max_name, 'a', sizeof(max_name));
    max_name[sizeof(max_name) - 1u] = '\0';
    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_BUTTON, 9u, max_name) == 0);
    assert(strlen(event.name) == ACCESSIBILITY_NAME_CAPACITY - 1u);
    assert(event.name[ACCESSIBILITY_NAME_CAPACITY - 1u] == '\0');

    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_BUTTON, 10u, "caf\xC3\xA9") == 0);
    const char *invalid_utf8[] = {
        "\xC0\xAF",             /* overlong two-byte sequence */
        "\xED\xA0\x80",       /* UTF-16 surrogate */
        "\xF4\x90\x80\x80", /* above U+10FFFF */
        "\xE2\x82"              /* truncated sequence */
    };
    for (size_t index = 0; index < sizeof(invalid_utf8) / sizeof(invalid_utf8[0]); ++index) {
        assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_BUTTON, 11u, invalid_utf8[index]) == -4);
    }
    return 0;
}
