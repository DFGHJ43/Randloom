//! Normal distribution via the Box-Muller transform.
//!
//! The call order matches the C reference exactly: draw `u1` until it is
//! strictly positive, then draw `u2` once. Only `z0` is used, as in C.

use std::f64::consts::PI;

use crate::rng::Rng;

pub fn sample(rng: &mut Rng, mean: f64, stddev: f64) -> f64 {
    let mut u1;
    loop {
        u1 = rng.next_f64();
        if u1 > 0.0 {
            break;
        }
    }
    let u2 = rng.next_f64();

    let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();
    mean + stddev * z0
}
