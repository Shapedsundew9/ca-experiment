//! Configuration module for EXP-2026-007a: Cascaded Boolean Logic Gate Composition and Planar Wire Crossing.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    ActiveComposed,
    BaselineUncompensatedDelay,
    AblationUnshieldedCrossing,
    AblationFixedTheta,
}

impl Condition {
    pub const ALL: [Condition; 4] = [
        Condition::ActiveComposed,
        Condition::BaselineUncompensatedDelay,
        Condition::AblationUnshieldedCrossing,
        Condition::AblationFixedTheta,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Condition::ActiveComposed => "active_composed",
            Condition::BaselineUncompensatedDelay => "baseline_uncompensated_delay",
            Condition::AblationUnshieldedCrossing => "ablation_unshielded_crossing",
            Condition::AblationFixedTheta => "ablation_fixed_theta",
        }
    }

    pub fn short_code(&self) -> &'static str {
        match self {
            Condition::ActiveComposed => "ACT",
            Condition::BaselineUncompensatedDelay => "UNCOMP",
            Condition::AblationUnshieldedCrossing => "UNSHIELD",
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
            "active_composed" | "act" => Ok(Condition::ActiveComposed),
            "baseline_uncompensated_delay" | "uncomp" | "nodelay" => {
                Ok(Condition::BaselineUncompensatedDelay)
            }
            "ablation_unshielded_crossing" | "unshield" | "uns" => {
                Ok(Condition::AblationUnshieldedCrossing)
            }
            "ablation_fixed_theta" | "fix" => Ok(Condition::AblationFixedTheta),
            _ => Err(format!("Unknown condition: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationMode {
    TruthTableSweep,
    CrosstalkProbe,
}

impl EvaluationMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            EvaluationMode::TruthTableSweep => "truth_table_sweep",
            EvaluationMode::CrosstalkProbe => "crosstalk_probe",
        }
    }
}

impl fmt::Display for EvaluationMode {
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
    pub mode: EvaluationMode,
    pub leak: f64,
    pub n_ref: usize,
    pub alpha_rho: f64,
    pub rho_target: f64,
    pub beta_theta: f64,
    pub theta_min: f64,
    pub theta_max: f64,
    pub theta_floor: f64,
    pub theta_init: f64,
    pub epochs_per_state: usize,
    pub t_warmup: usize,
    pub output_dir: String,
    pub num_seeds: usize,
}

impl ExperimentConfig {
    pub fn new(condition: Condition, noise_rate: f64, seed: u64) -> Self {
        let n_ref = 2;

        let beta_theta = match condition {
            Condition::AblationFixedTheta => 0.0,
            _ => 0.05,
        };

        let (theta_init, theta_floor) = match condition {
            Condition::ActiveComposed => (1.05, 1.02),
            Condition::AblationFixedTheta => (1.00, 0.50),
            _ => (1.05, 1.00),
        };

        let run_id = format!(
            "RUN-EXP-2026-007a-{}-N{:.2}-S{:02}",
            condition.short_code(),
            noise_rate,
            seed
        );

        Self {
            run_id,
            seed,
            condition,
            noise_rate,
            mode: EvaluationMode::TruthTableSweep,
            leak: 0.10,
            n_ref,
            alpha_rho: 0.02,
            rho_target: 0.10,
            beta_theta,
            theta_min: 0.50,
            theta_max: 2.50,
            theta_floor,
            theta_init,
            epochs_per_state: 50,
            t_warmup: 50,
            output_dir: "data/telemetry/EXP-2026-007a".to_string(),
            num_seeds: 30,
        }
    }

    pub fn new_crosstalk(condition: Condition, seed: u64) -> Self {
        let mut cfg = Self::new(condition, 0.0, seed);
        cfg.mode = EvaluationMode::CrosstalkProbe;
        cfg.run_id = format!(
            "RUN-EXP-2026-007a-XTLK-{}-S{:02}",
            condition.short_code(),
            seed
        );
        cfg
    }
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self::new(Condition::ActiveComposed, 0.0, 1)
    }
}
