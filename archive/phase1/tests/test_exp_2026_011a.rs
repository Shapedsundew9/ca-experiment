//! Integration test harness for EXP-2026-011a: Associative Key-Value Retrieval ("Needle in a Haystack").

use rust_3::experiments::exp_2026_011a_mva_associative_retrieval::*;

#[test]
fn test_graph_topology_invariants() {
    let cfg = ExperimentConfig::new(Condition::ActiveAssociative, 0.0, 64, 42);
    let substrate = build_circuit(cfg);

    // Invariant 1: Exactly 272 excitable nodes
    assert_eq!(
        substrate.nodes.len(),
        272,
        "Total node count must equal 272"
    );

    // Invariant 2: Fan-in degree bound k_in <= 4
    for (i, in_edges) in substrate.in_neighbors.iter().enumerate() {
        assert!(
            in_edges.len() <= 4,
            "Node {} ({}) exceeds max fan-in k_in <= 4 (got {})",
            i,
            substrate.nodes[i].name,
            in_edges.len()
        );
    }

    // Invariant 3: Memory partitions
    assert_eq!(substrate.slot_rings.len(), 16);
    assert_eq!(substrate.write_gates.len(), 16);
    assert_eq!(substrate.quench_nodes.len(), 16);
    assert_eq!(substrate.query_gates.len(), 16);
    assert_eq!(substrate.sink_nodes.len(), 3);
}

#[test]
fn test_clean_baseline_retrieval() {
    let cfg = ExperimentConfig::new(Condition::ActiveAssociative, 0.0, 64, 101);
    let mut rng = FastRng::seed_from_u64(101);
    let dataset = generate_dataset(64, 10, &mut rng);

    for (i, seq) in dataset.iter().enumerate() {
        let trial = evaluate_sequence_trial(&cfg, seq, i + 1);
        assert!(
            trial.correct,
            "Trial {} failed retrieval for queried key {} (target {}, pred {})",
            i + 1,
            trial.queried_key,
            trial.target_value,
            trial.predicted_value
        );
        assert_eq!(trial.distractor_leakage_events, 0);
        assert_eq!(trial.mutex_violation_ticks, 0);
    }
}

#[test]
fn test_distractor_immunity_long_horizon() {
    let cfg = ExperimentConfig::new(Condition::ActiveAssociative, 0.0, 256, 202);
    let mut rng = FastRng::seed_from_u64(202);
    let dataset = generate_dataset(256, 5, &mut rng);

    for (i, seq) in dataset.iter().enumerate() {
        let trial = evaluate_sequence_trial(&cfg, seq, i + 1);
        assert!(trial.correct);
        assert_eq!(
            trial.distractor_leakage_events,
            0,
            "Distractor leakage detected during trial {}",
            i + 1
        );
        assert_eq!(trial.mutex_violation_ticks, 0);
    }
}

#[test]
fn test_unindexed_baseline_collapses() {
    let cfg = ExperimentConfig::new(Condition::BaselineUnindexed, 0.0, 64, 303);
    let mut rng = FastRng::seed_from_u64(303);
    let dataset = generate_dataset(64, 50, &mut rng);

    let (rec, _) = execute_run(&cfg, &dataset);
    // Unindexed control should achieve ~0.531 accuracy, well below active 1.000
    assert!(
        rec.metrics.accuracy_retrieval < 0.70,
        "Unindexed baseline achieved unexpectedly high accuracy: {:.4}",
        rec.metrics.accuracy_retrieval
    );
}

#[test]
fn test_unshielded_ablation_fails() {
    let cfg = ExperimentConfig::new(Condition::AblationUnshieldedRetrieval, 0.0, 64, 404);
    let mut rng = FastRng::seed_from_u64(404);
    let dataset = generate_dataset(64, 30, &mut rng);

    let (rec, _) = execute_run(&cfg, &dataset);
    // Unshielded reflections cause mutex violations and readout collisions
    assert!(
        rec.metrics.mutex_violation_rate > 0.0,
        "Expected mutex violations in unshielded ablation, got 0"
    );
}

#[test]
fn test_noise_percolation_shielding() {
    let cfg_active = ExperimentConfig::new(Condition::ActiveAssociative, 0.05, 64, 505);
    let cfg_fixed = ExperimentConfig::new(Condition::AblationFixedTheta, 0.05, 64, 505);

    let mut rng = FastRng::seed_from_u64(505);
    let dataset = generate_dataset(64, 30, &mut rng);

    let (rec_active, trials) = execute_run(&cfg_active, &dataset);
    let (rec_fixed, _) = execute_run(&cfg_fixed, &dataset);

    let correct_count = trials.iter().filter(|t| t.correct).count();
    eprintln!(
        "Active accuracy: {:.4} ({}/{}), Fixed accuracy: {:.4}",
        rec_active.metrics.accuracy_retrieval,
        correct_count,
        trials.len(),
        rec_fixed.metrics.accuracy_retrieval
    );

    assert!(
        rec_active.metrics.accuracy_retrieval >= 0.90,
        "Active associative accuracy under eps=0.05 was {:.4} (< 0.90)",
        rec_active.metrics.accuracy_retrieval
    );
    assert!(
        rec_fixed.metrics.accuracy_retrieval < rec_active.metrics.accuracy_retrieval,
        "Fixed theta ({:.4}) should perform worse than active ({:.4}) under noise",
        rec_fixed.metrics.accuracy_retrieval,
        rec_active.metrics.accuracy_retrieval
    );
}
