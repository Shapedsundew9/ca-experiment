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

pub use crate::substrate::stats::{erfc_approx, sample_mean, sample_std, welch_t_test};
