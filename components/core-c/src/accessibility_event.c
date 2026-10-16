#include "accessibility_event.h"

#include <string.h>

/* Reject malformed UTF-8, overlong encodings, surrogate code points and
 * code points above U+10FFFF before storing a semantic name. */
static int accessibility_name_is_utf8(const unsigned char *text, size_t length) {
    size_t index = 0;

    while (index < length) {
        unsigned char first = text[index];
        if (first <= 0x7Fu) {
            ++index;
            continue;
        }
        if (first >= 0xC2u && first <= 0xDFu) {
            if (index + 1u >= length ||
                (text[index + 1u] & 0xC0u) != 0x80u) {
                return 0;
            }
            index += 2u;
            continue;
        }
        if (first >= 0xE0u && first <= 0xEFu) {
            unsigned char second;
            unsigned char third;
            if (index + 2u >= length) {
                return 0;
            }
            second = text[index + 1u];
            third = text[index + 2u];
            if ((second & 0xC0u) != 0x80u || (third & 0xC0u) != 0x80u) {
                return 0;
            }
            if ((first == 0xE0u && second < 0xA0u) ||
                (first == 0xEDu && second >= 0xA0u)) {
                return 0;
            }
            index += 3u;
            continue;
        }
        if (first >= 0xF0u && first <= 0xF4u) {
            unsigned char second;
            unsigned char third;
            unsigned char fourth;
            if (index + 3u >= length) {
                return 0;
            }
            second = text[index + 1u];
            third = text[index + 2u];
            fourth = text[index + 3u];
            if ((second & 0xC0u) != 0x80u ||
                (third & 0xC0u) != 0x80u ||
                (fourth & 0xC0u) != 0x80u) {
                return 0;
            }
            if ((first == 0xF0u && second < 0x90u) ||
                (first == 0xF4u && second >= 0x90u)) {
                return 0;
            }
            index += 4u;
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
    if (strlen(name) >= ACCESSIBILITY_NAME_CAPACITY) {
        return -3;
    }
    if (!accessibility_name_is_utf8((const unsigned char *)name, strlen(name))) {
        return -4;
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
