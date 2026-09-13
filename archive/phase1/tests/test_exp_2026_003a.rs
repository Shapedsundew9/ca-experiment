//! Integration tests for EXP-2026-003a Temporal XOR Benchmark.

use rust_3::experiments::exp_2026_003a_mva_temporal_xor::ca::FastRng;
use rust_3::experiments::exp_2026_003a_mva_temporal_xor::config::{
    ExperimentConfig, ExperimentalCondition,
};
use rust_3::experiments::exp_2026_003a_mva_temporal_xor::dataset::TemporalStream;
use rust_3::experiments::exp_2026_003a_mva_temporal_xor::readout::{
    OnlineRidgeAccumulator, solve_linear_system,
};
use rust_3::experiments::exp_2026_003a_mva_temporal_xor::run_single_experiment;

#[test]
fn test_fast_rng_determinism() {
    let mut rng1 = FastRng::seed_from_u64(12345);
    let mut rng2 = FastRng::seed_from_u64(12345);

    for _ in 0..100 {
        assert_eq!(rng1.next_u64(), rng2.next_u64());
        assert_eq!(rng1.bernoulli(0.3), rng2.bernoulli(0.3));
    }
}

#[test]
fn test_linear_system_solver() {
    // 2x2 system:
    // 2x + y = 5
    // x + 3y = 5
    // Solution: x = 2, y = 1
    let a = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
    let b = vec![5.0, 5.0];
    let sol = solve_linear_system(a, b).expect("Linear system should be solvable");
    assert!((sol[0] - 2.0).abs() < 1e-6);
    assert!((sol[1] - 1.0).abs() < 1e-6);
}

#[test]
fn test_dataset_generation_xor_and_identity() {
    let mut config = ExperimentConfig {
        seed: 42,
        delay_tau: 3,
        condition: ExperimentalCondition::ActiveXor,
        ..Default::default()
    };
    let stream_xor = TemporalStream::generate(&config, 100);

    for t in 3..100 {
        let expected = (stream_xor.inputs[t] as usize) ^ (stream_xor.inputs[t - 3] as usize);
        assert_eq!(stream_xor.targets[t] as usize, expected);
    }

    config.condition = ExperimentalCondition::ActiveIdentity;
    let stream_id = TemporalStream::generate(&config, 100);
    for t in 3..100 {
        assert_eq!(stream_id.targets[t], stream_id.inputs[t - 3]);
    }
}

#[test]
fn test_online_ridge_accumulator_matches_solution() {
    // Simple 1D line: y = 2x + 1
    let mut accum = OnlineRidgeAccumulator::new(1);
    for i in 0..10 {
        let x = i as f64;
        let y = 2.0 * x + 1.0;
        accum.update(&[x], y);
    }
    let model = accum.solve(1e-6).expect("Model should solve");
    assert!((model.weights[0] - 2.0).abs() < 1e-2);
    assert!((model.bias - 1.0).abs() < 1e-2);
}

#[test]
fn test_single_run_active_xor_execution() {
    let config = ExperimentConfig {
        seed: 42,
        condition: ExperimentalCondition::ActiveXor,
        delay_tau: 3,
        pulse_density: 0.20,
        washout_ticks: 100,
        train_ticks: 500,
        test_ticks: 300,
        ..Default::default()
    };

    let metrics = run_single_experiment(&config);
    assert!(
        metrics.test_accuracy > 0.60,
        "Active XOR at tau=3 should exceed base rate (0.60): got {}",
        metrics.test_accuracy
    );
    assert!(
        metrics.mean_firing_density >= 0.05 && metrics.mean_firing_density <= 0.25,
        "Firing density should be in homeostatic range: got {}",
        metrics.mean_firing_density
    );
}

#[test]
fn test_linear_direct_baseline_fails_xor() {
    let config = ExperimentConfig {
        seed: 42,
        condition: ExperimentalCondition::BaselineLinearDirect,
        delay_tau: 3,
        pulse_density: 0.50, // balanced 50/50 bits
        washout_ticks: 100,
        train_ticks: 500,
        test_ticks: 300,
        ..Default::default()
    };

    let metrics = run_single_experiment(&config);
    // On 50% density, XOR cannot be separated linearly -> test accuracy ~ 50%
    assert!(
        (metrics.test_accuracy - 0.50).abs() < 0.08,
        "Linear baseline must yield ~50% on balanced XOR: got {}",
        metrics.test_accuracy
    );
    assert!(
        metrics.capacity_k_xor < 0.05,
        "Linear baseline must have zero or near-zero capacity: got {}",
        metrics.capacity_k_xor
    );
}
