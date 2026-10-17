/*
 * braille.h — Braille dot-matrix canvas for graph rendering
 *
 * A Unicode braille pattern (U+2800..U+28FF) packs a 2x4 dot grid into a
 * single character cell, so a plot drawn on a BrailleCanvas has twice the
 * horizontal and four times the vertical resolution of the old '#' plot.
 * Because a character cell is roughly twice as tall as it is wide, the dots
 * land on a near-square grid: curves come out smooth instead of
 * stair-stepped, at no extra screen space.
 *
 * Dot space: x in [0, 2*cols), y in [0, 4*rows), origin at the canvas'
 * top-left cell. braille_render() writes only the cells that hold at least
 * one dot, so the caller's background fill shows through the empty cells.
 */

#ifndef BRAILLE_H
#define BRAILLE_H

typedef struct {
    int cols, rows;         /* canvas size in character cells */
    unsigned char *cells;   /* rows * cols dot masks (0 = empty) */
} BrailleCanvas;

/* Allocate a cols x rows canvas. Returns 0 on success; -1 if the size is
   empty or allocation fails (the caller skips drawing, nothing else is
   touched). Always pair with braille_free(). */
int  braille_init(BrailleCanvas *cv, int cols, int rows);
void braille_free(BrailleCanvas *cv);

/* Set a single dot. Coordinates outside the canvas are ignored. */
void braille_set(BrailleCanvas *cv, int x, int y);

/* Set the dots of column x between rows y0 and y1 (either order). */
void braille_vspan(BrailleCanvas *cv, int x, int y0, int y1);

/* Fill the w x h dot rectangle whose top-left dot is (x0, y0). */
void braille_rect(BrailleCanvas *cv, int x0, int y0, int w, int h);

/* Draw the canvas with its top-left cell at screen cell (row, col). */
void braille_render(const BrailleCanvas *cv, int row, int col);

#endif /* BRAILLE_H */
