//! Excitable node dynamics and network substrate for EXP-2026-011a.

use super::config::ExperimentConfig;
pub use crate::substrate::{CircuitNode, FastRng};

#[derive(Debug, Clone, Copy, Default)]
pub struct IngressInputs {
    pub keys: [f64; 16],
    pub vals: [f64; 2],
    pub distractor: f64,
    pub queries: [f64; 16],
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
    pub key_ports: [usize; 16],
    pub val_ports: [usize; 2],
    pub dist_port: usize,
    pub query_ports: [usize; 16],

    // Readout ports
    pub v_out_0: usize,
    pub v_out_1: usize,

    // Ring node groups per slot: (ring_0, ring_1)
    pub slot_rings: Vec<([usize; 4], [usize; 4])>,
    pub write_gates: Vec<(usize, usize)>,
    pub quench_nodes: Vec<(usize, usize)>,
    pub query_gates: Vec<(usize, usize)>,
    pub sink_nodes: [usize; 3],
    pub readout_hubs: ([usize; 4], [usize; 4]),

    pub is_ring_node: Vec<bool>,
    pub is_gate_node: Vec<bool>,

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
        key_ports: [usize; 16],
        val_ports: [usize; 2],
        dist_port: usize,
        query_ports: [usize; 16],
        v_out_0: usize,
        v_out_1: usize,
        slot_rings: Vec<([usize; 4], [usize; 4])>,
        write_gates: Vec<(usize, usize)>,
        quench_nodes: Vec<(usize, usize)>,
        query_gates: Vec<(usize, usize)>,
        sink_nodes: [usize; 3],
        readout_hubs: ([usize; 4], [usize; 4]),
    ) -> Self {
        let n = nodes.len();
        let prev_spikes = vec![0.0; n];
        let rng = FastRng::seed_from_u64(config.seed.wrapping_add(0xCA_2026_011A));

        let mut is_ring_node = vec![false; n];
        for (r0, r1) in &slot_rings {
            for &idx in r0 {
                is_ring_node[idx] = true;
            }
            for &idx in r1 {
                is_ring_node[idx] = true;
            }
        }

        let mut is_gate_node = vec![false; n];
        for &(g0, g1) in &write_gates {
            is_gate_node[g0] = true;
            is_gate_node[g1] = true;
        }
        for &(q0, q1) in &query_gates {
            is_gate_node[q0] = true;
            is_gate_node[q1] = true;
        }

        Self {
            config,
            nodes,
            in_neighbors,
            prev_spikes,
            rng,
            key_ports,
            val_ports,
            dist_port,
            query_ports,
            v_out_0,
            v_out_1,
            slot_rings,
            write_gates,
            quench_nodes,
            query_gates,
            sink_nodes,
            readout_hubs,
            is_ring_node,
            is_gate_node,
            total_spikes: 0,
            total_steps: 0,
            eval_spikes: 0,
            eval_steps: 0,
        }
    }

    /// Reset all dynamical node states to quiescent rest.
    pub fn reset_state(&mut self) {
        for node in &mut self.nodes {
            node.reset_state();
            node.theta = node.theta_floor.max(node.theta);
        }
        self.prev_spikes.fill(0.0);
    }

    /// Count current spikes in all memory slot resonant rings.
    pub fn count_memory_spikes(&self) -> usize {
        let mut count = 0;
        for (r0, r1) in &self.slot_rings {
            for &idx in r0 {
                if self.nodes[idx].spike > 0.0 {
                    count += 1;
                }
            }
            for &idx in r1 {
                if self.nodes[idx].spike > 0.0 {
                    count += 1;
                }
            }
        }
        count
    }

    /// Check if any slot has simultaneous activation in both Ring 0 and Ring 1 (mutex violation).
    pub fn check_mutex_violations(&self) -> usize {
        let mut violations = 0;
        for (r0, r1) in &self.slot_rings {
            let s0: usize = r0
                .iter()
                .map(|&idx| if self.nodes[idx].spike > 0.0 { 1 } else { 0 })
                .sum();
            let s1: usize = r1
                .iter()
                .map(|&idx| if self.nodes[idx].spike > 0.0 { 1 } else { 0 })
                .sum();
            if s0 > 0 && s1 > 0 {
                violations += 1;
            }
        }
        violations
    }

    /// Advance simulation by one discrete clock tick.
    /// Returns: Readout pulses at `(v_out_0, v_out_1)`.
    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self, inputs: IngressInputs, in_eval_window: bool) -> (f64, f64) {
        let n = self.nodes.len();
        let mut curr_spikes = vec![0.0; n];
        let alpha_rho = self.config.alpha_rho;

        for i in 0..n {
            let mut in_synaptic = 0.0;

            // Sensory ingress inputs
            for k in 0..16 {
                if i == self.key_ports[k] && inputs.keys[k] > 0.0 {
                    let w = self.nodes[i].theta.max(1.10);
                    in_synaptic += w * inputs.keys[k];
                }
                if i == self.query_ports[k] && inputs.queries[k] > 0.0 {
                    let w = self.nodes[i].theta.max(1.10);
                    in_synaptic += w * inputs.queries[k];
                }
            }
            if i == self.val_ports[0] && inputs.vals[0] > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_synaptic += w * inputs.vals[0];
            }
            if i == self.val_ports[1] && inputs.vals[1] > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_synaptic += w * inputs.vals[1];
            }
            if i == self.dist_port && inputs.distractor > 0.0 {
                let w = self.nodes[i].theta.max(1.10);
                in_synaptic += w * inputs.distractor;
            }

            // Presynaptic spikes from graph edges
            for &(src, w) in &self.in_neighbors[i] {
                if self.prev_spikes[src] > 0.0 {
                    in_synaptic += w * self.prev_spikes[src];
                }
            }

            // Channel noise pulse: solitary subthreshold thermal fluctuation on ring and gate nodes
            let mut noise_amp = 0.0;
            if self.config.noise_rate > 0.0
                && (self.is_ring_node[i] || self.is_gate_node[i])
                && self.rng.bernoulli(self.config.noise_rate)
            {
                noise_amp = self.rng.normal(0.40, 0.03).max(0.0);
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

        (
            self.nodes[self.v_out_0].spike,
            self.nodes[self.v_out_1].spike,
        )
    }
}
