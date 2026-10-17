/*
 * graph.h — Distribution graph rendering interface
 *
 * draw_graph: clear + dispatch to per-distribution renderers
 */

#ifndef GRAPH_MODULE_H
#define GRAPH_MODULE_H

#include "../tui.h"
#include "graph_axes.h"

void draw_graph(const TuiState *state);

#include "graph_uniform.h"
#include "graph_normal.h"
#include "graph_bernoulli.h"
#include "graph_poisson.h"

#endif /* GRAPH_MODULE_H */
