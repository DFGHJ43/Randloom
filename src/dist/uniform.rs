//! Uniform integer distribution.
//!
//! `[min, max]` is sampled with 64-bit rejection sampling so the result is
//! unbiased even for the full `i32` span. `min > max` is swapped first, just
//! like the C implementation.

use crate::rng::Rng;

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
