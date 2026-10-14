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
    assert(sr_announce(&focused, out, 0) == -1);

    /* Small output buffers are explicitly reported as truncated. */
    char tiny[4];
    assert(sr_announce(&focused, tiny, sizeof tiny) == 1);
    assert(tiny[sizeof tiny - 1] == '\0');

    /* Event queue copies data, preserves FIFO order, and wraps safely. */
    sr_event_queue queue = {0};
    sr_event event;
    assert(sr_queue_pop(&queue, &event) == 0);
    assert(sr_queue_push(&queue, SR_EVENT_FOCUS, &focused) == 0);
    focused.name = "Changed after enqueue";
    assert(sr_queue_push(&queue, SR_EVENT_VALUE_CHANGED, &focused) == 0);
    assert(sr_queue_pop(&queue, &event) == 1);
    assert(event.type == SR_EVENT_FOCUS);
    assert(strcmp(event.name, "Valider") == 0);
    assert(sr_queue_pop(&queue, &event) == 1);
    assert(event.type == SR_EVENT_VALUE_CHANGED);
    assert(strcmp(event.name, "Changed after enqueue") == 0);
    assert(sr_queue_pop(&queue, &event) == 0);

    sr_node too_long = {"bouton", "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", "", 1, 0};
    assert(sr_queue_push(&queue, SR_EVENT_FOCUS, &too_long) == -2);
    assert(sr_queue_push(&queue, (sr_event_type)99, &focused) == -1);
    assert(sr_queue_push(NULL, SR_EVENT_FOCUS, &focused) == -1);
    assert(sr_queue_pop(NULL, &event) == -1);
    assert(sr_queue_pop(&queue, NULL) == -1);

    for (size_t i = 0; i < SR_QUEUE_CAP; i++)
        assert(sr_queue_push(&queue, SR_EVENT_FOCUS, &focused) == 0);
    assert(sr_queue_push(&queue, SR_EVENT_FOCUS, &focused) == -3);
    for (size_t i = 0; i < SR_QUEUE_CAP; i++)
        assert(sr_queue_pop(&queue, &event) == 1);
    assert(sr_queue_pop(&queue, &event) == 0);

    /* Exercise ring wrap-around after a full drain. */
    assert(sr_queue_push(&queue, SR_EVENT_STATE_CHANGED, &disabled) == 0);
    assert(sr_queue_pop(&queue, &event) == 1);
    assert(event.type == SR_EVENT_STATE_CHANGED);
    assert(strcmp(event.name, "Wi-Fi") == 0);
    sr_node fallback = {NULL, "", "42", 1, 0};
    assert(sr_announce(&fallback, out, sizeof out) > 0);
    assert(strcmp(out, "élément, 42") == 0);

    return 0;
}
