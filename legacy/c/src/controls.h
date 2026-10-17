/*
 * controls.h — Controls panel & field editing interface
 */

#ifndef CONTROLS_H
#define CONTROLS_H

#include "tui.h"

/* ── Field visibility ─────────────────────────────────── */
int  field_is_visible(int fid, DistType dist);

/* ── Field editing ────────────────────────────────────── */
void field_insert(FieldId fid, TuiState *state, char c);
void field_backspace(FieldId fid, TuiState *state);
void parse_fields(TuiState *state);

/* ── Controls panel rendering ─────────────────────────── */
void draw_controls(TuiState *state);

#endif /* CONTROLS_H */
