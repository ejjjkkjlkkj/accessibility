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

    assert(accessibility_event_init(NULL, ACCESSIBILITY_ROLE_BUTTON, 1u, "x") != 0);
    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_BUTTON, 1u, NULL) != 0);
    assert(accessibility_event_init(&event, (accessibility_role)99, 1u, "x") == -2);
    assert(accessibility_event_init(&event, (accessibility_role)-1, 1u, "x") == -2);
    const char invalid_utf8[] = { (char)0xC0, (char)0xAF, '\0' };
    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_BUTTON, 1u, invalid_utf8) == -4);
    const char truncated_two[] = { (char)0xC2, '\0' };
    const char truncated_three[] = { (char)0xE2, (char)0x82, '\0' };
    const char truncated_four[] = { (char)0xF0, (char)0x9F, (char)0x92, '\0' };
    const char surrogate_utf8[] = { (char)0xED, (char)0xA0, (char)0x80, '\0' };
    const char too_large_codepoint[] = { (char)0xF4, (char)0x90, (char)0x80, (char)0x80, '\0' };
    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_BUTTON, 1u, truncated_two) == -4);
    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_BUTTON, 1u, truncated_three) == -4);
    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_BUTTON, 1u, truncated_four) == -4);
    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_BUTTON, 1u, surrogate_utf8) == -4);
    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_BUTTON, 1u, too_large_codepoint) == -4);
    const char valid_utf8[] = "Accès — 中文";
    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_BUTTON, 1u, valid_utf8) == 0);
    assert(strcmp(event.name, valid_utf8) == 0);
    char long_name[ACCESSIBILITY_NAME_CAPACITY + 1u];
    memset(long_name, 'x', sizeof(long_name));
    long_name[sizeof(long_name) - 1u] = '\0';
    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_BUTTON, 1u, long_name) == -3);
    return 0;
}
