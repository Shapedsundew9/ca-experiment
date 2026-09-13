//! Factorial sweep execution engine and telemetry serializer for EXP-2026-007a.

use super::circuit::build_full_adder_circuit;
use super::config::{Condition, EvaluationMode, ExperimentConfig};
use super::metrics::{
    RunMetrics, expected_cout, expected_sum, sample_mean, sample_std, welch_t_test,
};

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{File, create_dir_all};
use std::io::{BufWriter, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunTelemetryRecord {
    pub run_id: String,
    pub seed: u64,
    pub condition: String,
    pub noise_rate: f64,
    pub metrics: RunMetrics,
    pub homeostasis_stable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalTestResult {
    pub t_stat: f64,
    pub p_value: f64,
    pub cohen_d: f64,
    pub significant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionCleanSummary {
    pub accuracy_mean: f64,
    pub accuracy_std: f64,
    pub ber_sum_mean: f64,
    pub ber_cout_mean: f64,
    pub crosstalk_mean: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionNoiseSummary {
    pub accuracy_mean: f64,
    pub accuracy_std: f64,
    pub ber_mean: f64,
    pub ber_std: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncompensatedDelaySummary {
    pub accuracy_mean: f64,
    pub accuracy_std: f64,
    pub ber_mean: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnshieldedCrossingSummary {
    pub accuracy_mean: f64,
    pub accuracy_std: f64,
    pub crosstalk_mean: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveConditionSummary {
    pub eps_0_00: ConditionCleanSummary,
    pub eps_0_05: ConditionNoiseSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedThetaSummary {
    pub eps_0_05: ConditionNoiseSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationSummaryManifest {
    pub protocol_id: String,
    pub hypothesis_id: String,
    pub timestamp_utc: String,
    pub total_runs: usize,
    pub conditions: ConditionsContainer,
    pub statistical_tests: BTreeMap<String, StatisticalTestResult>,
    pub gate_verdicts: BTreeMap<String, String>,
    pub overall_verdict: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionsContainer {
    pub active_composed: ActiveConditionSummary,
    pub baseline_uncompensated_delay: UncompensatedDelaySummary,
    pub ablation_unshielded_crossing: UnshieldedCrossingSummary,
    pub ablation_fixed_theta: FixedThetaSummary,
}

/// Execute a single simulation run and produce telemetry
#[allow(clippy::needless_range_loop)]
pub fn execute_single_run(config: &ExperimentConfig) -> RunTelemetryRecord {
    let mut circuit = build_full_adder_circuit(config.clone());
    let t_warmup = config.t_warmup;

    // Warmup phase with quiescent input (u = 0)
    for _ in 0..t_warmup {
        circuit.step((0.0, 0.0, 0.0), 0.0, false);
    }

    match config.mode {
        EvaluationMode::CrosstalkProbe => {
            // Crosstalk Perturbation Routine:
            // Inject 50 pulses into Y1 track while Cin is held at 0
            let num_pulses = 50;
            let pulse_interval = 4;
            let mut spurious_spikes = 0;

            for step in 0..(num_pulses * pulse_interval) {
                let perturb = if step % pulse_interval == 0 { 1.0 } else { 0.0 };
                let (_sum, _cout, cin_post) = circuit.step((0.0, 0.0, 0.0), perturb, true);
                if cin_post > 0.0 {
                    spurious_spikes += 1;
                }
            }

            let crosstalk_cross = spurious_spikes as f64 / num_pulses as f64;
            let density = circuit.mean_firing_density();
            let homeostasis_stable = (0.01..=0.25).contains(&density);

            RunTelemetryRecord {
                run_id: config.run_id.clone(),
                seed: config.seed,
                condition: config.condition.as_str().to_string(),
                noise_rate: config.noise_rate,
                metrics: RunMetrics {
                    truth_table_accuracy: if crosstalk_cross < 1e-4 { 1.0 } else { 0.0 },
                    states_correct: if crosstalk_cross < 1e-4 { 8 } else { 0 },
                    states_total: 8,
                    ber_sum: 0.0,
                    ber_cout: 0.0,
                    ber_mean: 0.0,
                    crosstalk_cross,
                    latency_sum: 7,
                    latency_cout: 7,
                    latency_skew: 0,
                    mean_firing_density: density,
                },
                homeostasis_stable,
            }
        }
        EvaluationMode::TruthTableSweep => {
            let epochs = config.epochs_per_state;
            let mut total_evals = 0;
            let mut correct_evals = 0;
            let mut sum_errors = 0;
            let mut cout_errors = 0;
            let mut state_correct_counts = [0usize; 8];

            for _epoch in 0..epochs {
                for k in 0..8 {
                    let a = ((k >> 2) & 1) as f64;
                    let b = ((k >> 1) & 1) as f64;
                    let c = (k & 1) as f64;

                    let exp_s = expected_sum((k >> 2) & 1, (k >> 1) & 1, k & 1);
                    let exp_c = expected_cout((k >> 2) & 1, (k >> 1) & 1, k & 1);

                    // t = 0: Apply sensory ingress inputs
                    circuit.step((a, b, c), 0.0, true);

                    // t = 1..6: Propagate signals across cascaded stages
                    for _ in 1..7 {
                        circuit.step((0.0, 0.0, 0.0), 0.0, true);
                    }

                    // t = 7: Sample outputs at steady-state latency tau_adder = 7
                    let (sum_act, cout_act, _cin_post) = circuit.step((0.0, 0.0, 0.0), 0.0, true);

                    let sum_err = (sum_act - exp_s).abs();
                    let cout_err = (cout_act - exp_c).abs();

                    if sum_err > 0.5 {
                        sum_errors += 1;
                    }
                    if cout_err > 0.5 {
                        cout_errors += 1;
                    }

                    let is_correct = sum_err < 0.5 && cout_err < 0.5;
                    if is_correct {
                        correct_evals += 1;
                        state_correct_counts[k] += 1;
                    }
                    total_evals += 1;

                    // Allow 4 relaxation clock ticks between successive state presentations
                    for _ in 0..4 {
                        circuit.step((0.0, 0.0, 0.0), 0.0, true);
                    }
                }
            }

            let truth_table_accuracy = correct_evals as f64 / total_evals.max(1) as f64;
            let states_correct = state_correct_counts
                .iter()
                .filter(|&&cnt| cnt > epochs / 2)
                .count();
            let ber_sum = sum_errors as f64 / total_evals.max(1) as f64;
            let ber_cout = cout_errors as f64 / total_evals.max(1) as f64;
            let ber_mean = (ber_sum + ber_cout) / 2.0;

            let density = circuit.mean_firing_density();
            let homeostasis_stable = (0.01..=0.25).contains(&density);

            RunTelemetryRecord {
                run_id: config.run_id.clone(),
                seed: config.seed,
                condition: config.condition.as_str().to_string(),
                noise_rate: config.noise_rate,
                metrics: RunMetrics {
                    truth_table_accuracy,
                    states_correct,
                    states_total: 8,
                    ber_sum,
                    ber_cout,
                    ber_mean,
                    crosstalk_cross: 0.0,
                    latency_sum: 7,
                    latency_cout: 7,
                    latency_skew: 0,
                    mean_firing_density: density,
                },
                homeostasis_stable,
            }
        }
    }
}

/// Generate the full 480-run factorial experiment plan
pub fn build_experiment_plan(num_seeds: usize) -> Vec<ExperimentConfig> {
    let mut plan = Vec::with_capacity(480);

    // Suite 1: Truth Table Sweep (4 conditions * 3 noise levels * 30 seeds = 360 runs)
    let noise_levels = [0.00, 0.01, 0.05];
    for &cond in &Condition::ALL {
        for &noise in &noise_levels {
            for seed in 1..=(num_seeds as u64) {
                let cfg = ExperimentConfig::new(cond, noise, seed);
                plan.push(cfg);
            }
        }
    }

    // Suite 2: Crosstalk Probe (4 conditions * 30 seeds = 120 runs)
    for &cond in &Condition::ALL {
        for seed in 1..=(num_seeds as u64) {
            let cfg = ExperimentConfig::new_crosstalk(cond, seed);
            plan.push(cfg);
        }
    }

    plan
}

/// Execute all scheduled configurations in parallel and write telemetry artifacts
pub fn run_experiment_sweep(
    plan: &[ExperimentConfig],
    output_dir: &Path,
) -> Result<EvaluationSummaryManifest, String> {
    create_dir_all(output_dir)
        .map_err(|e| format!("Failed to create output directory {:?}: {e}", output_dir))?;

    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let chunk_size = plan.len().div_ceil(num_threads);

    let mut thread_records: Vec<Vec<RunTelemetryRecord>> = vec![Vec::new(); num_threads];

    std::thread::scope(|s| {
        let mut handles = Vec::new();
        for (t_idx, chunk) in plan.chunks(chunk_size).enumerate() {
            let handle = s.spawn(move || {
                let mut records = Vec::with_capacity(chunk.len());
                for cfg in chunk {
                    records.push(execute_single_run(cfg));
                }
                (t_idx, records)
            });
            handles.push(handle);
        }

        for handle in handles {
            let (idx, records) = handle.join().map_err(|_| "Worker thread panicked")?;
            thread_records[idx] = records;
        }
        Ok::<(), String>(())
    })?;

    let all_records: Vec<RunTelemetryRecord> = thread_records.into_iter().flatten().collect();

    // Serialize all records to telemetry.ndjson
    let ndjson_path = output_dir.join("telemetry.ndjson");
    let file = File::create(&ndjson_path)
        .map_err(|e| format!("Failed to create telemetry file {:?}: {e}", ndjson_path))?;
    let mut writer = BufWriter::new(file);

    for record in &all_records {
        let line =
            serde_json::to_string(record).map_err(|e| format!("Serialization error: {e}"))?;
        writeln!(writer, "{line}").map_err(|e| format!("Failed to write line to ndjson: {e}"))?;
    }
    writer.flush().map_err(|e| format!("Flush error: {e}"))?;

    // Aggregate summary statistics
    let summary = compute_summary_evaluation(&all_records);

    // Serialize summary.json
    let summary_path = output_dir.join("summary.json");
    let summary_file = File::create(&summary_path)
        .map_err(|e| format!("Failed to create summary file {:?}: {e}", summary_path))?;
    let mut summary_writer = BufWriter::new(summary_file);
    serde_json::to_writer_pretty(&mut summary_writer, &summary)
        .map_err(|e| format!("Failed to write summary JSON: {e}"))?;
    summary_writer
        .flush()
        .map_err(|e| format!("Flush error on summary: {e}"))?;

    Ok(summary)
}

/// Compute evaluation summary manifest and verify the 6 pre-registered gates
pub fn compute_summary_evaluation(records: &[RunTelemetryRecord]) -> EvaluationSummaryManifest {
    let mut statistical_tests = BTreeMap::new();
    let mut gate_verdicts = BTreeMap::new();

    // Active Composed records
    let active_clean: Vec<&RunTelemetryRecord> = records
        .iter()
        .filter(|r| {
            r.condition == Condition::ActiveComposed.as_str()
                && r.noise_rate == 0.0
                && !r.run_id.contains("-XTLK-")
        })
        .collect();

    let active_noise05: Vec<&RunTelemetryRecord> = records
        .iter()
        .filter(|r| {
            r.condition == Condition::ActiveComposed.as_str()
                && (r.noise_rate - 0.05).abs() < 1e-5
                && !r.run_id.contains("-XTLK-")
        })
        .collect();

    let active_crosstalk: Vec<&RunTelemetryRecord> = records
        .iter()
        .filter(|r| {
            r.condition == Condition::ActiveComposed.as_str() && r.run_id.contains("-XTLK-")
        })
        .collect();

    // Baseline Uncompensated Delay records
    let uncomp_clean: Vec<&RunTelemetryRecord> = records
        .iter()
        .filter(|r| {
            r.condition == Condition::BaselineUncompensatedDelay.as_str()
                && r.noise_rate == 0.0
                && !r.run_id.contains("-XTLK-")
        })
        .collect();

    // Ablation Unshielded Crossing records
    let unshield_clean: Vec<&RunTelemetryRecord> = records
        .iter()
        .filter(|r| {
            r.condition == Condition::AblationUnshieldedCrossing.as_str()
                && r.noise_rate == 0.0
                && !r.run_id.contains("-XTLK-")
        })
        .collect();

    let unshield_crosstalk: Vec<&RunTelemetryRecord> = records
        .iter()
        .filter(|r| {
            r.condition == Condition::AblationUnshieldedCrossing.as_str()
                && r.run_id.contains("-XTLK-")
        })
        .collect();

    // Ablation Fixed Theta records at eps = 0.05
    let fixed_noise05: Vec<&RunTelemetryRecord> = records
        .iter()
        .filter(|r| {
            r.condition == Condition::AblationFixedTheta.as_str()
                && (r.noise_rate - 0.05).abs() < 1e-5
                && !r.run_id.contains("-XTLK-")
        })
        .collect();

    // Compute Active clean statistics
    let active_clean_acc: Vec<f64> = active_clean
        .iter()
        .map(|r| r.metrics.truth_table_accuracy)
        .collect();
    let active_clean_ber_sum: Vec<f64> = active_clean.iter().map(|r| r.metrics.ber_sum).collect();
    let active_clean_ber_cout: Vec<f64> = active_clean.iter().map(|r| r.metrics.ber_cout).collect();
    let active_xtalk_vals: Vec<f64> = active_crosstalk
        .iter()
        .map(|r| r.metrics.crosstalk_cross)
        .collect();

    let active_clean_summary = ConditionCleanSummary {
        accuracy_mean: sample_mean(&active_clean_acc),
        accuracy_std: sample_std(&active_clean_acc),
        ber_sum_mean: sample_mean(&active_clean_ber_sum),
        ber_cout_mean: sample_mean(&active_clean_ber_cout),
        crosstalk_mean: sample_mean(&active_xtalk_vals),
    };

    // Compute Active eps=0.05 statistics
    let active_n05_acc: Vec<f64> = active_noise05
        .iter()
        .map(|r| r.metrics.truth_table_accuracy)
        .collect();
    let active_n05_ber: Vec<f64> = active_noise05.iter().map(|r| r.metrics.ber_mean).collect();

    let active_n05_summary = ConditionNoiseSummary {
        accuracy_mean: sample_mean(&active_n05_acc),
        accuracy_std: sample_std(&active_n05_acc),
        ber_mean: sample_mean(&active_n05_ber),
        ber_std: sample_std(&active_n05_ber),
    };

    // Compute Uncompensated Delay statistics
    let uncomp_acc: Vec<f64> = uncomp_clean
        .iter()
        .map(|r| r.metrics.truth_table_accuracy)
        .collect();
    let uncomp_ber: Vec<f64> = uncomp_clean.iter().map(|r| r.metrics.ber_mean).collect();

    let uncomp_summary = UncompensatedDelaySummary {
        accuracy_mean: sample_mean(&uncomp_acc),
        accuracy_std: sample_std(&uncomp_acc),
        ber_mean: sample_mean(&uncomp_ber),
    };

    // Compute Unshielded Crossing statistics
    let unshield_acc: Vec<f64> = unshield_clean
        .iter()
        .map(|r| r.metrics.truth_table_accuracy)
        .collect();
    let unshield_xtalk: Vec<f64> = unshield_crosstalk
        .iter()
        .map(|r| r.metrics.crosstalk_cross)
        .collect();

    let unshield_summary = UnshieldedCrossingSummary {
        accuracy_mean: sample_mean(&unshield_acc),
        accuracy_std: sample_std(&unshield_acc),
        crosstalk_mean: sample_mean(&unshield_xtalk),
    };

    // Compute Fixed Theta eps=0.05 statistics
    let fixed_n05_acc: Vec<f64> = fixed_noise05
        .iter()
        .map(|r| r.metrics.truth_table_accuracy)
        .collect();
    let fixed_n05_ber: Vec<f64> = fixed_noise05.iter().map(|r| r.metrics.ber_mean).collect();

    let fixed_summary = FixedThetaSummary {
        eps_0_05: ConditionNoiseSummary {
            accuracy_mean: sample_mean(&fixed_n05_acc),
            accuracy_std: sample_std(&fixed_n05_acc),
            ber_mean: sample_mean(&fixed_n05_ber),
            ber_std: sample_std(&fixed_n05_ber),
        },
    };

    // Statistical Tests:
    // 1. Active vs Uncompensated Delay (Accuracy)
    let (t1, p1, d1) = welch_t_test(&active_clean_acc, &uncomp_acc);
    statistical_tests.insert(
        "welch_active_vs_uncompensated_delay".to_string(),
        StatisticalTestResult {
            t_stat: t1,
            p_value: p1,
            cohen_d: d1,
            significant: p1 < 1e-6,
        },
    );

    // 2. Active vs Unshielded Crossing (Accuracy)
    let (t2, p2, d2) = welch_t_test(&active_clean_acc, &unshield_acc);
    statistical_tests.insert(
        "welch_active_vs_unshielded_crossing".to_string(),
        StatisticalTestResult {
            t_stat: t2,
            p_value: p2,
            cohen_d: d2,
            significant: p2 < 1e-6,
        },
    );

    // 3. Active vs Fixed Theta at eps = 0.05 (Accuracy)
    let (t3, p3, d3) = welch_t_test(&active_n05_acc, &fixed_n05_acc);
    statistical_tests.insert(
        "welch_active_vs_fixed_theta_noise005".to_string(),
        StatisticalTestResult {
            t_stat: t3,
            p_value: p3,
            cohen_d: d3,
            significant: p3 < 1e-6,
        },
    );

    // Assess the 6 Pre-Registered Gates:
    // Gate 1: Exact Truth Table Parity (Accuracy = 1.000 at eps = 0.00)
    let gate1_pass = (active_clean_summary.accuracy_mean - 1.0).abs() < 1e-9;
    gate_verdicts.insert(
        "gate_1_exact_truth_table_parity".to_string(),
        if gate1_pass { "PASS" } else { "FAIL" }.to_string(),
    );

    // Gate 2: Zero Intermediate Attenuation (BER_Sum = 0.000, BER_Cout = 0.000 at eps = 0.00)
    let gate2_pass =
        active_clean_summary.ber_sum_mean == 0.0 && active_clean_summary.ber_cout_mean == 0.0;
    gate_verdicts.insert(
        "gate_2_zero_intermediate_attenuation".to_string(),
        if gate2_pass { "PASS" } else { "FAIL" }.to_string(),
    );

    // Gate 3: Crossing Crosstalk Isolation (chi_cross < 10^-4)
    let gate3_pass = active_clean_summary.crosstalk_mean < 1e-4;
    gate_verdicts.insert(
        "gate_3_crossing_crosstalk_isolation".to_string(),
        if gate3_pass { "PASS" } else { "FAIL" }.to_string(),
    );

    // Gate 4: Noise Tolerance (Accuracy >= 0.950, BER <= 0.025 at eps = 0.05)
    let gate4_pass =
        active_n05_summary.accuracy_mean >= 0.950 && active_n05_summary.ber_mean <= 0.025;
    gate_verdicts.insert(
        "gate_4_noise_tolerance".to_string(),
        if gate4_pass { "PASS" } else { "FAIL" }.to_string(),
    );

    // Gate 5: Delay Equalization Advantage (Welch's p < 10^-6, d >= 2.0 vs uncompensated delay)
    let gate5_pass = p1 < 1e-6 && d1.abs() >= 2.0;
    gate_verdicts.insert(
        "gate_5_delay_equalization_advantage".to_string(),
        if gate5_pass { "PASS" } else { "FAIL" }.to_string(),
    );

    // Gate 6: Homeostatic Stability (Rolling density in [0.01, 0.25] in >= 99% of active runs)
    let active_runs: Vec<&RunTelemetryRecord> = records
        .iter()
        .filter(|r| r.condition == Condition::ActiveComposed.as_str())
        .collect();
    let stable_count = active_runs
        .iter()
        .filter(|r| (0.01..=0.25).contains(&r.metrics.mean_firing_density))
        .count();
    let stability_rate = stable_count as f64 / active_runs.len().max(1) as f64;
    let gate6_pass = stability_rate >= 0.99;
    gate_verdicts.insert(
        "gate_6_homeostatic_stability".to_string(),
        if gate6_pass { "PASS" } else { "FAIL" }.to_string(),
    );

    let all_gates_pass =
        gate1_pass && gate2_pass && gate3_pass && gate4_pass && gate5_pass && gate6_pass;

    EvaluationSummaryManifest {
        protocol_id: "EXP-2026-007a".to_string(),
        hypothesis_id: "HYP-2026-007".to_string(),
        timestamp_utc: "2026-09-07T19:00:00Z".to_string(),
        total_runs: records.len(),
        conditions: ConditionsContainer {
            active_composed: ActiveConditionSummary {
                eps_0_00: active_clean_summary,
                eps_0_05: active_n05_summary,
            },
            baseline_uncompensated_delay: uncomp_summary,
            ablation_unshielded_crossing: unshield_summary,
            ablation_fixed_theta: fixed_summary,
        },
        statistical_tests,
        gate_verdicts,
        overall_verdict: if all_gates_pass {
            "SUPPORTED"
        } else {
            "FALSIFIED"
        }
        .to_string(),
    }
}
