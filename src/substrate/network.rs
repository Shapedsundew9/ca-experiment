//! Network substrate and graph simulation engine.
//!
//! Handles presynaptic spike integration across weighted directed edges, refractory period
//! management, channel noise injection, Heaviside thresholding, and homeostatic adaptation.

use std::collections::HashMap;

use super::config::SubstrateConfig;
use super::node::CircuitNode;
use super::rng::FastRng;

#[derive(Debug, Clone)]
pub struct NetworkSubstrate {
    pub config: SubstrateConfig,
    pub nodes: Vec<CircuitNode>,
    /// in_neighbors[i] = list of (source_node_id, synaptic_weight)
    pub in_neighbors: Vec<Vec<(usize, f64)>>,
    pub prev_spikes: Vec<f64>,
    pub rng: FastRng,
    pub named_ports: HashMap<String, usize>,
    pub is_noisy_node: Vec<bool>,
    pub total_spikes: usize,
    pub total_steps: usize,
    pub eval_spikes: usize,
    pub eval_steps: usize,
}

impl NetworkSubstrate {
    /// Create a new network substrate with nodes, directed edges, and configuration.
    pub fn new(
        config: SubstrateConfig,
        nodes: Vec<CircuitNode>,
        in_neighbors: Vec<Vec<(usize, f64)>>,
    ) -> Self {
        let n = nodes.len();
        let prev_spikes = vec![0.0; n];
        let rng = FastRng::seed_from_u64(config.seed);
        let is_noisy_node = vec![false; n];

        Self {
            config,
            nodes,
            in_neighbors,
            prev_spikes,
            rng,
            named_ports: HashMap::new(),
            is_noisy_node,
            total_spikes: 0,
            total_steps: 0,
            eval_spikes: 0,
            eval_steps: 0,
        }
    }

    /// Register a named port mapping (e.g. "input_0", "accept", "data").
    pub fn set_port(&mut self, name: impl Into<String>, node_idx: usize) {
        self.named_ports.insert(name.into(), node_idx);
    }

    /// Retrieve the node index for a named port.
    pub fn port(&self, name: &str) -> Option<usize> {
        self.named_ports.get(name).copied()
    }

    /// Advance the simulation by one discrete clock tick.
    ///
    /// Accepts a list of external sensory injections `&[(node_idx, current_amplitude)]`
    /// and a flag indicating whether this step falls within the telemetry evaluation window.
    pub fn step(&mut self, external_inputs: &[(usize, f64)], in_eval_window: bool) {
        let n = self.nodes.len();
        let mut curr_spikes = vec![0.0; n];
        let alpha_rho = self.config.alpha_rho;

        // Map external inputs by node index
        let mut ext_map = vec![0.0; n];
        for &(idx, current) in external_inputs {
            if idx < n {
                ext_map[idx] += current;
            }
        }

        for i in 0..n {
            let mut in_synaptic = ext_map[i];

            // Presynaptic spikes from graph edges
            for &(src, w) in &self.in_neighbors[i] {
                if self.prev_spikes[src] > 0.0 {
                    in_synaptic += w * self.prev_spikes[src];
                }
            }

            // Thermal channel noise fluctuation on designated noisy nodes
            let mut noise_amp = 0.0;
            if self.config.noise_rate > 0.0
                && self.is_noisy_node[i]
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

            // Somatic threshold adaptation: increments on spike; relaxes when silent
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
    }

    /// Directly inject a spike into a specific node (e.g. for initial state seeding).
    pub fn force_spike(&mut self, node_idx: usize) {
        if node_idx < self.nodes.len() {
            self.prev_spikes[node_idx] = 1.0;
            self.nodes[node_idx].spike = 1.0;
            self.nodes[node_idx].refractory_counter = self.nodes[node_idx].n_ref;
        }
    }

    /// Reset all node voltages, refractory counters, and spikes to rest.
    pub fn reset_state(&mut self) {
        for node in &mut self.nodes {
            node.reset_state();
        }
        self.prev_spikes.fill(0.0);
    }

    /// Calculate mean firing density over the evaluation window.
    pub fn evaluation_firing_density(&self) -> f64 {
        if self.eval_steps == 0 || self.nodes.is_empty() {
            0.0
        } else {
            self.eval_spikes as f64 / (self.nodes.len() as f64 * self.eval_steps as f64)
        }
    }
}
