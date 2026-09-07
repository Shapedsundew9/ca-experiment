//! Configuration module for EXP-2026-011a: Associative Key-Value Retrieval ("Needle in a Haystack").

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    ActiveAssociative,
    BaselineUnindexed,
    AblationUnshieldedRetrieval,
    AblationFixedTheta,
}

impl Condition {
    pub const ALL: [Condition; 4] = [
        Condition::ActiveAssociative,
        Condition::BaselineUnindexed,
        Condition::AblationUnshieldedRetrieval,
        Condition::AblationFixedTheta,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Condition::ActiveAssociative => "active_associative",
            Condition::BaselineUnindexed => "baseline_unindexed",
            Condition::AblationUnshieldedRetrieval => "ablation_unshielded_retrieval",
            Condition::AblationFixedTheta => "ablation_fixed_theta",
        }
    }

    pub fn short_code(&self) -> &'static str {
        match self {
            Condition::ActiveAssociative => "ASSOC",
            Condition::BaselineUnindexed => "UNIDX",
            Condition::AblationUnshieldedRetrieval => "UNSHIELD",
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
            "active_associative" | "assoc" | "active" => Ok(Condition::ActiveAssociative),
            "baseline_unindexed" | "unindexed" | "unidx" => Ok(Condition::BaselineUnindexed),
            "ablation_unshielded_retrieval" | "unshield" | "unshielded" => {
                Ok(Condition::AblationUnshieldedRetrieval)
            }
            "ablation_fixed_theta" | "fixed_theta" | "fix" => Ok(Condition::AblationFixedTheta),
            _ => Err(format!("Unknown condition: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub run_id: String,
    pub seed: u64,
    pub condition: Condition,
    pub noise_rate: f64,
    pub seq_length: usize,
    pub t_token: usize,
    pub n_slots: usize,
    pub leak: f64,
    pub n_ref: usize,
    pub alpha_rho: f64,
    pub rho_target: f64,
    pub beta_theta: f64,
    pub theta_min: f64,
    pub theta_max: f64,
    pub theta_floor: f64,
    pub theta_init: f64,
    pub theta_gate_init: f64,
    pub w_fwd: f64,
    pub w_fb: f64,
    pub w_inh: f64,
    pub w_key: f64,
    pub w_val: f64,
    pub w_query: f64,
    pub w_sense: f64,
    pub w_dist: f64,
    pub output_dir: String,
    pub num_seeds: usize,
    pub sequences_per_run: usize,
}

impl ExperimentConfig {
    pub fn new(condition: Condition, noise_rate: f64, seq_length: usize, seed: u64) -> Self {
        let n_ref = match condition {
            Condition::AblationUnshieldedRetrieval => 0,
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
            _ => 1.00,
        };

        let theta_gate_init = match condition {
            Condition::AblationFixedTheta => 1.00,
            _ => 1.10,
        };

        let w_fwd = match condition {
            Condition::AblationFixedTheta => 1.00,
            _ => 1.10,
        };
        let w_fb = w_fwd;

        Self {
            run_id: format!(
                "EXP-2026-011a-{}-L{}-N{}-s{}",
                condition.short_code(),
                seq_length,
                (noise_rate * 100.0).round() as u64,
                seed
            ),
            seed,
            condition,
            noise_rate,
            seq_length,
            t_token: 16,
            n_slots: 16,
            leak: 0.05,
            n_ref,
            alpha_rho: 0.02,
            rho_target: 0.10,
            beta_theta,
            theta_min: 0.50,
            theta_max: 2.50,
            theta_floor,
            theta_init,
            theta_gate_init,
            w_fwd,
            w_fb,
            w_inh: -1.50,
            w_key: 0.60,
            w_val: 0.60,
            w_query: 0.60,
            w_sense: 0.60,
            w_dist: 0.30,
            output_dir: "data/telemetry/EXP-2026-011a".to_string(),
            num_seeds: 30,
            sequences_per_run: 50,
        }
    }
}
