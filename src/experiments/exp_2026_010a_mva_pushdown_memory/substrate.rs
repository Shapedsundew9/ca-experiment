//! Excitable node dynamics and network substrate for EXP-2026-010a.

use super::config::ExperimentConfig;
pub use crate::substrate::{CircuitNode, FastRng};

#[derive(Debug, Clone, Copy, Default)]
pub struct IngressInputs {
    pub open_rnd: f64,
    pub close_rnd: f64,
    pub open_sqr: f64,
    pub close_sqr: f64,
    pub eos: f64,
}

#[derive(Debug, Clone)]
pub struct CircuitSubstrate {
    pub config: ExperimentConfig,
    pub nodes: Vec<CircuitNode>,
    /// in_neighbors[i] = list of (source_node_id, synaptic_weight)
    pub in_neighbors: Vec<Vec<(usize, f64)>>,
    pub prev_spikes: Vec<f64>,
    pub rng: FastRng,

    // Ingress port node indices
    pub in_rnd_open_idx: usize,
    pub in_rnd_close_idx: usize,
    pub in_sqr_open_idx: usize,
    pub in_sqr_close_idx: usize,
    pub in_eos_idx: usize,

    // Readout sense port
    pub v_out_idx: usize,

    // Ring node groups
    pub pointer_rings: Vec<[usize; 4]>,
    pub frame_rnd_rings: Vec<[usize; 4]>,
    pub frame_sqr_rings: Vec<[usize; 4]>,
    pub trap_ring: [usize; 4],
    pub accept_ring: [usize; 4],
    pub is_ring_node: Vec<bool>,

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
        in_rnd_open_idx: usize,
        in_rnd_close_idx: usize,
        in_sqr_open_idx: usize,
        in_sqr_close_idx: usize,
        in_eos_idx: usize,
        v_out_idx: usize,
        pointer_rings: Vec<[usize; 4]>,
        frame_rnd_rings: Vec<[usize; 4]>,
        frame_sqr_rings: Vec<[usize; 4]>,
        trap_ring: [usize; 4],
        accept_ring: [usize; 4],
    ) -> Self {
        let n = nodes.len();
        let prev_spikes = vec![0.0; n];
        let rng = FastRng::seed_from_u64(config.seed.wrapping_add(0xCA_2026_010A));

        let mut is_ring_node = vec![false; n];
        for ring in &pointer_rings {
            for &idx in ring {
                is_ring_node[idx] = true;
            }
        }
        for ring in &frame_rnd_rings {
            for &idx in ring {
                is_ring_node[idx] = true;
            }
        }
        for ring in &frame_sqr_rings {
            for &idx in ring {
                is_ring_node[idx] = true;
            }
        }
        for &idx in &trap_ring {
            is_ring_node[idx] = true;
        }
        for &idx in &accept_ring {
            is_ring_node[idx] = true;
        }

        Self {
            config,
            nodes,
            in_neighbors,
            prev_spikes,
            rng,
            in_rnd_open_idx,
            in_rnd_close_idx,
            in_sqr_open_idx,
            in_sqr_close_idx,
            in_eos_idx,
            v_out_idx,
            pointer_rings,
            frame_rnd_rings,
            frame_sqr_rings,
            trap_ring,
            accept_ring,
            is_ring_node,
            total_spikes: 0,
            total_steps: 0,
            eval_spikes: 0,
            eval_steps: 0,
        }
    }

    /// Advance simulation by one discrete clock tick.
    /// Inputs: Ingress pulse values for this tick.
    /// Returns: Readout pulse at v_out.
    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self, inputs: IngressInputs, in_eval_window: bool) -> f64 {
        let n = self.nodes.len();
        let mut curr_spikes = vec![0.0; n];
        let alpha_rho = self.config.alpha_rho;

        for i in 0..n {
            let mut in_synaptic = 0.0;

            // Sensory ingress inputs
            if i == self.in_rnd_open_idx && inputs.open_rnd > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_synaptic += w * inputs.open_rnd;
            }
            if i == self.in_rnd_close_idx && inputs.close_rnd > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_synaptic += w * inputs.close_rnd;
            }
            if i == self.in_sqr_open_idx && inputs.open_sqr > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_synaptic += w * inputs.open_sqr;
            }
            if i == self.in_sqr_close_idx && inputs.close_sqr > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_synaptic += w * inputs.close_sqr;
            }
            if i == self.in_eos_idx && inputs.eos > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_synaptic += w * inputs.eos;
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
                && self.is_ring_node[i]
                && self.rng.bernoulli(self.config.noise_rate)
            {
                noise_amp = self.rng.normal(0.923, 0.0274).max(0.0);
            }

            // Somatic integration and Heaviside thresholding
            let node = &mut self.nodes[i];
            let leak = node.leak;
            let spike: f64;

            if node.refractory_counter > 0 {
                spike = 0.0;
                node.v = 0.0;
                node.refractory_counter -= 1;
            } else {
                let eff_leak = if node.v < 0.0 { 0.50 } else { leak };
                let v_cand = (1.0 - eff_leak) * node.v + in_synaptic + noise_amp;
                if v_cand >= node.theta {
                    spike = 1.0;
                    node.v = 0.0;
                    node.refractory_counter = node.n_ref;
                } else {
                    spike = 0.0;
                    let v_next = (1.0 - eff_leak) * node.v + in_synaptic;
                    node.v = v_next.clamp(-2.0, 4.0);
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

        self.nodes[self.v_out_idx].spike
    }

    /// Directly inject a spike into a specific node (e.g. for initial state seeding).
    pub fn force_spike(&mut self, node_idx: usize) {
        if node_idx < self.nodes.len() {
            self.prev_spikes[node_idx] = 1.0;
            self.nodes[node_idx].spike = 1.0;
            self.nodes[node_idx].refractory_counter = self.nodes[node_idx].n_ref;
        }
    }

    /// Returns the number of currently active pointer rings in the substrate.
    pub fn active_pointer_count(&self) -> usize {
        let mut count = 0;
        for ring in &self.pointer_rings {
            let active = ring.iter().any(|&idx| self.nodes[idx].spike > 0.0);
            if active {
                count += 1;
            }
        }
        count
    }

    /// Returns the active pointer depth level k if exactly one pointer is firing, or None.
    pub fn active_pointer_level(&self) -> Option<usize> {
        let mut active_level = None;
        for (k, ring) in self.pointer_rings.iter().enumerate() {
            if ring.iter().any(|&idx| self.nodes[idx].spike > 0.0) {
                if active_level.is_some() {
                    return None; // Crosstalk
                }
                active_level = Some(k);
            }
        }
        active_level
    }

    /// Mean substrate firing density over the evaluation window.
    pub fn evaluation_firing_density(&self) -> f64 {
        if self.eval_steps == 0 || self.nodes.is_empty() {
            0.0
        } else {
            self.eval_spikes as f64 / (self.eval_steps * self.nodes.len()) as f64
        }
    }
}
