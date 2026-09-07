//! Statistical metrics and hypothesis evaluation routines for EXP-2026-011a.

pub use crate::substrate::stats::{erfc_approx, sample_mean, sample_std, welch_t_test};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetrics {
    pub accuracy_retrieval: f64,
    pub ber: f64,
    pub distractor_leakage_rate: f64,
    pub mutex_violation_rate: f64,
    pub mean_firing_density: f64,
    pub total_sequences: usize,
    pub correct_sequences: usize,
    pub distractor_leakage_events: usize,
    pub distractor_ticks: usize,
    pub mutex_violations: usize,
    pub settled_ticks: usize,
    pub total_spikes: usize,
    pub total_steps: usize,
}
