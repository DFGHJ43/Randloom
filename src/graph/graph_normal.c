/*
 * graph_normal.c — Normal distribution PDF graph renderer
 *
 * draw_graph_normal: bell curve sampled once per plot column
 */

#include "graph_normal.h"
#include "block.h"

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

    draw_graph_axes(gx, gy, gh, gw);

    char title[48];
    snprintf(title, sizeof(title), "Normal PDF  mu=%.1f sigma=%.1f", mu, sigma);
    term_goto(gy - 1, gx);
    printf("%-*.*s", state->mw - 2, state->mw - 2, title);

    double x_min = mu - 3.0 * sigma;
    double x_max = mu + 3.0 * sigma;

    double pdf_max = 1.0 / (sigma * sqrt(2.0 * M_PI));

    /* Bell curve as a filled area on a half-block canvas: one half-cell
       column per character column, two half-cells per row. Each column is
       filled from the baseline up to its curve height, so the outline is
       twice as smooth vertically as plain '#' art and the fill is solid.
       The canvas covers rows gy+1..gy+gh-1: the top plot row stays free for
       the PDF value legend printed at (gy, mx). */
    BlockCanvas cv;
    int canvas_rows = gh - 1;                /* top plot row = PDF legend */
    if (block_init(&cv, gw, canvas_rows) == 0) {
        int sub_w = cv.cols;
        int sub_h = cv.rows * 2;

        for (int x = 0; x < sub_w; x++) {
            double t = sub_w > 1 ? (double)x / (double)(sub_w - 1) : 0.0;
            double xv = x_min + (x_max - x_min) * t;
            double z = (xv - mu) / sigma;
            double pdf = exp(-0.5 * z * z) / (sigma * sqrt(2.0 * M_PI));

            int h = (int)((pdf / pdf_max) * (double)(sub_h - 1) + 0.5);
            if (h < 0) h = 0;
            if (h > sub_h - 1) h = sub_h - 1;

            block_vspan(&cv, x, sub_h - 1 - h, sub_h - 1);
        }
        block_render(&cv, gy + 1, gx);
        block_free(&cv);
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
