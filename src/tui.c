/*
 * tui.c — RNG TUI main module
 *
 * Event loop, frame compositing (box + three columns),
 * results panel, generate/export actions.
 * Terminal control -> term.c, controls -> controls.c, graph -> graph.c.
 */

#include "tui.h"
#include "term.h"
#include "controls.h"
#include "graph/graph.h"
#include "random.h"
#include "output.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

/* ══════════════════════════════════════════════════════════
 * TUI lifecycle
 * ══════════════════════════════════════════════════════════ */


/*
 * Recompute the full layout from the actual terminal size.
 * Column split is proportional to the baseline 80-col layout:
 * left/mid/right = 22/32/22 of the 76 content columns.
 */
void tui_recompute_layout(TuiState *state) {
    int tw = state->term_cols;
    int th = state->term_rows;
    int content_w = tw - 4;  /* 4 = two borders + two dividers */

    state->lw = content_w * 22 / 76;
    state->mw = content_w * 32 / 76;
    state->rw = content_w - state->lw - state->mw;  /* absorbs rounding */

    state->lx   = 1;
    state->div1 = state->lx + state->lw;
    state->mx   = state->div1 + 1;
    state->div2 = state->mx + state->mw;
    state->rx   = state->div2 + 1;

    /* Graph area */
    state->gy = 4;
    state->gh = th - 14;               /* 10 rows at a 24-row terminal */
    if (state->gh < 6) state->gh = 6;  /* floor */
    state->gpx = state->mx + 2;        /* 2-col margin inside mid panel */
    state->gpw = state->mw - 4;        /* 2-col margin each side */
    if (state->gpw < 10) state->gpw = 10;  /* floor */

    /* Status bar */
    state->status_y = th - 2;

    /* Results panel */
    state->result_max_show = th - 14;
    if (state->result_max_show < 1) state->result_max_show = 1;
}

void tui_init(TuiState *state) {
#ifdef _WIN32
    term_enable_vt();
#endif
    term_clear();
    term_hide_cursor();

    /* Detect terminal size; refuse to run if too small */
    term_get_size(&state->term_rows, &state->term_cols);
    if (state->term_cols < TERM_MIN_COLS || state->term_rows < TERM_MIN_ROWS) {
        term_clear();
        term_show_cursor();
        printf("Terminal too small: %dx%d. Need at least %dx%d.\n",
               state->term_cols, state->term_rows,
               TERM_MIN_COLS, TERM_MIN_ROWS);
        exit(1);
    }
    tui_recompute_layout(state);

    state->dist         = DIST_UNIFORM;
    state->count        = 1;
    state->min_val      = 0;
    state->max_val      = 100;
    state->mean         = 0.0;
    state->stddev       = 1.0;
    state->prob         = 0.5;
    state->lambda       = 1.0;
    state->seed         = 0;
    state->result_count = 0;
    state->result_scroll= 0;
    state->focus_field  = FIELD_COUNT_NUM;
    state->running      = 1;
    state->graph_dirty  = 1;
    state->status[0]    = '\0';

    snprintf(state->field_buf[FIELD_COUNT_NUM], FIELD_BUF_LEN, "%d", state->count);
    snprintf(state->field_buf[FIELD_SEED],     FIELD_BUF_LEN, "%llu", state->seed);
    snprintf(state->field_buf[FIELD_MIN],     FIELD_BUF_LEN, "%d", state->min_val);
    snprintf(state->field_buf[FIELD_MAX],     FIELD_BUF_LEN, "%d", state->max_val);
    snprintf(state->field_buf[FIELD_MEAN],    FIELD_BUF_LEN, "%.1f", state->mean);
    snprintf(state->field_buf[FIELD_STDDEV],  FIELD_BUF_LEN, "%.1f", state->stddev);
    snprintf(state->field_buf[FIELD_PROB],    FIELD_BUF_LEN, "%.1f", state->prob);
    snprintf(state->field_buf[FIELD_LAMBDA],  FIELD_BUF_LEN, "%.1f", state->lambda);

    for (int i = 0; i < FIELD_COUNT; i++)
        state->field_cursor[i] = (int)strlen(state->field_buf[i]);

    rng_init(0);
    memset(state->results, 0, sizeof(state->results));
}

