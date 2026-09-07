//! Bitstream input pattern generators and noise perturbation for EXP-2026-002a.

use super::types::InputPatternId;

/// Generate base 32-tick bitstream for a given input pattern
pub fn generate_pattern(pattern: InputPatternId, seed: u64) -> [u8; 32] {
    let mut bits = [0u8; 32];
    match pattern {
        InputPatternId::PatternA => {
            // High-frequency burst: 11001100110011001100110011001100 (duty cycle 50%)
            for (t, bit) in bits.iter_mut().enumerate() {
                if t % 4 == 0 || t % 4 == 1 {
                    *bit = 1;
                }
            }
        }
        InputPatternId::PatternB => {
            // Prime pulse train: t in {2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31}
            let primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31];
            for &p in &primes {
                if p < 32 {
                    bits[p] = 1;
                }
            }
        }
        InputPatternId::PatternC => {
            // Galois LFSR: polynomial x^8 + x^6 + x^5 + x^4 + 1, seed 0x5A
            let mut lfsr: u8 = 0x5A;
            for bit in bits.iter_mut() {
                *bit = lfsr & 1;
                let lsb = lfsr & 1;
                lfsr >>= 1;
                if lsb == 1 {
                    lfsr ^= 0b10110000; // taps: 8, 6, 5, 4
                }
            }
        }
        InputPatternId::PatternD => {
            // Period-8 sparse pulse: 10000000100000001000000010000000
            for (t, bit) in bits.iter_mut().enumerate() {
                if t % 8 == 0 {
                    *bit = 1;
                }
            }
        }
        InputPatternId::Noise => {
            // Bernoulli noise with p = 0.12 generated from seed
            let mut s = seed;
            for bit in bits.iter_mut() {
                s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                let rand_val = ((s >> 32) as f64) / (u32::MAX as f64);
                if rand_val < 0.12 {
                    *bit = 1;
                }
            }
        }
    }
    bits
}

/// Apply bit-flip perturbation with probability epsilon
pub fn apply_noise(bits: &[u8; 32], epsilon: f64, seed: u64) -> [u8; 32] {
    if epsilon <= 0.0 {
        return *bits;
    }
    let mut noisy = *bits;
    let mut s = seed.wrapping_add(0xDEAD_BEEF);
    for bit in noisy.iter_mut() {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        let rand_val = ((s >> 32) as f64) / (u32::MAX as f64);
        if rand_val < epsilon {
            *bit ^= 1; // Flip bit
        }
    }
    noisy
}
