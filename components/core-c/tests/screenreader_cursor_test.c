#include "screenreader_cursor.h"

#include <assert.h>

int main(void) {
    screenreader_cursor cursor;
    uint32_t index = 99;

    screenreader_cursor_init(&cursor, 0);
    assert(!screenreader_cursor_first(&cursor, &index));
    assert(!cursor.has_item);
    assert(!screenreader_cursor_next(&cursor, &index));
    assert(!screenreader_cursor_previous(&cursor, &index));

    screenreader_cursor_init(&cursor, 2);
    assert(!screenreader_cursor_first(&cursor, 0));
    assert(!screenreader_cursor_next(&cursor, &index));
    assert(!screenreader_cursor_previous(&cursor, &index));
    assert(!cursor.has_item);
    assert(screenreader_cursor_first(&cursor, &index) && index == 0);
    assert(screenreader_cursor_next(&cursor, &index) && index == 1);
    assert(!screenreader_cursor_next(&cursor, &index));
    assert(index == 1);
    assert(screenreader_cursor_previous(&cursor, &index) && index == 0);
    assert(!screenreader_cursor_previous(&cursor, &index));
    assert(index == 0);
    assert(!screenreader_cursor_first(0, &index));
    assert(!screenreader_cursor_first(&cursor, 0));
    return 0;
}
