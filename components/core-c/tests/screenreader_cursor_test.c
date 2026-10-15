#include "screenreader_cursor.h"
#include <assert.h>
int main(void) {
 screenreader_cursor c; uint32_t i = 99;
 screenreader_cursor_init(&c, 0);
 assert(!screenreader_cursor_first(&c, &i) && i == 99);
 assert(!screenreader_cursor_next(&c, &i) && i == 99);
 assert(!screenreader_cursor_previous(&c, &i) && i == 99);
 assert(!c.has_item && c.current == 0);
 screenreader_cursor_init(&c, 2);
 assert(!screenreader_cursor_next(&c, &i) && i == 99);
 assert(!screenreader_cursor_previous(&c, &i) && i == 99);
 assert(!c.has_item && c.current == 0);
 assert(screenreader_cursor_first(&c, &i) && i == 0);
 assert(screenreader_cursor_first(&c, &i) && i == 0);
 assert(screenreader_cursor_next(&c, &i) && i == 1);
 i = 77;
 assert(!screenreader_cursor_next(&c, &i) && i == 77);
 assert(c.has_item && c.current == 1);
 assert(screenreader_cursor_previous(&c, &i) && i == 0);
 i = 88;
 assert(!screenreader_cursor_previous(&c, &i) && i == 88);
 assert(c.has_item && c.current == 0);
 assert(!screenreader_cursor_first(0, &i));
 return 0;
}
