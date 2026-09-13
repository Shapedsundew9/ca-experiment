//! Substrate simulation engine for EXP-2026-002a attractor mapping.

use super::types::{ExperimentConfig, HomeostaticMode};

pub const NUM_NODES: usize = 16;
pub const LATTICE_WIDTH: usize = 4;
pub const LATTICE_HEIGHT: usize = 4;

#[derive(Debug, Clone)]
pub struct SubstrateNode {
    pub id: usize,
    pub v: f64,
    pub refractory_counter: usize,
    pub rolling_rate: f64,
    pub v_thresh: f64,
    pub spike: u8,
}

impl SubstrateNode {
    pub fn new(id: usize, v_thresh_init: f64) -> Self {
        Self {
            id,
            v: 0.0,
            refractory_counter: 0,
            rolling_rate: 0.0,
            v_thresh: v_thresh_init,
            spike: 0,
        }
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn step(
        &mut self,
        incoming_charge: f64,
        leak: f64,
        n_ref: usize,
        mode: HomeostaticMode,
        target_rate: f64,
        eta: f64,
        v_min: f64,
        v_max: f64,
        alpha: f64,
    ) {
        let spike: u8;

        if self.refractory_counter > 0 {
            spike = 0;
            self.v *= 1.0 - leak;
            self.refractory_counter -= 1;
        } else {
            let v_pre = (1.0 - leak) * self.v + incoming_charge;
            if v_pre >= self.v_thresh {
                spike = 1;
                self.v = 0.0;
                self.refractory_counter = n_ref;
            } else {
                spike = 0;
                self.v = v_pre;
                self.refractory_counter = 0;
            }
        }

        self.spike = spike;

        // Exponential moving average for rolling firing rate
        self.rolling_rate = (1.0 - alpha) * self.rolling_rate + alpha * (spike as f64);

        // Homeostatic threshold regulation
        if mode == HomeostaticMode::Active {
            let delta = eta * (self.rolling_rate - target_rate);
            self.v_thresh = (self.v_thresh + delta).clamp(v_min, v_max);
        }
    }
}

pub struct TorusSubstrate {
    pub config: ExperimentConfig,
    pub nodes: [SubstrateNode; NUM_NODES],
    pub neighbors: [[usize; 4]; NUM_NODES],
    pub prev_spikes: [u8; NUM_NODES],
}

impl TorusSubstrate {
    pub fn new(config: ExperimentConfig) -> Self {
        let mut neighbors = [[0usize; 4]; NUM_NODES];
        for y in 0..LATTICE_HEIGHT {
            for x in 0..LATTICE_WIDTH {
                let idx = y * LATTICE_WIDTH + x;
                let north = ((y + 3) % LATTICE_HEIGHT) * LATTICE_WIDTH + x;
                let south = ((y + 1) % LATTICE_HEIGHT) * LATTICE_WIDTH + x;
                let east = y * LATTICE_WIDTH + ((x + 1) % LATTICE_WIDTH);
                let west = y * LATTICE_WIDTH + ((x + 3) % LATTICE_WIDTH);
                neighbors[idx] = [north, south, east, west];
            }
        }

        let nodes = std::array::from_fn(|i| SubstrateNode::new(i, config.v_init));

        Self {
            config,
            nodes,
            neighbors,
            prev_spikes: [0; NUM_NODES],
        }
    }

    /// Advance simulation by one tick given driving pattern bits
    #[inline]
    pub fn step(&mut self, tick: usize, drive_bits: &[u8; 32]) -> (u16, f64) {
        let mut incoming_charges = [0.0; NUM_NODES];
        for (i, charge) in incoming_charges.iter_mut().enumerate() {
            let mut sum_tracks = 0.0;
            for &nbr in &self.neighbors[i] {
                sum_tracks += self.prev_spikes[nbr] as f64;
            }

            let mut ext = 0.0;
            if tick < self.config.t_drive {
                // Sensory ingress on nodes 0 and 1
                if i == 0 {
                    ext = drive_bits[tick % 32] as f64;
                } else if i == 1 {
                    ext = drive_bits[(tick + 28) % 32] as f64; // t - 4 mod 32
                }
            } else {
                // Neutral carrier clock: periodic pulse on sensory nodes
                if (i == 0 || i == 1) && tick.is_multiple_of(2) {
                    ext = 1.0;
                }
            }

            *charge = sum_tracks + ext;
        }

        let mut current_spikes = [0u8; NUM_NODES];
        let mut spike_mask = 0u16;
        let mut total_spikes = 0usize;

        for (i, node) in self.nodes.iter_mut().enumerate() {
            node.step(
                incoming_charges[i],
                self.config.leak,
                self.config.n_ref,
                self.config.mode,
                self.config.r_target,
                self.config.eta,
                self.config.v_min,
                self.config.v_max,
                0.01,
            );

            current_spikes[i] = node.spike;
            if node.spike == 1 {
                spike_mask |= 1 << i;
                total_spikes += 1;
            }
        }

        self.prev_spikes = current_spikes;
        let density = (total_spikes as f64) / (NUM_NODES as f64);

        (spike_mask, density)
    }
}
