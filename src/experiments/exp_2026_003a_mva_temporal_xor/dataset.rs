//! Streaming dataset and temporal target generator for EXP-2026-003a.

use super::ca::FastRng;
use super::config::{ExperimentConfig, ExperimentalCondition};

#[derive(Debug, Clone)]
pub struct TemporalStream {
    pub inputs: Vec<f64>,
    pub targets: Vec<f64>,
}

impl TemporalStream {
    pub fn generate(config: &ExperimentConfig, total_ticks: usize) -> Self {
        let mut rng = FastRng::seed_from_u64(config.seed);
        let mut inputs = Vec::with_capacity(total_ticks);
        for _ in 0..total_ticks {
            let bit = if rng.bernoulli(config.pulse_density) {
                1.0
            } else {
                0.0
            };
            inputs.push(bit);
        }

        let mut targets = vec![0.0; total_ticks];
        for t in 0..total_ticks {
            if t >= config.delay_tau {
                match config.condition {
                    ExperimentalCondition::ActiveIdentity => {
                        targets[t] = inputs[t - config.delay_tau];
                    }
                    _ => {
                        let bit_now = inputs[t] as usize;
                        let bit_past = inputs[t - config.delay_tau] as usize;
                        targets[t] = (bit_now ^ bit_past) as f64;
                    }
                }
            } else {
                targets[t] = 0.0;
            }
        }

        Self { inputs, targets }
    }
}
