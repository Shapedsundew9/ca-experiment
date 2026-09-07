//! Adaptation modes for homeostatic threshold regulation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdaptationMode {
    Active,
    Fixed,
    RandomDrift,
    Inverted,
}

impl std::str::FromStr for AdaptationMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(AdaptationMode::Active),
            "fixed" => Ok(AdaptationMode::Fixed),
            "random_drift" | "random-drift" | "randomdrift" => Ok(AdaptationMode::RandomDrift),
            "inverted" => Ok(AdaptationMode::Inverted),
            other => Err(format!("Unknown adaptation mode: {other}")),
        }
    }
}

impl AdaptationMode {
    #[inline]
    pub fn compute_delta(
        &self,
        rolling_rate: f64,
        target_rate: f64,
        eta: f64,
        coin_flip: f64,
    ) -> f64 {
        match self {
            AdaptationMode::Active => eta * (rolling_rate - target_rate),
            AdaptationMode::Fixed => 0.0,
            AdaptationMode::RandomDrift => eta * coin_flip,
            AdaptationMode::Inverted => -eta * (rolling_rate - target_rate),
        }
    }
}
