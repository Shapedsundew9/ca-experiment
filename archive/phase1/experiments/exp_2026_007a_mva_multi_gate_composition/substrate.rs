//! Excitable node dynamics and network substrate for EXP-2026-007a.

use super::config::ExperimentConfig;

pub use crate::substrate::{CircuitNode, FastRng};

#[derive(Debug, Clone)]
pub struct CircuitSubstrate {
    pub config: ExperimentConfig,
    pub nodes: Vec<CircuitNode>,
    /// in_neighbors[i] = list of (source_node_id, synaptic_weight)
    pub in_neighbors: Vec<Vec<(usize, f64)>>,
    pub prev_spikes: Vec<f64>,
    pub rng: FastRng,
    pub in_a_idx: usize,
    pub in_b_idx: usize,
    pub in_c_idx: usize,
    pub crosstalk_inject_idx: usize,
    pub sum_idx: usize,
    pub cout_idx: usize,
    pub cin_post_cross_idx: usize,
    pub total_spikes: usize,
    pub total_steps: usize,
    pub eval_spikes: usize,
    pub eval_steps: usize,
}

impl CircuitSubstrate {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: ExperimentConfig,
        nodes: Vec<CircuitNode>,
        in_neighbors: Vec<Vec<(usize, f64)>>,
        in_a_idx: usize,
        in_b_idx: usize,
        in_c_idx: usize,
        crosstalk_inject_idx: usize,
        sum_idx: usize,
        cout_idx: usize,
        cin_post_cross_idx: usize,
    ) -> Self {
        let n = nodes.len();
        let prev_spikes = vec![0.0; n];
        let rng = FastRng::seed_from_u64(config.seed.wrapping_add(0xCA_2026_007A));

        Self {
            config,
            nodes,
            in_neighbors,
            prev_spikes,
            rng,
            in_a_idx,
            in_b_idx,
            in_c_idx,
            crosstalk_inject_idx,
            sum_idx,
            cout_idx,
            cin_post_cross_idx,
            total_spikes: 0,
            total_steps: 0,
            eval_spikes: 0,
            eval_steps: 0,
        }
    }

    /// Advance simulation by one discrete clock tick.
    /// Returns `(sum_spike, cout_spike, cin_post_cross_spike)`.
    #[allow(clippy::needless_range_loop)]
    pub fn step(
        &mut self,
        inputs: (f64, f64, f64),
        crosstalk_perturb: f64,
        in_eval_window: bool,
    ) -> (f64, f64, f64) {
        let n = self.nodes.len();
        let mut curr_spikes = vec![0.0; n];
        let leak = self.config.leak;
        let alpha_rho = self.config.alpha_rho;
        let rho_target = self.config.rho_target;

        // Channel noise: solitary noise pulse hits a random node with probability epsilon
        let noise_node =
            if self.config.noise_rate > 0.0 && self.rng.bernoulli(self.config.noise_rate) {
                Some((self.rng.next_u64() as usize) % n)
            } else {
                None
            };

        for i in 0..n {
            let mut in_charge = 0.0;

            // Sensory ingress inputs
            if i == self.in_a_idx && inputs.0 > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_charge += w * inputs.0;
            }
            if i == self.in_b_idx && inputs.1 > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_charge += w * inputs.1;
            }
            if i == self.in_c_idx && inputs.2 > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_charge += w * inputs.2;
            }

            // Crosstalk perturbation input
            if i == self.crosstalk_inject_idx && crosstalk_perturb > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_charge += w * crosstalk_perturb;
            }

            // Presynaptic spikes from graph edges
            for &(src, w) in &self.in_neighbors[i] {
                if self.prev_spikes[src] > 0.0 {
                    in_charge += w * self.prev_spikes[src];
                }
            }

            // Channel noise pulse
            if noise_node == Some(i) {
                in_charge += 1.0;
            }

            // Somatic integration and Heaviside thresholding
            let node = &mut self.nodes[i];
            let spike: f64;

            if node.refractory_counter > 0 {
                spike = 0.0;
                node.v = 0.0;
                node.refractory_counter -= 1;
            } else {
                let v_cand = (1.0 - leak) * node.v + in_charge;
                if v_cand >= node.theta {
                    spike = 1.0;
                    node.v = 0.0;
                    node.refractory_counter = node.n_ref;
                } else {
                    spike = 0.0;
                    node.v = v_cand.max(0.0);
                    node.refractory_counter = 0;
                }
            }

            node.spike = spike;
            curr_spikes[i] = spike;
            if spike > 0.0 {
                self.total_spikes += 1;
                if in_eval_window {
                    self.eval_spikes += 1;
                }
            }

            // Rolling firing rate EMA
            node.rolling_rate = (1.0 - alpha_rho) * node.rolling_rate + alpha_rho * spike;

            // Somatic threshold adaptation
            if node.beta_theta > 0.0 {
                let delta = node.beta_theta * (node.rolling_rate - rho_target);
                node.theta = (node.theta + delta).clamp(node.theta_floor, node.theta_max);
            }
        }

        self.prev_spikes = curr_spikes;
        self.total_steps += 1;
        if in_eval_window {
            self.eval_steps += 1;
        }

        (
            self.prev_spikes[self.sum_idx],
            self.prev_spikes[self.cout_idx],
            self.prev_spikes[self.cin_post_cross_idx],
        )
    }

    /// Substrate rolling activity density across evaluation steps
    pub fn mean_firing_density(&self) -> f64 {
        if self.eval_steps == 0 || self.nodes.is_empty() {
            0.0
        } else {
            self.eval_spikes as f64 / (self.eval_steps * self.nodes.len()) as f64
        }
    }

    /// Reset node potentials, refractory states, and spikes
    pub fn reset_state(&mut self) {
        for node in &mut self.nodes {
            node.v = 0.0;
            node.refractory_counter = 0;
            node.spike = 0.0;
        }
        self.prev_spikes.fill(0.0);
    }
}
