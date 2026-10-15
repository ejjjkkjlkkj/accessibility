#include "accessibility_event.h"

#include <assert.h>
#include <string.h>

int main(void) {
    accessibility_event event;
    const char french_name[] = "Élément accessible";
    const char invalid_overlong[] = {(char)0xc0, (char)0xaf, '\0'};
    const char invalid_surrogate[] = {(char)0xed, (char)0xa0, (char)0x80, '\0'};
    const char invalid_truncated[] = {(char)0xe2, (char)0x82, '\0'};

    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_BUTTON, 42u, "Continue") == 0);
    assert(event.abi_version == ACCESSIBILITY_EVENT_ABI_VERSION);
    assert(event.role == ACCESSIBILITY_ROLE_BUTTON);
    assert(ACCESSIBILITY_ROLE_BUTTON == 5);
    assert(ACCESSIBILITY_ROLE_MENU_ITEM == 17);
    assert(event.source_id == 42u);
    assert(strcmp(event.name, "Continue") == 0);

    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_TEXT, 43u, french_name) == 0);
    assert(strcmp(event.name, french_name) == 0);

    assert(accessibility_event_init(NULL, ACCESSIBILITY_ROLE_BUTTON, 1u, "x") == -1);
    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_BUTTON, 1u, NULL) == -1);
    assert(accessibility_event_init(&event, (accessibility_role)99, 1u, "x") == -2);

    char long_name[ACCESSIBILITY_NAME_CAPACITY + 1u];
    memset(long_name, 'x', sizeof(long_name));
    long_name[sizeof(long_name) - 1u] = '\0';
    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_BUTTON, 1u, long_name) == -3);

    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_TEXT, 1u, invalid_overlong) == -4);
    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_TEXT, 1u, invalid_surrogate) == -4);
    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_TEXT, 1u, invalid_truncated) == -4);
    return 0;
}
