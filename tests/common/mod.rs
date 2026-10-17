//! Shared helpers for integration tests.

#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn fixture_text(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("cannot read fixture {}: {err}", path.display()))
}

pub fn fixture_lines(name: &str) -> Vec<String> {
    fixture_text(name)
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

pub fn fixture_u64(name: &str) -> Vec<u64> {
    fixture_lines(name)
        .iter()
        .map(|line| line.parse::<u64>().expect("u64 fixture value"))
        .collect()
}

pub fn fixture_i32(name: &str) -> Vec<i32> {
    fixture_lines(name)
        .iter()
        .map(|line| line.parse::<i32>().expect("i32 fixture value"))
        .collect()
}

pub fn fixture_f64(name: &str) -> Vec<f64> {
    fixture_lines(name)
        .iter()
        .map(|line| line.parse::<f64>().expect("f64 fixture value"))
        .collect()
}

/// A unique path in the system temp directory for export tests.
pub fn unique_temp_path(prefix: &str, extension: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "{prefix}_{}_{nanos}_{counter}.{extension}",
        std::process::id()
    ))
}
