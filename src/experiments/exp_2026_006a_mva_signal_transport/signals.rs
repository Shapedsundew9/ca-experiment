//! Deterministic bitstream and perturbation generators for EXP-2026-006a.

use super::config::SignalPattern;

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
        if p <= 0.0 {
            false
        } else if p >= 1.0 {
            true
        } else {
            self.next_f64() < p
        }
    }
}

/// Generate the pre-registered input bitstream for a given pattern
pub fn generate_input_stream(pattern: SignalPattern, t_eval: usize, seed: u64) -> Vec<f64> {
    let mut stream = Vec::with_capacity(t_eval);

    match pattern {
        SignalPattern::SingleImpulse => {
            for t in 0..t_eval {
                stream.push(if t == 0 { 1.0 } else { 0.0 });
            }
        }
        SignalPattern::AlternatingClock => {
            for t in 0..t_eval {
                stream.push(if t % 4 == 0 { 1.0 } else { 0.0 });
            }
        }
        SignalPattern::BurstTrain => {
            for t in 0..t_eval {
                let phase = t % 12;
                stream.push(if phase == 0 || phase == 3 || phase == 6 {
                    1.0
                } else {
                    0.0
                });
            }
        }
        SignalPattern::PseudoRandom => {
            let mut rng = FastRng::seed_from_u64(seed.wrapping_add(0xCA_2026_006A));
            let mut last_spike: isize = -10;
            for t in 0..t_eval {
                if (t as isize) - last_spike >= 3 && rng.bernoulli(0.10) {
                    stream.push(1.0);
                    last_spike = t as isize;
                } else {
                    stream.push(0.0);
                }
            }
        }
    }

    stream
}

/// Generate the perturbation stream for inter-branch crosstalk isolation test:
/// 50 pulses at density 0.25 (ISI >= 3) injected into Branch 1
pub fn generate_perturbation_stream(t_eval: usize) -> Vec<f64> {
    let mut stream = Vec::with_capacity(t_eval);
    for t in 0..t_eval {
        stream.push(if t % 4 == 0 { 1.0 } else { 0.0 });
    }
    stream
}
