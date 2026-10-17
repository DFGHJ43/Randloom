/*
 * braille.c — Braille dot-matrix canvas (see braille.h)
 *
 * Dot mask of one cell (Unicode braille, code point 0x2800 + mask):
 *
 *     0x01 0x08        dots 1 4
 *     0x02 0x10        dots 2 5
 *     0x04 0x20        dots 3 6
 *     0x40 0x80        dots 7 8
 *
 * The mask of an empty cell stays 0 and braille_render() skips such cells:
 * the graph area is not repainted where there is no ink, so a frame only
 * writes the cells the plot actually covers.
 *
 * Output encoding: UTF-8. Terminals whose output path cannot carry it (a
 * Windows console stuck on a legacy code page — term_console_init() tries
 * to switch it to UTF-8) fall back to one '#' per touched cell, which keeps
 * the plot readable at cell resolution.
 */

#include "braille.h"
#include "../term.h"

#include <stdio.h>
#include <stdlib.h>

/* Bit for dot row y (0 = top) and column x (0 = left) inside a cell */
static const unsigned char DOT[2][4] = {
    { 0x01, 0x02, 0x04, 0x40 },   /* left column:  dots 1,2,3,7 */
    { 0x08, 0x10, 0x20, 0x80 }    /* right column: dots 4,5,6,8 */
};

int braille_init(BrailleCanvas *cv, int cols, int rows) {
    cv->cols = cols > 0 ? cols : 0;
    cv->rows = rows > 0 ? rows : 0;
    cv->cells = NULL;

    if (cv->cols == 0 || cv->rows == 0) return -1;

    cv->cells = (unsigned char *)calloc((size_t)cv->cols * (size_t)cv->rows, 1);
    if (cv->cells == NULL) {
        cv->cols = 0;
        cv->rows = 0;
        return -1;
    }
    return 0;
}

void braille_free(BrailleCanvas *cv) {
    free(cv->cells);
    cv->cells = NULL;
    cv->cols = 0;
    cv->rows = 0;
}

void braille_set(BrailleCanvas *cv, int x, int y) {
    if (cv == NULL || cv->cells == NULL) return;
    if (x < 0 || y < 0 || x >= cv->cols * 2 || y >= cv->rows * 4) return;

    cv->cells[(y >> 2) * cv->cols + (x >> 1)] |= DOT[x & 1][y & 3];
}

void braille_vspan(BrailleCanvas *cv, int x, int y0, int y1) {
    if (y0 > y1) { int t = y0; y0 = y1; y1 = t; }
    for (int y = y0; y <= y1; y++) braille_set(cv, x, y);
}

void braille_rect(BrailleCanvas *cv, int x0, int y0, int w, int h) {
    for (int y = y0; y < y0 + h; y++)
        for (int x = x0; x < x0 + w; x++)
            braille_set(cv, x, y);
}

/* Emit one braille pattern as UTF-8. Code points 0x2800..0x28FF are always
   three bytes: 1110_0010 10xx_xxxx 10xx_xxxx. */
static void emit_pattern(unsigned char mask) {
    unsigned int cp = 0x2800u + (unsigned int)mask;
    putchar((int)(0xE0u | (cp >> 12)));
    putchar((int)(0x80u | ((cp >> 6) & 0x3Fu)));
    putchar((int)(0x80u | (cp & 0x3Fu)));
}

void braille_render(const BrailleCanvas *cv, int row, int col) {
    if (cv == NULL || cv->cells == NULL) return;

    int utf8 = term_utf8_supported();

    for (int r = 0; r < cv->rows; r++) {
        int in_run = 0;   /* within a run the cursor advances by itself */
        for (int c = 0; c < cv->cols; c++) {
            unsigned char mask = cv->cells[r * cv->cols + c];
            if (mask == 0) {
                in_run = 0;
                continue;
            }
            if (!in_run) {
                term_goto(row + r, col + c);
                in_run = 1;
            }
            if (utf8) emit_pattern(mask);
            else      putchar('#');
        }
    }
}
