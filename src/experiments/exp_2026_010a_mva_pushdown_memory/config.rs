//! Configuration module for EXP-2026-010a: Pushdown Memory and Context-Free Dyck Language Recognition.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    ActivePda,
    BaselineFiniteState,
    AblationUnshieldedStack,
    AblationFixedTheta,
}

impl Condition {
    pub const ALL: [Condition; 4] = [
        Condition::ActivePda,
        Condition::BaselineFiniteState,
        Condition::AblationUnshieldedStack,
        Condition::AblationFixedTheta,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Condition::ActivePda => "active_pda",
            Condition::BaselineFiniteState => "baseline_finite_state",
            Condition::AblationUnshieldedStack => "ablation_unshielded_stack",
            Condition::AblationFixedTheta => "ablation_fixed_theta",
        }
    }

    pub fn short_code(&self) -> &'static str {
        match self {
            Condition::ActivePda => "PDA",
            Condition::BaselineFiniteState => "FSM",
            Condition::AblationUnshieldedStack => "UNSHIELD",
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
            "active_pda" | "pda" | "active" => Ok(Condition::ActivePda),
            "baseline_finite_state" | "fsm" | "finite_state" => Ok(Condition::BaselineFiniteState),
            "ablation_unshielded_stack" | "unshield" | "unshielded" => {
                Ok(Condition::AblationUnshieldedStack)
            }
            "ablation_fixed_theta" | "fix" | "fixed_theta" => Ok(Condition::AblationFixedTheta),
            _ => Err(format!("Unknown condition: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomatonTask {
    Dyck1,
    Dyck2,
    DepthGeneralization,
}

impl AutomatonTask {
    pub const ALL: [AutomatonTask; 3] = [
        AutomatonTask::Dyck1,
        AutomatonTask::Dyck2,
        AutomatonTask::DepthGeneralization,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            AutomatonTask::Dyck1 => "dyck_1",
            AutomatonTask::Dyck2 => "dyck_2",
            AutomatonTask::DepthGeneralization => "depth_generalization",
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
            "dyck_1" | "dyck1" => Ok(AutomatonTask::Dyck1),
            "dyck_2" | "dyck2" => Ok(AutomatonTask::Dyck2),
            "depth_generalization" | "depth" => Ok(AutomatonTask::DepthGeneralization),
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
    pub max_depth: usize,
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
        let max_depth = match condition {
            Condition::BaselineFiniteState => 2,
            _ => 8,
        };

        let n_ref = match condition {
            Condition::AblationUnshieldedStack => 0,
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
                "EXP-2026-010a-{}-{}-{}-s{}",
                condition.short_code(),
                task.as_str(),
                (noise_rate * 100.0).round() as u64,
                seed
            ),
            seed,
            condition,
            task,
            noise_rate,
            seq_length: 16,
            t_token: 16,
            t_warmup: 16,
            max_depth,
            leak: 0.05,
            n_ref,
            alpha_rho: 0.01,
            rho_target: 0.08,
            beta_theta,
            theta_min: 0.50,
            theta_max: 2.50,
            theta_floor,
            theta_init,
            output_dir: "data/telemetry/EXP-2026-010a".to_string(),
            num_seeds: 30,
        }
    }
}
