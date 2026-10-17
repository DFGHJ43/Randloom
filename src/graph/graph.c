/*
 * graph.c — Graph area dispatch
 *
 * draw_graph: clear graph area and dispatch to specific distribution
 */

#include "graph.h"
#include "../chars.h"
#include "../term.h"

#include <stdio.h>

void draw_graph_axes(int gx, int gy, int gh, int gw) {
    int utf8 = term_utf8_supported();

    if (utf8) {
        /* Row gy is the value-legend row: start the axis below it so a
           "0.399"-style legend cannot paint over the top of the y-axis. */
        for (int row = gy + 1; row < gy + gh; row++) {
            term_goto(row, gx - 1);
            fputs(UTF8_VBAR, stdout);
        }
        term_goto(gy + gh, gx - 1);
        fputs(UTF8_BL, stdout);

        term_goto(gy + gh, gx);
        for (int i = 0; i < gw; i++) fputs(UTF8_HBAR, stdout);

        term_goto(gy + gh, gx + gw);
        fputs(UTF8_ARROW, stdout);
    } else {
        for (int row = gy + 1; row < gy + gh; row++) {
            term_goto(row, gx - 1);
            putchar('|');
        }
        term_goto(gy + gh, gx - 1);
        putchar('+');

        term_goto(gy + gh, gx);
        for (int i = 0; i < gw; i++) putchar('-');

        term_goto(gy + gh, gx + gw);
        putchar('>');
    }
}

void draw_graph(const TuiState *state) {
    for (int row = state->gy - 1; row < state->gy + state->gh + 3; row++) {
        term_goto(row, state->mx);
        for (int col = 0; col < state->mw; col++) putchar(' ');
    }

    if (state->dist == DIST_NORMAL) {
        draw_graph_normal(state);
    } else if (state->dist == DIST_BERNOULLI) {
        draw_graph_bernoulli(state);
    } else if (state->dist == DIST_POISSON) {
        draw_graph_poisson(state);
    } else {
        draw_graph_uniform(state);
    }
}
