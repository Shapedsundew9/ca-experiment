//! Canonical pattern bitstreams and sensory driving functions for EXP-2026-004a.

use super::config::PatternId;

pub use crate::substrate::FastRng;

pub fn get_pattern_bits(pattern: PatternId) -> [f64; 32] {
    let word = pattern.bits();
    let mut bits = [0.0; 32];
    for (i, bit) in bits.iter_mut().enumerate() {
        // MSB first
        let b = (word >> (31 - i)) & 1;
        *bit = b as f64;
    }
    bits
}

pub fn apply_bit_flip_noise(bits: &[f64; 32], noise_rate: f64, rng: &mut FastRng) -> [f64; 32] {
    let mut noisy = *bits;
    if noise_rate <= 0.0 {
        return noisy;
    }
    for b in noisy.iter_mut() {
        if rng.bernoulli(noise_rate) {
            *b = 1.0 - *b;
        }
    }
    noisy
}
