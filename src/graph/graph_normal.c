/*
 * graph_normal.c — Normal distribution PDF graph renderer
 *
 * draw_graph_normal: bell curve via Box-Muller PDF (28 sample columns)
 */

#include "graph_normal.h"
#include "braille.h"

#include <math.h>
#include <stdio.h>
#include <string.h>

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

void draw_graph_normal(const TuiState *state) {
    int gx = state->gpx;
    int gy = state->gy;
    int gw = state->gpw;
    int gh = state->gh;
    double mu = state->mean;
    double sigma = state->stddev;

    if (sigma <= 0.0) return;

    for (int i = 0; i <= gh; i++) {
        term_goto(gy + gh - i, gx - 1);
        putchar('|');
    }

    term_goto(gy + gh, gx);
    for (int i = 0; i < gw; i++) putchar('-');
    term_goto(gy + gh, gx + gw);
    putchar('>');

    char title[48];
    snprintf(title, sizeof(title), "Normal PDF  mu=%.1f sigma=%.1f", mu, sigma);
    term_goto(gy - 1, gx);
    printf("%-*.*s", state->mw - 2, state->mw - 2, title);

    double x_min = mu - 3.0 * sigma;
    double x_max = mu + 3.0 * sigma;

    double pdf_max = 1.0 / (sigma * sqrt(2.0 * M_PI));

    /* Bell curve as a filled area on a braille canvas: two dots per cell
       column, four per cell row. Each column is filled from the baseline up
       to its curve height, so the outline is 4x smoother vertically than
       the old '#' columns and the flanks stop looking stair-stepped. The
       canvas covers rows gy+1..gy+gh-1: the top plot row stays free for the
       PDF value legend printed at (gy, mx). */
    BrailleCanvas cv;
    int canvas_rows = gh - 1;                /* top plot row = PDF legend */
    if (braille_init(&cv, gw, canvas_rows) == 0) {
        int dot_w = cv.cols * 2;
        int dot_h = cv.rows * 4;

        for (int x = 0; x < dot_w; x++) {
            double t = (double)x / (double)(dot_w - 1);
            double xv = x_min + (x_max - x_min) * t;
            double z = (xv - mu) / sigma;
            double pdf = exp(-0.5 * z * z) / (sigma * sqrt(2.0 * M_PI));

            int h = (int)((pdf / pdf_max) * (double)(dot_h - 1) + 0.5);
            if (h < 0) h = 0;
            if (h > dot_h - 1) h = dot_h - 1;

            braille_vspan(&cv, x, dot_h - 1 - h, dot_h - 1);
        }
        braille_render(&cv, gy + 1, gx);
        braille_free(&cv);
    }

    term_goto(gy + gh + 1, gx);
    printf("%.0f", x_min);
    char buf[16];
    snprintf(buf, sizeof(buf), "%.0f", x_max);
    term_goto(gy + gh + 1, gx + gw - (int)strlen(buf));
    printf("%s", buf);

    int mu_col = (int)((mu - x_min) / (x_max - x_min) * (double)(gw - 1));
    if (mu_col >= 0 && mu_col < gw) {
        term_goto(gy + gh + 1, gx + mu_col);
        printf("mu");
    }

    term_goto(gy, state->mx);
    printf("%.3f", pdf_max);
}
