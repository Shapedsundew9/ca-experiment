//! Integration tests for EXP-2026-005a: Continual Multi-Pattern Learning & Metaplasticity.

use rust_3::experiments::exp_2026_005a_mva_continual_learning::*;

#[test]
fn test_substrate_initialization_and_invariants() {
    let config = ExperimentConfig::default();
    let sub = TorusSubstrate::new(config);

    assert_eq!(sub.nodes.len(), 16);
    for i in 0..16 {
        assert_eq!(sub.neighbors_in[i].len(), 4);
        assert_eq!(sub.nodes[i].theta, 1.0);
        let mut in_weight_sum = 0.0;
        for &j in &sub.neighbors_in[i] {
            assert_eq!(sub.weights[i][j], 1.0);
            in_weight_sum += sub.weights[i][j];
        }
        assert!((in_weight_sum - 4.0).abs() < 1e-12);
    }
}

#[test]
fn test_curriculum_patterns_orthogonality() {
    let nodes_a = PatternId::A.nodes();
    let nodes_b = PatternId::B.nodes();
    let nodes_c = PatternId::C.nodes();

    // Check disjoint sensory ingress
    for &na in &nodes_a {
        assert!(!nodes_b.contains(&na));
        assert!(!nodes_c.contains(&na));
    }
    for &nb in &nodes_b {
        assert!(!nodes_c.contains(&nb));
    }

    // Check bitstream periods
    for t in 0..16 {
        assert_eq!(PatternId::A.bit_at(t), PatternId::A.bit_at(t % 8));
        assert_eq!(PatternId::B.bit_at(t), PatternId::B.bit_at(t % 8));
        assert_eq!(PatternId::C.bit_at(t), PatternId::C.bit_at(t % 8));
    }
}

#[test]
fn test_welch_t_test_and_overlap() {
    let sample1 = vec![1.0, 1.1, 1.2, 0.9, 1.0, 1.05];
    let sample2 = vec![0.5, 0.6, 0.55, 0.48, 0.52, 0.58];

    let (t_stat, p_val, cohen_d) = welch_t_test(&sample1, &sample2);
    assert!(t_stat > 5.0);
    assert!(p_val < 1e-4);
    assert!(cohen_d > 2.0);

    // Overlap test
    let mut w1 = [[1.0; NUM_NODES]; NUM_NODES];
    let mut w2 = [[1.0; NUM_NODES]; NUM_NODES];
    let sub = TorusSubstrate::new(ExperimentConfig::default());

    // Identical displacement should give overlap 1.0
    w1[0][sub.neighbors_in[0][0]] = 2.0;
    w2[0][sub.neighbors_in[0][0]] = 2.0;
    let overlap = compute_subspace_overlap(&w1, &w2, &sub.neighbors_in);
    assert!((overlap - 1.0).abs() < 1e-6);

    // Orthogonal displacement should give overlap 0.0
    let mut w3 = [[1.0; NUM_NODES]; NUM_NODES];
    w3[0][sub.neighbors_in[0][1]] = 2.0;
    let overlap_orth = compute_subspace_overlap(&w1, &w3, &sub.neighbors_in);
    assert!(overlap_orth.abs() < 1e-6);
}

#[test]
fn test_fast_sweep_and_telemetry_emission() {
    let temp_dir = std::env::temp_dir().join("exp_2026_005a_test_smoke");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let configs = generate_factorial_configs(1, &[50]);
    assert_eq!(configs.len(), 7); // 1 static + 3 plastic + 3 joint = 7 configs for 1 t_train, 1 seed

    let (results, manifest) = run_factorial_sweep(configs, 2);
    assert_eq!(results.len(), 7);
    assert_eq!(manifest.total_runs, 7);

    emit_telemetry(&temp_dir, &results, &manifest).expect("Failed to emit telemetry");

    assert!(temp_dir.join("run_results.jsonl").exists());
    assert!(temp_dir.join("telemetry.ndjson").exists());
    assert!(temp_dir.join("summary_evaluation.json").exists());
    assert!(temp_dir.join("summary.json").exists());

    let _ = std::fs::remove_dir_all(&temp_dir);
}
