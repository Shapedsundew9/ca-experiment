//! Configuration module for EXP-2026-008a: Bistable Resonant Latching and Nondestructive Dynamic Bit Storage.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    ActiveLatch,
    BaselineFeedforwardLoss,
    AblationUnshieldedFeedback,
    AblationFixedTheta,
}

impl Condition {
    pub const ALL: [Condition; 4] = [
        Condition::ActiveLatch,
        Condition::BaselineFeedforwardLoss,
        Condition::AblationUnshieldedFeedback,
        Condition::AblationFixedTheta,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Condition::ActiveLatch => "active_latch",
            Condition::BaselineFeedforwardLoss => "baseline_feedforward_loss",
            Condition::AblationUnshieldedFeedback => "ablation_unshielded_feedback",
            Condition::AblationFixedTheta => "ablation_fixed_theta",
        }
    }

    pub fn short_code(&self) -> &'static str {
        match self {
            Condition::ActiveLatch => "ACT",
            Condition::BaselineFeedforwardLoss => "LOSS",
            Condition::AblationUnshieldedFeedback => "UNSHIELD",
            Condition::AblationFixedTheta => "FIX",
        }
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Condition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active_latch" | "act" => Ok(Condition::ActiveLatch),
            "baseline_feedforward_loss" | "loss" | "ff_loss" => {
                Ok(Condition::BaselineFeedforwardLoss)
            }
            "ablation_unshielded_feedback" | "unshield" | "uns" => {
                Ok(Condition::AblationUnshieldedFeedback)
            }
            "ablation_fixed_theta" | "fix" => Ok(Condition::AblationFixedTheta),
            _ => Err(format!("Unknown condition: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationSuite {
    QuiescentRetentionSweep,
    StateTransitionSuite,
}

impl EvaluationSuite {
    pub fn as_str(&self) -> &'static str {
        match self {
            EvaluationSuite::QuiescentRetentionSweep => "quiescent_retention_sweep",
            EvaluationSuite::StateTransitionSuite => "state_transition_suite",
        }
    }
}

impl fmt::Display for EvaluationSuite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub run_id: String,
    pub seed: u64,
    pub condition: Condition,
    pub noise_rate: f64,
    pub suite: EvaluationSuite,
    pub delta_t: usize,
    pub t_warmup: usize,
    pub leak: f64,
    pub n_ref: usize,
    pub alpha_rho: f64,
    pub rho_target: f64,
    pub beta_theta: f64,
    pub theta_min: f64,
    pub theta_max: f64,
    pub theta_floor: f64,
    pub theta_init: f64,
    pub output_dir: String,
    pub num_seeds: usize,
}

impl ExperimentConfig {
    pub fn new(condition: Condition, noise_rate: f64, seed: u64) -> Self {
        let n_ref = match condition {
            Condition::AblationUnshieldedFeedback => 0,
            _ => 2,
        };

        let beta_theta = match condition {
            Condition::AblationFixedTheta => 0.0,
            _ => 0.05,
        };

        let (theta_init, theta_floor) = match condition {
            Condition::AblationFixedTheta => (1.00, 0.50),
            _ => (1.05, 1.02),
        };

        let run_id = format!(
            "RUN-EXP-2026-008a-{}-N{:.2}-S{:02}",
            condition.short_code(),
            noise_rate,
            seed
        );

        Self {
            run_id,
            seed,
            condition,
            noise_rate,
            suite: EvaluationSuite::QuiescentRetentionSweep,
            delta_t: 1000,
            t_warmup: 50,
            leak: 0.10,
            n_ref,
            alpha_rho: 0.02,
            rho_target: 0.10,
            beta_theta,
            theta_min: 0.50,
            theta_max: 2.50,
            theta_floor,
            theta_init,
            output_dir: "data/telemetry/EXP-2026-008a".to_string(),
            num_seeds: 30,
        }
    }

    pub fn new_transition(condition: Condition, seed: u64) -> Self {
        let mut cfg = Self::new(condition, 0.0, seed);
        cfg.suite = EvaluationSuite::StateTransitionSuite;
        cfg.run_id = format!(
            "RUN-EXP-2026-008a-TRANS-{}-S{:02}",
            condition.short_code(),
            seed
        );
        cfg
    }
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self::new(Condition::ActiveLatch, 0.0, 1)
    }
}