void tui_restore(void) {
    term_clear();
    term_show_cursor();
    term_flush();
}

/* ══════════════════════════════════════════════════════════
 * Screen rendering
 * ══════════════════════════════════════════════════════════ */

static void draw_box(const TuiState *state) {
    int w = state->term_cols;
    int h = state->term_rows;
    int d1 = state->div1;
    int d2 = state->div2;

    term_goto(0, 0); putchar('+');
    for (int c = 1; c <= w - 2; c++) {
        if (c == d1 || c == d2) putchar('+');
        else                    putchar('-');
    }
    term_goto(0, w - 1); putchar('+');

    for (int r = 1; r <= h - 2; r++) {
        term_goto(r, 0);      putchar('|');
        term_goto(r, d1);     putchar('|');
        term_goto(r, d2);     putchar('|');
        term_goto(r, w - 1);  putchar('|');
    }

    term_goto(h - 1, 0); putchar('+');
    for (int c = 1; c <= w - 2; c++) {
        if (c == d1 || c == d2) putchar('+');
        else                    putchar('-');
    }
    term_goto(h - 1, w - 1); putchar('+');
}

static void draw_results(const TuiState *state) {
    int x = state->rx;
    int y = 3;
    int max_show = state->result_max_show;

    /* Clear one extra row: the "(N total, arrows)" hint below the list
       would otherwise survive when the count drops to fit on screen */
    for (int i = 0; i < max_show + 3; i++) {
        term_goto(y + i, x);
        printf("%-*s", state->rw, "");
    }

    if (state->result_count == 0) {
        term_goto(y + 1, x);
        printf("(no results)");
        return;
    }

    int start = state->result_scroll;
    if (start > state->result_count - max_show)
        start = state->result_count - max_show;
    if (start < 0) start = 0;

    for (int i = 0; i < max_show && (start + i) < state->result_count; i++) {
        term_goto(y + i + 1, x);
        printf("%3d: %g", start + i + 1, state->results[start + i]);
    }

    if (state->result_count > max_show) {
        char hint[48];
        snprintf(hint, sizeof(hint), "(%d total, arrows)", state->result_count);
        term_goto(y + max_show + 2, x);
        /* Truncate to the panel width (rw=17 at 60 cols): a full
           "(1000 total, arrows)" wraps past the border. Same pattern
           as draw_option in controls.c. */
        printf("%-*.*s", state->rw, state->rw, hint);
    }
}

/* ══════════════════════════════════════════════════════════
 * Full frame
 * ══════════════════════════════════════════════════════════ */

static void draw_frame(TuiState *state) {
    draw_box(state);

    term_goto(0, state->lx + 2);
    printf("Random Number Generator");
    term_goto(2, state->mx + 2);
    printf("Distribution Graph");
    term_goto(2, state->rx + 2);
    printf("Results");

    draw_controls(state);
    draw_results(state);

    if (state->graph_dirty) {
        draw_graph(state);
        state->graph_dirty = 0;
    }

    term_goto(state->status_y, state->lx);
    if (state->status[0] != '\0')
        printf("\033[1m%-*s\033[0m", state->term_cols - 2, state->status);
    else
        printf("%-*s", state->term_cols - 2, "");

    term_flush();
}

/* ══════════════════════════════════════════════════════════
 * Actions
 * ══════════════════════════════════════════════════════════ */

