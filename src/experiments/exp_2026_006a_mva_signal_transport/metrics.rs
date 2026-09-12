//! Quantitative evaluation metrics and statistical hypothesis testing for EXP-2026-006a.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetrics {
    pub ber_branch1: f64,
    pub ber_branch2: f64,
    pub ber_mean: f64,
    pub fidelity: f64,
    pub latency_branch1: usize,
    pub latency_branch2: usize,
    pub latency_skew: usize,
    pub crosstalk_1_to_2: f64,
    pub mean_firing_density: f64,
}

/// Calculate Bit Error Rate (BER) between aligned actual output and input bitstream
pub fn ber(actual: &[f64], expected: &[f64]) -> f64 {
    if actual.is_empty() || expected.is_empty() {
        return 0.0;
    }
    let len = actual.len().min(expected.len());
    let mut err_sum = 0.0;
    for i in 0..len {
        err_sum += (actual[i] - expected[i]).abs();
    }
    err_sum / len as f64
}

/// Calculate combined Transmission Fidelity F = (1 - BER_1)(1 - BER_2)
pub fn transmission_fidelity(ber1: f64, ber2: f64) -> f64 {
    let f1 = (1.0 - ber1).clamp(0.0, 1.0);
    let f2 = (1.0 - ber2).clamp(0.0, 1.0);
    f1 * f2
}

/// Measure temporal latency via cross-correlation maximization:
/// tau = argmax_{delta in [0, max_delay]} sum u(t) s(t + delta)
pub fn measure_latency(
    u_input: &[f64],
    s_history: &[f64],
    t_warmup: usize,
    nominal_latency: usize,
    max_delay: usize,
) -> usize {
    let t_eval = u_input.len();
    if s_history.len() < t_warmup + max_delay + t_eval {
        return nominal_latency;
    }

    let mut best_corr = -1.0;
    let mut best_lags = Vec::new();

    for delta in 0..=max_delay {
        let mut corr = 0.0;
        let start = t_warmup + delta;
        for t in 0..t_eval {
            corr += u_input[t] * s_history[start + t];
        }

        if corr > best_corr {
            best_corr = corr;
            best_lags.clear();
            best_lags.push(delta);
        } else if (corr - best_corr).abs() < 1e-9 {
            best_lags.push(delta);
        }
    }

    if best_corr <= 0.0 {
        // Complete signal extinction: default to nominal latency
        nominal_latency
    } else {
        // In case of periodic ties (e.g. clock signals), pick lag closest to nominal
        *best_lags
            .iter()
            .min_by_key(|&&lag| (lag as isize - nominal_latency as isize).abs())
            .unwrap_or(&nominal_latency)
    }
}

/// Calculate inter-branch crosstalk isolation coefficient chi_{1 -> 2}
pub fn crosstalk_coefficient(s_branch2: &[f64]) -> f64 {
    if s_branch2.is_empty() {
        0.0
    } else {
        s_branch2.iter().sum::<f64>() / s_branch2.len() as f64
    }
}

pub use crate::substrate::stats::{erfc_approx, sample_mean, sample_std, welch_t_test};
