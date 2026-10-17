/*
 * stats.h — Statistics computation interface
 */

#ifndef STATS_H
#define STATS_H

/*
 * Holds the computed statistics for a set of numbers.
 */
typedef struct {
    double min;
    double max;
    double mean;
    double stddev;
    int    count;
} StatsResult;

/*
 * Compute statistics (min, max, mean, stddev) for an array of doubles.
 */
StatsResult stats_compute(const double *data, int count);

#endif /* STATS_H */
