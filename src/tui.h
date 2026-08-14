/*
 * tui.h — Layout fields, field enums, distribution types, TuiState
 *
 * All layout coordinates are computed at runtime by tui_recompute_layout()
 * from the actual terminal size — nothing is hardcoded to 80x24.
 *
 * Column grid (0-indexed): col 0 = border, left panel (lx..lx+lw-1),
 * div1, mid panel (mx..mx+mw-1), div2, right panel (rx..rx+rw-1),
 * last col = border.
 */

#ifndef TUI_H
#define TUI_H

/* ── Minimum terminal size ──────────────────────────────
 * Rows: the controls panel occupies rows 3-19 (title, radios,
 * fields, buttons); 22 rows leave room for the status bar and
 * bottom border. Cols: below ~60 the panels get too cramped. */
#define TERM_MIN_COLS  60
#define TERM_MIN_ROWS  22

/* ── Input fields ─────────────────────────────────────── */
#define FIELD_COUNT    8
#define FIELD_BUF_LEN  32

typedef enum {
    FIELD_COUNT_NUM = 0,
    FIELD_SEED,
    FIELD_MIN,
    FIELD_MAX,
    FIELD_MEAN,
    FIELD_STDDEV,
    FIELD_PROB,
    FIELD_LAMBDA
} FieldId;

/* ── Distribution type ────────────────────────────────── */
typedef enum {
    DIST_UNIFORM    = 0,
    DIST_NORMAL     = 1,
    DIST_BERNOULLI  = 2,
    DIST_POISSON    = 3
} DistType;

/* ── Main application state ───────────────────────────── */
#define MAX_RESULTS 1000

typedef struct {
    /* layout (recomputed at startup and on resize) */
    int term_rows, term_cols;   /* actual terminal size */
    int lx, lw;                 /* left panel (controls): start col, width */
    int mx, mw;                 /* middle panel (graph): start col, width */
    int rx, rw;                 /* right panel (results): start col, width */
    int div1, div2;             /* divider columns */
    int gy, gh;                 /* graph area: start row, height */
    int gpx, gpw;               /* graph plot: start col, width */
    int status_y;               /* status bar row */
    int result_max_show;        /* max results visible in right panel */

    /* parameters */
    DistType dist;
    int      count;
    int      min_val;
    int      max_val;
    double   mean;
    double   stddev;
    double   prob;
    double   lambda;
    unsigned long long seed;    /* 0 = auto (time-based); non-zero = fixed */

    /* results */
    double   results[MAX_RESULTS];
    int      result_count;
    int      result_scroll;

    /* input editing */
    int      focus_field;
    char     field_buf[FIELD_COUNT][FIELD_BUF_LEN];
    int      field_cursor[FIELD_COUNT];

    /* status message */
    char     status[128];

    /* running flag */
    int      running;

    /* dirty flags for selective redraw */
    int      graph_dirty;
} TuiState;

/* ── TUI API ──────────────────────────────────────────── */
void tui_init(TuiState *state);
void tui_restore(void);
void tui_run(void);
void tui_recompute_layout(TuiState *state);

#endif /* TUI_H */
