#include "accessibility_event.h"

#include <assert.h>
#include <string.h>

int main(void) {
    accessibility_event event;

    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_BUTTON, 42u, "Continue") == 0);
    assert(event.abi_version == ACCESSIBILITY_EVENT_ABI_VERSION);
    assert(event.role == ACCESSIBILITY_ROLE_BUTTON);
    assert(event.source_id == 42u);
    assert(strcmp(event.name, "Continue") == 0);

    assert(accessibility_event_init(NULL, ACCESSIBILITY_ROLE_BUTTON, 1u, "x") != 0);
    assert(accessibility_event_init(&event, ACCESSIBILITY_ROLE_BUTTON, 1u, NULL) != 0);
    assert(accessibility_event_init(&event, (accessibility_role)99, 1u, "x") != 0);
    return 0;
}
