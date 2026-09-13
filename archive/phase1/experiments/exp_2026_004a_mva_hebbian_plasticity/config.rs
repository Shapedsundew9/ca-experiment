//! Configuration structures and condition types for EXP-2026-004a Hebbian plasticity experiment.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlasticityCondition {
    ActiveHebbian,
    BaselineStatic,
    AblationRandomDrift,
    AblationAntiHebbian,
    NovelPatternControl,
}

impl PlasticityCondition {
    pub const ALL: [PlasticityCondition; 5] = [
        PlasticityCondition::ActiveHebbian,
        PlasticityCondition::BaselineStatic,
        PlasticityCondition::AblationRandomDrift,
        PlasticityCondition::AblationAntiHebbian,
        PlasticityCondition::NovelPatternControl,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            PlasticityCondition::ActiveHebbian => "active_hebbian",
            PlasticityCondition::BaselineStatic => "baseline_static",
            PlasticityCondition::AblationRandomDrift => "ablation_random_drift",
            PlasticityCondition::AblationAntiHebbian => "ablation_anti_hebbian",
            PlasticityCondition::NovelPatternControl => "novel_pattern_control",
        }
    }
}

impl std::str::FromStr for PlasticityCondition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "active_hebbian" | "hebbian" => Ok(PlasticityCondition::ActiveHebbian),
            "baseline_static" | "static" => Ok(PlasticityCondition::BaselineStatic),
            "ablation_random_drift" | "random_drift" | "drift" => {
                Ok(PlasticityCondition::AblationRandomDrift)
            }
            "ablation_anti_hebbian" | "anti_hebbian" => {
                Ok(PlasticityCondition::AblationAntiHebbian)
            }
            "novel_pattern_control" | "novel_pattern" | "novel" => {
                Ok(PlasticityCondition::NovelPatternControl)
            }
            other => Err(format!("Unknown condition: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternId {
    A,
    B,
    C,
    D,
}

impl PatternId {
    pub const ALL: [PatternId; 4] = [PatternId::A, PatternId::B, PatternId::C, PatternId::D];

    pub fn name(&self) -> &'static str {
        match self {
            PatternId::A => "A",
            PatternId::B => "B",
            PatternId::C => "C",
            PatternId::D => "D",
        }
    }

    pub fn bits(&self) -> u32 {
        match self {
            PatternId::A => 0xCA69_5A53,
            PatternId::B => 0xB4CA_9665,
            PatternId::C => 0x69A5_CA9C,
            PatternId::D => 0x5C6A_A6C5,
        }
    }
}

impl std::str::FromStr for PatternId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "A" => Ok(PatternId::A),
            "B" => Ok(PatternId::B),
            "C" => Ok(PatternId::C),
            "D" => Ok(PatternId::D),
            other => Err(format!("Unknown pattern: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub seed: u64,
    pub condition: PlasticityCondition,
    pub pattern: PatternId,
    pub plasticity_rate: f64,
    pub exposure_ticks: usize,
    pub noise_level: f64,
    pub leak: f64,
    pub n_ref: usize,
    pub target_rate: f64,
    pub homeo_rate: f64,
    pub decay_factor: f64,
    pub weight_budget: f64,
    pub weight_max: f64,
    pub drive_ticks: usize,
    pub relax_ticks: usize,
    pub sample_window: usize,
    pub v_min: f64,
    pub v_max: f64,
    pub v_init: f64,
    pub alpha_ema: f64,
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            condition: PlasticityCondition::ActiveHebbian,
            pattern: PatternId::A,
            plasticity_rate: 0.01,
            exposure_ticks: 1000,
            noise_level: 0.05,
            leak: 0.05,
            n_ref: 2,
            target_rate: 0.12,
            homeo_rate: 0.02,
            decay_factor: 0.12,
            weight_budget: 4.0,
            weight_max: 2.5,
            drive_ticks: 32,
            relax_ticks: 200,
            sample_window: 128,
            v_min: 0.50,
            v_max: 3.5,
            v_init: 1.15,
            alpha_ema: 0.01,
        }
    }
}
