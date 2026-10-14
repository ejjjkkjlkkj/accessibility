#include <stddef.h>
#include <stdio.h>
#include <string.h>

/* Portable core only: no OS accessibility API or audio driver is wired yet. */
typedef struct {
    const char *role;
    const char *name;
    const char *value;
    int focused;
    int disabled;
} sr_node;

int sr_announce(const sr_node *node, char *out, size_t capacity) {
    if (!node || !out || capacity == 0) return -1;
    out[0] = '\0';
    if (!node->focused) return 0;
    int n = snprintf(out, capacity, "%s%s%s%s%s%s",
        node->role ? node->role : "élément",
        node->name && node->name[0] ? ": " : "",
        node->name && node->name[0] ? node->name : "",
        node->value && node->value[0] ? ", " : "",
        node->value && node->value[0] ? node->value : "",
        node->disabled ? ", désactivé" : "");
    if (n < 0) return -1;
    return (size_t)n >= capacity ? 1 : n;
}
