/*
 * graph_bernoulli.c — Bernoulli distribution PMF graph renderer
 *
 * draw_graph_bernoulli: dual-bar PMF (P(0)=1-p, P(1)=p)
 */

#include "graph_bernoulli.h"
#include "braille.h"

#include <math.h>
#include <stdio.h>

void draw_graph_bernoulli(const TuiState *state) {
    int gx = state->gpx;
    int gy = state->gy;
    int gw = state->gpw;
    int gh = state->gh;
    double p = state->prob;

    if (p < 0.0) p = 0.0;
    if (p > 1.0) p = 1.0;

    for (int i = 0; i <= gh; i++) {
        term_goto(gy + gh - i, gx - 1);
        putchar('|');
    }

    term_goto(gy + gh, gx);
    for (int i = 0; i < gw; i++) putchar('-');
    term_goto(gy + gh, gx + gw);
    putchar('>');

    char title[48];
    snprintf(title, sizeof(title), "Bernoulli PMF  p=%.2f", p);
    term_goto(gy - 1, gx);
    printf("%-*.*s", state->mw - 2, state->mw - 2, title);

    /* The bars are laid out in dot space (2 dots per cell column, 4 per cell
       row) on the proportions of the original 80-column geometry — bar width
       6/28 of the plot width, left edges at 4/28 and 20/28 — so the shape is
       unchanged while each bar top gains four positions per cell row. The
       canvas covers rows gy+1..gy+gh-1: the top plot row stays free for the
       y-axis labels printed at (gy, mx) and (gy+gh, mx). */
    int canvas_rows = gh - 1;                /* top plot row = y-axis legend */
    int dot_h = 4 * canvas_rows;

    /* Geometry in cells first, then doubled: whole-cell bar widths and
       positions keep every bar edge on a cell boundary. */
    int bar_cw  = 6 * gw / 28;
    if (bar_cw < 1) bar_cw = 1;
    int bar_dw  = 2 * bar_cw;
    int bar0_dx = 2 * (4 * gw / 28);
    int bar1_dx = 2 * (20 * gw / 28);

    int h0 = (int)((1.0 - p) * (double)(dot_h - 1) + 0.5);
    int h1 = (int)(p * (double)(dot_h - 1) + 0.5);
    if (h0 < 2) h0 = 2;              /* a zero-probability bar stays visible */
    if (h1 < 2) h1 = 2;
    if (h0 > dot_h - 1) h0 = dot_h - 1;
    if (h1 > dot_h - 1) h1 = dot_h - 1;

    BrailleCanvas cv;
    if (braille_init(&cv, gw, canvas_rows) == 0) {
        braille_rect(&cv, bar0_dx, dot_h - h0, bar_dw, h0);
        braille_rect(&cv, bar1_dx, dot_h - h1, bar_dw, h1);
        braille_render(&cv, gy + 1, gx);
        braille_free(&cv);
    }

    int bar0_x = gx + bar0_dx / 2;   /* bars in cell columns, for labels */
    int bar1_x = gx + bar1_dx / 2;
    int bar_w  = bar_dw / 2;

    term_goto(gy + gh + 1, bar0_x + (bar_w - 1) / 2);
    printf("0");
    term_goto(gy + gh + 1, bar1_x + (bar_w - 1) / 2);
    printf("1");

    term_goto(gy, state->mx);
    printf("1.0");
    term_goto(gy + gh, state->mx);
    printf("0.0");

    /* Value labels sit one cell row above each bar top. A bar tall enough to
       reach the top of the canvas puts its label in the free legend row. */
    int row0 = gy + 1 + (dot_h - h0) / 4 - 1;
    int row1 = gy + 1 + (dot_h - h1) / 4 - 1;
    if (row0 < gy) row0 = gy;
    if (row1 < gy) row1 = gy;

    term_goto(row0, bar0_x);
    printf("%.2f", 1.0 - p);
    term_goto(row1, bar1_x);
    printf("%.2f", p);
}
