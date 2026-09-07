//! Configuration module for EXP-2026-009a: Finite State Automata and Regular Language Recognition.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    ActiveDfa,
    BaselineFeedforwardLoss,
    AblationUnshieldedFeedback,
    AblationFixedTheta,
    BaselineMemoryless,
}

impl Condition {
    pub const ALL: [Condition; 5] = [
        Condition::ActiveDfa,
        Condition::BaselineFeedforwardLoss,
        Condition::AblationUnshieldedFeedback,
        Condition::AblationFixedTheta,
        Condition::BaselineMemoryless,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Condition::ActiveDfa => "active_dfa",
            Condition::BaselineFeedforwardLoss => "baseline_feedforward_loss",
            Condition::AblationUnshieldedFeedback => "ablation_unshielded_feedback",
            Condition::AblationFixedTheta => "ablation_fixed_theta",
            Condition::BaselineMemoryless => "baseline_memoryless",
        }
    }

    pub fn short_code(&self) -> &'static str {
        match self {
            Condition::ActiveDfa => "ACT",
            Condition::BaselineFeedforwardLoss => "LOSS",
            Condition::AblationUnshieldedFeedback => "UNSHIELD",
            Condition::AblationFixedTheta => "FIX",
            Condition::BaselineMemoryless => "MEMLESS",
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
            "active_dfa" | "act" => Ok(Condition::ActiveDfa),
            "baseline_feedforward_loss" | "ff_loss" | "loss" => {
                Ok(Condition::BaselineFeedforwardLoss)
            }
            "ablation_unshielded_feedback" | "unshield" | "uns" => {
                Ok(Condition::AblationUnshieldedFeedback)
            }
            "ablation_fixed_theta" | "fix" => Ok(Condition::AblationFixedTheta),
            "baseline_memoryless" | "memless" => Ok(Condition::BaselineMemoryless),
            _ => Err(format!("Unknown condition: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomatonTask {
    ParityDfa,
    RegexDfa,
}

impl AutomatonTask {
    pub const ALL: [AutomatonTask; 2] = [AutomatonTask::ParityDfa, AutomatonTask::RegexDfa];

    pub fn as_str(&self) -> &'static str {
        match self {
            AutomatonTask::ParityDfa => "parity_dfa",
            AutomatonTask::RegexDfa => "regex_dfa",
        }
    }
}

impl fmt::Display for AutomatonTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for AutomatonTask {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "parity_dfa" | "parity" => Ok(AutomatonTask::ParityDfa),
            "regex_dfa" | "regex" => Ok(AutomatonTask::RegexDfa),
            _ => Err(format!("Unknown task: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub run_id: String,
    pub seed: u64,
    pub condition: Condition,
    pub task: AutomatonTask,
    pub noise_rate: f64,
    pub seq_length: usize,
    pub t_token: usize,
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
    pub fn new(condition: Condition, task: AutomatonTask, noise_rate: f64, seed: u64) -> Self {
        let n_ref = match condition {
            Condition::AblationUnshieldedFeedback => 0,
            _ => 2,
        };

        let beta_theta = match condition {
            Condition::AblationFixedTheta => 0.0,
            _ => 0.05,
        };

        let theta_floor = match condition {
            Condition::AblationFixedTheta => 0.50,
            _ => 1.02,
        };

        let theta_init = match condition {
            Condition::AblationFixedTheta => 1.00,
            _ => 1.05,
        };

        Self {
            run_id: format!(
                "EXP-2026-009a-{}-{}-s{}",
                condition.short_code(),
                task.as_str(),
                seed
            ),
            seed,
            condition,
            task,
            noise_rate,
            seq_length: 10,
            t_token: 16,
            t_warmup: 16,
            leak: 0.10,
            n_ref,
            alpha_rho: 0.02,
            rho_target: 0.10,
            beta_theta,
            theta_min: 0.50,
            theta_max: 2.50,
            theta_floor,
            theta_init,
            output_dir: "data/telemetry/EXP-2026-009a".to_string(),
            num_seeds: 30,
        }
    }
}
