//! Excitable node dynamics and network substrate for EXP-2026-011a.
//!
//! Refactored to delegate core somatic dynamics, synaptic propagation, and
//! channel noise to `crate::substrate::NetworkSubstrate`.

use std::ops::{Deref, DerefMut};

use super::config::ExperimentConfig;
pub use crate::substrate::{CircuitNode, FastRng, NetworkSubstrate};

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
    pub net: NetworkSubstrate,

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
}

impl Deref for CircuitSubstrate {
    type Target = NetworkSubstrate;
    fn deref(&self) -> &Self::Target {
        &self.net
    }
}

impl DerefMut for CircuitSubstrate {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.net
    }
}

impl CircuitSubstrate {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: ExperimentConfig,
        mut net: NetworkSubstrate,
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
        // Mark ring and gate nodes as susceptible to channel noise
        for (r0, r1) in &slot_rings {
            net.mark_noisy_nodes(r0);
            net.mark_noisy_nodes(r1);
        }
        for &(g0, g1) in &write_gates {
            net.mark_noisy_nodes(&[g0, g1]);
        }
        for &(q0, q1) in &query_gates {
            net.mark_noisy_nodes(&[q0, q1]);
        }

        Self {
            config,
            net,
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
        }
    }

    /// Reset all dynamical node states to quiescent rest.
    pub fn reset_state(&mut self) {
        self.net.reset_state();
        for node in &mut self.net.nodes {
            node.theta = node.theta_floor.max(node.theta);
        }
    }

    /// Count current spikes in all memory slot resonant rings.
    pub fn count_memory_spikes(&self) -> usize {
        let mut count = 0;
        for (r0, r1) in &self.slot_rings {
            count += self.net.count_spikes_in_nodes(r0) + self.net.count_spikes_in_nodes(r1);
        }
        count
    }

    /// Check if any slot has simultaneous activation in both Ring 0 and Ring 1 (mutex violation).
    pub fn check_mutex_violations(&self) -> usize {
        let mut violations = 0;
        for (r0, r1) in &self.slot_rings {
            let s0 = self.net.count_spikes_in_nodes(r0);
            let s1 = self.net.count_spikes_in_nodes(r1);
            if s0 > 0 && s1 > 0 {
                violations += 1;
            }
        }
        violations
    }

    /// Advance simulation by one discrete clock tick.
    /// Returns: Readout pulses at `(v_out_0, v_out_1)`.
    pub fn step(&mut self, inputs: IngressInputs, in_eval_window: bool) -> (f64, f64) {
        let mut ext = Vec::with_capacity(35);

        for k in 0..16 {
            if inputs.keys[k] > 0.0 {
                let w = self.net.nodes[self.key_ports[k]].theta.max(1.10);
                ext.push((self.key_ports[k], w * inputs.keys[k]));
            }
            if inputs.queries[k] > 0.0 {
                let w = self.net.nodes[self.query_ports[k]].theta.max(1.10);
                ext.push((self.query_ports[k], w * inputs.queries[k]));
            }
        }
        if inputs.vals[0] > 0.0 {
            let w = self.net.nodes[self.val_ports[0]].theta.max(1.10);
            ext.push((self.val_ports[0], w * inputs.vals[0]));
        }
        if inputs.vals[1] > 0.0 {
            let w = self.net.nodes[self.val_ports[1]].theta.max(1.10);
            ext.push((self.val_ports[1], w * inputs.vals[1]));
        }
        if inputs.distractor > 0.0 {
            let w = self.net.nodes[self.dist_port].theta.max(1.10);
            ext.push((self.dist_port, w * inputs.distractor));
        }

        self.net.step(&ext, in_eval_window);

        (
            self.net.nodes[self.v_out_0].spike,
            self.net.nodes[self.v_out_1].spike,
        )
    }
}
