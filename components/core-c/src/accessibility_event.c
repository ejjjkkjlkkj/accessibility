#include "accessibility_event.h"

#include <string.h>

int accessibility_state_flags_valid(uint64_t state_flags) {
    if ((state_flags & ~ACCESSIBILITY_STATE_KNOWN_MASK) != 0u) return 0;
    if ((state_flags & ACCESSIBILITY_STATE_ENABLED) != 0u &&
        (state_flags & ACCESSIBILITY_STATE_DISABLED) != 0u) return 0;
    if ((state_flags & ACCESSIBILITY_STATE_FOCUSED) != 0u &&
        ((state_flags & ACCESSIBILITY_STATE_FOCUSABLE) == 0u ||
         (state_flags & ACCESSIBILITY_STATE_ENABLED) == 0u ||
         (state_flags & ACCESSIBILITY_STATE_DISABLED) != 0u)) return 0;
    return 1;
}

int accessibility_event_init(accessibility_event *event,
                             accessibility_role role,
                             uint64_t source_id,
                             const char *name) {
    size_t i = 0;

    if (event == NULL || name == NULL) {
        return -1;
    }
    if ((int)role < ACCESSIBILITY_ROLE_UNKNOWN || (int)role > ACCESSIBILITY_ROLE_MENU_ITEM) {
        return -2;
    }
    /* Bound the scan before copying; do not call unbounded strlen on external input. */
    while (i < ACCESSIBILITY_NAME_CAPACITY && name[i] != '\0') {
        ++i;
    }
    if (i >= ACCESSIBILITY_NAME_CAPACITY) {
        return -3;
    }

    memset(event, 0, sizeof(*event));
    event->abi_version = ACCESSIBILITY_EVENT_ABI_VERSION;
    event->role = (uint32_t)role;
    event->source_id = source_id;
    memcpy(event->name, name, i);
    event->name[i] = '\0';
    return 0;
}
