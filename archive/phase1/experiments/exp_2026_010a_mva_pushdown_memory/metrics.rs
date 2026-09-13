//! Statistical metrics and hypothesis evaluation routines for EXP-2026-010a.

pub use crate::substrate::stats::{erfc_approx, sample_mean, sample_std, welch_t_test};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetrics {
    pub accuracy_seq: f64,
    pub fidelity_reject: f64,
    pub fidelity_underflow: f64,
    pub fidelity_mismatch: f64,
    pub fidelity_unclosed: f64,
    pub pointer_crosstalk_rate: f64,
    pub ber: f64,
    pub mean_firing_density: f64,
    pub sequences_total: usize,
    pub sequences_correct: usize,
    pub invalid_total: usize,
    pub invalid_rejected: usize,
    pub underflow_total: usize,
    pub underflow_rejected: usize,
    pub mismatch_total: usize,
    pub mismatch_rejected: usize,
    pub unclosed_total: usize,
    pub unclosed_rejected: usize,
    pub crosstalk_ticks: usize,
    pub settled_ticks: usize,
}
