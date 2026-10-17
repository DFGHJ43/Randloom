/*
 * graph_axes.h — Shared graph axes interface
 *
 * Every distribution renderer draws the same coordinate frame, so the axes
 * live in graph.c and are exposed here.
 */

#ifndef GRAPH_AXES_H
#define GRAPH_AXES_H

#include "../term.h"

/* Draw the y-axis in column gx-1 (rows gy+1..gy+gh-1) and the x-axis on row
   gy+gh (columns gx..gx+gw-1), with a corner at the origin and an arrow at
   the right end. Row gy stays free for each graph's value legend, so the
   legend never paints over the top of the y-axis. Uses box-drawing
   characters, or '|' '-' '+' '>' where UTF-8 cannot reach the screen. */
void draw_graph_axes(int gx, int gy, int gh, int gw);

#endif /* GRAPH_AXES_H */
