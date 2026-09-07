//! 16-node 2D Torus cellular automaton engine for EXP-2026-003a.

use super::config::{ExperimentConfig, ExperimentalCondition};

pub const NUM_NODES: usize = 16;
pub const LATTICE_WIDTH: usize = 4;
pub const LATTICE_HEIGHT: usize = 4;

/// Deterministic pseudo-random number generator based on Xoshiro256++
#[derive(Debug, Clone)]
pub struct FastRng {
    s: [u64; 4],
}

impl FastRng {
    pub fn seed_from_u64(seed: u64) -> Self {
        let mut sm = seed;
        let mut next_sm = || {
            sm = sm.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = sm;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            z ^ (z >> 31)
        };
        let s0 = next_sm();
        let s1 = next_sm();
        let s2 = next_sm();
        let s3 = next_sm();
        Self {
            s: [if (s0 | s1 | s2 | s3) == 0 { 1 } else { s0 }, s1, s2, s3],
        }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let res = (self.s[0].wrapping_add(self.s[3]))
            .rotate_left(23)
            .wrapping_add(self.s[0]);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        res
    }

    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    #[inline]
    pub fn bernoulli(&mut self, p: f64) -> bool {
        self.next_f64() < p
    }
}

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
        eta: f64,
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
        if eta > 0.0 {
            let delta = eta * (self.rolling_rate - target_rate);
            self.v_thresh = (self.v_thresh + delta).clamp(v_min, v_max);
        }
    }
}

pub struct TorusSubstrate {
    pub config: ExperimentConfig,
    pub nodes: [TorusNode; NUM_NODES],
    pub neighbors: [[usize; 4]; NUM_NODES],
    pub prev_spikes: [f64; NUM_NODES],
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

        let nodes = std::array::from_fn(|i| TorusNode::new(i, config.v_init));

        Self {
            config,
            nodes,
            neighbors,
            prev_spikes: [0.0; NUM_NODES],
        }
    }

    #[inline]
    pub fn step(&mut self, input_bit: f64) -> [f64; NUM_NODES] {
        let k_in = match self.config.condition {
            ExperimentalCondition::AblationMemoryless => 0.0,
            _ => 1.0,
        };

        let eta = match self.config.condition {
            ExperimentalCondition::AblationStaticThresh => 0.0,
            _ => self.config.eta,
        };

        let mut incoming = [0.0; NUM_NODES];
        for (i, inc) in incoming.iter_mut().enumerate() {
            let mut sum_tracks = 0.0;
            for &nbr in &self.neighbors[i] {
                sum_tracks += self.prev_spikes[nbr];
            }
            if i == 0 {
                sum_tracks += k_in * input_bit;
            }
            *inc = sum_tracks;
        }

        let mut curr_spikes = [0.0; NUM_NODES];
        for (i, node) in self.nodes.iter_mut().enumerate() {
            node.step(
                incoming[i],
                self.config.leak,
                self.config.n_ref,
                eta,
                self.config.target_rate,
                self.config.v_min,
                self.config.v_max,
                self.config.alpha_ema,
            );
            curr_spikes[i] = node.spike;
        }

        self.prev_spikes = curr_spikes;
        curr_spikes
    }
}
