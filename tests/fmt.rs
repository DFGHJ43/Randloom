//! `%g` formatting parity with C `printf`.

use randloom::fmt::{g, g_precision};

#[test]
fn matches_c_printf_cases() {
    let cases: &[(f64, &str)] = &[
        (0.0, "0"),
        (-0.0, "-0"),
        (1.0, "1"),
        (1.5, "1.5"),
        (0.0001, "0.0001"),
        (0.00001, "1e-05"),
        (0.000_123_456, "0.000123456"),
        (999_999.0, "999999"),
        (999_999.4, "999999"),
        (999_999.5, "1e+06"),
        (1_000_000.0, "1e+06"),
        (123.456_789, "123.457"),
        (0.000_000_123_456_789, "1.23457e-07"),
        (1e300, "1e+300"),
        (1e-300, "1e-300"),
        (f64::INFINITY, "inf"),
        (f64::NEG_INFINITY, "-inf"),
    ];

    for (value, expected) in cases {
        assert_eq!(g(*value), *expected, "g({value})");
    }

    assert_eq!(g(f64::NAN), "nan");
}

#[test]
fn supports_explicit_precision() {
    assert_eq!(g_precision(1.234_567, 4), "1.235");
    assert_eq!(g_precision(123_456.789, 4), "1.235e+05");
    assert_eq!(g_precision(0.000_123_456, 4), "0.0001235");
}

#[test]
fn trims_fractional_zeros() {
    assert_eq!(g(1.0), "1");
    assert_eq!(g(1.2300), "1.23");
    assert_eq!(g(-1.500_000), "-1.5");
}
