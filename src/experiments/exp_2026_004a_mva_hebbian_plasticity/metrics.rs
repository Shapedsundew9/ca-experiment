//! Evaluation metrics, statistical tests, and telemetry schemas for EXP-2026-004a.

use super::plasticity::NUM_NODES;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetrics {
    pub replay_distance: f64,
    pub min_separation: f64,
    pub basin_depth: f64,
    pub perturbation_distance: f64,
    pub perturbation_resistance: f64,
    pub mean_firing_density: f64,
    pub mean_threshold: f64,
    pub weight_variance: f64,
    pub weight_peak: f64,
    pub budget_conserved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub seed: u64,
    pub condition: String,
    pub pattern: String,
    pub plasticity_rate: f64,
    pub exposure_ticks: usize,
    pub noise_level: f64,
    pub metrics: RunMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateEvaluation {
    pub mean_relative_replay_reduction: f64,
    pub basin_deepening_gate_passed: bool,
    pub mean_relative_perturbation_gain: f64,
    pub perturbation_resistance_gate_passed: bool,
    pub p_value_vs_static_baseline: f64,
    pub static_separation_passed: bool,
    pub pattern_specificity_p_value: f64,
    pub pattern_specificity_passed: bool,
    pub p_value_vs_anti_hebbian: f64,
    pub anti_hebbian_separation_passed: bool,
    pub p_value_vs_random_drift: f64,
    pub random_drift_separation_passed: bool,
    pub critical_density_all_passed: bool,
    pub overall_hypothesis_verdict: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryEvaluation {
    pub protocol_id: String,
    pub hypothesis_id: String,
    pub execution_timestamp: String,
    pub total_runs: usize,
    pub gate_evaluation: GateEvaluation,
    pub condition_summaries:
        std::collections::BTreeMap<String, std::collections::BTreeMap<String, serde_json::Value>>,
}

/// Compute L1 normalized distance between two node occupancy vectors
pub fn occupancy_distance(m1: &[f64; NUM_NODES], m2: &[f64; NUM_NODES]) -> f64 {
    let mut sum_diff = 0.0;
    for i in 0..NUM_NODES {
        sum_diff += (m1[i] - m2[i]).abs();
    }
    sum_diff / NUM_NODES as f64
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

    2.0 * normal_cdf(-t)
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
