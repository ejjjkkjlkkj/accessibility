#include "screenreader_cursor.h"
void screenreader_cursor_init(screenreader_cursor *c, uint32_t count) {
 if (c == 0) return; c->count = count; c->current = 0; c->has_item = false;
}
bool screenreader_cursor_first(screenreader_cursor *c, uint32_t *index) {
 if (c == 0 || index == 0 || c->count == 0) return false;
 c->current = 0; c->has_item = true; *index = 0; return true;
}
bool screenreader_cursor_next(screenreader_cursor *c, uint32_t *index) {
 if (c == 0 || index == 0 || !c->has_item || c->current >= c->count - 1) return false;
 *index = ++c->current; return true;
}
bool screenreader_cursor_previous(screenreader_cursor *c, uint32_t *index) {
 if (c == 0 || index == 0 || !c->has_item || c->current == 0) return false;
 *index = --c->current; return true;
}
