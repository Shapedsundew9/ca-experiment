//! Data structures and configurations for EXP-2026-002a attractor mapping.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputPatternId {
    PatternA,
    PatternB,
    PatternC,
    PatternD,
    Noise,
}

impl InputPatternId {
    pub const ALL: [InputPatternId; 5] = [
        InputPatternId::PatternA,
        InputPatternId::PatternB,
        InputPatternId::PatternC,
        InputPatternId::PatternD,
        InputPatternId::Noise,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            InputPatternId::PatternA => "Pattern_A",
            InputPatternId::PatternB => "Pattern_B",
            InputPatternId::PatternC => "Pattern_C",
            InputPatternId::PatternD => "Pattern_D",
            InputPatternId::Noise => "Noise",
        }
    }
}

impl std::str::FromStr for InputPatternId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('_', "").as_str() {
            "patterna" | "a" => Ok(InputPatternId::PatternA),
            "patternb" | "b" => Ok(InputPatternId::PatternB),
            "patternc" | "c" => Ok(InputPatternId::PatternC),
            "patternd" | "d" => Ok(InputPatternId::PatternD),
            "noise" => Ok(InputPatternId::Noise),
            other => Err(format!("Unknown pattern: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HomeostaticMode {
    Active,
    Fixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub pattern: InputPatternId,
    pub t_drive: usize,
    pub t_relax: usize,
    pub noise_rate: f64,
    pub mode: HomeostaticMode,
    pub seed: u64,
    pub n_ref: usize,
    pub leak: f64,
    pub r_target: f64,
    pub eta: f64,
    pub v_init: f64,
    pub v_min: f64,
    pub v_max: f64,
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self {
            pattern: InputPatternId::PatternA,
            t_drive: 32,
            t_relax: 500,
            noise_rate: 0.0,
            mode: HomeostaticMode::Active,
            seed: 1,
            n_ref: 2,
            leak: 0.05,
            r_target: 0.12,
            eta: 0.02,
            v_init: 1.15,
            v_min: 0.5,
            v_max: 3.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummaryRecord {
    pub run_id: String,
    pub pattern: String,
    pub t_relax: usize,
    pub noise: f64,
    pub mode: String,
    pub seed: u64,
    pub period: usize,
    pub settling_time: usize,
    pub mean_firing_rate: f64,
    pub mean_threshold: f64,
    pub cycle_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotRecord {
    pub run_id: String,
    pub tick: usize,
    pub spikes: u16,
}
