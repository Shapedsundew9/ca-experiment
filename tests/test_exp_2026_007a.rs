//! Integration test harness for EXP-2026-007a: Cascaded Boolean Logic Gate Composition and Planar Wire Crossing.

use rust_3::experiments::exp_2026_007a_mva_multi_gate_composition::*;

#[test]
fn test_full_adder_truth_table_all_8_states() {
    let cfg = ExperimentConfig::new(Condition::ActiveComposed, 0.0, 42);
    let mut circuit = build_full_adder_circuit(cfg);

    // Warmup settling window
    for _ in 0..50 {
        circuit.step((0.0, 0.0, 0.0), 0.0, false);
    }

    for k in 0..8 {
        let a = ((k >> 2) & 1) as f64;
        let b = ((k >> 1) & 1) as f64;
        let c = (k & 1) as f64;

        let exp_s = expected_sum((k >> 2) & 1, (k >> 1) & 1, k & 1);
        let exp_c = expected_cout((k >> 2) & 1, (k >> 1) & 1, k & 1);

        // Inject inputs at t = 0
        circuit.step((a, b, c), 0.0, false);

        // Advance 6 clock ticks
        for _ in 1..7 {
            circuit.step((0.0, 0.0, 0.0), 0.0, false);
        }

        // Sample at t = 7 (steady-state latency tau_adder = 7 ticks)
        let (sum_act, cout_act, _cin_post) = circuit.step((0.0, 0.0, 0.0), 0.0, false);

        assert_eq!(
            sum_act, exp_s,
            "State {k} ({a},{b},{c}): Sum mismatch! Expected {exp_s}, got {sum_act}"
        );
        assert_eq!(
            cout_act, exp_c,
            "State {k} ({a},{b},{c}): Cout mismatch! Expected {exp_c}, got {cout_act}"
        );

        // Relaxation interval
        for _ in 0..4 {
            circuit.step((0.0, 0.0, 0.0), 0.0, false);
        }
    }
}

#[test]
fn test_gate_1_and_2_active_clean_parity_and_zero_attenuation() {
    let cfg = ExperimentConfig::new(Condition::ActiveComposed, 0.0, 1);
    let rec = execute_single_run(&cfg);

    // Gate 1: Exact truth table parity
    assert_eq!(
        rec.metrics.truth_table_accuracy, 1.0,
        "Active condition failed Gate 1 parity"
    );
    assert_eq!(rec.metrics.states_correct, 8);
    assert_eq!(rec.metrics.states_total, 8);

    // Gate 2: Zero intermediate attenuation
    assert_eq!(
        rec.metrics.ber_sum, 0.0,
        "Active condition failed Gate 2 BER_sum"
    );
    assert_eq!(
        rec.metrics.ber_cout, 0.0,
        "Active condition failed Gate 2 BER_cout"
    );
    assert_eq!(rec.metrics.ber_mean, 0.0);

    // Latency matching
    assert_eq!(rec.metrics.latency_sum, 7);
    assert_eq!(rec.metrics.latency_cout, 7);
    assert_eq!(rec.metrics.latency_skew, 0);

    // Gate 6: Homeostasis
    assert!(rec.homeostasis_stable);
}

#[test]
fn test_gate_3_crossing_crosstalk_isolation() {
    // Active condition: shielded crossing bridge has zero crosstalk
    let active_cfg = ExperimentConfig::new_crosstalk(Condition::ActiveComposed, 1);
    let active_rec = execute_single_run(&active_cfg);
    assert!(
        active_rec.metrics.crosstalk_cross < 1e-4,
        "Active crosstalk isolation failed: chi={}",
        active_rec.metrics.crosstalk_cross
    );

    // Unshielded ablation: 4-way collision node causes severe leakage
    let unshielded_cfg = ExperimentConfig::new_crosstalk(Condition::AblationUnshieldedCrossing, 1);
    let unshielded_rec = execute_single_run(&unshielded_cfg);
    assert!(
        unshielded_rec.metrics.crosstalk_cross >= 0.20,
        "Unshielded ablation failed to exhibit crosstalk leakage: chi={}",
        unshielded_rec.metrics.crosstalk_cross
    );
}

#[test]
fn test_gate_4_noise_tolerance() {
    let active_noise_cfg = ExperimentConfig::new(Condition::ActiveComposed, 0.05, 1);
    let active_rec = execute_single_run(&active_noise_cfg);

    assert!(
        active_rec.metrics.truth_table_accuracy >= 0.950,
        "Active condition failed Gate 4 accuracy under noise: {}",
        active_rec.metrics.truth_table_accuracy
    );
    assert!(
        active_rec.metrics.ber_mean <= 0.025,
        "Active condition failed Gate 4 BER under noise: {}",
        active_rec.metrics.ber_mean
    );

    // Fixed theta ablation under noise has worse performance
    let fixed_noise_cfg = ExperimentConfig::new(Condition::AblationFixedTheta, 0.05, 1);
    let fixed_rec = execute_single_run(&fixed_noise_cfg);
    assert!(
        fixed_rec.metrics.truth_table_accuracy < active_rec.metrics.truth_table_accuracy,
        "Fixed theta unexpectedly outperformed active under noise"
    );
}

#[test]
fn test_gate_5_delay_equalization_advantage() {
    let uncomp_cfg = ExperimentConfig::new(Condition::BaselineUncompensatedDelay, 0.0, 1);
    let uncomp_rec = execute_single_run(&uncomp_cfg);

    assert!(
        uncomp_rec.metrics.truth_table_accuracy <= 0.500,
        "Uncompensated delay unexpectedly achieved high accuracy: {}",
        uncomp_rec.metrics.truth_table_accuracy
    );
    assert!(
        uncomp_rec.metrics.ber_mean >= 0.20,
        "Uncompensated delay unexpectedly had low BER: {}",
        uncomp_rec.metrics.ber_mean
    );
}
