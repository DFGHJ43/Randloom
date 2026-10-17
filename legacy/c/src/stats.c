/*
 * stats.c — Statistics computation
 *
 * stats_compute(data,count): computes min, max, mean, stddev
 * Returns StatsResult{min, max, mean, stddev, count}
 */

#include "stats.h"

#include <math.h>
#include <float.h>

StatsResult stats_compute(const double *data, int count) {
    StatsResult result;
    result.count = count;

    if (count <= 0) {
        result.min   = 0.0;
        result.max   = 0.0;
        result.mean  = 0.0;
        result.stddev = 0.0;
        return result;
    }

    /* Min, max, and sum for mean */
    double sum = data[0];
    double min = data[0];
    double max = data[0];

    for (int i = 1; i < count; i++) {
        sum += data[i];
        if (data[i] < min) min = data[i];
        if (data[i] > max) max = data[i];
    }

    double mean = sum / (double)count;

    /* Standard deviation */
    double sum_sq_diff = 0.0;
    for (int i = 0; i < count; i++) {
        double diff = data[i] - mean;
        sum_sq_diff += diff * diff;
    }
    double stddev = sqrt(sum_sq_diff / (double)count);

    result.min    = min;
    result.max    = max;
    result.mean   = mean;
    result.stddev = stddev;
    return result;
}
