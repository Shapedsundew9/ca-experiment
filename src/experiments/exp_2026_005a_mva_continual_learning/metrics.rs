//! Metrics and evaluation module for EXP-2026-005a continual learning.

use super::curriculum::{
    FastRng, NUM_NODES, PatternId, generate_carrier_ingress, generate_pattern_ingress,
};
use super::substrate::TorusSubstrate;

/// Normalized Hamming distance between two node occupancy / firing state vectors
#[inline]
pub fn hamming_dist(u: &[f64; NUM_NODES], v: &[f64; NUM_NODES]) -> f64 {
    let mut sum = 0.0;
    for i in 0..NUM_NODES {
        sum += (u[i] - v[i]).abs();
    }
    sum / NUM_NODES as f64
}

/// Probe settled attractor firing profile for a pattern under frozen plasticity
pub fn probe_occupancy(
    substrate: &mut TorusSubstrate,
    pattern: PatternId,
    noise_rate: f64,
    rng: &mut FastRng,
) -> [f64; NUM_NODES] {
    let t_drive = substrate.config.t_drive;
    let t_relax = substrate.config.t_relax;
    let t_total = t_drive + t_relax;

    let mut spike_sums = [0.0; NUM_NODES];
    let mut rng_opt = if noise_rate > 0.0 { Some(rng) } else { None };

    for t in 0..t_total {
        let ext_ingress = if t < t_drive {
            generate_pattern_ingress(pattern, t, noise_rate, &mut rng_opt)
        } else {
            generate_carrier_ingress(pattern, t)
        };

        let spikes = substrate.step(&ext_ingress, false);

        if t >= t_drive {
            for i in 0..NUM_NODES {
                spike_sums[i] += spikes[i];
            }
        }
    }

    let mut occ = [0.0; NUM_NODES];
    for i in 0..NUM_NODES {
        occ[i] = spike_sums[i] / t_relax as f64;
    }
    occ
}

/// Measure attractor basin depth under perturbation noise:
/// B(X | W) = 1.0 - (1/K) \sum D_H(m_clean, m_noisy)
pub fn measure_basin_depth(
    substrate: &mut TorusSubstrate,
    pattern: PatternId,
    num_noisy: usize,
    noise_rate: f64,
    rng: &mut FastRng,
) -> ([f64; NUM_NODES], f64) {
    substrate.reset_dynamics(100);
    let clean_occ = probe_occupancy(substrate, pattern, 0.0, rng);

    if num_noisy == 0 || noise_rate <= 0.0 {
        return (clean_occ, 1.0);
    }

    let mut total_d = 0.0;
    for k in 0..num_noisy {
        substrate.reset_dynamics(200 + k as u64 * 17);
        let noisy_occ = probe_occupancy(substrate, pattern, noise_rate, rng);
        total_d += hamming_dist(&clean_occ, &noisy_occ);
    }

    let avg_d = total_d / num_noisy as f64;
    let basin_depth = (1.0 - avg_d).clamp(0.0, 1.0);
    (clean_occ, basin_depth)
}

/// Compute cosine similarity between weight deviations from baseline:
/// Delta W = W - 1.0
pub fn compute_subspace_overlap(
    w1: &[[f64; NUM_NODES]; NUM_NODES],
    w2: &[[f64; NUM_NODES]; NUM_NODES],
    neighbors_in: &[[usize; 4]; NUM_NODES],
) -> f64 {
    let mut dot = 0.0;
    let mut norm1_sq = 0.0;
    let mut norm2_sq = 0.0;

    for i in 0..NUM_NODES {
        for &j in &neighbors_in[i] {
            let dw1 = w1[i][j] - 1.0;
            let dw2 = w2[i][j] - 1.0;
            dot += dw1 * dw2;
            norm1_sq += dw1 * dw1;
            norm2_sq += dw2 * dw2;
        }
    }

    let denom = (norm1_sq.sqrt() * norm2_sq.sqrt()).max(1e-12);
    if denom <= 1e-11 {
        0.0
    } else {
        (dot / denom).clamp(-1.0, 1.0)
    }
}

/// Two-tailed Welch's t-test comparing sample1 and sample2
/// Returns (t_statistic, p_value, cohen_d)
pub fn welch_t_test(sample1: &[f64], sample2: &[f64]) -> (f64, f64, f64) {
    let n1 = sample1.len() as f64;
    let n2 = sample2.len() as f64;

    if n1 < 2.0 || n2 < 2.0 {
        return (0.0, 1.0, 0.0);
    }

    let m1 = sample1.iter().sum::<f64>() / n1;
    let m2 = sample2.iter().sum::<f64>() / n2;

    let v1 = sample1.iter().map(|x| (x - m1).powi(2)).sum::<f64>() / (n1 - 1.0);
    let v2 = sample2.iter().map(|x| (x - m2).powi(2)).sum::<f64>() / (n2 - 1.0);

    let se = ((v1 / n1) + (v2 / n2)).sqrt();
    if se < 1e-12 {
        return (0.0, 1.0, 0.0);
    }

    let t_stat = (m1 - m2) / se;

    // Cohen's d effect size
    let pooled_s = (((n1 - 1.0) * v1 + (n2 - 1.0) * v2) / (n1 + n2 - 2.0))
        .sqrt()
        .max(1e-12);
    let cohen_d = (m1 - m2) / pooled_s;

    // Normal approximation to p-value via complementary error function (Abramowitz & Stegun 7.1.26)
    let z = (t_stat.abs()) / std::f64::consts::SQRT_2;
    let p_val = erfc_approx(z);

    (t_stat, p_val, cohen_d)
}

/// Rational Chebyshev approximation to erfc(x) for x >= 0
fn erfc_approx(x: f64) -> f64 {
    if x <= 0.0 {
        return 1.0;
    }
    if x > 20.0 {
        return 0.0;
    }
    // Formula from Numerical Recipes / Abramowitz & Stegun
    let t = 1.0 / (1.0 + 0.5 * x);
    let tau = t
        * (-x * x - 1.26551223
            + t * (1.00002368
                + t * (0.37409196
                    + t * (0.09678418
                        + t * (-0.18628806
                            + t * (0.27886807
                                + t * (-1.13520398
                                    + t * (1.48851587 + t * (-0.82215223 + t * 0.17087277)))))))))
            .exp();
    tau.clamp(0.0, 1.0)
}
