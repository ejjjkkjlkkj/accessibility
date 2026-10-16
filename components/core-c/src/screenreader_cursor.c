#include "screenreader_cursor.h"

void screenreader_cursor_init(screenreader_cursor *cursor, uint32_t count) {
    if (cursor == 0) {
        return;
    }
    cursor->count = count;
    cursor->current = 0;
    cursor->has_item = false;
}

bool screenreader_cursor_first(screenreader_cursor *cursor, uint32_t *index) {
    if (cursor == 0 || index == 0 || cursor->count == 0) {
        if (cursor != 0 && cursor->count == 0) {
            cursor->current = 0;
            cursor->has_item = false;
        }
        return false;
    }
    cursor->current = 0;
    cursor->has_item = true;
    *index = 0;
    return true;
}

bool screenreader_cursor_next(screenreader_cursor *cursor, uint32_t *index) {
    if (cursor == 0 || index == 0 || !cursor->has_item ||
        cursor->count == 0 || cursor->current >= cursor->count - 1u) {
        return false;
    }
    ++cursor->current;
    *index = cursor->current;
    return true;
}

bool screenreader_cursor_previous(screenreader_cursor *cursor, uint32_t *index) {
    if (cursor == 0 || index == 0 || !cursor->has_item || cursor->current == 0) {
        return false;
    }
    --cursor->current;
    *index = cursor->current;
    return true;
}
