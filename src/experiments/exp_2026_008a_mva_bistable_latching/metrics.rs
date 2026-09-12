//! Quantitative evaluation metrics and statistical hypothesis testing for EXP-2026-008a.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetrics {
    pub retention_accuracy: f64,
    pub q0_retained: bool,
    pub q1_retained: bool,
    pub ber_read: f64,
    pub pulse_persists_post_read: bool,
    pub transition_fidelity: f64,
    pub transitions_correct: usize,
    pub transitions_total: usize,
    pub mean_firing_density: f64,
}

pub use crate::substrate::stats::{erfc_approx, sample_mean, sample_std, welch_t_test};
