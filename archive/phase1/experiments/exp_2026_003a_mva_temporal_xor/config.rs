//! Configuration structures and condition types for EXP-2026-003a temporal XOR benchmark.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExperimentalCondition {
    ActiveXor,
    ActiveIdentity,
    AblationMemoryless,
    AblationStaticThresh,
    BaselineLinearDirect,
}

impl ExperimentalCondition {
    pub const ALL: [ExperimentalCondition; 5] = [
        ExperimentalCondition::ActiveXor,
        ExperimentalCondition::ActiveIdentity,
        ExperimentalCondition::AblationMemoryless,
        ExperimentalCondition::AblationStaticThresh,
        ExperimentalCondition::BaselineLinearDirect,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            ExperimentalCondition::ActiveXor => "active_xor",
            ExperimentalCondition::ActiveIdentity => "active_identity",
            ExperimentalCondition::AblationMemoryless => "ablation_memoryless",
            ExperimentalCondition::AblationStaticThresh => "ablation_static_thresh",
            ExperimentalCondition::BaselineLinearDirect => "baseline_linear_direct",
        }
    }
}

impl std::str::FromStr for ExperimentalCondition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "active_xor" | "xor" => Ok(ExperimentalCondition::ActiveXor),
            "active_identity" | "identity" => Ok(ExperimentalCondition::ActiveIdentity),
            "ablation_memoryless" | "memoryless" => Ok(ExperimentalCondition::AblationMemoryless),
            "ablation_static_thresh" | "static" | "static_thresh" => {
                Ok(ExperimentalCondition::AblationStaticThresh)
            }
            "baseline_linear_direct" | "baseline" | "direct" => {
                Ok(ExperimentalCondition::BaselineLinearDirect)
            }
            other => Err(format!("Unknown condition: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub seed: u64,
    pub condition: ExperimentalCondition,
    pub delay_tau: usize,
    pub pulse_density: f64,
    pub ridge_alpha: f64,
    pub washout_ticks: usize,
    pub train_ticks: usize,
    pub test_ticks: usize,
    pub leak: f64,
    pub n_ref: usize,
    pub target_rate: f64,
    pub eta: f64,
    pub v_min: f64,
    pub v_max: f64,
    pub v_init: f64,
    pub alpha_ema: f64,
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            condition: ExperimentalCondition::ActiveXor,
            delay_tau: 5,
            pulse_density: 0.20,
            ridge_alpha: 0.01,
            washout_ticks: 200,
            train_ticks: 2000,
            test_ticks: 1000,
            leak: 0.05,
            n_ref: 2,
            target_rate: 0.12,
            eta: 0.02,
            v_min: 0.50,
            v_max: 3.0,
            v_init: 1.15,
            alpha_ema: 0.01,
        }
    }
}
