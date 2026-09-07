//! Integration tests for EXP-2026-004a Hebbian Plasticity Experiment.

use rust_3::experiments::exp_2026_004a_mva_hebbian_plasticity::ca::TorusSubstrate;
use rust_3::experiments::exp_2026_004a_mva_hebbian_plasticity::config::{
    ExperimentConfig, PatternId, PlasticityCondition,
};
use rust_3::experiments::exp_2026_004a_mva_hebbian_plasticity::metrics::occupancy_distance;
use rust_3::experiments::exp_2026_004a_mva_hebbian_plasticity::patterns::{
    FastRng, apply_bit_flip_noise, get_pattern_bits,
};
use rust_3::experiments::exp_2026_004a_mva_hebbian_plasticity::plasticity::NUM_NODES;
use rust_3::experiments::exp_2026_004a_mva_hebbian_plasticity::run_single_experiment;

#[test]
fn test_patterns_distinctness_and_noise() {
    let mut rng = FastRng::seed_from_u64(42);
    let pat_a = get_pattern_bits(PatternId::A);
    let pat_b = get_pattern_bits(PatternId::B);
    let pat_c = get_pattern_bits(PatternId::C);
    let pat_d = get_pattern_bits(PatternId::D);

    assert_ne!(pat_a, pat_b);
    assert_ne!(pat_a, pat_c);
    assert_ne!(pat_b, pat_d);

    let noisy_a = apply_bit_flip_noise(&pat_a, 0.5, &mut rng);
    assert_ne!(pat_a, noisy_a);
}

#[test]
fn test_synaptic_budget_conservation_invariant() {
    let mut config = ExperimentConfig::default();
    config.exposure_ticks = 500;
    config.plasticity_rate = 0.05; // aggressive rate to test clamping
    let substrate = TorusSubstrate::new(config);

    let (_, peak, budget_conserved) = substrate.plasticity.compute_weight_stats();
    assert!(budget_conserved, "Initial budget must be conserved");
    assert!(peak <= 2.5, "Peak weight must respect W_max");
}

#[test]
fn test_single_run_active_hebbian_execution() {
    let config = ExperimentConfig {
        seed: 42,
        condition: PlasticityCondition::ActiveHebbian,
        pattern: PatternId::A,
        exposure_ticks: 200,
        noise_level: 0.01,
        ..Default::default()
    };

    let metrics = run_single_experiment(&config);
    assert!(
        metrics.mean_firing_density >= 0.05 && metrics.mean_firing_density <= 0.20,
        "Firing density should remain in homeostatic critical range: got {}",
        metrics.mean_firing_density
    );
    assert!(
        metrics.budget_conserved,
        "Synaptic budget must be conserved"
    );
    assert!(
        metrics.replay_distance <= 0.08,
        "Replay distance should be low: got {}",
        metrics.replay_distance
    );
}

#[test]
fn test_occupancy_distance_metric() {
    let m1 = [0.1; NUM_NODES];
    let m2 = [0.2; NUM_NODES];
    let dist = occupancy_distance(&m1, &m2);
    assert!((dist - 0.1).abs() < 1e-6);

    let dist_self = occupancy_distance(&m1, &m1);
    assert!(dist_self < 1e-12);
}
