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

pub fn sample_mean(data: &[f64]) -> f64 {
    if data.is_empty() {
        0.0
    } else {
        data.iter().sum::<f64>() / data.len() as f64
    }
}

pub fn sample_std(data: &[f64]) -> f64 {
    if data.len() < 2 {
        0.0
    } else {
        let m = sample_mean(data);
        let var = data.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (data.len() - 1) as f64;
        var.sqrt()
    }
}

/// Two-tailed Welch's t-test comparing sample1 and sample2.
/// Returns (t_stat, p_value, cohen_d).
pub fn welch_t_test(sample1: &[f64], sample2: &[f64]) -> (f64, f64, f64) {
    let n1 = sample1.len() as f64;
    let n2 = sample2.len() as f64;

    if n1 < 2.0 || n2 < 2.0 {
        return (0.0, 1.0, 0.0);
    }

    let m1 = sample_mean(sample1);
    let m2 = sample_mean(sample2);

    let v1 = sample1.iter().map(|x| (x - m1).powi(2)).sum::<f64>() / (n1 - 1.0);
    let v2 = sample2.iter().map(|x| (x - m2).powi(2)).sum::<f64>() / (n2 - 1.0);

    let se = ((v1 / n1) + (v2 / n2)).sqrt();
    if se < 1e-12 {
        return (0.0, 1.0, 0.0);
    }

    let t_stat = (m1 - m2) / se;

    // Cohen's d effect size
    let pooled_s = (((n1 - 1.0) * v1 + (n2 - 1.0) * v2) / (n1 + n2 - 2.0))
        .sqrt()
        .max(1e-12);
    let cohen_d = (m1 - m2) / pooled_s;

    // Normal approximation to p-value via complementary error function
    let z = t_stat.abs() / std::f64::consts::SQRT_2;
    let p_val = erfc_approx(z);

    (t_stat, p_val, cohen_d)
}

/// Rational Chebyshev approximation to erfc(x) for x >= 0
fn erfc_approx(x: f64) -> f64 {
    if x <= 0.0 {
        return 1.0;
    }
    if x > 20.0 {
        return 0.0;
    }
    let t = 1.0 / (1.0 + 0.5 * x);
    let tau = t
        * (-x * x - 1.26551223
            + t * (1.00002368
                + t * (0.37409196
                    + t * (0.09678418
                        + t * (-0.18628806
                            + t * (0.27886807
                                + t * (-1.13520398
                                    + t * (1.48851587 + t * (-0.82215223 + t * 0.17087277)))))))))
            .exp();
    tau.clamp(0.0, 1.0)
}