static void do_generate(TuiState *state) {
    parse_fields(state);

    int n = state->count;
    if (n > MAX_RESULTS) n = MAX_RESULTS;

    state->result_count = n;
    state->result_scroll = 0;

    /* Seed 0 = auto: reseed from the clock every run. Non-zero = fixed:
       the same seed (with same dist + params) reproduces the sequence. */
    unsigned long long seed = state->seed;
    if (seed == 0) seed = (unsigned long long)time(NULL);
    rng_init(seed);

    if (state->dist == DIST_NORMAL) {
        for (int i = 0; i < n; i++)
            state->results[i] = rng_normal(state->mean, state->stddev);
    } else if (state->dist == DIST_BERNOULLI) {
        for (int i = 0; i < n; i++)
            state->results[i] = (double)rng_bernoulli(state->prob);
    } else if (state->dist == DIST_POISSON) {
        for (int i = 0; i < n; i++)
            state->results[i] = (double)rng_poisson(state->lambda);
    } else {
        for (int i = 0; i < n; i++)
            state->results[i] = (double)rng_uniform(state->min_val, state->max_val);
    }

    snprintf(state->status, sizeof(state->status),
             "Gen %d %s numbers. Seed %llu%s.", n,
             state->dist == DIST_NORMAL    ? "normal" :
             state->dist == DIST_BERNOULLI ? "bernoulli" :
             state->dist == DIST_POISSON   ? "poisson" : "uniform",
             seed, state->seed == 0 ? " (auto)" : "");
}

static void do_export(TuiState *state) {
    if (state->result_count == 0) {
        snprintf(state->status, sizeof(state->status), "Nothing to export!");
        return;
    }

    time_t now = time(NULL);
    struct tm *t = localtime(&now);
    char fname[64];
    snprintf(fname, sizeof(fname), "rng_export_%04d%02d%02d_%02d%02d%02d.csv",
             t->tm_year + 1900, t->tm_mon + 1, t->tm_mday,
             t->tm_hour, t->tm_min, t->tm_sec);

    if (output_file(state->results, state->result_count, fname) == 0)
        snprintf(state->status, sizeof(state->status), "Exported %d to CSV.", state->result_count);
    else
        snprintf(state->status, sizeof(state->status), "Export failed!");
}

/* ══════════════════════════════════════════════════════════
 * Main event loop
 * ══════════════════════════════════════════════════════════ */

/*
 * Poll for terminal resize. Returns:
 *   0 — size unchanged
 *   1 — size changed, layout recomputed (full redraw needed)
 *  -1 — terminal too small (caller shows the overlay)
 */
static int check_resize(TuiState *state) {
    int cur_rows, cur_cols;
    term_get_size(&cur_rows, &cur_cols);

    /* Report too-small on every call, even when the size is unchanged,
       so the caller never falls through to rendering a garbled frame. */
    if (cur_rows < TERM_MIN_ROWS || cur_cols < TERM_MIN_COLS) {
        state->term_rows = cur_rows;
        state->term_cols = cur_cols;
        return -1;
    }

    if (cur_rows == state->term_rows && cur_cols == state->term_cols)
        return 0;

    state->term_rows = cur_rows;
    state->term_cols = cur_cols;

    tui_recompute_layout(state);
    term_clear();
    state->graph_dirty = 1;
    if (state->result_scroll > state->result_count - state->result_max_show)
        state->result_scroll = state->result_count - state->result_max_show;
    if (state->result_scroll < 0) state->result_scroll = 0;
    return 1;
}

static void show_too_small(const TuiState *state) {
    term_clear();
    printf("Terminal too small: %dx%d. Need at least %dx%d.\n",
           state->term_cols, state->term_rows,
           TERM_MIN_COLS, TERM_MIN_ROWS);
    term_flush();
}

