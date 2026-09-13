//! Metric structures, evaluation routines, and statistical tests for EXP-2026-003a.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetrics {
    pub train_accuracy: f64,
    pub test_accuracy: f64,
    pub train_mse: f64,
    pub test_mse: f64,
    pub capacity_k_xor: f64,
    pub capacity_m_tau: f64,
    pub mean_firing_density: f64,
    pub min_firing_density: f64,
    pub max_firing_density: f64,
    pub mean_threshold: f64,
    pub weight_l2_norm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub seed: u64,
    pub condition: String,
    pub delay_tau: usize,
    pub pulse_density: f64,
    pub ridge_alpha: f64,
    pub metrics: RunMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionDelaySummary {
    pub mean_acc: f64,
    pub std_acc: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateEvaluation {
    pub tau_5_active_xor_mean_accuracy: f64,
    pub tau_5_active_xor_ci_95: [f64; 2],
    pub tau_5_gate_passed: bool,
    pub p_value_vs_memoryless: f64,
    pub memoryless_separation_passed: bool,
    pub p_value_vs_direct_linear: f64,
    pub direct_linear_separation_passed: bool,
    pub p_value_vs_static_thresh: f64,
    pub static_thresh_advantage_passed: bool,
    pub overall_hypothesis_verdict: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryEvaluation {
    pub protocol_id: String,
    pub hypothesis_id: String,
    pub execution_timestamp: String,
    pub total_runs: usize,
    pub gate_evaluation: GateEvaluation,
    pub condition_summaries: std::collections::BTreeMap<
        String,
        std::collections::BTreeMap<String, ConditionDelaySummary>,
    >,
}

/// Compute two-sample Student's t-test p-value
pub fn two_sample_t_test(group_a: &[f64], group_b: &[f64]) -> f64 {
    if group_a.is_empty() || group_b.is_empty() {
        return 1.0;
    }
    let n_a = group_a.len() as f64;
    let n_b = group_b.len() as f64;
    let mean_a = group_a.iter().sum::<f64>() / n_a;
    let mean_b = group_b.iter().sum::<f64>() / n_b;

    let var_a = group_a.iter().map(|x| (x - mean_a).powi(2)).sum::<f64>() / (n_a - 1.0).max(1.0);
    let var_b = group_b.iter().map(|x| (x - mean_b).powi(2)).sum::<f64>() / (n_b - 1.0).max(1.0);

    let se = (var_a / n_a + var_b / n_b).sqrt();
    if se < 1e-12 {
        return if (mean_a - mean_b).abs() < 1e-12 {
            1.0
        } else {
            0.0
        };
    }
    let t = (mean_a - mean_b).abs() / se;

    // Approximate p-value from t using normal approximation for large df (df >= 30)
    let z = t;
    2.0 * normal_cdf(-z)
}

/// Standard normal cumulative distribution function approximation (erfc)
pub fn normal_cdf(x: f64) -> f64 {
    0.5 * erfc(-x / std::f64::consts::SQRT_2)
}

/// Complementary error function Chebyshev approximation
pub fn erfc(x: f64) -> f64 {
    let z = x.abs();
    let t = 1.0 / (1.0 + 0.5 * z);
    let r = t
        * (-z * z - 1.26551223
            + t * (1.00002368
                + t * (0.37409196
                    + t * (0.09678418
                        + t * (-0.18628806
                            + t * (0.27886807
                                + t * (-1.13520398
                                    + t * (1.48851587 + t * (-0.82215223 + t * 0.17087277)))))))))
            .exp();
    if x >= 0.0 { r } else { 2.0 - r }
}
