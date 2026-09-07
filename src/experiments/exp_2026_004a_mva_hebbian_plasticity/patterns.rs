//! Canonical pattern bitstreams and sensory driving functions for EXP-2026-004a.

use super::config::PatternId;

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
