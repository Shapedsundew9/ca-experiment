//! Configuration module for EXP-2026-006a: Directed Regenerative Transmission Tracks and Branching Fan-Out.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    ActiveRegenerative,
    BaselinePassive,
    AblationUnshielded,
    AblationFixedTheta,
}

impl Condition {
    pub const ALL: [Condition; 4] = [
        Condition::ActiveRegenerative,
        Condition::BaselinePassive,
        Condition::AblationUnshielded,
        Condition::AblationFixedTheta,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Condition::ActiveRegenerative => "active_regenerative",
            Condition::BaselinePassive => "baseline_passive",
            Condition::AblationUnshielded => "ablation_unshielded",
            Condition::AblationFixedTheta => "ablation_fixed_theta",
        }
    }

    pub fn short_code(&self) -> &'static str {
        match self {
            Condition::ActiveRegenerative => "ACT",
            Condition::BaselinePassive => "PAS",
            Condition::AblationUnshielded => "UNS",
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
            "active_regenerative" | "act" => Ok(Condition::ActiveRegenerative),
            "baseline_passive" | "pas" => Ok(Condition::BaselinePassive),
            "ablation_unshielded" | "uns" => Ok(Condition::AblationUnshielded),
            "ablation_fixed_theta" | "fix" => Ok(Condition::AblationFixedTheta),
            _ => Err(format!("Unknown condition: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalPattern {
    SingleImpulse,
    AlternatingClock,
    BurstTrain,
    PseudoRandom,
}

impl SignalPattern {
    pub const ALL: [SignalPattern; 4] = [
        SignalPattern::SingleImpulse,
        SignalPattern::AlternatingClock,
        SignalPattern::BurstTrain,
        SignalPattern::PseudoRandom,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            SignalPattern::SingleImpulse => "single_impulse",
            SignalPattern::AlternatingClock => "alternating_clock",
            SignalPattern::BurstTrain => "burst_train",
            SignalPattern::PseudoRandom => "pseudo_random",
        }
    }

    pub fn short_code(&self) -> &'static str {
        match self {
            SignalPattern::SingleImpulse => "PUL",
            SignalPattern::AlternatingClock => "CLK",
            SignalPattern::BurstTrain => "BST",
            SignalPattern::PseudoRandom => "RND",
        }
    }
}

impl fmt::Display for SignalPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for SignalPattern {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "single_impulse" | "pul" => Ok(SignalPattern::SingleImpulse),
            "alternating_clock" | "clk" => Ok(SignalPattern::AlternatingClock),
            "burst_train" | "bst" => Ok(SignalPattern::BurstTrain),
            "pseudo_random" | "rnd" => Ok(SignalPattern::PseudoRandom),
            _ => Err(format!("Unknown pattern: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub run_id: String,
    pub seed: u64,
    pub condition: Condition,
    pub distance: usize,
    pub jitter: usize,
    pub noise_rate: f64,
    pub pattern: SignalPattern,
    pub leak: f64,
    pub n_ref: usize,
    pub alpha_rho: f64,
    pub rho_target: f64,
    pub beta_theta: f64,
    pub theta_min: f64,
    pub theta_max: f64,
    pub theta_init: f64,
    pub w_fwd: f64,
    pub w_back: f64,
    pub t_warmup: usize,
    pub t_eval: usize,
    pub is_crosstalk_test: bool,
    pub output_dir: String,
    pub num_seeds: usize,
}

impl ExperimentConfig {
    pub fn new(
        condition: Condition,
        distance: usize,
        jitter: usize,
        noise_rate: f64,
        pattern: SignalPattern,
        seed: u64,
    ) -> Self {
        let n_ref = match condition {
            Condition::AblationUnshielded => 0,
            _ => 2,
        };

        let beta_theta = match condition {
            Condition::AblationFixedTheta => 0.0,
            _ => 0.05,
        };

        let w_back = match condition {
            Condition::AblationUnshielded => 1.0,
            _ => 0.0,
        };

        let run_id = format!(
            "RUN-EXP-2026-006a-{}-D{}-J{}-N{:.2}-{}-S{:02}",
            condition.short_code(),
            distance,
            jitter,
            noise_rate,
            pattern.short_code(),
            seed
        );

        Self {
            run_id,
            seed,
            condition,
            distance,
            jitter,
            noise_rate,
            pattern,
            leak: 0.10,
            n_ref,
            alpha_rho: 0.02,
            rho_target: 0.10,
            beta_theta,
            theta_min: 0.50,
            theta_max: 3.00,
            theta_init: 1.00,
            w_fwd: 1.00,
            w_back,
            t_warmup: 50,
            t_eval: 200,
            is_crosstalk_test: false,
            output_dir: "data/telemetry/EXP-2026-006a".to_string(),
            num_seeds: 30,
        }
    }

    pub fn new_crosstalk(condition: Condition, seed: u64) -> Self {
        let mut cfg = Self::new(condition, 30, 0, 0.0, SignalPattern::AlternatingClock, seed);
        cfg.is_crosstalk_test = true;
        cfg.run_id = format!(
            "RUN-EXP-2026-006a-XTLK-{}-S{:02}",
            condition.short_code(),
            seed
        );
        cfg
    }
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self::new(
            Condition::ActiveRegenerative,
            30,
            0,
            0.0,
            SignalPattern::AlternatingClock,
            1,
        )
    }
}
