//! Excitable node dynamics and network substrate for EXP-2026-010a.
//!
//! Refactored to delegate somatic integration and channel noise to `crate::substrate::NetworkSubstrate`.

use std::ops::{Deref, DerefMut};

use super::config::ExperimentConfig;
pub use crate::substrate::{CircuitNode, FastRng, NetworkSubstrate};

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
    pub net: NetworkSubstrate,

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
        // Mark ring nodes as susceptible to channel noise
        for ring in &pointer_rings {
            net.mark_noisy_nodes(ring);
        }
        for ring in &frame_rnd_rings {
            net.mark_noisy_nodes(ring);
        }
        for ring in &frame_sqr_rings {
            net.mark_noisy_nodes(ring);
        }
        net.mark_noisy_nodes(&trap_ring);
        net.mark_noisy_nodes(&accept_ring);

        Self {
            config,
            net,
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
        }
    }

    /// Advance simulation by one discrete clock tick.
    /// Inputs: Ingress pulse values for this tick.
    /// Returns: Readout pulse at v_out.
    pub fn step(&mut self, inputs: IngressInputs, in_eval_window: bool) -> f64 {
        let mut ext = Vec::with_capacity(5);

        if inputs.open_rnd > 0.0 {
            let w = self.net.nodes[self.in_rnd_open_idx].theta.max(1.10);
            ext.push((self.in_rnd_open_idx, w * inputs.open_rnd));
        }
        if inputs.close_rnd > 0.0 {
            let w = self.net.nodes[self.in_rnd_close_idx].theta.max(1.10);
            ext.push((self.in_rnd_close_idx, w * inputs.close_rnd));
        }
        if inputs.open_sqr > 0.0 {
            let w = self.net.nodes[self.in_sqr_open_idx].theta.max(1.10);
            ext.push((self.in_sqr_open_idx, w * inputs.open_sqr));
        }
        if inputs.close_sqr > 0.0 {
            let w = self.net.nodes[self.in_sqr_close_idx].theta.max(1.10);
            ext.push((self.in_sqr_close_idx, w * inputs.close_sqr));
        }
        if inputs.eos > 0.0 {
            let w = self.net.nodes[self.in_eos_idx].theta.max(1.10);
            ext.push((self.in_eos_idx, w * inputs.eos));
        }

        self.net.step(&ext, in_eval_window);

        self.net.nodes[self.v_out_idx].spike
    }

    /// Returns the number of currently active pointer rings in the substrate.
    pub fn active_pointer_count(&self) -> usize {
        self.pointer_rings
            .iter()
            .filter(|ring| self.net.any_spike_in_nodes(&ring[..]))
            .count()
    }

    /// Returns the active pointer depth level k if exactly one pointer is firing, or None.
    pub fn active_pointer_level(&self) -> Option<usize> {
        let mut active_level = None;
        for (k, ring) in self.pointer_rings.iter().enumerate() {
            if self.net.any_spike_in_nodes(&ring[..]) {
                if active_level.is_some() {
                    return None; // Crosstalk
                }
                active_level = Some(k);
            }
        }
        active_level
    }
}
