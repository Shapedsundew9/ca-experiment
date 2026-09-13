//! Configuration module for EXP-2026-005a: Continual Multi-Pattern Learning & Lifelong Adaptation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    BaselineStatic,
    AblationUnconstrained,
    ActiveMetaplastic,
    ControlJointTraining,
}

impl Condition {
    pub fn as_str(&self) -> &'static str {
        match self {
            Condition::BaselineStatic => "baseline_static",
            Condition::AblationUnconstrained => "ablation_unconstrained",
            Condition::ActiveMetaplastic => "active_metaplastic",
            Condition::ControlJointTraining => "control_joint_training",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub run_id: String,
    pub seed: u64,
    pub condition: Condition,
    pub kappa: f64,
    pub t_train: usize,
    pub eta0: f64,
    pub gamma_decay: f64,
    pub w_min: f64,
    pub w_max: f64,
    pub w_sum_max: f64,
    pub leak: f64,
    pub n_ref: usize,
    pub alpha_rho: f64,
    pub rho_target: f64,
    pub beta_theta: f64,
    pub theta_min: f64,
    pub theta_max: f64,
    pub theta_init: f64,
    pub t_drive: usize,
    pub t_relax: usize,
    pub noise_level: f64,
    pub num_noisy_trials: usize,
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self {
            run_id: "EXP-2026-005a-DEFAULT".to_string(),
            seed: 1,
            condition: Condition::ActiveMetaplastic,
            kappa: 1.0,
            t_train: 1000,
            eta0: 0.05,
            gamma_decay: 0.50,
            w_min: 0.1,
            w_max: 3.0,
            w_sum_max: 4.0,
            leak: 0.10,
            n_ref: 2,
            alpha_rho: 0.02,
            rho_target: 0.10,
            beta_theta: 0.05,
            theta_min: 0.5,
            theta_max: 3.0,
            theta_init: 1.0,
            t_drive: 32,
            t_relax: 64,
            noise_level: 0.05,
            num_noisy_trials: 10,
        }
    }
}
