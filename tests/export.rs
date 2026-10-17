//! CSV export tests: filename shape, content format and failure handling.

mod common;

use std::fs;

use chrono::NaiveDateTime;
use randloom::export::{filename, write_csv};
use randloom::fmt::NumberStyle;

#[test]
fn timestamped_filename_matches_c_pattern() {
    let now = NaiveDateTime::parse_from_str("2026-10-17 21:08:24", "%Y-%m-%d %H:%M:%S")
        .expect("valid fixed timestamp");
    assert_eq!(filename(now), "randloom_export_20261017_210824.csv");
}

#[test]
fn csv_contains_one_compact_value_per_line() {
    let path = common::unique_temp_path("randloom_export", "csv");
    write_csv(
        &[0.0, 1.5, 1_000_000.0, 0.00001],
        &path,
        NumberStyle::Compact,
    )
    .expect("write CSV");

    let contents = fs::read_to_string(&path).expect("read CSV");
    assert_eq!(contents, "0\n1.5\n1e+06\n1e-05\n");

    fs::remove_file(&path).expect("remove temp CSV");
}

#[test]
fn csv_uses_six_decimals_for_continuous_results() {
    let path = common::unique_temp_path("randloom_export_float", "csv");
    write_csv(
        &[0.0, 1.5, -2.25, 1e-7, 123.456_789],
        &path,
        NumberStyle::Decimal6,
    )
    .expect("write CSV");

    let contents = fs::read_to_string(&path).expect("read CSV");
    assert_eq!(
        contents,
        "0.000000\n1.500000\n-2.250000\n1.000000e-07\n123.456789\n"
    );

    fs::remove_file(&path).expect("remove temp CSV");
}

#[test]
fn write_failure_is_reported() {
    let dir = common::unique_temp_path("randloom_export_dir", "tmp");
    fs::create_dir(&dir).expect("create temp dir");

    let result = write_csv(&[1.0], &dir, NumberStyle::Compact);
    assert!(result.is_err(), "writing to a directory must fail");

    fs::remove_dir(&dir).expect("remove temp dir");
}

#[test]
fn empty_export_creates_an_empty_file() {
    let path = common::unique_temp_path("randloom_export_empty", "csv");
    write_csv(&[], &path, NumberStyle::Decimal6).expect("write empty CSV");
    assert_eq!(fs::read_to_string(&path).expect("read CSV"), "");
    fs::remove_file(&path).expect("remove temp CSV");
}
