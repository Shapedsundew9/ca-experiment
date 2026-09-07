//! 16-node Torus CA substrate with directed weights and homeostatic regulation for EXP-2026-004a.

use super::config::ExperimentConfig;
use super::patterns::FastRng;
use super::plasticity::{NUM_NODES, PlasticityEngine};

pub const LATTICE_WIDTH: usize = 4;
pub const LATTICE_HEIGHT: usize = 4;

#[derive(Debug, Clone)]
pub struct TorusNode {
    pub id: usize,
    pub v: f64,
    pub refractory_counter: usize,
    pub rolling_rate: f64,
    pub v_thresh: f64,
    pub spike: f64,
}

impl TorusNode {
    pub fn new(id: usize, v_thresh_init: f64) -> Self {
        Self {
            id,
            v: 0.0,
            refractory_counter: 0,
            rolling_rate: 0.0,
            v_thresh: v_thresh_init,
            spike: 0.0,
        }
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn step(
        &mut self,
        incoming_charge: f64,
        leak: f64,
        n_ref: usize,
        homeo_rate: f64,
        target_rate: f64,
        v_min: f64,
        v_max: f64,
        alpha_ema: f64,
    ) {
        let spike: f64;
        if self.refractory_counter > 0 {
            spike = 0.0;
            self.v *= 1.0 - leak;
            self.refractory_counter -= 1;
        } else {
            let v_pre = (1.0 - leak) * self.v + incoming_charge;
            if v_pre >= self.v_thresh {
                spike = 1.0;
                self.v = 0.0;
                self.refractory_counter = n_ref;
            } else {
                spike = 0.0;
                self.v = v_pre;
                self.refractory_counter = 0;
            }
        }
        self.spike = spike;

        self.rolling_rate = (1.0 - alpha_ema) * self.rolling_rate + alpha_ema * spike;
        if homeo_rate > 0.0 {
            let delta = homeo_rate * (self.rolling_rate - target_rate);
            self.v_thresh = (self.v_thresh + delta).clamp(v_min, v_max);
        }
    }
}

pub struct TorusSubstrate {
    pub config: ExperimentConfig,
    pub nodes: [TorusNode; NUM_NODES],
    pub neighbors_in: [[usize; 4]; NUM_NODES],
    pub prev_spikes: [f64; NUM_NODES],
    pub plasticity: PlasticityEngine,
    pub rng: FastRng,
}

impl TorusSubstrate {
    pub fn new(config: ExperimentConfig) -> Self {
        let mut neighbors_in = [[0usize; 4]; NUM_NODES];
        for y in 0..LATTICE_HEIGHT {
            for x in 0..LATTICE_WIDTH {
                let idx = y * LATTICE_WIDTH + x;
                let north = ((y + 3) % LATTICE_HEIGHT) * LATTICE_WIDTH + x;
                let south = ((y + 1) % LATTICE_HEIGHT) * LATTICE_WIDTH + x;
                let east = y * LATTICE_WIDTH + ((x + 1) % LATTICE_WIDTH);
                let west = y * LATTICE_WIDTH + ((x + 3) % LATTICE_WIDTH);
                neighbors_in[idx] = [north, south, east, west];
            }
        }

        let nodes = std::array::from_fn(|i| TorusNode::new(i, config.v_init));
        let plasticity = PlasticityEngine::new(
            neighbors_in,
            config.weight_budget,
            config.weight_max,
            config.decay_factor,
        );
        let rng = FastRng::seed_from_u64(config.seed);

        Self {
            config,
            nodes,
            neighbors_in,
            prev_spikes: [0.0; NUM_NODES],
            plasticity,
            rng,
        }
    }

    /// Reset node somatic dynamics (membrane, refractory, rolling rate) while preserving learned synaptic weights
    pub fn reset_dynamics(&mut self, seed_offset: u64) {
        for node in self.nodes.iter_mut() {
            node.v = 0.0;
            node.refractory_counter = 0;
            node.spike = 0.0;
            node.rolling_rate = self.config.target_rate;
            // Retain adapted threshold v_thresh to preserve homeostatic equilibrium
        }
        self.prev_spikes = [0.0; NUM_NODES];
        self.rng = FastRng::seed_from_u64(self.config.seed.wrapping_add(seed_offset));
    }

    /// Advance one clock tick with optional sensory ingress
    pub fn step(
        &mut self,
        ext_ingress: &[f64; NUM_NODES],
        update_plasticity: bool,
    ) -> [f64; NUM_NODES] {
        let mut incoming = [0.0; NUM_NODES];
        for (i, inc) in incoming.iter_mut().enumerate() {
            let mut sum_tracks = 0.0;
            for &j in &self.neighbors_in[i] {
                sum_tracks += self.plasticity.weights[i][j] * self.prev_spikes[j];
            }
            *inc = sum_tracks + ext_ingress[i];
        }

        let mut curr_spikes = [0.0; NUM_NODES];
        for (i, node) in self.nodes.iter_mut().enumerate() {
            node.step(
                incoming[i],
                self.config.leak,
                self.config.n_ref,
                self.config.homeo_rate,
                self.config.target_rate,
                self.config.v_min,
                self.config.v_max,
                self.config.alpha_ema,
            );
            curr_spikes[i] = node.spike;
        }

        if update_plasticity {
            self.plasticity.update_weights(
                &self.config,
                &curr_spikes,
                &self.prev_spikes,
                &mut self.rng,
            );
        }

        self.prev_spikes = curr_spikes;
        curr_spikes
    }
}
