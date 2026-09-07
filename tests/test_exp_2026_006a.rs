//! Integration test harness for EXP-2026-006a: Directed Regenerative Transmission Tracks and Branching Fan-Out.

use rust_3::experiments::exp_2026_006a_mva_signal_transport::*;

#[test]
fn test_topology_graph_construction_and_invariants() {
    for &d in &[10, 20, 30, 50] {
        for &j in &[0, 2, 5] {
            let topo = TransportTopology::new(d, j);
            let l_stem = d / 2;
            let l_b1 = d - l_stem;
            let l_b2 = l_b1 + j;
            let expected_nodes = l_stem + l_b1 + l_b2;

            assert_eq!(topo.total_nodes, expected_nodes);
            assert_eq!(topo.fork_idx, l_stem - 1);
            assert_eq!(topo.branch1_root_idx, l_stem);
            assert_eq!(topo.branch1_readout_idx, l_stem + l_b1 - 1);
            assert_eq!(topo.branch2_root_idx, l_stem + l_b1);
            assert_eq!(topo.branch2_readout_idx, expected_nodes - 1);

            // Node 0 has no forward in-neighbors from lattice (sensory ingress port)
            assert!(
                topo.in_neighbors[0]
                    .iter()
                    .all(|&(_, w_fwd, _)| w_fwd == 0.0)
            );
            assert_eq!(topo.in_neighbors[topo.branch1_root_idx][0].0, topo.fork_idx);
            assert_eq!(topo.in_neighbors[topo.branch2_root_idx][0].0, topo.fork_idx);
        }
    }
}

#[test]
fn test_signal_generators_and_nyquist_refractory_limit() {
    for &pat in &SignalPattern::ALL {
        let stream = generate_input_stream(pat, 200, 42);
        assert_eq!(stream.len(), 200);

        // Verify Nyquist refractory limit: no consecutive spikes (ISI >= 3 ticks)
        let mut last_spike: Option<usize> = None;
        for (t, &val) in stream.iter().enumerate() {
            assert!(val == 0.0 || val == 1.0);
            if val == 1.0 {
                if let Some(prev) = last_spike {
                    let isi = t - prev;
                    assert!(
                        isi >= 3,
                        "Pattern {:?} violated refractory Nyquist limit with ISI={} at tick {}",
                        pat,
                        isi,
                        t
                    );
                }
                last_spike = Some(t);
            }
        }
    }
}

#[test]
fn test_zero_attenuation_and_fanout_gates() {
    for &d in &[30, 50] {
        for &pat in &SignalPattern::ALL {
            let cfg = ExperimentConfig::new(Condition::ActiveRegenerative, d, 0, 0.0, pat, 1);
            let rec = execute_single_run(&cfg);

            // Gate 1: Zero Attenuation BER_1 = BER_2 = 0.000
            assert_eq!(
                rec.metrics.ber_branch1, 0.0,
                "Failed Gate 1 BER_1 for pattern {:?} at D={}",
                pat, d
            );
            assert_eq!(
                rec.metrics.ber_branch2, 0.0,
                "Failed Gate 1 BER_2 for pattern {:?} at D={}",
                pat, d
            );

            // Gate 2: Fidelity F = 1.000
            assert!(
                (rec.metrics.fidelity - 1.0).abs() < 1e-9,
                "Failed Gate 2 Fidelity for pattern {:?} at D={}",
                pat,
                d
            );

            // Constant velocity latency invariant: tau = D
            assert_eq!(rec.metrics.latency_branch1, d);
            assert_eq!(rec.metrics.latency_branch2, d);
            assert_eq!(rec.metrics.latency_skew, 0);
        }
    }
}

#[test]
fn test_crosstalk_isolation_gate() {
    // Gate 3: Active condition inter-branch crosstalk chi_{1 -> 2} < 10^-4
    let active_cfg = ExperimentConfig::new_crosstalk(Condition::ActiveRegenerative, 1);
    let active_rec = execute_single_run(&active_cfg);
    assert!(
        active_rec.metrics.crosstalk_1_to_2 < 1e-4,
        "Active crosstalk isolation failed: chi={}",
        active_rec.metrics.crosstalk_1_to_2
    );

    // Unshielded ablation exhibits retrograde reflection leakage
    let unshielded_cfg = ExperimentConfig::new_crosstalk(Condition::AblationUnshielded, 1);
    let unshielded_rec = execute_single_run(&unshielded_cfg);
    assert!(
        unshielded_rec.metrics.crosstalk_1_to_2 >= 0.05,
        "Unshielded ablation failed to exhibit crosstalk leakage: chi={}",
        unshielded_rec.metrics.crosstalk_1_to_2
    );
}

#[test]
fn test_jitter_constant_velocity_skew() {
    for &j in &[0, 2, 5] {
        let cfg = ExperimentConfig::new(
            Condition::ActiveRegenerative,
            30,
            j,
            0.0,
            SignalPattern::AlternatingClock,
            1,
        );
        let rec = execute_single_run(&cfg);

        assert_eq!(rec.metrics.latency_branch1, 30);
        assert_eq!(rec.metrics.latency_branch2, 30 + j);
        assert_eq!(rec.metrics.latency_skew, j);
        assert_eq!(rec.metrics.ber_branch1, 0.0);
        assert_eq!(rec.metrics.ber_branch2, 0.0);
        assert_eq!(rec.metrics.fidelity, 1.0);
    }
}

#[test]
fn test_passive_baseline_attenuation_extinction() {
    // At D = 30, passive analog cable decays to extinction, causing bit dropouts
    let cfg = ExperimentConfig::new(
        Condition::BaselinePassive,
        30,
        0,
        0.0,
        SignalPattern::AlternatingClock,
        1,
    );
    let rec = execute_single_run(&cfg);

    assert!(
        rec.metrics.ber_mean > 0.10,
        "Passive baseline unexpectedly propagated: BER={}",
        rec.metrics.ber_mean
    );
    assert!(
        rec.metrics.fidelity < 0.80,
        "Passive baseline fidelity too high: F={}",
        rec.metrics.fidelity
    );
}

#[test]
fn test_welch_t_test_and_cohen_d_computation() {
    let s1 = (0..30).map(|i| 0.01 * (i % 3) as f64).collect::<Vec<_>>();
    let s2 = (0..30)
        .map(|i| 0.25 + 0.01 * (i % 3) as f64)
        .collect::<Vec<_>>();

    let (t, p, d) = welch_t_test(&s1, &s2);
    assert!(t < -20.0, "Expected large negative t, got {t}");
    assert!(p < 1e-6, "Expected p < 1e-6, got {p}");
    assert!(d.abs() > 2.0, "Expected |d| > 2.0, got {d}");
}
