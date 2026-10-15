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

int accessibility_event_set_state_flags(accessibility_event *event, uint64_t state_flags) {
    if (event == NULL) return -1;
    if (!accessibility_state_flags_valid(state_flags)) return -2;
    event->state_flags = state_flags;
    return 0;
}

static int accessibility_utf8_valid(const unsigned char *text, size_t length) {
    size_t i = 0;
    while (i < length) {
        unsigned char first = text[i];
        unsigned char second;
        if (first <= 0x7Fu) {
            ++i;
            continue;
        }
        if (first >= 0xC2u && first <= 0xDFu) {
            if (i + 1u >= length) return 0;
            second = text[i + 1u];
            if (second < 0x80u || second > 0xBFu) return 0;
            i += 2u;
            continue;
        }
        if (first >= 0xE0u && first <= 0xEFu) {
            unsigned char third;
            if (i + 2u >= length) return 0;
            second = text[i + 1u];
            third = text[i + 2u];
            if (third < 0x80u || third > 0xBFu) return 0;
            if (first == 0xE0u) {
                if (second < 0xA0u || second > 0xBFu) return 0;
            } else if (first == 0xEDu) {
                if (second < 0x80u || second > 0x9Fu) return 0;
            } else if (second < 0x80u || second > 0xBFu) {
                return 0;
            }
            i += 3u;
            continue;
        }
        if (first >= 0xF0u && first <= 0xF4u) {
            unsigned char third, fourth;
            if (i + 3u >= length) return 0;
            second = text[i + 1u];
            third = text[i + 2u];
            fourth = text[i + 3u];
            if (third < 0x80u || third > 0xBFu ||
                fourth < 0x80u || fourth > 0xBFu) return 0;
            if (first == 0xF0u) {
                if (second < 0x90u || second > 0xBFu) return 0;
            } else if (first == 0xF4u) {
                if (second < 0x80u || second > 0x8Fu) return 0;
            } else if (second < 0x80u || second > 0xBFu) {
                return 0;
            }
            i += 4u;
            continue;
        }
        return 0;
    }
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
    if (!accessibility_utf8_valid((const unsigned char *)name, i)) {
        return -4;
    }

    memset(event, 0, sizeof(*event));
    event->abi_version = ACCESSIBILITY_EVENT_ABI_VERSION;
    event->role = (uint32_t)role;
    event->source_id = source_id;
    memcpy(event->name, name, i);
    event->name[i] = '\0';
    return 0;
}
