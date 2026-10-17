//! Bernoulli distribution: 1 with probability `p`, else 0.

use crate::rng::Rng;

pub fn sample(rng: &mut Rng, p: f64) -> bool {
    let p = p.clamp(0.0, 1.0);
    rng.next_f64() < p
}
