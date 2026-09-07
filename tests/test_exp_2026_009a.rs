//! Integration test harness for EXP-2026-009a: Finite State Automata and Regular Language Recognition.

use rust_3::experiments::exp_2026_009a_mva_finite_state_automata::*;

#[test]
fn test_parity_dfa_clean_execution() {
    let cfg = ExperimentConfig::new(Condition::ActiveDfa, AutomatonTask::ParityDfa, 0.0, 42);

    // Test even parity bitstream: [1, 0, 1, 0] -> Parity 0 (Even, Reject)
    let seq_even = EvaluationSequence {
        tokens: vec![1, 0, 1, 0],
        target_label: 0,
        expected_states: vec![1, 1, 0, 0],
    };
    let trial_even = evaluate_sequence_trial(&cfg, &seq_even, 1);
    assert_eq!(trial_even.target_label, 0);
    assert_eq!(trial_even.predicted_label, 0);
    assert!(trial_even.correct);
    assert_eq!(trial_even.crosstalk_ticks, 0);

    // Test odd parity bitstream: [1, 0, 1, 1] -> Parity 1 (Odd, Accept)
    let seq_odd = EvaluationSequence {
        tokens: vec![1, 0, 1, 1],
        target_label: 1,
        expected_states: vec![1, 1, 0, 1],
    };
    let trial_odd = evaluate_sequence_trial(&cfg, &seq_odd, 2);
    assert_eq!(trial_odd.target_label, 1);
    assert_eq!(trial_odd.predicted_label, 1);
    assert!(trial_odd.correct);
    assert_eq!(trial_odd.crosstalk_ticks, 0);
}

#[test]
fn test_regex_dfa_clean_execution() {
    let cfg = ExperimentConfig::new(Condition::ActiveDfa, AutomatonTask::RegexDfa, 0.0, 42);

    // Test positive string: 1 0 1 0 1 -> Matches (10)+1 -> Accept (1)
    let seq_pos = EvaluationSequence {
        tokens: vec![1, 0, 1, 0, 1],
        target_label: 1,
        expected_states: vec![1, 2, 3, 2, 3],
    };
    let trial_pos = evaluate_sequence_trial(&cfg, &seq_pos, 1);
    assert_eq!(trial_pos.target_label, 1);
    assert_eq!(trial_pos.predicted_label, 1);
    assert!(trial_pos.correct);
    assert_eq!(trial_pos.crosstalk_ticks, 0);

    // Test negative string: 1 0 1 0 0 -> Ends in trap -> Reject (0)
    let seq_neg = EvaluationSequence {
        tokens: vec![1, 0, 1, 0, 0],
        target_label: 0,
        expected_states: vec![1, 2, 3, 2, 4],
    };
    let trial_neg = evaluate_sequence_trial(&cfg, &seq_neg, 2);
    assert_eq!(trial_neg.target_label, 0);
    assert_eq!(trial_neg.predicted_label, 0);
    assert!(trial_neg.correct);
}

#[test]
fn test_gates_1_to_3_and_6_active_clean() {
    let mut cfg = ExperimentConfig::new(Condition::ActiveDfa, AutomatonTask::ParityDfa, 0.0, 1);
    cfg.seq_length = 10;
    let (run_rec, _) = execute_run(&cfg, 20);

    // Gate 1: Sequence accuracy
    assert_eq!(
        run_rec.metrics.accuracy_seq, 1.0,
        "Active condition failed Gate 1 accuracy: {}",
        run_rec.metrics.accuracy_seq
    );

    // Gate 2: Transition fidelity
    assert_eq!(
        run_rec.metrics.fidelity_trans, 1.0,
        "Active condition failed Gate 2 fidelity: {}",
        run_rec.metrics.fidelity_trans
    );

    // Gate 3: Mutual exclusivity (zero crosstalk)
    assert_eq!(
        run_rec.metrics.crosstalk_violation_rate, 0.0,
        "Active condition failed Gate 3 crosstalk: {}",
        run_rec.metrics.crosstalk_violation_rate
    );

    // Gate 6: Homeostatic density
    assert!(
        run_rec.homeostasis_stable,
        "Active condition violated homeostatic density envelope: {}",
        run_rec.metrics.mean_firing_density
    );
}

#[test]
fn test_gate_5_feedforward_loss_collapse() {
    let mut cfg = ExperimentConfig::new(
        Condition::BaselineFeedforwardLoss,
        AutomatonTask::ParityDfa,
        0.0,
        1,
    );
    cfg.seq_length = 10;
    let (run_rec, _) = execute_run(&cfg, 20);

    // Feedforward loss fails on odd sequences because Ring 1 cannot maintain resonance
    assert!(
        run_rec.metrics.accuracy_seq <= 0.60,
        "Feedforward loss did not collapse as expected: {}",
        run_rec.metrics.accuracy_seq
    );
}
