//! Uniform distribution: discrete integers and continuous reals.
//!
//! The integer mode samples `[min, max]` with 64-bit rejection sampling so the
//! result is unbiased even for the full `i32` span. The continuous mode maps
//! the 53-bit `f64` output of the engine linearly onto `[min, max)`. Both modes
//! swap reversed bounds first, just like the C implementation.

use crate::rng::Rng;

/// Sample one integer uniformly from the inclusive `[min, max]` range.
pub fn sample(rng: &mut Rng, min: i32, max: i32) -> i32 {
    let (min, max) = if min > max { (max, min) } else { (min, max) };

    // The inclusive range of two i32 values fits in u64 (at most 2^32).
    let range = (max as i64 - min as i64 + 1) as u64;
    let limit = u64::MAX - (u64::MAX % range);
    loop {
        let r = rng.next_u64();
        if r < limit {
            return (min as i64 + (r % range) as i64) as i32;
        }
    }
}

/// Sample one real number uniformly from the half-open `[min, max)` range.
///
/// `next_f64` already covers `[0, 1)` with 53 random bits, so a single linear
/// mapping gives a uniform real on the interval without extra rejection. A
/// zero-width interval (or a non-finite span) yields `min`.
pub fn sample_continuous(rng: &mut Rng, min: f64, max: f64) -> f64 {
    let (min, max) = if min > max { (max, min) } else { (min, max) };
    let span = max - min;
    if !span.is_finite() || span <= 0.0 {
        return min;
    }

    let value = min + span * rng.next_f64();
    if value >= max {
        // `min + span * u` can round up to `max` when the span is only a few
        // ulps wide. `max` is excluded, so step to the next representable
        // value below it; that is still >= `min` because min < max.
        next_down(max)
    } else {
        value
    }
}

/// The largest `f64` strictly below `value`.
///
/// `f64::next_down` is newer than the crate's MSRV, so the bit-level step is
/// done by hand. NaN and negative infinity are returned unchanged.
fn next_down(value: f64) -> f64 {
    if value.is_nan() || value == f64::NEG_INFINITY {
        return value;
    }
    if value == 0.0 {
        // The next value below zero is the smallest negative subnormal.
        return -f64::from_bits(1);
    }

    let bits = value.to_bits();
    if value > 0.0 {
        f64::from_bits(bits - 1)
    } else {
        f64::from_bits(bits + 1)
    }
}
