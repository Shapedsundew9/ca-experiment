//! 16-node Torus CA substrate with local synaptic metaplasticity for EXP-2026-005a.

use super::config::{Condition, ExperimentConfig};
use super::curriculum::{FastRng, NUM_NODES};

pub const LATTICE_WIDTH: usize = 4;
pub const LATTICE_HEIGHT: usize = 4;

#[derive(Debug, Clone)]
pub struct TorusNode {
    pub id: usize,
    pub v: f64,
    pub refractory_counter: usize,
    pub rolling_rate: f64,
    pub theta: f64,
    pub spike: f64,
}

impl TorusNode {
    pub fn new(id: usize, theta_init: f64, rho_target: f64) -> Self {
        Self {
            id,
            v: 0.0,
            refractory_counter: 0,
            rolling_rate: rho_target,
            theta: theta_init,
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
        alpha_rho: f64,
        rho_target: f64,
        beta_theta: f64,
        theta_min: f64,
        theta_max: f64,
    ) -> f64 {
        let spike: f64;
        if self.refractory_counter > 0 {
            spike = 0.0;
            self.v = 0.0;
            self.refractory_counter -= 1;
        } else {
            let v_cand = (1.0 - leak) * self.v + incoming_charge;
            if v_cand >= self.theta {
                spike = 1.0;
                self.v = 0.0;
                self.refractory_counter = n_ref;
            } else {
                spike = 0.0;
                self.v = v_cand;
                self.refractory_counter = 0;
            }
        }
        self.spike = spike;

        // Rolling firing rate EMA
        self.rolling_rate = (1.0 - alpha_rho) * self.rolling_rate + alpha_rho * spike;

        // Dynamic threshold adaptation
        if beta_theta > 0.0 {
            let delta = beta_theta * (self.rolling_rate - rho_target);
            self.theta = (self.theta + delta).clamp(theta_min, theta_max);
        }

        spike
    }
}

#[derive(Debug, Clone)]
pub struct TorusSubstrate {
    pub config: ExperimentConfig,
    pub nodes: [TorusNode; NUM_NODES],
    pub neighbors_in: [[usize; 4]; NUM_NODES],
    pub prev_spikes: [f64; NUM_NODES],
    /// Directed weight matrix: weights[i][j] is weight of track from j to i
    pub weights: [[f64; NUM_NODES]; NUM_NODES],
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

        let nodes =
            std::array::from_fn(|i| TorusNode::new(i, config.theta_init, config.rho_target));

        // Initialize directed weights to uncommitted baseline 1.0
        let mut weights = [[0.0; NUM_NODES]; NUM_NODES];
        for i in 0..NUM_NODES {
            for &j in &neighbors_in[i] {
                weights[i][j] = 1.0;
            }
        }

        let rng = FastRng::seed_from_u64(config.seed);

        Self {
            config,
            nodes,
            neighbors_in,
            prev_spikes: [0.0; NUM_NODES],
            weights,
            rng,
        }
    }

    /// Reset somatic microstate dynamics while retaining adapted thresholds and synaptic weights
    pub fn reset_dynamics(&mut self, seed_offset: u64) {
        for node in self.nodes.iter_mut() {
            node.v = 0.0;
            node.refractory_counter = 0;
            node.spike = 0.0;
            node.rolling_rate = self.config.rho_target;
        }
        self.prev_spikes = [0.0; NUM_NODES];
        self.rng = FastRng::seed_from_u64(self.config.seed.wrapping_add(seed_offset));
    }

    /// Advance one clock tick
    #[allow(clippy::needless_range_loop)]
    pub fn step(
        &mut self,
        ext_ingress: &[f64; NUM_NODES],
        plasticity_on: bool,
    ) -> [f64; NUM_NODES] {
        let mut incoming = [0.0; NUM_NODES];
        for (i, inc) in incoming.iter_mut().enumerate() {
            let mut sum_tracks = 0.0;
            for &j in &self.neighbors_in[i] {
                sum_tracks += self.weights[i][j] * self.prev_spikes[j];
            }
            *inc = sum_tracks + ext_ingress[i];
        }

        let mut curr_spikes = [0.0; NUM_NODES];
        for (i, node) in self.nodes.iter_mut().enumerate() {
            curr_spikes[i] = node.step(
                incoming[i],
                self.config.leak,
                self.config.n_ref,
                self.config.alpha_rho,
                self.config.rho_target,
                self.config.beta_theta,
                self.config.theta_min,
                self.config.theta_max,
            );
        }

        // Metaplastic synaptic update
        if plasticity_on
            && self.config.condition != Condition::BaselineStatic
            && self.config.eta0 > 0.0
        {
            let eta0 = self.config.eta0;
            let kappa = self.config.kappa;
            let decay = self.config.gamma_decay;
            let w_min = self.config.w_min;
            let w_max = self.config.w_max;
            let w_sum_max = self.config.w_sum_max;

            for i in 0..NUM_NODES {
                let post_spike = curr_spikes[i];

                for &j in &self.neighbors_in[i] {
                    let pre_spike = self.prev_spikes[j];
                    let w_curr = self.weights[i][j];

                    // Metaplastic damping: eta_ij = eta_0 / (1 + kappa * |W_ij - 1.0|)
                    let eta_ij = eta0 / (1.0 + kappa * (w_curr - 1.0).abs());
                    let delta = eta_ij * post_spike * (pre_spike - decay);
                    self.weights[i][j] = (w_curr + delta).max(w_min);
                }

                // Heterosynaptic dendritic scaling
                let mut sum_in = 0.0;
                for &j in &self.neighbors_in[i] {
                    sum_in += self.weights[i][j];
                }

                if sum_in > w_sum_max && sum_in > 1e-12 {
                    let scale = w_sum_max / sum_in;
                    for &j in &self.neighbors_in[i] {
                        self.weights[i][j] = (self.weights[i][j] * scale).clamp(w_min, w_max);
                    }
                } else {
                    for &j in &self.neighbors_in[i] {
                        self.weights[i][j] = self.weights[i][j].min(w_max);
                    }
                }
            }
        }

        self.prev_spikes = curr_spikes;
        curr_spikes
    }

    /// Snapshot current weights
    pub fn snapshot_weights(&self) -> [[f64; NUM_NODES]; NUM_NODES] {
        self.weights
    }

    /// Restore weights from a snapshot
    pub fn restore_weights(&mut self, snap: &[[f64; NUM_NODES]; NUM_NODES]) {
        self.weights = *snap;
    }
}
