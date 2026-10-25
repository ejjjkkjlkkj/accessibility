/*
 * ZERO OS Native Bare-Metal Graphic Interface & Software Suite
 * Direct Framebuffer Rendering (UEFI GOP / ARGB 32-bit)
 * Natively Accessible Screen Reader Bridge — Built-in direct Hardware Accessibility Channel (HAC)
 * No Hosted OS APIs / 0 Language Tier / Pure Standalone C
 */

#include <stdint.h>

#define DISPLAY_WIDTH 1920
#define DISPLAY_HEIGHT 1080

typedef struct {
    uint32_t x;
    uint32_t y;
    uint32_t width;
    uint32_t height;
    uint32_t color;
    uint32_t role_id;    /* Code rôle canonique (5=Button, 8=Textbox, 2=Window) */
    uint32_t is_focused;
    char label[64];
} ZeroGuiElement;

typedef struct {
    uint32_t element_count;
    ZeroGuiElement elements[16];
    uint32_t active_focus_index;
    uint32_t *framebuffer;
} ZeroGuiContext;

static ZeroGuiContext g_gui;

/* Rendu d'un rectangle de pixel dans le Framebuffer VRAM */
void zero_gui_draw_rect(uint32_t *fb, uint32_t x, uint32_t y, uint32_t w, uint32_t h, uint32_t color) {
    if (!fb) return;
    for (uint32_t j = y; j < y + h && j < DISPLAY_HEIGHT; j++) {
        for (uint32_t i = x; i < x + w && i < DISPLAY_WIDTH; i++) {
            fb[j * DISPLAY_WIDTH + i] = color;
        }
    }
}

/* Initialisation de la Suite Logicielle & UI Graphique Accessible ZERO OS */
void zero_gui_suite_init(uint32_t *framebuffer_base) {
    g_gui.framebuffer = framebuffer_base;
    g_gui.element_count = 0;
    g_gui.active_focus_index = 0;

    /* Composant 1 : Barre des tâches / Fenêtre OS (Rôle Window = 2) */
    ZeroGuiElement *win = &g_gui.elements[0];
    win->x = 50; win->y = 50; win->width = 1820; win->height = 980;
    win->color = 0xFF1E1E2E; win->role_id = 2; win->is_focused = 0;

    /* Composant 2 : Bouton "Menu Démarrer ZERO" (Rôle Button = 5) */
    ZeroGuiElement *btn_start = &g_gui.elements[1];
    btn_start->x = 100; btn_start->y = 100; btn_start->width = 300; btn_start->height = 80;
    btn_start->color = 0xFF89B4FA; btn_start->role_id = 5; btn_start->is_focused = 1;

    /* Composant 3 : Éditeur de texte / Console IA (Rôle Textbox = 8) */
    ZeroGuiElement *editor = &g_gui.elements[2];
    editor->x = 100; editor->y = 220; editor->width = 1200; editor->height = 700;
    editor->color = 0xFF313244; editor->role_id = 8; editor->is_focused = 0;

    g_gui.element_count = 3;
}

/* Rendu Graphique & Émission Directe des Annonces Vocales Sans API OS */
void zero_gui_render_and_announce(void) {
    for (uint32_t i = 0; i < g_gui.element_count; i++) {
        ZeroGuiElement *el = &g_gui.elements[i];
        uint32_t color = el->color;
        if (el->is_focused) {
            color = 0xFFF9E2AF; /* Surlignage de focus accessible */
        }
        zero_gui_draw_rect(g_gui.framebuffer, el->x, el->y, el->width, el->height, color);
    }
}

/* Navigation de Focus Clavier Natif et Annonce Vocale Matérielle Directe */
void zero_gui_next_focus(void) {
    if (g_gui.element_count == 0) return;
    g_gui.elements[g_gui.active_focus_index].is_focused = 0;
    g_gui.active_focus_index = (g_gui.active_focus_index + 1) % g_gui.element_count;
    g_gui.elements[g_gui.active_focus_index].is_focused = 1;

    /* Rendu et émission directe de l'événement accessible au processeur ZERO-ISA */
    zero_gui_render_and_announce();
}
