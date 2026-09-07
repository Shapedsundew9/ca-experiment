//! Integration test harness for EXP-2026-008a: Bistable Resonant Latching and Nondestructive Dynamic Bit Storage.

use rust_3::experiments::exp_2026_008a_mva_bistable_latching::*;

#[test]
fn test_bistable_latch_quiescent_retention_clean() {
    let cfg = ExperimentConfig::new(Condition::ActiveLatch, 0.0, 42);
    let mut circuit = build_bistable_latch_circuit(cfg);

    // Warmup settling window
    for _ in 0..50 {
        circuit.step((0.0, 0.0, 0.0, 0.0, 0.0), false);
    }

    // State 1: Write 1 (D=1, WE=1)
    circuit.step((1.0, 1.0, 0.0, 0.0, 0.0), true);

    // Quiescent horizon (1000 ticks)
    for _ in 0..1000 {
        circuit.step((0.0, 0.0, 0.0, 0.0, 0.0), true);
    }

    // Read probe over 4 ticks: verify pulse circulating
    let mut s1_detected = false;
    for _ in 0..4 {
        let (vs, vr) = circuit.step((0.0, 0.0, 0.0, 0.0, 1.0), true);
        if vs > 0.0 || vr > 0.0 {
            s1_detected = true;
        }
    }
    assert!(
        s1_detected,
        "State 1 pulse extinguished after 1000 quiescent ticks!"
    );

    // State 0: Write 0 (D=0, WE=1)
    circuit.step((0.0, 1.0, 0.0, 0.0, 0.0), true);

    // Quiescent horizon (1000 ticks)
    for _ in 0..1000 {
        circuit.step((0.0, 0.0, 0.0, 0.0, 0.0), true);
    }

    // Read probe over 4 ticks: verify silence
    let mut s0_detected = false;
    for _ in 0..4 {
        let (vs, vr) = circuit.step((0.0, 0.0, 0.0, 0.0, 1.0), true);
        if vs > 0.0 || vr > 0.0 {
            s0_detected = true;
        }
    }
    assert!(
        !s0_detected,
        "State 0 ignited spurious limit cycle after 1000 quiescent ticks!"
    );
}

#[test]
fn test_gate_1_and_2_active_clean_retention_and_nondestructive_read() {
    let cfg = ExperimentConfig::new(Condition::ActiveLatch, 0.0, 1);
    let rec = execute_single_run(&cfg);

    // Gate 1: State retention accuracy
    assert_eq!(
        rec.metrics.retention_accuracy, 1.0,
        "Active condition failed Gate 1 retention accuracy: {}",
        rec.metrics.retention_accuracy
    );
    assert!(rec.metrics.q0_retained, "Q=0 not retained");
    assert!(rec.metrics.q1_retained, "Q=1 not retained");

    // Gate 2: Nondestructive readout fidelity
    assert_eq!(
        rec.metrics.ber_read, 0.0,
        "Active condition failed Gate 2 BER: {}",
        rec.metrics.ber_read
    );
    assert!(
        rec.metrics.pulse_persists_post_read,
        "Active condition failed Gate 2 post-read pulse persistence"
    );

    // Gate 6: Homeostasis
    assert!(
        rec.homeostasis_stable,
        "Active condition violated homeostatic density envelope"
    );
}

#[test]
fn test_gate_3_state_transition_reliability() {
    let cfg = ExperimentConfig::new_transition(Condition::ActiveLatch, 1);
    let rec = execute_single_run(&cfg);

    assert_eq!(
        rec.metrics.transition_fidelity, 1.0,
        "Active condition failed Gate 3 transition fidelity: {}",
        rec.metrics.transition_fidelity
    );
    assert_eq!(rec.metrics.transitions_correct, 4);
    assert_eq!(rec.metrics.transitions_total, 4);
}

#[test]
fn test_gate_4_noise_resilience_under_critical_percolation() {
    let active_noise_cfg = ExperimentConfig::new(Condition::ActiveLatch, 0.05, 2);
    let active_rec = execute_single_run(&active_noise_cfg);

    assert!(
        active_rec.metrics.retention_accuracy >= 0.950,
        "Active condition failed Gate 4 accuracy under noise: {}",
        active_rec.metrics.retention_accuracy
    );
    assert!(
        active_rec.metrics.ber_read <= 0.025,
        "Active condition failed Gate 4 BER under noise: {}",
        active_rec.metrics.ber_read
    );

    // Fixed theta ablation under noise has worse performance
    let fixed_noise_cfg = ExperimentConfig::new(Condition::AblationFixedTheta, 0.05, 2);
    let fixed_rec = execute_single_run(&fixed_noise_cfg);
    assert!(
        fixed_rec.metrics.retention_accuracy < active_rec.metrics.retention_accuracy
            || fixed_rec.metrics.ber_read > active_rec.metrics.ber_read,
        "Fixed theta unexpectedly outperformed active under noise"
    );
}

#[test]
fn test_gate_5_feedforward_loss_baseline_failure() {
    let loss_cfg = ExperimentConfig::new(Condition::BaselineFeedforwardLoss, 0.0, 1);
    let loss_rec = execute_single_run(&loss_cfg);

    assert_eq!(
        loss_rec.metrics.retention_accuracy, 0.500,
        "Feedforward loss unexpectedly retained state: {}",
        loss_rec.metrics.retention_accuracy
    );
    assert!(
        !loss_rec.metrics.q1_retained,
        "Q=1 was retained in feedforward loss"
    );
    assert!(
        loss_rec.metrics.q0_retained,
        "Q=0 was not retained in feedforward loss"
    );
    assert!(
        loss_rec.metrics.ber_read >= 0.50,
        "Feedforward loss unexpectedly achieved low BER: {}",
        loss_rec.metrics.ber_read
    );
}

#[test]
fn test_ablation_unshielded_feedback_degradation() {
    let cfg = ExperimentConfig::new_transition(Condition::AblationUnshieldedFeedback, 1);
    let rec = execute_single_run(&cfg);

    // Unshielded feedback fails reset operations due to retrograde reflections and standing waves
    assert!(
        rec.metrics.transition_fidelity < 1.0,
        "Unshielded feedback unexpectedly passed all transitions: {}",
        rec.metrics.transition_fidelity
    );
    assert_eq!(
        rec.metrics.transitions_correct, 2,
        "Expected exactly 2/4 transitions correct in unshielded condition"
    );
}
