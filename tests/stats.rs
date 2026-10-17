//! Statistics helper tests.

use randloom::stats::compute;

#[test]
fn empty_data_returns_zeros() {
    let stats = compute(&[]);
    assert_eq!(stats.count, 0);
    assert_eq!(stats.min, 0.0);
    assert_eq!(stats.max, 0.0);
    assert_eq!(stats.mean, 0.0);
    assert_eq!(stats.stddev, 0.0);
}

#[test]
fn computes_population_statistics() {
    let stats = compute(&[1.0, 2.0, 3.0, 4.0]);
    assert_eq!(stats.count, 4);
    assert_eq!(stats.min, 1.0);
    assert_eq!(stats.max, 4.0);
    assert_eq!(stats.mean, 2.5);
    assert!((stats.stddev - 1.118_033_988_749_895).abs() < 1e-15);
}

#[test]
fn single_value_has_zero_stddev() {
    let stats = compute(&[42.5]);
    assert_eq!(stats.count, 1);
    assert_eq!(stats.min, 42.5);
    assert_eq!(stats.max, 42.5);
    assert_eq!(stats.mean, 42.5);
    assert_eq!(stats.stddev, 0.0);
}
