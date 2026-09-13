//! Deterministic bitstream and perturbation generators for EXP-2026-006a.

use super::config::SignalPattern;

pub use crate::substrate::FastRng;

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
