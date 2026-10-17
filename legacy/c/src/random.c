/*
 * random.c — Random number generation (xoshiro256** engine)
 *
 * Engine: xoshiro256** by Blackman & Vigna (public domain, prng.di.unimi.it).
 *   - 4x64-bit state, period 2^256 - 1, passes TestU01 BigCrush
 *   - the state update is a linear xor/shift/rotate transform; the output
 *     goes through a nonlinear multiply scrambler (hence "**"), which hides
 *     the linear low bits that the xoshiro256+ variant would leak
 * Seeding: SplitMix64 expands the 64-bit seed into the four state words.
 *   The all-zero state is absorbing (outputs 0 forever), so state words are
 *   derived from the seed, never filled directly.
 *
 * Distributions:
 *   rng_uniform(min, max)    uniform integer in [min, max]
 *   rng_normal(mean, stddev) normal via Box-Muller transform
 *   rng_bernoulli(p)         Bernoulli, returns 1 with probability p, else 0
 *   rng_poisson(lambda)      Knuth (lambda <= 10), PTRS rejection (lambda > 10)
 *
 * Not cryptographically secure: outputs are predictable from observed
 * values. Use an OS CSPRNG if keys or passwords are ever generated.
 */

#include "random.h"

#include <limits.h>
#include <math.h>
#include <stdint.h>
#include <time.h>

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

/* ══════════════════════════════════════════════════════════
 * xoshiro256** engine
 * ══════════════════════════════════════════════════════════ */

static uint64_t s[4];  /* 4x64-bit internal state */

static inline uint64_t rotl64(uint64_t x, int k) {
    return (x << k) | (x >> (64 - k));
}

static uint64_t next_u64(void) {
    /* Output scrambler: multiply-rotate-multiply breaks up the linear
       structure of the state below (low bits stay well distributed). */
    uint64_t result = rotl64(s[1] * 5, 7) * 9;

    /* Linear state update (period 2^256 - 1) */
    uint64_t t = s[1] << 17;
    s[2] ^= s[0];
    s[3] ^= s[1];
    s[1] ^= s[2];
    s[0] ^= s[3];
    s[2] ^= t;
    s[3] = rotl64(s[3], 45);
    return result;
}

/* [0, 1): top 53 bits = the full significand of a double, lossless.
   Replaces rand()/RAND_MAX, whose 15-bit resolution on MSVC visibly
   quantized the normal and Bernoulli outputs. */
static double next_double(void) {
    return (double)(next_u64() >> 11) * 0x1.0p-53;
}

/* (0, 1): open interval for the PTRS branch (keeps log((1-u)/u) finite) */
static double next_double_open(void) {
    return ((double)(next_u64() >> 11) + 0.5) * 0x1.0p-53;
}

/* ══════════════════════════════════════════════════════════
 * Seeding
 * ══════════════════════════════════════════════════════════ */

/* SplitMix64 (Steele et al.): scrambles a 64-bit seed stream into well
   distributed state words. */
static uint64_t splitmix64(uint64_t *x) {
    uint64_t z = (*x += 0x9e3779b97f4a7c15ULL);  /* golden ratio constant */
    z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
    z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
    return z ^ (z >> 31);
}

void rng_init(uint64_t seed) {
    if (seed == 0) {
        seed = (uint64_t)time(NULL);
    }
    uint64_t x = seed;
    for (int i = 0; i < 4; i++)
        s[i] = splitmix64(&x);
}

/* ══════════════════════════════════════════════════════════
 * Distributions
 * ══════════════════════════════════════════════════════════ */

int rng_uniform(int min, int max) {
    if (min > max) {
        int tmp = min;
        min = max;
        max = tmp;
    }
    /*
     * Range computed in 64-bit: (max - min + 1) reaches 2^32 for the full
     * int span, which overflows int (min=INT_MIN, max=INT_MAX wraps it to 0
     * and % 0 crashes). The 64-bit engine output is rejection-sampled to
     * remove the modulo bias.
     */
    unsigned long long range = (unsigned long long)max - (unsigned long long)min + 1;
    unsigned long long limit = ULLONG_MAX - ULLONG_MAX % range;
    unsigned long long r;
    do {
        r = next_u64();
    } while (r >= limit);
    return (int)((long long)min + (long long)(r % range));
}

double rng_normal(double mean, double stddev) {
    /*
     * Box-Muller transform:
     * Given two independent uniform(0,1) random numbers u1, u2,
     * z0 = sqrt(-2 * ln(u1)) * cos(2 * pi * u2)
     * produces a standard normal(0,1) random variable.
     * Then scale: result = mean + stddev * z0.
     */
    /* Avoid log(0): keep generating until u1 > 0 (probability 2^-53) */
    double u1;
    do {
        u1 = next_double();
    } while (u1 <= 0.0);

    double u2 = next_double();
    double z0 = sqrt(-2.0 * log(u1)) * cos(2.0 * M_PI * u2);

    return mean + stddev * z0;
}

int rng_bernoulli(double p) {
    /* Clamp p to [0, 1] */
    if (p < 0.0) p = 0.0;
    if (p > 1.0) p = 1.0;

    return (next_double() < p) ? 1 : 0;
}

int rng_poisson(double lambda) {
    /* Clamp lambda to positive */
    if (lambda <= 0.0) lambda = 0.001;

    if (lambda > 10.0) {
        /*
         * PTRS (Hörmann 1993): transformed-rejection sampling from a
         * truncated logistic distribution - exact for any lambda.
         * Knuth's loop below is O(lambda) and exp(-lambda) underflows to
         * 0.0 past lambda ~ 745, silently capping results at ~745.
         */
        const double c     = 0.767 - 3.36 / lambda;
        const double beta  = M_PI / sqrt(3.0 * lambda);
        const double alpha = beta * lambda;
        const double k_log = log(c) - lambda - log(beta);

        for (;;) {
            /* u in (0,1): keeps log((1-u)/u) finite */
            double u = next_double_open();
            double x = (alpha - log((1.0 - u) / u)) / beta;
            if (x > 9.0e18) return INT_MAX;  /* absurd lambda via direct call */
            long long n = (long long)floor(x + 0.5);
            if (n < 0) continue;

            double v = next_double_open();
            double y = alpha - beta * x;
            double lhs = y + log(v) - 2.0 * log(1.0 + exp(y));
            double rhs = k_log + (double)n * log(lambda) - lgamma((double)n + 1.0);
            if (lhs <= rhs) {
                /* Guard direct callers passing an absurd lambda */
                if (n > INT_MAX) return INT_MAX;
                return (int)n;
            }
        }
    }

    /*
     * Knuth's algorithm for Poisson distribution:
     * L = e^(-lambda), k = 0, p = 1
     * do { k++; p *= U } while (p > L)
     * return k - 1
     */
    double L = exp(-lambda);
    int k = 0;
    double p = 1.0;

    do {
        k++;
        p *= next_double();
    } while (p > L);

    return k - 1;
}
