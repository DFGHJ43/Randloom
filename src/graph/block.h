/*
 * block.h — Half-block canvas for graph rendering
 *
 * A character cell is split into two vertically stacked half-cells, drawn
 * with the Unicode block elements ▀ (upper half), ▄ (lower half) and █
 * (both).  The canvas therefore has a 1x2 sub-cell grid: twice the vertical
 * resolution of plain text art, with solid fills instead of a dot texture.
 *
 * Sub-cell space: x in [0, cols), y in [0, 2*rows), origin at the canvas'
 * top-left cell.  block_render() writes only the cells that hold at least
 * one half-cell, so the caller's background fill shows through the empty
 * cells.
 */

#ifndef BLOCK_H
#define BLOCK_H

typedef struct {
    int cols, rows;         /* canvas size in character cells */
    unsigned char *cells;   /* rows * cols half-cell masks (0 = empty) */
} BlockCanvas;

/* Allocate a cols x rows canvas. Returns 0 on success; -1 if the size is
   empty or allocation fails (the caller skips drawing, nothing else is
   touched). Always pair with block_free(). */
int  block_init(BlockCanvas *cv, int cols, int rows);
void block_free(BlockCanvas *cv);

/* Set a single half-cell. Coordinates outside the canvas are ignored. */
void block_set(BlockCanvas *cv, int x, int y);

/* Set the half-cells of column x between rows y0 and y1 (either order). */
void block_vspan(BlockCanvas *cv, int x, int y0, int y1);

/* Fill the w x h half-cell rectangle whose top-left half-cell is (x0, y0). */
void block_rect(BlockCanvas *cv, int x0, int y0, int w, int h);

/* Draw the canvas with its top-left cell at screen cell (row, col). */
void block_render(const BlockCanvas *cv, int row, int col);

#endif /* BLOCK_H */
