/*
 * random.h — Random number generation interface
 */

#ifndef RANDOM_H
#define RANDOM_H

#include <stdint.h>

/*
 * Initialize the random number generator with a 64-bit seed.
 * Call once before generating numbers. If seed is 0, uses time(NULL).
 * The seed is expanded into the 256-bit engine state via SplitMix64.
 */
void rng_init(uint64_t seed);

/*
 * Generate a uniformly distributed random integer in [min, max].
 */
int rng_uniform(int min, int max);

/*
 * Generate a normally distributed random number
 * with the given mean and standard deviation.
 * Uses the Box-Muller transform.
 */
double rng_normal(double mean, double stddev);

/*
 * Generate a Bernoulli-distributed random integer.
 * Returns 1 with probability p, 0 with probability (1-p).
 */
int rng_bernoulli(double p);

/*
 * Generate a Poisson-distributed random integer
 * with the given rate parameter lambda (lambda > 0).
 * Uses Knuth's algorithm for lambda <= 10, exact PTRS
 * rejection sampling (Hörmann) for larger lambda.
 */
int rng_poisson(double lambda);

#endif /* RANDOM_H */
