//! Runner and factorial sweep orchestrator for EXP-2026-005a.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::config::{Condition, ExperimentConfig};
use super::curriculum::{FastRng, NUM_NODES, PatternId, generate_pattern_ingress};
use super::metrics::{compute_subspace_overlap, hamming_dist, measure_basin_depth, welch_t_test};
use super::substrate::TorusSubstrate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotBasins {
    pub basin_a: f64,
    pub basin_b: f64,
    pub basin_c: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshots {
    pub after_a: SnapshotBasins,
    pub after_b: SnapshotBasins,
    pub after_c: SnapshotBasins,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetricsRecord {
    pub bwt: f64,
    pub ar: f64,
    pub forgetting_a: f64,
    pub forgetting_b: f64,
    pub forgetting_mean: f64,
    pub d_sep_min: f64,
    pub d_sep_ab: f64,
    pub d_sep_bc: f64,
    pub d_sep_ac: f64,
    pub overlap_ac: f64,
    pub mean_firing_density: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunTelemetryRecord {
    pub run_id: String,
    pub seed: u64,
    pub condition: String,
    pub kappa: f64,
    pub t_train: usize,
    pub snapshots: Snapshots,
    pub metrics: RunMetricsRecord,
    pub homeostasis_stable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionSummary {
    pub bwt_mean: f64,
    pub bwt_std: f64,
    pub ar_mean: f64,
    pub ar_std: f64,
    pub d_sep_mean: f64,
    pub forgetting_mean: f64,
    pub sample_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatTestSummary {
    pub t_stat: f64,
    pub p_value: f64,
    pub cohen_d: f64,
    pub significant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryManifest {
    pub protocol_id: String,
    pub hypothesis_id: String,
    pub timestamp_utc: String,
    pub total_runs: usize,
    pub conditions: BTreeMap<String, ConditionSummary>,
    pub statistical_tests: BTreeMap<String, StatTestSummary>,
    pub gate_verdicts: BTreeMap<String, String>,
    pub overall_verdict: String,
}

/// Execute a single experiment run
pub fn run_single_experiment(config: &ExperimentConfig) -> RunTelemetryRecord {
    let mut substrate = TorusSubstrate::new(config.clone());
    let mut rng = FastRng::seed_from_u64(config.seed);

    let t_train = config.t_train;
    let total_ticks = 3 * t_train;
    let mut total_spikes = 0.0;

    let mut snap_w_a = substrate.snapshot_weights();
    let mut snap_w_c = substrate.snapshot_weights();

    let mut snapshot_after_a = SnapshotBasins {
        basin_a: 0.0,
        basin_b: 0.0,
        basin_c: 0.0,
    };
    let mut snapshot_after_b = SnapshotBasins {
        basin_a: 0.0,
        basin_b: 0.0,
        basin_c: 0.0,
    };
    let mut snapshot_after_c = SnapshotBasins {
        basin_a: 0.0,
        basin_b: 0.0,
        basin_c: 0.0,
    };

    let mut occ_a_final = [0.0; NUM_NODES];
    let mut occ_b_final = [0.0; NUM_NODES];
    let mut occ_c_final = [0.0; NUM_NODES];

    let is_joint = config.condition == Condition::ControlJointTraining;
    let plasticity_active = config.condition != Condition::BaselineStatic;

    for t in 0..total_ticks {
        let pattern = if is_joint {
            // Joint interleaved curriculum: rotate pattern every 8 ticks (1 period)
            PatternId::ALL[(t / 8) % 3]
        } else if t < t_train {
            PatternId::A
        } else if t < 2 * t_train {
            PatternId::B
        } else {
            PatternId::C
        };

        let mut rng_opt = None;
        let ext_ingress = generate_pattern_ingress(pattern, t, 0.0, &mut rng_opt);
        let spikes = substrate.step(&ext_ingress, plasticity_active);

        for &s in &spikes {
            total_spikes += s;
        }

        // End of Phase 1
        if t == t_train - 1 {
            snap_w_a = substrate.snapshot_weights();
            let mut probe_sub = substrate.clone();
            let (_, b_a) = measure_basin_depth(
                &mut probe_sub,
                PatternId::A,
                config.num_noisy_trials,
                config.noise_level,
                &mut rng,
            );
            let (_, b_b) = measure_basin_depth(
                &mut probe_sub,
                PatternId::B,
                config.num_noisy_trials,
                config.noise_level,
                &mut rng,
            );
            let (_, b_c) = measure_basin_depth(
                &mut probe_sub,
                PatternId::C,
                config.num_noisy_trials,
                config.noise_level,
                &mut rng,
            );
            snapshot_after_a = SnapshotBasins {
                basin_a: b_a,
                basin_b: b_b,
                basin_c: b_c,
            };
        }

        // End of Phase 2
        if t == 2 * t_train - 1 {
            let mut probe_sub = substrate.clone();
            let (_, b_a) = measure_basin_depth(
                &mut probe_sub,
                PatternId::A,
                config.num_noisy_trials,
                config.noise_level,
                &mut rng,
            );
            let (_, b_b) = measure_basin_depth(
                &mut probe_sub,
                PatternId::B,
                config.num_noisy_trials,
                config.noise_level,
                &mut rng,
            );
            let (_, b_c) = measure_basin_depth(
                &mut probe_sub,
                PatternId::C,
                config.num_noisy_trials,
                config.noise_level,
                &mut rng,
            );
            snapshot_after_b = SnapshotBasins {
                basin_a: b_a,
                basin_b: b_b,
                basin_c: b_c,
            };
        }

        // End of Phase 3
        if t == 3 * t_train - 1 {
            snap_w_c = substrate.snapshot_weights();
            let mut probe_sub = substrate.clone();
            let (m_a, b_a) = measure_basin_depth(
                &mut probe_sub,
                PatternId::A,
                config.num_noisy_trials,
                config.noise_level,
                &mut rng,
            );
            let (m_b, b_b) = measure_basin_depth(
                &mut probe_sub,
                PatternId::B,
                config.num_noisy_trials,
                config.noise_level,
                &mut rng,
            );
            let (m_c, b_c) = measure_basin_depth(
                &mut probe_sub,
                PatternId::C,
                config.num_noisy_trials,
                config.noise_level,
                &mut rng,
            );
            snapshot_after_c = SnapshotBasins {
                basin_a: b_a,
                basin_b: b_b,
                basin_c: b_c,
            };
            occ_a_final = m_a;
            occ_b_final = m_b;
            occ_c_final = m_c;
        }
    }

    let mean_firing_density = total_spikes / (total_ticks * NUM_NODES) as f64;
    let homeostasis_stable = (0.05..=0.20).contains(&mean_firing_density);

    // Compute continual learning metrics
    let b_a_init = snapshot_after_a.basin_a;
    let b_b_init = snapshot_after_b.basin_b;

    let b_a_final = snapshot_after_c.basin_a;
    let b_b_final = snapshot_after_c.basin_b;

    let bwt = 0.5 * ((b_a_final - b_a_init) + (b_b_final - b_b_init));
    let ar = 0.5 * ((b_a_final / (b_a_init + 1e-6)) + (b_b_final / (b_b_init + 1e-6)));

    let forgetting_a = (b_a_init - b_a_final).max(0.0);
    let forgetting_b = (b_b_init - b_b_final).max(0.0);
    let forgetting_mean = 0.5 * (forgetting_a + forgetting_b);

    let d_sep_ab = hamming_dist(&occ_a_final, &occ_b_final);
    let d_sep_bc = hamming_dist(&occ_b_final, &occ_c_final);
    let d_sep_ac = hamming_dist(&occ_a_final, &occ_c_final);
    let d_sep_min = d_sep_ab.min(d_sep_bc).min(d_sep_ac);

    let overlap_ac = compute_subspace_overlap(&snap_w_a, &snap_w_c, &substrate.neighbors_in);

    RunTelemetryRecord {
        run_id: config.run_id.clone(),
        seed: config.seed,
        condition: config.condition.as_str().to_string(),
        kappa: config.kappa,
        t_train,
        snapshots: Snapshots {
            after_a: snapshot_after_a,
            after_b: snapshot_after_b,
            after_c: snapshot_after_c,
        },
        metrics: RunMetricsRecord {
            bwt,
            ar,
            forgetting_a,
            forgetting_b,
            forgetting_mean,
            d_sep_min,
            d_sep_ab,
            d_sep_bc,
            d_sep_ac,
            overlap_ac,
            mean_firing_density,
        },
        homeostasis_stable,
    }
}

/// Generate the full 420-run factorial configurations
pub fn generate_factorial_configs(seeds: usize, t_train_values: &[usize]) -> Vec<ExperimentConfig> {
    let mut configs = Vec::new();

    // 1. Baseline static (60 runs: 2 t_train * 30 seeds)
    for &t_train in t_train_values {
        for s in 1..=seeds {
            configs.push(ExperimentConfig {
                run_id: format!("RUN-EXP-2026-005a-STATIC-T{}-S{:02}", t_train, s),
                seed: s as u64,
                condition: Condition::BaselineStatic,
                kappa: 0.0,
                t_train,
                ..Default::default()
            });
        }
    }

    // 2. Sequential Plastic conditions (180 runs: 3 kappas * 2 t_train * 30 seeds)
    // kappa = 0.0 is ablation_unconstrained (60 runs)
    // kappa = 1.0 is active_metaplastic (60 runs)
    // kappa = 5.0 is active_metaplastic (60 runs)
    for &kappa in &[0.0, 1.0, 5.0] {
        let cond = if kappa == 0.0 {
            Condition::AblationUnconstrained
        } else {
            Condition::ActiveMetaplastic
        };
        let tag = if kappa == 0.0 { "ABLATION" } else { "ACTIVE" };

        for &t_train in t_train_values {
            for s in 1..=seeds {
                configs.push(ExperimentConfig {
                    run_id: format!(
                        "RUN-EXP-2026-005a-{}-K{:.1}-T{}-S{:02}",
                        tag, kappa, t_train, s
                    ),
                    seed: s as u64,
                    condition: cond,
                    kappa,
                    t_train,
                    ..Default::default()
                });
            }
        }
    }

    // 3. Control joint training (180 runs: 3 kappas * 2 t_train * 30 seeds)
    for &kappa in &[0.0, 1.0, 5.0] {
        for &t_train in t_train_values {
            for s in 1..=seeds {
                configs.push(ExperimentConfig {
                    run_id: format!(
                        "RUN-EXP-2026-005a-JOINT-K{:.1}-T{}-S{:02}",
                        kappa, t_train, s
                    ),
                    seed: s as u64,
                    condition: Condition::ControlJointTraining,
                    kappa,
                    t_train,
                    ..Default::default()
                });
            }
        }
    }

    configs
}

/// Execute factorial sweep across threads and compile summary manifest
pub fn run_factorial_sweep(
    configs: Vec<ExperimentConfig>,
    num_threads: usize,
) -> (Vec<RunTelemetryRecord>, SummaryManifest) {
    let total_runs = configs.len();

    // Execute in parallel batches using scoped threads
    let chunk_size = total_runs.div_ceil(num_threads);
    let mut results: Vec<RunTelemetryRecord> = Vec::with_capacity(total_runs);

    std::thread::scope(|s| {
        let mut handles = Vec::new();
        for chunk in configs.chunks(chunk_size) {
            let chunk_vec = chunk.to_vec();
            handles.push(s.spawn(move || {
                chunk_vec
                    .iter()
                    .map(run_single_experiment)
                    .collect::<Vec<_>>()
            }));
        }

        for handle in handles {
            let mut chunk_res = handle.join().expect("Thread panicked");
            results.append(&mut chunk_res);
        }
    });

    // Aggregate statistics by condition key
    let mut condition_groups: BTreeMap<String, Vec<&RunTelemetryRecord>> = BTreeMap::new();
    for r in &results {
        let key = if r.condition == "active_metaplastic" || r.condition == "control_joint_training"
        {
            format!("{}_k{:.0}", r.condition, r.kappa)
        } else {
            r.condition.clone()
        };
        condition_groups.entry(key).or_default().push(r);
    }

    let mut conditions_summary: BTreeMap<String, ConditionSummary> = BTreeMap::new();
    for (key, group) in &condition_groups {
        let n = group.len() as f64;
        let bwt_mean = group.iter().map(|r| r.metrics.bwt).sum::<f64>() / n;
        let bwt_std = (group
            .iter()
            .map(|r| (r.metrics.bwt - bwt_mean).powi(2))
            .sum::<f64>()
            / (n - 1.0).max(1.0))
        .sqrt();

        let ar_mean = group.iter().map(|r| r.metrics.ar).sum::<f64>() / n;
        let ar_std = (group
            .iter()
            .map(|r| (r.metrics.ar - ar_mean).powi(2))
            .sum::<f64>()
            / (n - 1.0).max(1.0))
        .sqrt();

        let d_sep_mean = group.iter().map(|r| r.metrics.d_sep_min).sum::<f64>() / n;
        let forgetting_mean = group.iter().map(|r| r.metrics.forgetting_mean).sum::<f64>() / n;

        conditions_summary.insert(
            key.clone(),
            ConditionSummary {
                bwt_mean,
                bwt_std,
                ar_mean,
                ar_std,
                d_sep_mean,
                forgetting_mean,
                sample_size: group.len(),
            },
        );
    }

    // Statistical tests
    let mut statistical_tests: BTreeMap<String, StatTestSummary> = BTreeMap::new();

    let ablation_bwt: Vec<f64> = condition_groups
        .get("ablation_unconstrained")
        .map(|g| g.iter().map(|r| r.metrics.bwt).collect())
        .unwrap_or_default();

    let k1_bwt: Vec<f64> = condition_groups
        .get("active_metaplastic_k1")
        .map(|g| g.iter().map(|r| r.metrics.bwt).collect())
        .unwrap_or_default();

    let k5_bwt: Vec<f64> = condition_groups
        .get("active_metaplastic_k5")
        .map(|g| g.iter().map(|r| r.metrics.bwt).collect())
        .unwrap_or_default();

    let (t_k1, p_k1, d_k1) = welch_t_test(&k1_bwt, &ablation_bwt);
    statistical_tests.insert(
        "welch_t_test_bwt_k1_vs_ablation".to_string(),
        StatTestSummary {
            t_stat: t_k1,
            p_value: p_k1,
            cohen_d: d_k1,
            significant: p_k1 < 1e-6,
        },
    );

    let (t_k5, p_k5, d_k5) = welch_t_test(&k5_bwt, &ablation_bwt);
    statistical_tests.insert(
        "welch_t_test_bwt_k5_vs_ablation".to_string(),
        StatTestSummary {
            t_stat: t_k5,
            p_value: p_k5,
            cohen_d: d_k5,
            significant: p_k5 < 1e-6,
        },
    );

    // Gate evaluations
    let mut gate_verdicts = BTreeMap::new();

    let k5_summary = conditions_summary.get("active_metaplastic_k5");
    let k1_summary = conditions_summary.get("active_metaplastic_k1");

    let bwt_pass = (k5_summary.map(|s| s.bwt_mean >= -0.10).unwrap_or(false))
        && (k1_summary.map(|s| s.bwt_mean >= -0.10).unwrap_or(false));
    gate_verdicts.insert(
        "gate_1_bwt".to_string(),
        if bwt_pass { "PASS" } else { "FAIL" }.to_string(),
    );

    let ar_pass = (k5_summary.map(|s| s.ar_mean >= 0.85).unwrap_or(false))
        && (k1_summary.map(|s| s.ar_mean >= 0.85).unwrap_or(false));
    gate_verdicts.insert(
        "gate_2_ar".to_string(),
        if ar_pass { "PASS" } else { "FAIL" }.to_string(),
    );

    let f_pass = (k5_summary
        .map(|s| s.forgetting_mean <= 0.12)
        .unwrap_or(false))
        && (k1_summary
            .map(|s| s.forgetting_mean <= 0.12)
            .unwrap_or(false));
    gate_verdicts.insert(
        "gate_3_forgetting".to_string(),
        if f_pass { "PASS" } else { "FAIL" }.to_string(),
    );

    let sep_pass = (k5_summary.map(|s| s.d_sep_mean >= 0.05).unwrap_or(false))
        && (k1_summary.map(|s| s.d_sep_mean >= 0.05).unwrap_or(false));
    gate_verdicts.insert(
        "gate_4_separation".to_string(),
        if sep_pass { "PASS" } else { "FAIL" }.to_string(),
    );

    let stat_pass = p_k5 < 1e-6 || p_k1 < 1e-6;
    gate_verdicts.insert(
        "gate_5_p_value".to_string(),
        if stat_pass { "PASS" } else { "FAIL" }.to_string(),
    );

    let active_runs: Vec<&RunTelemetryRecord> = results
        .iter()
        .filter(|r| r.condition == "active_metaplastic")
        .collect();
    let stable_count = active_runs.iter().filter(|r| r.homeostasis_stable).count();
    let homeo_pass =
        !active_runs.is_empty() && (stable_count as f64 / active_runs.len() as f64) >= 0.99;
    gate_verdicts.insert(
        "gate_6_homeostasis".to_string(),
        if homeo_pass { "PASS" } else { "FAIL" }.to_string(),
    );

    let all_passed = bwt_pass && ar_pass && f_pass && sep_pass && stat_pass && homeo_pass;
    let overall_verdict = if all_passed { "SUPPORTED" } else { "FALSIFIED" }.to_string();

    let manifest = SummaryManifest {
        protocol_id: "EXP-2026-005a".to_string(),
        hypothesis_id: "HYP-2026-005".to_string(),
        timestamp_utc: "2026-09-07T01:00:00Z".to_string(),
        total_runs,
        conditions: conditions_summary,
        statistical_tests,
        gate_verdicts,
        overall_verdict,
    };

    (results, manifest)
}

/// Write telemetry results and summary manifest to disk
pub fn emit_telemetry<P: AsRef<Path>>(
    output_dir: P,
    results: &[RunTelemetryRecord],
    manifest: &SummaryManifest,
) -> std::io::Result<()> {
    let dir = output_dir.as_ref();
    std::fs::create_dir_all(dir)?;

    // 1. run_results.jsonl (and telemetry.ndjson)
    let file_ndjson = File::create(dir.join("run_results.jsonl"))?;
    let mut writer = BufWriter::new(file_ndjson);
    for r in results {
        serde_json::to_writer(&mut writer, r)?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;

    // Also write telemetry.ndjson copy
    let file_copy = File::create(dir.join("telemetry.ndjson"))?;
    let mut writer_copy = BufWriter::new(file_copy);
    for r in results {
        serde_json::to_writer(&mut writer_copy, r)?;
        writer_copy.write_all(b"\n")?;
    }
    writer_copy.flush()?;

    // 2. summary_evaluation.json (and summary.json)
    let file_summary = File::create(dir.join("summary_evaluation.json"))?;
    let mut summary_writer = BufWriter::new(file_summary);
    serde_json::to_writer_pretty(&mut summary_writer, manifest)?;
    summary_writer.flush()?;

    let file_summary2 = File::create(dir.join("summary.json"))?;
    let mut summary_writer2 = BufWriter::new(file_summary2);
    serde_json::to_writer_pretty(&mut summary_writer2, manifest)?;
    summary_writer2.flush()?;

    Ok(())
}
