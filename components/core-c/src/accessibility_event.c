#include "accessibility_event.h"

#include <string.h>

int accessibility_event_init(accessibility_event *event,
                             accessibility_role role,
                             uint64_t source_id,
                             const char *name) {
    size_t i = 0;

    if (event == NULL || name == NULL) {
        return -1;
    }
    if (role < ACCESSIBILITY_ROLE_UNKNOWN || role > ACCESSIBILITY_ROLE_DIALOG) {
        return -2;
    }

    memset(event, 0, sizeof(*event));
    event->abi_version = ACCESSIBILITY_EVENT_ABI_VERSION;
    event->role = (uint32_t)role;
    event->source_id = source_id;

    while (i < ACCESSIBILITY_NAME_CAPACITY - 1u && name[i] != '\0') {
        event->name[i] = name[i];
        ++i;
    }
    event->name[i] = '\0';
    return 0;
}
