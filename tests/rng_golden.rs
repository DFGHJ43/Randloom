//! Golden-vector tests for the raw engine and seeding helpers.

mod common;

use randloom::rng::{Rng, effective_seed, seed_from_clock};

#[test]
fn xoshiro_engine_seed42_matches_c_reference() {
    let expected = common::fixture_u64("engine_seed42.golden");
    let mut rng = Rng::from_seed(42);
    for (index, expected) in expected.into_iter().enumerate() {
        assert_eq!(
            rng.next_u64(),
            expected,
            "raw u64 #{index} differs from the C reference"
        );
    }
}

#[test]
fn next_f64_seed42_matches_c_reference_bits() {
    let expected = common::fixture_f64("f64_seed42.golden");
    let mut rng = Rng::from_seed(42);
    for (index, expected) in expected.into_iter().enumerate() {
        assert_eq!(
            rng.next_f64().to_bits(),
            expected.to_bits(),
            "next_f64 #{index} differs from the C reference"
        );
    }
}

#[test]
fn effective_seed_uses_clock_only_for_zero() {
    assert_eq!(effective_seed(0, 1_234), 1_234);
    assert_eq!(effective_seed(42, 1_234), 42);
    assert_eq!(effective_seed(0, 0), 1);
}

#[test]
fn clock_seed_is_a_real_timestamp() {
    // Far enough past the Unix epoch to catch a broken implementation.
    assert!(seed_from_clock() > 1_600_000_000);
}
