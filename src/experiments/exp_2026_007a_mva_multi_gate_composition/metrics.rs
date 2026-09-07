//! Quantitative evaluation metrics and statistical hypothesis testing for EXP-2026-007a.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetrics {
    pub truth_table_accuracy: f64,
    pub states_correct: usize,
    pub states_total: usize,
    pub ber_sum: f64,
    pub ber_cout: f64,
    pub ber_mean: f64,
    pub crosstalk_cross: f64,
    pub latency_sum: usize,
    pub latency_cout: usize,
    pub latency_skew: usize,
    pub mean_firing_density: f64,
}

/// Ground-truth Full Adder Boolean functions:
/// Sum = A ^ B ^ Cin
#[inline]
pub fn expected_sum(a: usize, b: usize, cin: usize) -> f64 {
    ((a ^ b ^ cin) & 1) as f64
}

/// Cout = (A & B) | (Cin & (A ^ B))
#[inline]
pub fn expected_cout(a: usize, b: usize, cin: usize) -> f64 {
    (((a & b) | (cin & (a ^ b))) & 1) as f64
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

    let diff = m1 - m2;
    if diff.abs() < 1e-12 {
        return (0.0, 1.0, 0.0);
    }

    // Minimum variance floor for discrete finite proportions:
    // Prevents division by zero when clean deterministic baselines produce zero sample variance
    let var_floor = 0.001139; // corresponding to s_floor = 0.03375
    let v1_reg = v1.max(var_floor);
    let v2_reg = v2.max(var_floor);

    let se = ((v1_reg / n1) + (v2_reg / n2)).sqrt();
    let t_stat = diff / se;

    // Cohen's d effect size
    let pooled_s = (((n1 - 1.0) * v1_reg + (n2 - 1.0) * v2_reg) / (n1 + n2 - 2.0))
        .sqrt()
        .max(1e-12);
    let cohen_d = diff / pooled_s;

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
