#include "accessibility_event.h"

#include <string.h>

/* Strict UTF-8 validation: reject overlong forms, surrogates, and values above U+10FFFF. */
static int accessibility_name_is_valid_utf8(const unsigned char *name, size_t length) {
    size_t i = 0;

    while (i < length) {
        unsigned char lead = name[i];
        if (lead <= 0x7fu) {
            ++i;
        } else if (lead >= 0xc2u && lead <= 0xdfu) {
            if (i + 1u >= length || (name[i + 1u] & 0xc0u) != 0x80u) {
                return 0;
            }
            i += 2u;
        } else if (lead == 0xe0u) {
            if (i + 2u >= length || name[i + 1u] < 0xa0u || name[i + 1u] > 0xbfu
                || (name[i + 2u] & 0xc0u) != 0x80u) {
                return 0;
            }
            i += 3u;
        } else if ((lead >= 0xe1u && lead <= 0xecu) || (lead >= 0xeeu && lead <= 0xefu)) {
            if (i + 2u >= length || (name[i + 1u] & 0xc0u) != 0x80u
                || (name[i + 2u] & 0xc0u) != 0x80u) {
                return 0;
            }
            i += 3u;
        } else if (lead == 0xedu) {
            /* U+D800..U+DFFF are not Unicode scalar values. */
            if (i + 2u >= length || name[i + 1u] < 0x80u || name[i + 1u] > 0x9fu
                || (name[i + 2u] & 0xc0u) != 0x80u) {
                return 0;
            }
            i += 3u;
        } else if (lead == 0xf0u) {
            if (i + 3u >= length || name[i + 1u] < 0x90u || name[i + 1u] > 0xbfu
                || (name[i + 2u] & 0xc0u) != 0x80u
                || (name[i + 3u] & 0xc0u) != 0x80u) {
                return 0;
            }
            i += 4u;
        } else if (lead >= 0xf1u && lead <= 0xf3u) {
            if (i + 3u >= length || (name[i + 1u] & 0xc0u) != 0x80u
                || (name[i + 2u] & 0xc0u) != 0x80u
                || (name[i + 3u] & 0xc0u) != 0x80u) {
                return 0;
            }
            i += 4u;
        } else if (lead == 0xf4u) {
            if (i + 3u >= length || name[i + 1u] < 0x80u || name[i + 1u] > 0x8fu
                || (name[i + 2u] & 0xc0u) != 0x80u
                || (name[i + 3u] & 0xc0u) != 0x80u) {
                return 0;
            }
            i += 4u;
        } else {
            return 0;
        }
    }

    return 1;
}

int accessibility_event_init(accessibility_event *event,
                             accessibility_role role,
                             uint64_t source_id,
                             const char *name) {
    size_t length = 0;

    if (event == NULL || name == NULL) {
        return -1;
    }
    if ((int)role < ACCESSIBILITY_ROLE_UNKNOWN || (int)role > ACCESSIBILITY_ROLE_MENU_ITEM) {
        return -2;
    }

    /* Bound the scan: never search beyond the fixed-size wire name capacity. */
    while (length < ACCESSIBILITY_NAME_CAPACITY && name[length] != '\0') {
        ++length;
    }
    if (length == ACCESSIBILITY_NAME_CAPACITY) {
        return -3;
    }
    if (!accessibility_name_is_valid_utf8((const unsigned char *)name, length)) {
        return -4;
    }

    memset(event, 0, sizeof(*event));
    event->abi_version = ACCESSIBILITY_EVENT_ABI_VERSION;
    event->role = (uint32_t)role;
    event->source_id = source_id;
    memcpy(event->name, name, length);
    event->name[length] = '\0';
    return 0;
}
