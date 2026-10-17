/*
 * graph_uniform.c — Uniform distribution PDF graph renderer
 *
 * draw_graph_uniform: flat bar (constant PDF = 1/(max-min))
 */

#include "graph_uniform.h"
#include "block.h"

#include <stdio.h>
#include <string.h>

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

void draw_graph_uniform(const TuiState *state) {
    int gx = state->gpx;
    int gy = state->gy;
    int gw = state->gpw;
    int gh = state->gh;
    int min = state->min_val;
    int max = state->max_val;

    if (min >= max) return;

    draw_graph_axes(gx, gy, gh, gw);

    char title[48];
    snprintf(title, sizeof(title), "Uniform PDF  [%d, %d]", min, max);
    term_goto(gy - 1, gx);
    printf("%-*.*s", state->mw - 2, state->mw - 2, title);

    /* The PDF is flat, so the bar is a solid rectangle: every half-cell of
       the canvas gets set, which renders as a wall of full blocks. The
       canvas covers rows gy+1..gy+gh-1 — the top plot row stays free for
       the PDF value legend printed at (gy, mx) further down. */
    BlockCanvas cv;
    int canvas_rows = gh - 1;                /* top plot row = PDF legend */
    if (block_init(&cv, gw, canvas_rows) == 0) {
        block_rect(&cv, 0, 0, cv.cols, cv.rows * 2);
        block_render(&cv, gy + 1, gx);
        block_free(&cv);
    }

    term_goto(gy + gh + 1, gx);
    printf("%d", min);
    char buf[16];
    snprintf(buf, sizeof(buf), "%d", max);
    term_goto(gy + gh + 1, gx + gw - (int)strlen(buf));
    printf("%s", buf);

    double pdf_val = 1.0 / (max - min);
    term_goto(gy, state->mx);
    printf("%4s", "");
    term_goto(gy, state->mx);
    printf("%.2f", pdf_val);
}
