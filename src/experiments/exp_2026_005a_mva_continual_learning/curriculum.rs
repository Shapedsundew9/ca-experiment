//! Curriculum module for EXP-2026-005a: Patterns, ingress generation, and deterministic RNG.

use serde::{Deserialize, Serialize};

pub const NUM_NODES: usize = 16;

pub use crate::substrate::FastRng;

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
