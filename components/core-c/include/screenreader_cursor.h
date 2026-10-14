#ifndef SCREENREADER_CURSOR_H
#define SCREENREADER_CURSOR_H
#include <stdbool.h>
#include <stdint.h>
typedef struct screenreader_cursor { uint32_t count; uint32_t current; bool has_item; } screenreader_cursor;
void screenreader_cursor_init(screenreader_cursor *cursor, uint32_t count);
bool screenreader_cursor_first(screenreader_cursor *cursor, uint32_t *index);
bool screenreader_cursor_next(screenreader_cursor *cursor, uint32_t *index);
bool screenreader_cursor_previous(screenreader_cursor *cursor, uint32_t *index);
#endif
