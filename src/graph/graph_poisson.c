/*
 * graph_poisson.c — Poisson distribution PMF graph renderer
 *
 * draw_graph_poisson: discrete bar chart of P(k) = (λ^k * e^(-λ)) / k!
 */

#include "graph_poisson.h"
#include "block.h"

#include <math.h>
#include <stdio.h>
#include <string.h>

void draw_graph_poisson(const TuiState *state) {
    int gx = state->gpx;
    int gy = state->gy;
    int gw = state->gpw;
    int gh = state->gh;
    double lambda = state->lambda;

    if (lambda <= 0.0) return;

    draw_graph_axes(gx, gy, gh, gw);

    char title[48];
    snprintf(title, sizeof(title), "Poisson PMF  lambda=%.1f", lambda);
    term_goto(gy - 1, gx);
    printf("%-*.*s", state->mw - 2, state->mw - 2, title);

    int k_max = (int)(lambda + 4.0 * sqrt(lambda));
    if (k_max < (int)lambda + 1) k_max = (int)lambda + 1;

    double pmf[64];
    int count = 0;

    pmf[0] = exp(-lambda);
    count = 1;

    for (int k = 1; k <= k_max && count < 64; k++) {
        pmf[count] = pmf[count - 1] * lambda / (double)k;
        count++;
    }

    double pmf_max = pmf[0];
    for (int i = 1; i < count; i++) {
        if (pmf[i] > pmf_max) pmf_max = pmf[i];
    }
    if (pmf_max <= 0.0) pmf_max = 1.0;

    /* Bars are placed in half-cell space (one half-cell column per cell, two
       half-cells per row): every bar gets one slot of cell columns and can
       use both half-cells of a row for its height, so P(k) spikes keep their
       shape instead of being rounded to whole cells. One blank cell is left
       between bars when the bars are still wide enough; only when the
       distribution has more bars than the plot can hold — a large lambda —
       does the slot shrink to a single cell column and every step-th bar get
       sampled, as the text grid did. The canvas covers rows gy+1..gy+gh-1:
       the top plot row stays free for the PMF value legend at (gy, mx). */
    int canvas_rows = gh - 1;                /* top plot row = PMF legend */
    int sub_w = gw;
    int sub_h = 2 * canvas_rows;

    int step = 1;
    if (count > sub_w) step = (count + sub_w - 1) / sub_w;
    int shown = (count + step - 1) / step;
    int pitch = sub_w / shown;               /* cell columns per bar, >= 1 */
    if (pitch < 1) pitch = 1;
    int gap    = pitch >= 2 ? 1 : 0;         /* one blank cell when roomy */
    int bar_dw = pitch - gap;

    int x = (sub_w - (shown * pitch - gap)) / 2;   /* centre the row of bars */

    BlockCanvas cv;
    if (block_init(&cv, gw, canvas_rows) == 0) {
        for (int i = 0; i < count; i += step) {
            int h = (int)((pmf[i] / pmf_max) * (double)(sub_h - 1) + 0.5);
            if (h < 1 && pmf[i] > 0.0) h = 1;
            if (h > sub_h - 1) h = sub_h - 1;

            int w = bar_dw;
            if (x + w > sub_w) w = sub_w - x;
            if (w > 0) block_rect(&cv, x, sub_h - h, w, h);

            x += pitch;
        }
        block_render(&cv, gy + 1, gx);
        block_free(&cv);
    }

    term_goto(gy + gh + 1, gx);
    printf("0");
    char buf[16];
    snprintf(buf, sizeof(buf), "%d", k_max);
    term_goto(gy + gh + 1, gx + gw - (int)strlen(buf));
    printf("%s", buf);

    int lambda_col = (int)((lambda / (double)k_max) * (double)(gw - 1));
    if (lambda_col >= 0 && lambda_col < gw) {
        term_goto(gy + gh + 1, gx + lambda_col);
        printf("L");
    }

    term_goto(gy, state->mx);
    printf("%.3f", pmf_max);
}
