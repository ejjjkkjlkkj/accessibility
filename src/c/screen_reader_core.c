#include <stddef.h>
#include <stdio.h>
#include <string.h>

/*
 * Portable semantic core only. Native UIA/AT-SPI adapters and audio are not
 * wired. Queue payloads own bounded copies; callers need not retain strings.
 */
#define SR_TEXT_CAP 96
#define SR_QUEUE_CAP 16

typedef struct {
    const char *role;
    const char *name;
    const char *value;
    int focused;
    int disabled;
} sr_node;

typedef enum {
    SR_EVENT_FOCUS = 1,
    SR_EVENT_VALUE_CHANGED = 2,
    SR_EVENT_STATE_CHANGED = 3
} sr_event_type;

typedef struct {
    sr_event_type type;
    char role[SR_TEXT_CAP];
    char name[SR_TEXT_CAP];
    char value[SR_TEXT_CAP];
    int focused;
    int disabled;
} sr_event;

typedef struct {
    sr_event items[SR_QUEUE_CAP];
    size_t head;
    size_t count;
} sr_event_queue;

/* Return -1 for invalid arguments, 1 for output truncation, otherwise length. */
int sr_announce(const sr_node *node, char *out, size_t capacity) {
    if (!node || !out || capacity == 0) return -1;
    out[0] = '\0';
    if (!node->focused) return 0;
    int n = snprintf(out, capacity, "%s%s%s%s%s%s",
        node->role && node->role[0] ? node->role : "élément",
        node->name && node->name[0] ? ": " : "",
        node->name && node->name[0] ? node->name : "",
        node->value && node->value[0] ? ", " : "",
        node->value && node->value[0] ? node->value : "",
        node->disabled ? ", désactivé" : "");
    if (n < 0) return -1;
    return (size_t)n >= capacity ? 1 : n;
}

static int sr_copy_text(char *dst, size_t capacity, const char *src) {
    if (!dst || capacity == 0) return -1;
    if (!src) src = "";
    size_t length = strlen(src);
    if (length >= capacity) return -2;
    memcpy(dst, src, length + 1);
    return 0;
}

/* 0=queued, -1=invalid input/type, -2=text too long, -3=queue full. */
int sr_queue_push(sr_event_queue *queue, sr_event_type type, const sr_node *node) {
    if (!queue || !node) return -1;
    if (type != SR_EVENT_FOCUS && type != SR_EVENT_VALUE_CHANGED &&
        type != SR_EVENT_STATE_CHANGED) return -1;
    if (queue->count >= SR_QUEUE_CAP) return -3;

    sr_event event;
    event.type = type;
    event.focused = node->focused != 0;
    event.disabled = node->disabled != 0;
    if (sr_copy_text(event.role, sizeof event.role, node->role) != 0 ||
        sr_copy_text(event.name, sizeof event.name, node->name) != 0 ||
        sr_copy_text(event.value, sizeof event.value, node->value) != 0) return -2;

    size_t tail = (queue->head + queue->count) % SR_QUEUE_CAP;
    queue->items[tail] = event;
    queue->count++;
    return 0;
}

/* 1=event returned, 0=empty, -1=invalid arguments. */
int sr_queue_pop(sr_event_queue *queue, sr_event *out) {
    if (!queue || !out) return -1;
    if (queue->count == 0) return 0;
    *out = queue->items[queue->head];
    queue->head = (queue->head + 1) % SR_QUEUE_CAP;
    queue->count--;
    return 1;
}
