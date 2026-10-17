/*
 * dump_sequences.c — golden-vector generator for the Rust rewrite.
 *
 * Includes the C reference implementation directly so even the static
 * engine helpers (next_u64, next_double, next_double_open) are callable.
 *
 * Build (from legacy/c/tools):
 *   gcc -O2 -std=c99 -o dump_sequences dump_sequences.c -lm
 *
 * Usage:
 *   dump_sequences engine     <seed> <n>
 *   dump_sequences f64        <seed> <n>
 *   dump_sequences uniform    <seed> <n> <min> <max>
 *   dump_sequences normal     <seed> <n> <mean> <stddev>
 *   dump_sequences bernoulli  <seed> <n> <p>
 *   dump_sequences poisson    <seed> <n> <lambda>
 *
 * Every mode seeds first, then prints exactly n values, one per line.
 * Floating-point values use %.17g so the golden files round-trip.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "../src/random.c"

static unsigned long long parse_u64(const char *s) {
    return strtoull(s, NULL, 10);
}

static int parse_int(const char *s) {
    return (int)strtol(s, NULL, 10);
}

static double parse_double(const char *s) {
    return strtod(s, NULL);
}

int main(int argc, char **argv) {
    if (argc < 4) {
        fprintf(stderr,
                "usage: %s <engine|f64|uniform|normal|bernoulli|poisson> "
                "<seed> <n> [params...]\n",
                argv[0]);
        return 2;
    }

    const char *mode = argv[1];
    unsigned long long seed = parse_u64(argv[2]);
    int n = parse_int(argv[3]);
    if (n < 0) n = 0;

    rng_init((uint64_t)seed);

    if (strcmp(mode, "engine") == 0) {
        for (int i = 0; i < n; i++) {
            printf("%llu\n", (unsigned long long)next_u64());
        }
        return 0;
    }

    if (strcmp(mode, "f64") == 0) {
        for (int i = 0; i < n; i++) {
            printf("%.17g\n", next_double());
        }
        return 0;
    }

    if (strcmp(mode, "uniform") == 0) {
        if (argc < 6) return 2;
        int min = parse_int(argv[4]);
        int max = parse_int(argv[5]);
        for (int i = 0; i < n; i++) {
            printf("%d\n", rng_uniform(min, max));
        }
        return 0;
    }

    if (strcmp(mode, "normal") == 0) {
        if (argc < 6) return 2;
        double mean = parse_double(argv[4]);
        double stddev = parse_double(argv[5]);
        for (int i = 0; i < n; i++) {
            printf("%.17g\n", rng_normal(mean, stddev));
        }
        return 0;
    }

    if (strcmp(mode, "bernoulli") == 0) {
        if (argc < 5) return 2;
        double p = parse_double(argv[4]);
        for (int i = 0; i < n; i++) {
            printf("%d\n", rng_bernoulli(p));
        }
        return 0;
    }

    if (strcmp(mode, "poisson") == 0) {
        if (argc < 5) return 2;
        double lambda = parse_double(argv[4]);
        for (int i = 0; i < n; i++) {
            printf("%d\n", rng_poisson(lambda));
        }
        return 0;
    }

    fprintf(stderr, "unknown mode: %s\n", mode);
    return 2;
}
