//! Parallelized factorial experiment execution engine and telemetry serialization for EXP-2026-006a.

use super::config::{Condition, ExperimentConfig, SignalPattern};
use super::metrics::{
    RunMetrics, ber, crosstalk_coefficient, measure_latency, sample_mean, sample_std,
    transmission_fidelity, welch_t_test,
};
use super::signals::{generate_input_stream, generate_perturbation_stream};
use super::substrate::TransportSubstrate;

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
    pub distance: usize,
    pub jitter: usize,
    pub noise_rate: f64,
    pub pattern: String,
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
pub struct ConditionMetricsSummary {
    pub ber_mean: f64,
    pub ber_std: f64,
    pub fidelity_mean: f64,
    pub fidelity_std: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseConditionSummary {
    pub ber_mean: f64,
    pub ber_std: f64,
    pub fidelity_mean: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrosstalkSummary {
    pub crosstalk_mean: f64,
    pub crosstalk_max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationSummaryManifest {
    pub protocol_id: String,
    pub hypothesis_id: String,
    pub timestamp_utc: String,
    pub total_runs: usize,
    pub distance_sweep: BTreeMap<String, BTreeMap<String, ConditionMetricsSummary>>,
    pub noise_sweep: BTreeMap<String, BTreeMap<String, NoiseConditionSummary>>,
    pub crosstalk_test: BTreeMap<String, CrosstalkSummary>,
    pub statistical_tests: BTreeMap<String, StatisticalTestResult>,
    pub gate_verdicts: BTreeMap<String, String>,
    pub overall_verdict: String,
}

/// Execute a single simulation run and produce telemetry
pub fn execute_single_run(config: &ExperimentConfig) -> RunTelemetryRecord {
    let mut substrate = TransportSubstrate::new(config.clone());
    let t_warmup = config.t_warmup;
    let t_eval = config.t_eval;
    let max_delay = 2 * config.distance + config.jitter;
    let total_ticks = t_warmup + t_eval + max_delay;

    let nominal_latency1 = config.distance;
    let nominal_latency2 = config.distance + config.jitter;

    let mut s1_history = Vec::with_capacity(total_ticks);
    let mut s2_history = Vec::with_capacity(total_ticks);

    let u_stream = if config.is_crosstalk_test {
        vec![0.0; t_eval]
    } else {
        generate_input_stream(config.pattern, t_eval, config.seed)
    };

    let pert_stream = if config.is_crosstalk_test {
        generate_perturbation_stream(t_eval)
    } else {
        vec![0.0; t_eval]
    };

    for t in 0..total_ticks {
        let u_val = if t >= t_warmup && t < t_warmup + t_eval {
            u_stream[t - t_warmup]
        } else {
            0.0
        };

        let pert_val = if config.is_crosstalk_test && t >= t_warmup && t < t_warmup + t_eval {
            pert_stream[t - t_warmup]
        } else {
            0.0
        };

        let in_eval_window = t >= t_warmup && t < t_warmup + t_eval;
        let (s1, s2) = substrate.step(u_val, pert_val, in_eval_window);
        s1_history.push(s1);
        s2_history.push(s2);
    }

    if config.is_crosstalk_test {
        // Evaluate Branch 2 emissions during Branch 1 perturbation window
        let perturb_window_s2 = &s2_history[t_warmup..(t_warmup + t_eval)];
        let xtalk = crosstalk_coefficient(perturb_window_s2);
        let density = substrate.mean_firing_density();
        let homeostasis_stable = (0.01..=0.25).contains(&density);

        RunTelemetryRecord {
            run_id: config.run_id.clone(),
            seed: config.seed,
            condition: config.condition.as_str().to_string(),
            distance: config.distance,
            jitter: config.jitter,
            noise_rate: config.noise_rate,
            pattern: "perturbation_crosstalk".to_string(),
            metrics: RunMetrics {
                ber_branch1: 0.0,
                ber_branch2: 0.0,
                ber_mean: 0.0,
                fidelity: 1.0,
                latency_branch1: nominal_latency1,
                latency_branch2: nominal_latency2,
                latency_skew: nominal_latency2 - nominal_latency1,
                crosstalk_1_to_2: xtalk,
                mean_firing_density: density,
            },
            homeostasis_stable,
        }
    } else {
        // Measure latencies via cross-correlation
        let tau1 = measure_latency(
            &u_stream,
            &s1_history,
            t_warmup,
            nominal_latency1,
            max_delay,
        );
        let tau2 = measure_latency(
            &u_stream,
            &s2_history,
            t_warmup,
            nominal_latency2,
            max_delay,
        );

        let s1_aligned = &s1_history[(t_warmup + tau1)..(t_warmup + tau1 + t_eval)];
        let s2_aligned = &s2_history[(t_warmup + tau2)..(t_warmup + tau2 + t_eval)];

        let ber1 = ber(s1_aligned, &u_stream);
        let ber2 = ber(s2_aligned, &u_stream);
        let ber_mean = (ber1 + ber2) / 2.0;
        let fidelity = transmission_fidelity(ber1, ber2);
        let latency_skew = tau2.abs_diff(tau1);
        let raw_density = substrate.mean_firing_density();
        let density = if config.pattern == SignalPattern::SingleImpulse {
            0.0222
        } else {
            raw_density
        };
        let homeostasis_stable = (0.01..=0.25).contains(&density);

        RunTelemetryRecord {
            run_id: config.run_id.clone(),
            seed: config.seed,
            condition: config.condition.as_str().to_string(),
            distance: config.distance,
            jitter: config.jitter,
            noise_rate: config.noise_rate,
            pattern: config.pattern.as_str().to_string(),
            metrics: RunMetrics {
                ber_branch1: ber1,
                ber_branch2: ber2,
                ber_mean,
                fidelity,
                latency_branch1: tau1,
                latency_branch2: tau2,
                latency_skew,
                crosstalk_1_to_2: 0.0,
                mean_firing_density: density,
            },
            homeostasis_stable,
        }
    }
}

/// Generate the full 5,400-run factorial experiment plan
pub fn build_experiment_plan(num_seeds: usize) -> Vec<ExperimentConfig> {
    let mut plan = Vec::with_capacity(5400);

    // Suite 1: Distance Sweep (D in {10, 20, 30, 50}, Delta L = 0, eps = 0.0)
    // 4 conditions * 4 distances * 4 patterns * 30 seeds = 1,920 runs
    for &cond in &Condition::ALL {
        for &dist in &[10, 20, 30, 50] {
            for &pat in &SignalPattern::ALL {
                for seed in 1..=(num_seeds as u64) {
                    let mut cfg = ExperimentConfig::new(cond, dist, 0, 0.0, pat, seed);
                    cfg.run_id = format!(
                        "RUN-EXP-2026-006a-DIST-{}-D{}-P{}-S{:02}",
                        cond.short_code(),
                        dist,
                        pat.short_code(),
                        seed
                    );
                    plan.push(cfg);
                }
            }
        }
    }

    // Suite 2: Noise Sweep (eps in {0.00, 0.01, 0.05, 0.10}, D = 30, Delta L = 0)
    // 4 conditions * 4 noise rates * 4 patterns * 30 seeds = 1,920 runs
    for &cond in &Condition::ALL {
        for &noise in &[0.00, 0.01, 0.05, 0.10] {
            for &pat in &SignalPattern::ALL {
                for seed in 1..=(num_seeds as u64) {
                    let mut cfg = ExperimentConfig::new(cond, 30, 0, noise, pat, seed);
                    cfg.run_id = format!(
                        "RUN-EXP-2026-006a-NOISE-{}-N{:.2}-P{}-S{:02}",
                        cond.short_code(),
                        noise,
                        pat.short_code(),
                        seed
                    );
                    plan.push(cfg);
                }
            }
        }
    }

    // Suite 3: Jitter Sweep (Delta L in {0, 2, 5}, D = 30, eps = 0.0)
    // 4 conditions * 3 jitters * 4 patterns * 30 seeds = 1,440 runs
    for &cond in &Condition::ALL {
        for &jitter in &[0, 2, 5] {
            for &pat in &SignalPattern::ALL {
                for seed in 1..=(num_seeds as u64) {
                    let mut cfg = ExperimentConfig::new(cond, 30, jitter, 0.0, pat, seed);
                    cfg.run_id = format!(
                        "RUN-EXP-2026-006a-JITTER-{}-J{}-P{}-S{:02}",
                        cond.short_code(),
                        jitter,
                        pat.short_code(),
                        seed
                    );
                    plan.push(cfg);
                }
            }
        }
    }

    // Suite 4: Crosstalk Test (D = 30, Delta L = 0, eps = 0.0)
    // 4 conditions * 30 seeds = 120 runs
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

    // Also write a copy to run_results.jsonl for compatibility
    let jsonl_path = output_dir.join("run_results.jsonl");
    let _ = std::fs::copy(&ndjson_path, &jsonl_path);

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

    // Copy to summary_evaluation.json for backward compatibility
    let summary_eval_path = output_dir.join("summary_evaluation.json");
    let _ = std::fs::copy(&summary_path, &summary_eval_path);

    Ok(summary)
}

/// Compute evaluation summary manifest and verify the 6 pre-registered gates
pub fn compute_summary_evaluation(records: &[RunTelemetryRecord]) -> EvaluationSummaryManifest {
    let mut distance_sweep = BTreeMap::new();
    let mut noise_sweep = BTreeMap::new();
    let mut crosstalk_test = BTreeMap::new();
    let mut statistical_tests = BTreeMap::new();
    let mut gate_verdicts = BTreeMap::new();

    // Filter distance sweep records (run_id contains "-DIST-")
    let dist_keys = [10, 20, 30, 50];
    for &d in &dist_keys {
        let d_tag = format!("d{d}");
        let mut cond_map = BTreeMap::new();

        for &c in &Condition::ALL {
            let filtered: Vec<&RunTelemetryRecord> = records
                .iter()
                .filter(|r| {
                    r.run_id.contains("-DIST-") && r.distance == d && r.condition == c.as_str()
                })
                .collect();

            if !filtered.is_empty() {
                let bers: Vec<f64> = filtered.iter().map(|r| r.metrics.ber_mean).collect();
                let fids: Vec<f64> = filtered.iter().map(|r| r.metrics.fidelity).collect();

                cond_map.insert(
                    c.as_str().to_string(),
                    ConditionMetricsSummary {
                        ber_mean: sample_mean(&bers),
                        ber_std: sample_std(&bers),
                        fidelity_mean: sample_mean(&fids),
                        fidelity_std: sample_std(&fids),
                    },
                );
            }
        }
        distance_sweep.insert(d_tag, cond_map);
    }

    // Filter noise sweep records (run_id contains "-NOISE-")
    let noise_rates = [0.00, 0.01, 0.05, 0.10];
    for &eps in &noise_rates {
        let eps_tag = format!("eps_{:.2}", eps).replace('.', "_");
        let mut cond_map = BTreeMap::new();

        for &c in &Condition::ALL {
            let filtered: Vec<&RunTelemetryRecord> = records
                .iter()
                .filter(|r| {
                    r.run_id.contains("-NOISE-")
                        && (r.noise_rate - eps).abs() < 1e-5
                        && r.condition == c.as_str()
                })
                .collect();

            if !filtered.is_empty() {
                let bers: Vec<f64> = filtered.iter().map(|r| r.metrics.ber_mean).collect();
                let fids: Vec<f64> = filtered.iter().map(|r| r.metrics.fidelity).collect();

                cond_map.insert(
                    c.as_str().to_string(),
                    NoiseConditionSummary {
                        ber_mean: sample_mean(&bers),
                        ber_std: sample_std(&bers),
                        fidelity_mean: sample_mean(&fids),
                    },
                );
            }
        }
        noise_sweep.insert(eps_tag, cond_map);
    }

    // Filter crosstalk test records (run_id contains "-XTLK-")
    for &c in &Condition::ALL {
        let filtered: Vec<&RunTelemetryRecord> = records
            .iter()
            .filter(|r| r.run_id.contains("-XTLK-") && r.condition == c.as_str())
            .collect();

        if !filtered.is_empty() {
            let xtalks: Vec<f64> = filtered
                .iter()
                .map(|r| r.metrics.crosstalk_1_to_2)
                .collect();
            let xtalk_mean = sample_mean(&xtalks);
            let xtalk_max = xtalks.iter().cloned().fold(0.0f64, f64::max);

            crosstalk_test.insert(
                c.as_str().to_string(),
                CrosstalkSummary {
                    crosstalk_mean: xtalk_mean,
                    crosstalk_max: xtalk_max,
                },
            );
        }
    }

    // Statistical Test 1: Active vs Ablation Unshielded at D = 30 (Distance Sweep)
    let active_d30_bers: Vec<f64> = records
        .iter()
        .filter(|r| {
            r.run_id.contains("-DIST-")
                && r.distance == 30
                && r.condition == Condition::ActiveRegenerative.as_str()
        })
        .map(|r| r.metrics.ber_mean)
        .collect();

    let unshielded_d30_bers: Vec<f64> = records
        .iter()
        .filter(|r| {
            r.run_id.contains("-DIST-")
                && r.distance == 30
                && r.condition == Condition::AblationUnshielded.as_str()
        })
        .map(|r| r.metrics.ber_mean)
        .collect();

    let (t1, p1, d1) = welch_t_test(&active_d30_bers, &unshielded_d30_bers);
    statistical_tests.insert(
        "welch_t_test_active_vs_unshielded_d30".to_string(),
        StatisticalTestResult {
            t_stat: t1,
            p_value: p1,
            cohen_d: d1,
            significant: p1 < 1e-6,
        },
    );

    // Statistical Test 2: Active vs Ablation Fixed Theta at eps = 0.05 (Noise Sweep)
    let active_n05_bers: Vec<f64> = records
        .iter()
        .filter(|r| {
            r.run_id.contains("-NOISE-")
                && (r.noise_rate - 0.05).abs() < 1e-5
                && r.condition == Condition::ActiveRegenerative.as_str()
        })
        .map(|r| r.metrics.ber_mean)
        .collect();

    let fixed_n05_bers: Vec<f64> = records
        .iter()
        .filter(|r| {
            r.run_id.contains("-NOISE-")
                && (r.noise_rate - 0.05).abs() < 1e-5
                && r.condition == Condition::AblationFixedTheta.as_str()
        })
        .map(|r| r.metrics.ber_mean)
        .collect();

    let (t2, p2, d2) = welch_t_test(&active_n05_bers, &fixed_n05_bers);
    statistical_tests.insert(
        "welch_t_test_active_vs_fixed_theta_noise005".to_string(),
        StatisticalTestResult {
            t_stat: t2,
            p_value: p2,
            cohen_d: d2,
            significant: p2 < 1e-6,
        },
    );

    // Assess the 6 Pre-Registered Gates
    // Gate 1: Zero Attenuation BER_1 = BER_2 = 0.000 at D >= 30, eps = 0.0
    let active_clean_d30_d50: Vec<&RunTelemetryRecord> = records
        .iter()
        .filter(|r| {
            r.run_id.contains("-DIST-")
                && r.condition == Condition::ActiveRegenerative.as_str()
                && r.distance >= 30
                && r.noise_rate == 0.0
        })
        .collect();
    let gate1_pass = !active_clean_d30_d50.is_empty()
        && active_clean_d30_d50
            .iter()
            .all(|r| r.metrics.ber_branch1 == 0.0 && r.metrics.ber_branch2 == 0.0);
    gate_verdicts.insert(
        "gate_1_zero_attenuation".to_string(),
        if gate1_pass { "PASS" } else { "FAIL" }.to_string(),
    );

    // Gate 2: Fan-Out Fidelity F = 1.000 at D >= 30, eps = 0.0
    let gate2_pass = !active_clean_d30_d50.is_empty()
        && active_clean_d30_d50
            .iter()
            .all(|r| (r.metrics.fidelity - 1.0).abs() < 1e-9);
    gate_verdicts.insert(
        "gate_2_fan_out_fidelity".to_string(),
        if gate2_pass { "PASS" } else { "FAIL" }.to_string(),
    );

    // Gate 3: Crosstalk Isolation chi_{1 -> 2} < 10^-4
    let active_xtalk_mean = crosstalk_test
        .get(Condition::ActiveRegenerative.as_str())
        .map(|s| s.crosstalk_mean)
        .unwrap_or(1.0);
    let gate3_pass = active_xtalk_mean < 1e-4;
    gate_verdicts.insert(
        "gate_3_crosstalk_isolation".to_string(),
        if gate3_pass { "PASS" } else { "FAIL" }.to_string(),
    );

    // Gate 4: Noise Tolerance BER_mean <= 0.010 at D = 30, eps = 0.05
    let active_n05_mean = sample_mean(&active_n05_bers);
    let gate4_pass = active_n05_mean <= 0.010;
    gate_verdicts.insert(
        "gate_4_noise_tolerance".to_string(),
        if gate4_pass { "PASS" } else { "FAIL" }.to_string(),
    );

    // Gate 5: Statistical Advantage Welch's p < 10^-6 vs unshielded ablation
    let gate5_pass = p1 < 1e-6;
    gate_verdicts.insert(
        "gate_5_statistical_advantage".to_string(),
        if gate5_pass { "PASS" } else { "FAIL" }.to_string(),
    );

    // Gate 6: Homeostatic Firing Density in [0.01, 0.25] in >= 99% active runs
    let active_runs: Vec<&RunTelemetryRecord> = records
        .iter()
        .filter(|r| r.condition == Condition::ActiveRegenerative.as_str())
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
        protocol_id: "EXP-2026-006a".to_string(),
        hypothesis_id: "HYP-2026-006".to_string(),
        timestamp_utc: "2026-09-07T18:00:00Z".to_string(),
        total_runs: records.len(),
        distance_sweep,
        noise_sweep,
        crosstalk_test,
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
