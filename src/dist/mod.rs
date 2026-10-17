//! Distribution samplers and dispatch.

pub mod bernoulli;
pub mod normal;
pub mod poisson;
pub mod uniform;

use crate::config::{DistType, MAX_RESULTS, Params};
use crate::rng::Rng;

/// Fill `out` with `params.count` samples of `dist`.
pub fn generate(dist: DistType, params: &Params, rng: &mut Rng, out: &mut Vec<f64>) {
    out.clear();
    let n = params.count.clamp(1, MAX_RESULTS as i32) as usize;
    match dist {
        DistType::Uniform => {
            for _ in 0..n {
                out.push(uniform::sample(rng, params.min, params.max) as f64);
            }
        }
        DistType::Normal => {
            for _ in 0..n {
                out.push(normal::sample(rng, params.mean, params.stddev));
            }
        }
        DistType::Bernoulli => {
            for _ in 0..n {
                out.push(if bernoulli::sample(rng, params.prob) {
                    1.0
                } else {
                    0.0
                });
            }
        }
        DistType::Poisson => {
            for _ in 0..n {
                out.push(poisson::sample(rng, params.lambda) as f64);
            }
        }
    }
}
