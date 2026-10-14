#include <assert.h>
#include <string.h>
#include "screen_reader_core.c"

int main(void) {
    char out[256];
    sr_node focused = {"bouton", "Valider", "", 1, 0};
    assert(sr_announce(&focused, out, sizeof out) > 0);
    assert(strcmp(out, "bouton: Valider") == 0);

    sr_node unfocused = {"bouton", "Valider", "", 0, 0};
    assert(sr_announce(&unfocused, out, sizeof out) == 0);
    assert(strcmp(out, "") == 0);

    sr_node disabled = {"case à cocher", "Wi-Fi", "désactivé", 1, 1};
    assert(sr_announce(&disabled, out, sizeof out) > 0);
    assert(strstr(out, "désactivé") != NULL);

    assert(sr_announce(NULL, out, sizeof out) == -1);
    assert(sr_announce(&focused, NULL, sizeof out) == -1);
    return 0;
}
