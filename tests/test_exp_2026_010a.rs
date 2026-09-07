//! Integration test harness for EXP-2026-010a: Pushdown Memory and Context-Free Dyck Language Recognition.

use rust_3::experiments::exp_2026_010a_mva_pushdown_memory::*;

fn parse_tokens(s: &str) -> Vec<Token> {
    s.chars()
        .map(|c| match c {
            '(' => Token::OpenRnd,
            ')' => Token::CloseRnd,
            '[' => Token::OpenSqr,
            ']' => Token::CloseSqr,
            _ => panic!("Unknown token char: {c}"),
        })
        .collect()
}

#[test]
fn test_dyck1_clean_execution() {
    let cfg = ExperimentConfig::new(Condition::ActivePda, AutomatonTask::Dyck1, 0.0, 42);

    // Valid balanced sequences -> Accept (1)
    let valid_cases = vec!["()", "(())", "(()())", "((()))", "()()()()"];
    for s in valid_cases {
        let seq = EvaluationSequence {
            tokens: parse_tokens(s),
            target_label: 1,
            violation_type: "none".to_string(),
            nesting_depth: 2,
        };
        let trial = evaluate_sequence_trial(&cfg, &seq, 1);
        assert_eq!(
            trial.predicted_label, 1,
            "Valid Dyck-1 string '{s}' was rejected!"
        );
        assert!(trial.correct);
        assert_eq!(trial.pointer_crosstalk_ticks, 0);
    }

    // Invalid sequences -> Reject (0)
    let invalid_cases = vec![
        (")(", "underflow"),
        ("())(()", "underflow"),
        ("((", "unclosed"),
        ("(()", "unclosed"),
        ("())", "mismatch"),
        ("(()))", "mismatch"),
    ];
    for (s, v_type) in invalid_cases {
        let seq = EvaluationSequence {
            tokens: parse_tokens(s),
            target_label: 0,
            violation_type: v_type.to_string(),
            nesting_depth: 1,
        };
        let trial = evaluate_sequence_trial(&cfg, &seq, 2);
        assert_eq!(
            trial.predicted_label, 0,
            "Invalid Dyck-1 string '{s}' ({v_type}) was accepted!"
        );
        assert!(trial.correct);
    }
}

#[test]
fn test_dyck2_clean_execution() {
    let cfg = ExperimentConfig::new(Condition::ActivePda, AutomatonTask::Dyck2, 0.0, 42);

    // Valid Dyck-2 nested sequences -> Accept (1)
    let valid_cases = vec!["[]", "()", "[()]", "[()[]]", "(([]))", "[[()]()]"];
    for s in valid_cases {
        let seq = EvaluationSequence {
            tokens: parse_tokens(s),
            target_label: 1,
            violation_type: "none".to_string(),
            nesting_depth: 3,
        };
        let trial = evaluate_sequence_trial(&cfg, &seq, 1);
        assert_eq!(
            trial.predicted_label, 1,
            "Valid Dyck-2 string '{s}' was rejected!"
        );
        assert!(trial.correct);
        assert_eq!(trial.pointer_crosstalk_ticks, 0);
    }

    // Invalid Dyck-2 non-LIFO and mismatch violations -> Reject (0)
    let invalid_cases = vec![
        ("([)]", "cross_bracket"),
        ("[(])", "cross_bracket"),
        ("[([)])]", "cross_bracket"),
        ("(]", "mismatch"),
        ("[)", "mismatch"),
        ("[([))] ", "mismatch"),
        ("][", "underflow"),
        (")[", "underflow"),
        ("[(", "unclosed"),
        ("[[()]", "unclosed"),
    ];
    for (s, v_type) in invalid_cases {
        let s_clean = s.trim();
        let seq = EvaluationSequence {
            tokens: parse_tokens(s_clean),
            target_label: 0,
            violation_type: v_type.to_string(),
            nesting_depth: 2,
        };
        let trial = evaluate_sequence_trial(&cfg, &seq, 2);
        assert_eq!(
            trial.predicted_label, 0,
            "Invalid Dyck-2 string '{s_clean}' ({v_type}) was accepted!"
        );
        assert!(trial.correct);
    }
}

#[test]
fn test_depth_generalization_scaling() {
    let cfg = ExperimentConfig::new(
        Condition::ActivePda,
        AutomatonTask::DepthGeneralization,
        0.0,
        100,
    );

    // Test depths D in [1, 8]
    let depth_strings = vec![
        (1, "()"),
        (2, "[()]"),
        (3, "([()])"),
        (4, "[[()]]"),
        (5, "[([()])]"),
        (6, "([([()])])"),
        (7, "[[([([()])])]]"),
        (8, "([[([([()])])]])"),
    ];

    for (d, s) in depth_strings {
        let seq = EvaluationSequence {
            tokens: parse_tokens(s),
            target_label: 1,
            violation_type: "none".to_string(),
            nesting_depth: d,
        };
        let trial = evaluate_sequence_trial(&cfg, &seq, d);
        assert_eq!(
            trial.predicted_label, 1,
            "Active PDA failed on nesting depth D={d} ('{s}')!"
        );
        assert!(trial.correct);
        assert_eq!(trial.pointer_crosstalk_ticks, 0);
    }
}

#[test]
fn test_baseline_finite_state_depth_limitation() {
    let cfg = ExperimentConfig::new(
        Condition::BaselineFiniteState,
        AutomatonTask::DepthGeneralization,
        0.0,
        42,
    );

    // D <= 2 works
    let seq_d2 = EvaluationSequence {
        tokens: parse_tokens("[()]"),
        target_label: 1,
        violation_type: "none".to_string(),
        nesting_depth: 2,
    };
    let trial_d2 = evaluate_sequence_trial(&cfg, &seq_d2, 1);
    assert_eq!(
        trial_d2.predicted_label, 1,
        "Baseline FSM should accept D=2 within its physical capacity!"
    );

    // D >= 3 fails (overflows capacity D_cap = 2)
    let seq_d3 = EvaluationSequence {
        tokens: parse_tokens("((()))"),
        target_label: 1,
        violation_type: "none".to_string(),
        nesting_depth: 3,
    };
    let trial_d3 = evaluate_sequence_trial(&cfg, &seq_d3, 2);
    assert_eq!(
        trial_d3.predicted_label, 0,
        "Baseline FSM should fail/reject D=3 due to capacity overflow!"
    );
    assert!(
        !trial_d3.correct,
        "Baseline FSM unexpectedly marked correct on D=3!"
    );
}

#[test]
fn test_active_pda_clean_gates() {
    let mut cfg = ExperimentConfig::new(Condition::ActivePda, AutomatonTask::Dyck1, 0.0, 1);
    cfg.seq_length = 8;
    let dataset = generate_dyck1_dataset(1, 8, 20);
    let (run_rec, _) = execute_run(&cfg, &dataset);

    assert_eq!(
        run_rec.metrics.accuracy_seq, 1.0,
        "Active PDA failed accuracy gate: {}",
        run_rec.metrics.accuracy_seq
    );
    assert_eq!(
        run_rec.metrics.fidelity_reject, 1.0,
        "Active PDA failed rejection fidelity: {}",
        run_rec.metrics.fidelity_reject
    );
    assert_eq!(
        run_rec.metrics.pointer_crosstalk_rate, 0.0,
        "Active PDA failed zero-crosstalk invariant: {}",
        run_rec.metrics.pointer_crosstalk_rate
    );
    assert!(
        run_rec.homeostasis_stable,
        "Active PDA violated homeostatic density bounds: {}",
        run_rec.metrics.mean_firing_density
    );
}
