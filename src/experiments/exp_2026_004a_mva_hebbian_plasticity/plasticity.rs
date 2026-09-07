//! Hebbian plasticity engine, synaptic scaling, and weight matrix updates for EXP-2026-004a.

use super::config::{ExperimentConfig, PlasticityCondition};
use super::patterns::FastRng;

pub const NUM_NODES: usize = 16;

#[derive(Debug, Clone)]
pub struct PlasticityEngine {
    /// Directed weight matrix: W[i][j] is weight of track from j to i
    pub weights: [[f64; NUM_NODES]; NUM_NODES],
    pub neighbors_in: [[usize; 4]; NUM_NODES],
    pub weight_budget: f64,
    pub weight_max: f64,
    pub decay_factor: f64,
}

impl PlasticityEngine {
    pub fn new(
        neighbors_in: [[usize; 4]; NUM_NODES],
        weight_budget: f64,
        weight_max: f64,
        decay_factor: f64,
    ) -> Self {
        let mut weights = [[0.0; NUM_NODES]; NUM_NODES];
        // Initialize active tracks to uniform 1.0
        for i in 0..NUM_NODES {
            for &j in &neighbors_in[i] {
                weights[i][j] = 1.0;
            }
        }

        Self {
            weights,
            neighbors_in,
            weight_budget,
            weight_max,
            decay_factor,
        }
    }

    /// Update synaptic weights based on spike-timing correlations
    #[allow(clippy::needless_range_loop)]
    pub fn update_weights(
        &mut self,
        config: &ExperimentConfig,
        curr_spikes: &[f64; NUM_NODES],
        prev_spikes: &[f64; NUM_NODES],
        rng: &mut FastRng,
    ) {
        if config.condition == PlasticityCondition::BaselineStatic {
            return;
        }

        let eta = config.plasticity_rate;

        for i in 0..NUM_NODES {
            let post_spike = curr_spikes[i];

            for &j in &self.neighbors_in[i] {
                let pre_spike = prev_spikes[j];
                let w = self.weights[i][j];

                let delta = match config.condition {
                    PlasticityCondition::ActiveHebbian
                    | PlasticityCondition::NovelPatternControl => {
                        eta * (post_spike * pre_spike - self.decay_factor * post_spike)
                    }
                    PlasticityCondition::AblationAntiHebbian => {
                        -eta * (post_spike * pre_spike - self.decay_factor * post_spike)
                    }
                    PlasticityCondition::AblationRandomDrift => {
                        let sigma = eta * 0.5;
                        sigma * rng.next_gaussian()
                    }
                    PlasticityCondition::BaselineStatic => 0.0,
                };

                self.weights[i][j] = (w + delta).clamp(0.0, self.weight_max);
            }

            // Synaptic scaling: constrain total incoming weight to weight_budget
            let mut sum_in = 0.0;
            for &j in &self.neighbors_in[i] {
                sum_in += self.weights[i][j];
            }

            if sum_in > self.weight_budget && sum_in > 1e-12 {
                let scale = self.weight_budget / sum_in;
                for &j in &self.neighbors_in[i] {
                    self.weights[i][j] *= scale;
                }
            }
        }
    }

    /// Compute summary statistics of current weight distribution
    pub fn compute_weight_stats(&self) -> (f64, f64, bool) {
        let mut active_weights = Vec::with_capacity(NUM_NODES * 4);
        let mut budget_conserved = true;

        for i in 0..NUM_NODES {
            let mut sum_in = 0.0;
            for &j in &self.neighbors_in[i] {
                let w = self.weights[i][j];
                active_weights.push(w);
                sum_in += w;
            }
            if sum_in > self.weight_budget + 1e-4 {
                budget_conserved = false;
            }
        }

        let n = active_weights.len() as f64;
        let mean = active_weights.iter().sum::<f64>() / n;
        let variance = active_weights
            .iter()
            .map(|w| (w - mean).powi(2))
            .sum::<f64>()
            / n;
        let peak = active_weights.iter().copied().fold(0.0f64, f64::max);

        (variance, peak, budget_conserved)
    }
}
