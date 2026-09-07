//! Deterministic pseudo-random number generator for substrate simulations.
//!
//! Implements Xoshiro256++ seeded via SplitMix64. Provides fast uniform floating-point
//! generation, Bernoulli sampling, and standard normal distribution sampling (Box-Muller).

#[derive(Debug, Clone)]
pub struct FastRng {
    s: [u64; 4],
}

impl FastRng {
    /// Initialize PRNG from a 64-bit seed using SplitMix64.
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

    /// Generate next pseudo-random 64-bit unsigned integer via Xoshiro256++.
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

    /// Generate pseudo-random f64 in range [0.0, 1.0).
    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Sample boolean with probability `p`.
    #[inline]
    pub fn bernoulli(&mut self, p: f64) -> bool {
        if p <= 0.0 {
            false
        } else if p >= 1.0 {
            true
        } else {
            self.next_f64() < p
        }
    }

    /// Sample uniform float in range [low, high).
    #[inline]
    pub fn uniform(&mut self, low: f64, high: f64) -> f64 {
        low + (high - low) * self.next_f64()
    }

    /// Sample standard normal distribution with mean and standard deviation via Box-Muller transform.
    #[inline]
    pub fn normal(&mut self, mean: f64, std_dev: f64) -> f64 {
        let u1 = self.next_f64().max(1e-15);
        let u2 = self.next_f64();
        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        mean + z0 * std_dev
    }
}
