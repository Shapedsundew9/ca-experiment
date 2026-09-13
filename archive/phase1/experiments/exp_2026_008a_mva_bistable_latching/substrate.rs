//! Excitable node dynamics and network substrate for EXP-2026-008a.

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
    pub in_d_idx: usize,
    pub in_we_idx: usize,
    pub in_s_idx: usize,
    pub in_r_idx: usize,
    pub in_re_idx: usize,
    pub gate_set_idx: usize,
    pub gate_rst_idx: usize,
    pub v_inh_idx: usize,
    pub r0_idx: usize,
    pub r1_idx: usize,
    pub r2_idx: usize,
    pub r3_idx: usize,
    pub v_sense_idx: usize,
    pub v_read_idx: usize,
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
        in_d_idx: usize,
        in_we_idx: usize,
        in_s_idx: usize,
        in_r_idx: usize,
        in_re_idx: usize,
        gate_set_idx: usize,
        gate_rst_idx: usize,
        v_inh_idx: usize,
        r0_idx: usize,
        r1_idx: usize,
        r2_idx: usize,
        r3_idx: usize,
        v_sense_idx: usize,
        v_read_idx: usize,
    ) -> Self {
        let n = nodes.len();
        let prev_spikes = vec![0.0; n];
        let rng = FastRng::seed_from_u64(config.seed.wrapping_add(0xCA_2026_008A));

        Self {
            config,
            nodes,
            in_neighbors,
            prev_spikes,
            rng,
            in_d_idx,
            in_we_idx,
            in_s_idx,
            in_r_idx,
            in_re_idx,
            gate_set_idx,
            gate_rst_idx,
            v_inh_idx,
            r0_idx,
            r1_idx,
            r2_idx,
            r3_idx,
            v_sense_idx,
            v_read_idx,
            total_spikes: 0,
            total_steps: 0,
            eval_spikes: 0,
            eval_steps: 0,
        }
    }

    /// Advance simulation by one discrete clock tick.
    /// Inputs: `(in_d, in_we, in_s, in_r, in_re)`
    /// Returns `(v_sense_spike, v_read_spike)`.
    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self, inputs: (f64, f64, f64, f64, f64), in_eval_window: bool) -> (f64, f64) {
        let n = self.nodes.len();
        let mut curr_spikes = vec![0.0; n];
        let leak = self.config.leak;
        let alpha_rho = self.config.alpha_rho;

        for i in 0..n {
            let mut in_synaptic = 0.0;

            // Sensory ingress inputs
            if i == self.in_d_idx && inputs.0 > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_synaptic += w * inputs.0;
            }
            if i == self.in_we_idx && inputs.1 > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_synaptic += w * inputs.1;
            }
            if i == self.in_s_idx && inputs.2 > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_synaptic += w * inputs.2;
            }
            if i == self.in_r_idx && inputs.3 > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_synaptic += w * inputs.3;
            }
            if i == self.in_re_idx && inputs.4 > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_synaptic += w * inputs.4;
            }

            // Presynaptic spikes from graph edges
            for &(src, w) in &self.in_neighbors[i] {
                if self.prev_spikes[src] > 0.0 {
                    in_synaptic += w * self.prev_spikes[src];
                }
            }

            // Channel noise pulse: solitary subthreshold thermal fluctuation on ring nodes
            let mut noise_amp = 0.0;
            if self.config.noise_rate > 0.0
                && (i == self.r0_idx || i == self.r1_idx || i == self.r2_idx || i == self.r3_idx)
                && self.rng.bernoulli(self.config.noise_rate)
            {
                noise_amp = self.rng.normal(0.923, 0.0274).max(0.0);
            }

            // Somatic integration and Heaviside thresholding
            let node = &mut self.nodes[i];
            let spike: f64;

            if node.refractory_counter > 0 {
                spike = 0.0;
                node.v = 0.0;
                node.refractory_counter -= 1;
            } else {
                let v_cand = (1.0 - leak) * node.v + in_synaptic + noise_amp;
                if v_cand >= node.theta {
                    spike = 1.0;
                    node.v = 0.0;
                    node.refractory_counter = node.n_ref;
                } else {
                    spike = 0.0;
                    node.v = ((1.0 - leak) * node.v + in_synaptic).clamp(-2.0, 4.0);
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

            // Somatic threshold adaptation:
            // Increments by homeostatic quantum on firing; relaxes toward resting baseline when silent
            if node.beta_theta > 0.0 {
                if spike > 0.0 {
                    node.theta = (node.theta + node.beta_theta * alpha_rho).min(node.theta_max);
                } else {
                    node.theta = (node.theta - node.beta_theta * 0.01).max(node.theta_floor);
                }
            }
        }

        self.prev_spikes = curr_spikes;
        self.total_steps += 1;
        if in_eval_window {
            self.eval_steps += 1;
        }

        (
            self.prev_spikes[self.v_sense_idx],
            self.prev_spikes[self.v_read_idx],
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
