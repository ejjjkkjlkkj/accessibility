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
    ACCESSIBILITY_ROLE_BUTTON = 1,
    ACCESSIBILITY_ROLE_TEXT = 2,
    ACCESSIBILITY_ROLE_CHECKBOX = 3,
    ACCESSIBILITY_ROLE_DIALOG = 4
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
