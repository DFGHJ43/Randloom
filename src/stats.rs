//! Statistics helpers, matching the C implementation's population standard
//! deviation (`sqrt(sum((x - mean)^2) / n)`).

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stats {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub stddev: f64,
    pub count: usize,
}

/// Compute min, max, mean and population standard deviation.
pub fn compute(data: &[f64]) -> Stats {
    if data.is_empty() {
        return Stats::default();
    }

    let mut sum = data[0];
    let mut min = data[0];
    let mut max = data[0];
    for &value in &data[1..] {
        sum += value;
        if value < min {
            min = value;
        }
        if value > max {
            max = value;
        }
    }

    let mean = sum / data.len() as f64;
    let mut sum_sq_diff = 0.0;
    for &value in data {
        let diff = value - mean;
        sum_sq_diff += diff * diff;
    }
    let stddev = (sum_sq_diff / data.len() as f64).sqrt();

    Stats {
        min,
        max,
        mean,
        stddev,
        count: data.len(),
    }
}
