/*
 * output.h — Output interface
 */

#ifndef OUTPUT_H
#define OUTPUT_H

#include "stats.h"

/*
 * Print an array of numbers to the console, one per line.
 */
void output_console(const double *numbers, int count);

/*
 * Write an array of numbers to a file, one per line.
 * Returns 0 on success, -1 on error.
 */
int output_file(const double *numbers, int count, const char *filename);

/*
 * Print a statistics summary to the console.
 */
void output_stats(const StatsResult *stats);

#endif /* OUTPUT_H */
