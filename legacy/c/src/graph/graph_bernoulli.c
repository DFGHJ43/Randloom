/*
 * graph_bernoulli.c — Bernoulli distribution PMF graph renderer
 *
 * draw_graph_bernoulli: dual-bar PMF (P(0)=1-p, P(1)=p)
 */

#include "graph_bernoulli.h"
#include "block.h"

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

    draw_graph_axes(gx, gy, gh, gw);

    char title[48];
    snprintf(title, sizeof(title), "Bernoulli PMF  p=%.2f", p);
    term_goto(gy - 1, gx);
    printf("%-*.*s", state->mw - 2, state->mw - 2, title);

    /* The bars are laid out in half-cell space (one half-cell column per
       cell, two half-cells per row) on the proportions of the original
       80-column geometry — bar width 6/28 of the plot width, left edges at
       4/28 and 20/28 — so the shape is unchanged while each bar top gains a
       half-cell step. The canvas covers rows gy+1..gy+gh-1: the top plot row
       stays free for the y-axis labels printed at (gy, mx) and
       (gy+gh, mx). */
    int canvas_rows = gh - 1;                /* top plot row = y-axis legend */
    int sub_h = 2 * canvas_rows;

    /* Geometry in cells: whole-cell bar widths and positions keep every bar
       edge on a cell boundary. */
    int bar_cw  = 6 * gw / 28;
    if (bar_cw < 1) bar_cw = 1;
    int bar0_cx = 4 * gw / 28;
    int bar1_cx = 20 * gw / 28;

    int h0 = (int)((1.0 - p) * (double)(sub_h - 1) + 0.5);
    int h1 = (int)(p * (double)(sub_h - 1) + 0.5);
    if (h0 < 2) h0 = 2;              /* a zero-probability bar stays visible */
    if (h1 < 2) h1 = 2;
    if (h0 > sub_h - 1) h0 = sub_h - 1;
    if (h1 > sub_h - 1) h1 = sub_h - 1;

    BlockCanvas cv;
    if (block_init(&cv, gw, canvas_rows) == 0) {
        block_rect(&cv, bar0_cx, sub_h - h0, bar_cw, h0);
        block_rect(&cv, bar1_cx, sub_h - h1, bar_cw, h1);
        block_render(&cv, gy + 1, gx);
        block_free(&cv);
    }

    int bar0_x = gx + bar0_cx;
    int bar1_x = gx + bar1_cx;

    term_goto(gy + gh + 1, bar0_x + (bar_cw - 1) / 2);
    printf("0");
    term_goto(gy + gh + 1, bar1_x + (bar_cw - 1) / 2);
    printf("1");

    /* One-character scale labels: they sit in the two-column margin left of
       the y-axis, so even "0" at the baseline cannot cover the corner where
       the axes meet (a "0.0" there would paint over it). */
    term_goto(gy, state->mx);
    printf("1");
    term_goto(gy + gh, state->mx);
    printf("0");

    /* Value labels sit one cell row above each bar top. A bar tall enough to
       reach the top of the canvas puts its label in the free legend row. */
    int row0 = gy + 1 + (sub_h - h0) / 2 - 1;
    int row1 = gy + 1 + (sub_h - h1) / 2 - 1;
    if (row0 < gy) row0 = gy;
    if (row1 < gy) row1 = gy;

    term_goto(row0, bar0_x);
    printf("%.2f", 1.0 - p);
    term_goto(row1, bar1_x);
    printf("%.2f", p);
}