void tui_run(void) {
    TuiState state;
    tui_init(&state);

    while (state.running) {
        if (check_resize(&state) < 0) {
            /* Too small: show the overlay once, drain any keys pressed
               while it is up, then wait for the window to grow back.
               No redraw here — flashing it every frame is needless. */
            show_too_small(&state);
            while (state.running && check_resize(&state) < 0) {
                /* Keep draining keys the user presses while the overlay
                   is up, so none get replayed after the window grows */
                while (kb_hit()) kb_get();
                term_sleep_ms(200);
            }
            if (!state.running) break;
            /* check_resize() recomputed the layout and cleared the
               screen — fall through to draw the fresh frame */
        }

        draw_frame(&state);

        int resized = 0;
        while (!kb_hit() && state.running) {
            term_sleep_ms(10);
            /* Detect resize while idle so a shrunken window never
               keeps showing a stale (garbled) frame */
            if (check_resize(&state) != 0) {
                resized = 1;
                break;
            }
        }
        if (!state.running) break;

        /* A resize ended the wait: loop back so the overlay is shown
           (if too small) or the frame is redrawn at the new size.
           Skipping read_key() here keeps buffered keys from being
           processed against a stale frame. */
        if (resized)
            continue;

        int key = read_key();

        switch (key) {
        case 'q': case 'Q': case KEY_ESC:
            state.running = 0; break;

        case 'u': case 'U':
            state.dist = DIST_UNIFORM;
            state.focus_field = FIELD_COUNT_NUM;
            state.graph_dirty = 1;
            snprintf(state.status, sizeof(state.status), "Switched to Uniform."); break;

        case 'n': case 'N':
            state.dist = DIST_NORMAL;
            state.focus_field = FIELD_COUNT_NUM;
            state.graph_dirty = 1;
            snprintf(state.status, sizeof(state.status), "Switched to Normal."); break;

        case 'b': case 'B':
            state.dist = DIST_BERNOULLI;
            state.focus_field = FIELD_COUNT_NUM;
            state.graph_dirty = 1;
            snprintf(state.status, sizeof(state.status), "Switched to Bernoulli."); break;

        case 'p': case 'P':
            state.dist = DIST_POISSON;
            state.focus_field = FIELD_COUNT_NUM;
            state.graph_dirty = 1;
            snprintf(state.status, sizeof(state.status), "Switched to Poisson."); break;

        case 'g': case 'G':
            do_generate(&state); break;

        case 'e': case 'E':
            do_export(&state); break;

        case '\t':
            do {
                state.focus_field = (state.focus_field + 1) % FIELD_COUNT;
            } while (!field_is_visible(state.focus_field, state.dist));
            break;

        case KEY_UP:
            if (state.result_count > 0 && state.result_scroll > 0)
                state.result_scroll--;
            break;

        case KEY_DOWN:
            if (state.result_count > 0 &&
                state.result_scroll < state.result_count - state.result_max_show)
                state.result_scroll++;
            break;

        case KEY_LEFT:
            if (state.field_cursor[state.focus_field] > 0)
                state.field_cursor[state.focus_field]--;
            break;

        case KEY_RIGHT: {
            int len = (int)strlen(state.field_buf[state.focus_field]);
            if (state.field_cursor[state.focus_field] < len)
                state.field_cursor[state.focus_field]++;
            break;
        }

        case '\b': case 127:
            field_backspace((FieldId)state.focus_field, &state);
            parse_fields(&state);
            if (field_is_visible(state.focus_field, state.dist))
                state.graph_dirty = 1;
            break;

        default:
            /* Seed takes digits only: strtoull("-5") would otherwise wrap
               to ULLONG_MAX, which looks like a plausible-but-wrong seed */
            if (state.focus_field == FIELD_SEED) {
                if (key >= '0' && key <= '9') {
                    field_insert(FIELD_SEED, &state, (char)key);
                    parse_fields(&state);
                }
            } else if ((key >= '0' && key <= '9') || key == '-' || key == '.') {
                field_insert((FieldId)state.focus_field, &state, (char)key);
                parse_fields(&state);
                if (field_is_visible(state.focus_field, state.dist))
                    state.graph_dirty = 1;
            }
            break;
        }
    }

    tui_restore();
}

int main(void) {
    tui_run();
    return 0;
}
