//! Distribution parity and statistical sanity tests.

mod common;

use randloom::config::{DistType, Params};
use randloom::dist;
use randloom::rng::Rng;

fn base_params(count: i32, seed: u64) -> Params {
    let mut params = Params {
        count,
        seed,
        ..Params::default()
    };
    params.validate();
    params
}

fn generate(dist: DistType, params: Params) -> Vec<f64> {
    let mut rng = Rng::from_seed(params.seed);
    let mut out = Vec::new();
    dist::generate(dist, &params, &mut rng, &mut out);
    out
}

#[test]
fn uniform_seed42_matches_c_reference() {
    let expected = common::fixture_i32("uniform_seed42.golden");
    let mut params = base_params(expected.len() as i32, 42);
    params.min = 0;
    params.max = 100;
    let actual = generate(DistType::Uniform, params);
    assert_eq!(actual.len(), expected.len());
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(*actual, expected as f64, "uniform sample #{index}");
    }
}

#[test]
fn bernoulli_seed42_matches_c_reference() {
    let expected = common::fixture_i32("bernoulli_seed42.golden");
    let mut params = base_params(expected.len() as i32, 42);
    params.prob = 0.5;
    let actual = generate(DistType::Bernoulli, params);
    assert_eq!(actual.len(), expected.len());
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(*actual, expected as f64, "bernoulli sample #{index}");
    }
}

#[test]
fn normal_seed42_matches_c_reference_within_tolerance() {
    let expected = common::fixture_f64("normal_seed42.golden");
    let mut params = base_params(expected.len() as i32, 42);
    params.mean = 0.0;
    params.stddev = 1.0;
    let actual = generate(DistType::Normal, params);
    assert_eq!(actual.len(), expected.len());
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        let tolerance = 1e-12 * expected.abs().max(1.0);
        assert!(
            (actual - expected).abs() <= tolerance,
            "normal sample #{index}: {actual} vs {expected}"
        );
    }
}

fn poisson_golden(parameter: f64, fixture: &str) {
    let expected = common::fixture_i32(fixture);
    let mut params = base_params(expected.len() as i32, 42);
    params.lambda = parameter;
    let actual = generate(DistType::Poisson, params);
    assert_eq!(actual.len(), expected.len());
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(
            *actual, expected as f64,
            "poisson(lambda={parameter}) sample #{index}"
        );
    }
}

#[test]
fn poisson_knuth_matches_c_reference() {
    poisson_golden(1.0, "poisson_seed42_lambda1.golden");
    poisson_golden(10.0, "poisson_seed42_lambda10.golden");
}

#[test]
fn poisson_ptrs_matches_c_reference() {
    poisson_golden(10.0001, "poisson_seed42_lambda10_0001.golden");
    poisson_golden(100.0, "poisson_seed42_lambda100.golden");
    poisson_golden(1_000_000.0, "poisson_seed42_lambda1000000.golden");
}

#[test]
fn uniform_swaps_reversed_bounds_and_respects_range() {
    let mut rng = Rng::from_seed(7);
    for _ in 0..1000 {
        let value = dist::uniform::sample(&mut rng, 10, -10);
        assert!((-10..=10).contains(&value));
    }
}

#[test]
fn uniform_is_roughly_flat() {
    let mut rng = Rng::from_seed(0xC0FFEE);
    let mut buckets = [0_u32; 10];
    const N: u32 = 100_000;
    for _ in 0..N {
        let value = dist::uniform::sample(&mut rng, 0, 9);
        buckets[value as usize] += 1;
    }
    let expected = N as f64 / 10.0;
    let chi_square: f64 = buckets
        .iter()
        .map(|&count| {
            let diff = count as f64 - expected;
            diff * diff / expected
        })
        .sum();
    assert!(chi_square < 30.0, "chi-square too large: {chi_square}");
}

#[test]
fn normal_has_expected_moments() {
    let mut rng = Rng::from_seed(12345);
    const N: usize = 100_000;
    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    for _ in 0..N {
        let value = dist::normal::sample(&mut rng, 0.0, 1.0);
        sum += value;
        sum_sq += value * value;
    }
    let mean = sum / N as f64;
    let variance = sum_sq / N as f64 - mean * mean;
    assert!(mean.abs() < 0.02, "normal mean off: {mean}");
    assert!(
        (variance.sqrt() - 1.0).abs() < 0.02,
        "normal stddev off: {}",
        variance.sqrt()
    );
}

#[test]
fn bernoulli_frequency_tracks_probability() {
    let mut rng = Rng::from_seed(99);
    const N: u32 = 100_000;
    let successes = (0..N)
        .filter(|_| dist::bernoulli::sample(&mut rng, 0.3))
        .count();
    let frequency = successes as f64 / N as f64;
    assert!(
        (frequency - 0.3).abs() < 0.008,
        "bernoulli frequency off: {frequency}"
    );
}

#[test]
fn poisson_mean_tracks_lambda() {
    for lambda in [1.0, 10.0, 1000.0] {
        let mut rng = Rng::from_seed(2024);
        const N: usize = 100_000;
        let sum: u64 = (0..N)
            .map(|_| dist::poisson::sample(&mut rng, lambda) as u64)
            .sum();
        let mean = sum as f64 / N as f64;
        let tolerance = if lambda < 2.0 { 0.02 } else { lambda * 0.01 };
        assert!(
            (mean - lambda).abs() < tolerance,
            "poisson mean off for lambda={lambda}: {mean}"
        );
    }
}
