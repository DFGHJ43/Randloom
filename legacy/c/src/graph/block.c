/*
 * block.c — Half-block canvas (see block.h)
 *
 * One cell holds two independent half-cells:
 *
 *     bit 0  top half     => ▀ (top only)
 *     bit 1  bottom half  => ▄ (bottom only)
 *     both                => █
 *
 * A mask of 0 stays empty and block_render() skips such cells: the graph
 * area is not repainted where there is no ink, so a frame only writes the
 * cells the plot actually covers.
 *
 * Output encoding: UTF-8. Terminals whose output path cannot carry it (a
 * Windows console stuck on a legacy code page — term_console_init() tries
 * to switch it to UTF-8) fall back to one '#' per touched cell, which keeps
 * the plot readable at cell resolution.
 */

#include "block.h"
#include "../chars.h"
#include "../term.h"

#include <stdio.h>
#include <stdlib.h>

#define BLOCK_TOP     0x01
#define BLOCK_BOTTOM  0x02
#define BLOCK_BOTH    (BLOCK_TOP | BLOCK_BOTTOM)

int block_init(BlockCanvas *cv, int cols, int rows) {
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

void block_free(BlockCanvas *cv) {
    free(cv->cells);
    cv->cells = NULL;
    cv->cols = 0;
    cv->rows = 0;
}

void block_set(BlockCanvas *cv, int x, int y) {
    if (cv == NULL || cv->cells == NULL) return;
    if (x < 0 || y < 0 || x >= cv->cols || y >= cv->rows * 2) return;

    cv->cells[(y >> 1) * cv->cols + x] |=
        (unsigned char)(y & 1 ? BLOCK_BOTTOM : BLOCK_TOP);
}

void block_vspan(BlockCanvas *cv, int x, int y0, int y1) {
    if (y0 > y1) { int t = y0; y0 = y1; y1 = t; }
    for (int y = y0; y <= y1; y++) block_set(cv, x, y);
}

void block_rect(BlockCanvas *cv, int x0, int y0, int w, int h) {
    for (int y = y0; y < y0 + h; y++)
        for (int x = x0; x < x0 + w; x++)
            block_set(cv, x, y);
}

/* The character for one cell mask. Empty masks are never passed here. */
static const char *block_glyph(unsigned char mask) {
    switch (mask & BLOCK_BOTH) {
    case BLOCK_BOTH:   return UTF8_FULL_BLOCK;
    case BLOCK_TOP:    return UTF8_UPPER_HALF;
    case BLOCK_BOTTOM: return UTF8_LOWER_HALF;
    default:           return " ";
    }
}

void block_render(const BlockCanvas *cv, int row, int col) {
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
            if (utf8) fputs(block_glyph(mask), stdout);
            else      putchar('#');
        }
    }
}
