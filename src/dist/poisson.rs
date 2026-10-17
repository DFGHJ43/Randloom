//! Poisson distribution.
//!
//! Knuth's multiplication algorithm is used for `lambda <= 10`; larger rates
//! use Hörmann's PTRS transformed-rejection algorithm. Knuth's algorithm is
//! O(lambda) and `exp(-lambda)` underflows past lambda ~745, which is why the
//! PTRS branch exists.

use std::f64::consts::PI;

use crate::rng::Rng;

/// Sample one Poisson value. `lambda <= 0` is clamped to 0.001 like C.
pub fn sample(rng: &mut Rng, lambda: f64) -> u32 {
    let lambda = if lambda <= 0.0 { 0.001 } else { lambda };

    if lambda > 10.0 {
        // PTRS (Hörmann 1993): exact for any lambda.
        let c = 0.767 - 3.36 / lambda;
        let beta = PI / (3.0 * lambda).sqrt();
        let alpha = beta * lambda;
        let k_log = c.ln() - lambda - beta.ln();

        loop {
            let u = rng.next_f64_open();
            let x = (alpha - ((1.0 - u) / u).ln()) / beta;
            if x > 9.0e18 {
                return i32::MAX as u32;
            }
            let n = (x + 0.5).floor();
            if n < 0.0 {
                continue;
            }

            let v = rng.next_f64_open();
            let y = alpha - beta * x;
            let lhs = y + v.ln() - 2.0 * (1.0 + y.exp()).ln();
            let rhs = k_log + n * lambda.ln() - ln_factorial(n as u64);
            if lhs <= rhs {
                if n > i32::MAX as f64 {
                    return i32::MAX as u32;
                }
                return n as u32;
            }
        }
    }

    // Knuth: L = e^-lambda; k=0; p=1; do { k++; p *= U } while (p > L).
    let l = (-lambda).exp();
    let mut k = 0_u32;
    let mut p = 1.0_f64;
    loop {
        k += 1;
        p *= rng.next_f64();
        if p <= l {
            break;
        }
    }
    k - 1
}

/// Natural log of `n!` for a non-negative integer.
///
/// `f64` has no `lgamma` in the standard library, so small values are summed
/// directly and large values use the Stirling series. Accuracy is far better
/// than needed for PTRS acceptance comparisons.
fn ln_factorial(n: u64) -> f64 {
    if n < 30 {
        let mut sum = 0.0;
        for k in 2..=n {
            sum += (k as f64).ln();
        }
        return sum;
    }

    let nf = n as f64;
    let two_pi = 2.0 * PI;
    let inv = 1.0 / nf;
    let inv2 = inv * inv;
    let inv3 = inv2 * inv;
    let inv5 = inv3 * inv2;
    let inv7 = inv5 * inv2;

    (nf + 0.5) * nf.ln() - nf + 0.5 * two_pi.ln() + inv / 12.0 - inv3 / 360.0 + inv5 / 1260.0
        - inv7 / 1680.0
}
