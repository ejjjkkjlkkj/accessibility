#ifndef ACCESSIBILITY_EVENT_H
#define ACCESSIBILITY_EVENT_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define ACCESSIBILITY_EVENT_ABI_VERSION 1u
#define ACCESSIBILITY_NAME_CAPACITY 128u

typedef enum accessibility_role {
    ACCESSIBILITY_ROLE_UNKNOWN = 0,
    ACCESSIBILITY_ROLE_APPLICATION = 1,
    ACCESSIBILITY_ROLE_WINDOW = 2,
    ACCESSIBILITY_ROLE_GROUP = 3,
    ACCESSIBILITY_ROLE_TEXT = 4,
    ACCESSIBILITY_ROLE_BUTTON = 5,
    ACCESSIBILITY_ROLE_CHECKBOX = 6,
    ACCESSIBILITY_ROLE_RADIO = 7,
    ACCESSIBILITY_ROLE_TEXTBOX = 8,
    ACCESSIBILITY_ROLE_LINK = 9,
    ACCESSIBILITY_ROLE_HEADING = 10,
    ACCESSIBILITY_ROLE_LIST = 11,
    ACCESSIBILITY_ROLE_LIST_ITEM = 12,
    ACCESSIBILITY_ROLE_TABLE = 13,
    ACCESSIBILITY_ROLE_ROW = 14,
    ACCESSIBILITY_ROLE_CELL = 15,
    ACCESSIBILITY_ROLE_MENU = 16,
    ACCESSIBILITY_ROLE_MENU_ITEM = 17
} accessibility_role;

typedef struct accessibility_event {
    uint32_t abi_version;
    uint32_t role;
    uint64_t state_flags;
    uint64_t source_id;
    char name[ACCESSIBILITY_NAME_CAPACITY];
} accessibility_event;

/* Initializes a deterministic, versioned event structure. Returns 0 on success. */
int accessibility_event_init(accessibility_event *event,
                             accessibility_role role,
                             uint64_t source_id,
                             const char *name);

#ifdef __cplusplus
}
#endif

#endif
