//! xoshiro256** engine and seeding.
//!
//! The engine is a direct port of the C reference: the state update is a
//! linear xor/shift/rotate transform and the output passes through a
//! multiply-rotate-multiply scrambler. SplitMix64 expands the 64-bit seed
//! into the four state words, so the all-zero absorbing state is never
//! reached from a normal seed.

use std::time::{SystemTime, UNIX_EPOCH};

/// `2^-53`, the scale applied to the top 53 bits of a `u64`.
const F64_SCALE: f64 = 1.0 / 9_007_199_254_740_992.0;

/// xoshiro256** generator state.
#[derive(Clone, Debug)]
pub struct Rng {
    state: [u64; 4],
}

impl Rng {
    /// Seed the engine from a 64-bit value via SplitMix64.
    ///
    /// A seed of 0 is valid here; the application layer is responsible for
    /// resolving "0 = automatic" to a clock value before calling this.
    pub fn from_seed(seed: u64) -> Self {
        let mut x = seed;
        let mut state = [0_u64; 4];
        for slot in &mut state {
            *slot = splitmix64(&mut x);
        }
        Self { state }
    }

    /// Next raw 64-bit output.
    pub fn next_u64(&mut self) -> u64 {
        let result = self.state[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);

        let t = self.state[1] << 17;
        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];
        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(45);
        result
    }

    /// Uniform `[0, 1)` with the full 53-bit significand of an `f64`.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * F64_SCALE
    }

    /// Uniform `(0, 1)`; the half-offset keeps `ln((1-u)/u)` finite.
    pub fn next_f64_open(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64 + 0.5) * F64_SCALE
    }
}

/// Current Unix time in seconds, used for the automatic seed.
pub fn seed_from_clock() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

/// Resolve configured seed 0 to the current clock value.
///
/// The clock value is clamped away from 0 so a displayed seed is never the
/// ambiguous automatic value. A real wall clock is far from the Unix epoch.
pub fn effective_seed(configured: u64, now: u64) -> u64 {
    if configured == 0 {
        now.max(1)
    } else {
        configured
    }
}

/// SplitMix64 (Steele et al.): scrambles a 64-bit seed stream into well
/// distributed state words.
fn splitmix64(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
