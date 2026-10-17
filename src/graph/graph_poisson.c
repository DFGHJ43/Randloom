/*
 * graph_poisson.c — Poisson distribution PMF graph renderer
 *
 * draw_graph_poisson: discrete bar chart of P(k) = (λ^k * e^(-λ)) / k!
 */

#include "graph_poisson.h"
#include "braille.h"

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

    for (int i = 0; i <= gh; i++) {
        term_goto(gy + gh - i, gx - 1);
        putchar('|');
    }

    term_goto(gy + gh, gx);
    for (int i = 0; i < gw; i++) putchar('-');
    term_goto(gy + gh, gx + gw);
    putchar('>');

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

    /* Bars are placed in dot space (2 dots per cell column, 4 per cell row):
       every bar gets one slot of dot columns and can use all four dots of a
       row for its height, so P(k) spikes keep their shape instead of being
       rounded to whole cells. The slot is kept an even number of dots wide
       (and one blank cell is left between bars when the bars are still wide
       enough), so bar edges land on cell boundaries; only when the
       distribution has more bars than the plot can hold — a large lambda —
       does the slot shrink to a single dot column and every step-th bar get
       sampled, as the text grid did. The canvas covers rows gy+1..gy+gh-1:
       the top plot row stays free for the PMF value legend at (gy, mx). */
    int canvas_rows = gh - 1;                /* top plot row = PMF legend */
    int dot_w = 2 * gw;
    int dot_h = 4 * canvas_rows;

    int step = 1;
    if (count > dot_w) step = (count + dot_w - 1) / dot_w;
    int shown = (count + step - 1) / step;
    int pitch = dot_w / shown;               /* dot columns per bar, >= 1 */
    if (pitch >= 2) pitch -= pitch & 1;      /* even pitch -> cell-aligned bars */
    if (pitch < 1) pitch = 1;
    int gap    = pitch >= 4 ? 2 : 0;         /* one blank cell when roomy */
    int bar_dw = pitch - gap;

    int x = (dot_w - (shown * pitch - gap)) / 2;   /* centre the row of bars */
    x -= x & 1;                                    /* left edges on cell lines */

    BrailleCanvas cv;
    if (braille_init(&cv, gw, canvas_rows) == 0) {
        for (int i = 0; i < count; i += step) {
            int h = (int)((pmf[i] / pmf_max) * (double)(dot_h - 1) + 0.5);
            if (h < 1 && pmf[i] > 0.0) h = 1;
            if (h > dot_h - 1) h = dot_h - 1;

            int w = bar_dw;
            if (x + w > dot_w) w = dot_w - x;
            if (w > 0) braille_rect(&cv, x, dot_h - h, w, h);

            x += pitch;
        }
        braille_render(&cv, gy + 1, gx);
        braille_free(&cv);
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
