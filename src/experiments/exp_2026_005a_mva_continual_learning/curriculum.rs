//! Curriculum module for EXP-2026-005a: Patterns, ingress generation, and deterministic RNG.

use serde::{Deserialize, Serialize};

pub const NUM_NODES: usize = 16;

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

    /// Box-Muller transform for standard normal sample
    pub fn next_gaussian(&mut self) -> f64 {
        let u1 = self.next_f64().max(1e-15);
        let u2 = self.next_f64();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternId {
    A,
    B,
    C,
}

impl PatternId {
    pub const ALL: [PatternId; 3] = [PatternId::A, PatternId::B, PatternId::C];

    #[inline]
    pub fn nodes(&self) -> [usize; 2] {
        match self {
            PatternId::A => [0, 1],
            PatternId::B => [4, 8],
            PatternId::C => [2, 3],
        }
    }

    #[inline]
    pub fn bit_at(&self, tick: usize) -> f64 {
        const SEQ_A: [f64; 8] = [1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        const SEQ_B: [f64; 8] = [1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0];
        const SEQ_C: [f64; 8] = [1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0];

        match self {
            PatternId::A => SEQ_A[tick % 8],
            PatternId::B => SEQ_B[tick % 8],
            PatternId::C => SEQ_C[tick % 8],
        }
    }
}

/// Generate external sensory ingress vector for a specific pattern at tick `t`.
/// If `noise_rate > 0.0`, flips pattern bits with probability `noise_rate`.
pub fn generate_pattern_ingress(
    pattern: PatternId,
    tick: usize,
    noise_rate: f64,
    rng: &mut Option<&mut FastRng>,
) -> [f64; NUM_NODES] {
    let mut ingress = [0.0; NUM_NODES];
    let nodes = pattern.nodes();
    let mut b = pattern.bit_at(tick);

    if noise_rate > 0.0
        && let Some(r) = rng
        && r.bernoulli(noise_rate)
    {
        b = 1.0 - b;
    }

    for &node in &nodes {
        ingress[node] = b;
    }

    ingress
}

/// Generate neutral carrier clock pulse on sensory nodes during relaxation
pub fn generate_carrier_ingress(pattern: PatternId, tick: usize) -> [f64; NUM_NODES] {
    let mut ingress = [0.0; NUM_NODES];
    if tick.is_multiple_of(2) {
        for &node in &pattern.nodes() {
            ingress[node] = 1.0;
        }
    }
    ingress
}
