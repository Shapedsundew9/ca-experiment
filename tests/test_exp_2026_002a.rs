//! Integration tests for EXP-2026-002a Attractor Mapping.

use rust_3::experiments::exp_2026_002a_mva_attractor_mapping::attractor::{
    detect_period, hamming_distance, occupancy_distance,
};
use rust_3::experiments::exp_2026_002a_mva_attractor_mapping::patterns::{
    apply_noise, generate_pattern,
};
use rust_3::experiments::exp_2026_002a_mva_attractor_mapping::runner::execute_single_run;
use rust_3::experiments::exp_2026_002a_mva_attractor_mapping::types::{
    ExperimentConfig, HomeostaticMode, InputPatternId,
};

#[test]
fn test_pattern_generation_distinctness() {
    let pat_a = generate_pattern(InputPatternId::PatternA, 1);
    let pat_b = generate_pattern(InputPatternId::PatternB, 1);
    let pat_c = generate_pattern(InputPatternId::PatternC, 1);
    let pat_d = generate_pattern(InputPatternId::PatternD, 1);

    assert_ne!(pat_a, pat_b, "Pattern A and B must differ");
    assert_ne!(pat_a, pat_c, "Pattern A and C must differ");
    assert_ne!(pat_b, pat_c, "Pattern B and C must differ");
    assert_ne!(pat_c, pat_d, "Pattern C and D must differ");

    // Noise perturbation flip check
    let noisy = apply_noise(&pat_a, 0.5, 42);
    assert_ne!(pat_a, noisy, "50% noise must flip bits");
}

#[test]
fn test_cycle_period_detection() {
    // Artificial period 4 orbit
    let mut snapshots = Vec::new();
    let orbit = [0b0001, 0b0010, 0b0100, 0b1000];
    for _ in 0..32 {
        for &val in &orbit {
            snapshots.push(val);
        }
    }
    let (period, detected) = detect_period(&snapshots);
    assert!(detected, "Must detect periodic orbit");
    assert_eq!(period, 4, "Detected period must equal 4");
}

#[test]
fn test_distance_metrics() {
    let mask1 = 0b0000_0000_0000_0001u16;
    let mask2 = 0b0000_0000_0000_0011u16;
    let d_h = hamming_distance(mask1, mask2);
    assert!((d_h - 1.0 / 16.0).abs() < 1e-6);

    let occ1 = [0.1; 16];
    let occ2 = [0.2; 16];
    let d_occ = occupancy_distance(&occ1, &occ2);
    assert!((d_occ - 0.1).abs() < 1e-6);
}

#[test]
fn test_single_run_active_settling() {
    let mut config = ExperimentConfig::default();
    config.pattern = InputPatternId::PatternA;
    config.t_relax = 200;
    config.mode = HomeostaticMode::Active;
    config.seed = 42;

    let res = execute_single_run(config);
    assert!(
        res.summary.cycle_detected,
        "Must detect limit cycle or bounded attractor"
    );
    assert!(
        res.summary.mean_firing_rate >= 0.05 && res.summary.mean_firing_rate <= 0.20,
        "Firing rate must be within homeostatic range: {}",
        res.summary.mean_firing_rate
    );
}
