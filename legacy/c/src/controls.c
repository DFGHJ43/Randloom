/*
 * controls.c — Left panel rendering + field editing
 *
 * draw_controls: distribution selector + parameter fields + buttons
 * field_insert/backspace: character insert/delete in field buffers
 * parse_fields: field buffer text -> numeric values (strtol/strtod)
 * field_is_visible: whether a field is shown for current distribution
 */

#include "controls.h"
#include "term.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Upper bound for lambda: keeps k_max = (int)(lambda + 4*sqrt(lambda)) in
   graph_poisson.c below INT_MAX (that cast is UB beyond ~2.1e9). */
#define LAMBDA_MAX 1e9

int field_is_visible(int fid, DistType dist) {
    /* Count and Seed are global parameters: visible for every distribution */
    if (fid == FIELD_COUNT_NUM || fid == FIELD_SEED) return 1;
    switch (dist) {
    case DIST_UNIFORM:
        return fid == FIELD_MIN || fid == FIELD_MAX;
    case DIST_NORMAL:
        return fid == FIELD_MEAN || fid == FIELD_STDDEV;
    case DIST_BERNOULLI:
        return fid == FIELD_PROB;
    case DIST_POISSON:
        return fid == FIELD_LAMBDA;
    }
    return 0;
}

void field_insert(FieldId fid, TuiState *state, char c) {
    char *buf = state->field_buf[fid];
    int *cur  = &state->field_cursor[fid];
    int len   = (int)strlen(buf);

    if (len >= FIELD_BUF_LEN - 1) return;

    for (int i = len; i >= *cur; i--)
        buf[i + 1] = buf[i];
    buf[*cur] = c;
    (*cur)++;
}

void field_backspace(FieldId fid, TuiState *state) {
    char *buf = state->field_buf[fid];
    int *cur  = &state->field_cursor[fid];
    int len   = (int)strlen(buf);

    if (*cur <= 0) return;

    for (int i = *cur - 1; i < len; i++)
        buf[i] = buf[i + 1];
    (*cur)--;
}

void parse_fields(TuiState *state) {
    state->count  = (int)strtol(state->field_buf[FIELD_COUNT_NUM], NULL, 10);
    state->seed   = strtoull(state->field_buf[FIELD_SEED], NULL, 10);
    state->min_val= (int)strtol(state->field_buf[FIELD_MIN],     NULL, 10);
    state->max_val= (int)strtol(state->field_buf[FIELD_MAX],     NULL, 10);
    state->mean   = strtod(state->field_buf[FIELD_MEAN],   NULL);
    state->stddev = strtod(state->field_buf[FIELD_STDDEV], NULL);
    state->prob   = strtod(state->field_buf[FIELD_PROB],   NULL);
    state->lambda = strtod(state->field_buf[FIELD_LAMBDA], NULL);

    if (state->count < 1)  state->count = 1;
    if (state->count > MAX_RESULTS) state->count = MAX_RESULTS;
    if (state->prob < 0.0) state->prob = 0.0;
    if (state->prob > 1.0) state->prob = 1.0;
    if (state->stddev <= 0.0) state->stddev = 1.0;
    if (state->lambda <= 0.0) state->lambda = 1.0;
    if (state->lambda > LAMBDA_MAX) state->lambda = LAMBDA_MAX;
}

/*
 * Draw one distribution radio row. The key hint "(X)" is dropped and the
 * label truncated when the left panel is too narrow, so text never spills
 * into the divider column.
 */
static void draw_option(const TuiState *state, int row, const char *name,
                        const char *hint, int selected) {
    char buf[24];
    if (state->lw >= 17)
        snprintf(buf, sizeof(buf), "%s %s (%s)",
                 selected ? "[*]" : "[ ]", name, hint);
    else
        snprintf(buf, sizeof(buf), "%s %s", selected ? "[*]" : "[ ]", name);
    term_goto(row, state->lx + 2);
    printf("%-*.*s", state->lw - 2, state->lw - 2, buf);
}

void draw_controls(TuiState *state) {
    int x = state->lx;
    int y = 3;

    term_goto(y, x);
    printf("%-*s", state->lw, "Distribution:");

    draw_option(state, y + 1, "Uniform",   "U", state->dist == DIST_UNIFORM);
    draw_option(state, y + 2, "Normal",    "N", state->dist == DIST_NORMAL);
    draw_option(state, y + 3, "Bernoulli", "B", state->dist == DIST_BERNOULLI);
    draw_option(state, y + 4, "Poisson",   "P", state->dist == DIST_POISSON);

    static const char *labels[FIELD_COUNT] = {
        "Count:  ", "Seed:   ", "Min:    ", "Max:    ", "Mean:   ", "StdDev: ", "Prob:   ", "Lambda: "
    };

    int value_w = state->lw - 12;   /* label(8) + gap(2) + brackets(2) */
    if (value_w < 1) value_w = 1;

    /* 8 fields (incl. Seed) start one row earlier than the old 7 so the
       last field stays above the [ Generate ] button at y+14 */
    int ry = y + 6;
    for (int i = 0; i < FIELD_COUNT; i++) {
        if (!field_is_visible(i, state->dist)) {
            term_goto(ry, x);
            printf("%-*s", state->lw, "");
            ry++;
            continue;
        }
        term_goto(ry, x);
        printf("%s", labels[i]);

        term_goto(ry, x + 10);
        printf("[");
        if (i == state->focus_field)
            term_style_reverse();
        printf("%-*.*s", value_w, value_w, state->field_buf[i]);
        if (i == state->focus_field)
            term_style_reset();
        printf("]");

        ry++;
    }

    for (; ry < y + 12; ry++) {
        term_goto(ry, x);
        printf("%-*s", state->lw, "");
    }

    int by = y + 14;
    term_goto(by, x);
    printf("[ Generate ]  G");
    term_goto(by + 1, x);
    printf("[ Export   ]  E");
    term_goto(by + 2, x);
    printf("[ Quit     ]  Q");

    for (int row = by + 3; row < state->term_rows - 1; row++) {
        term_goto(row, x);
        printf("%-*s", state->lw, "");
    }
}
