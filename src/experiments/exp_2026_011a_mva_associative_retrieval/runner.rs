//! Factorial sweep execution engine and telemetry serializer for EXP-2026-011a.

use super::circuit::build_circuit;
use super::config::{Condition, ExperimentConfig};
use super::metrics::{RunMetrics, sample_mean, welch_t_test};
use super::substrate::{FastRng, IngressInputs};

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{File, create_dir_all};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    KeyValue { key: usize, val: usize },
    Distractor,
}

#[derive(Debug, Clone)]
pub struct EvaluationSequence {
    pub tokens: Vec<Token>,
    pub queried_key: usize,
    pub target_val: usize,
    pub seq_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceTrialRecord {
    pub seed: u64,
    pub condition: String,
    pub seq_length: usize,
    pub noise_rate: f64,
    pub seq_id: usize,
    pub queried_key: usize,
    pub target_value: usize,
    pub predicted_value: usize,
    pub correct: bool,
    pub distractor_leakage_events: usize,
    pub mutex_violation_ticks: usize,
    pub mean_firing_density: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunTelemetryRecord {
    pub run_id: String,
    pub seed: u64,
    pub condition: String,
    pub seq_length: usize,
    pub noise_rate: f64,
    pub metrics: RunMetrics,
    pub homeostasis_stable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseSweepEntry {
    pub accuracy: f64,
    pub ber: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionSummary {
    pub overall_accuracy: f64,
    pub horizon_breakdown: BTreeMap<String, f64>,
    pub distractor_leakage_rate: f64,
    pub noise_sweep: BTreeMap<String, NoiseSweepEntry>,
    pub mutex_violation_rate: f64,
    pub mean_firing_density: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub welch_t_stat_vs_unindexed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub welch_p_val_vs_unindexed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cohens_d_vs_unindexed: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate1Evaluation {
    pub passed: bool,
    pub value: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate2Evaluation {
    pub passed: bool,
    pub value: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate3Evaluation {
    pub passed: bool,
    pub value: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate4Evaluation {
    pub passed: bool,
    pub accuracy_at_005: f64,
    pub ber_at_005: f64,
    pub threshold_accuracy: f64,
    pub threshold_ber: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate5Evaluation {
    pub passed: bool,
    pub p_value: f64,
    pub cohens_d: f64,
    pub threshold_p: f64,
    pub threshold_d: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate6Evaluation {
    pub passed: bool,
    pub within_envelope_pct: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateEvaluations {
    pub gate_1_retrieval_accuracy: Gate1Evaluation,
    pub gate_2_long_horizon_retention: Gate2Evaluation,
    pub gate_3_distractor_immunity: Gate3Evaluation,
    pub gate_4_noise_resilience: Gate4Evaluation,
    pub gate_5_associative_advantage: Gate5Evaluation,
    pub gate_6_homeostatic_stability: Gate6Evaluation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationSummaryManifest {
    pub experiment_id: String,
    pub timestamp: String,
    pub conditions: BTreeMap<String, ConditionSummary>,
    pub gate_evaluations: GateEvaluations,
    pub overall_verdict: String,
}

/// Generate a synthetic dataset of `m_sequences` streaming sequences for a given length `L`.
pub fn generate_dataset(
    seq_length: usize,
    m_sequences: usize,
    rng: &mut FastRng,
) -> Vec<EvaluationSequence> {
    assert!(
        seq_length >= 16,
        "Sequence length L must be >= 16 to accommodate 16 key-value pairs"
    );

    let mut dataset = Vec::with_capacity(m_sequences);

    for _ in 0..m_sequences {
        // Sample values for 16 keys i.i.d. Bernoulli(0.5)
        let mut values = [0usize; 16];
        for v in &mut values {
            *v = if rng.bernoulli(0.5) { 1 } else { 0 };
        }

        // Permute key presentation order uniformly at random
        let mut key_order: Vec<usize> = (0..16).collect();
        for i in (1..16).rev() {
            let j = (rng.next_u64() as usize) % (i + 1);
            key_order.swap(i, j);
        }

        // Create 16 key-value pairs
        let kv_tokens: Vec<Token> = key_order
            .iter()
            .map(|&k| Token::KeyValue {
                key: k,
                val: values[k],
            })
            .collect();

        // Distribute 16 key-value pairs across L positions
        let mut positions: Vec<usize> = (0..seq_length).collect();
        for i in (1..seq_length).rev() {
            let j = (rng.next_u64() as usize) % (i + 1);
            positions.swap(i, j);
        }
        let mut needle_positions = positions[0..16].to_vec();
        needle_positions.sort_unstable();

        let mut tokens = vec![Token::Distractor; seq_length];
        for (idx, &pos) in needle_positions.iter().enumerate() {
            tokens[pos] = kv_tokens[idx];
        }

        // Sample target query uniformly from 0..15
        let queried_key = (rng.next_u64() as usize) % 16;
        let target_val = values[queried_key];

        dataset.push(EvaluationSequence {
            tokens,
            queried_key,
            target_val,
            seq_length,
        });
    }

    dataset
}

/// Execute a single evaluation sequence trial on the substrate.
pub fn evaluate_sequence_trial(
    config: &ExperimentConfig,
    seq: &EvaluationSequence,
    seq_id: usize,
) -> SequenceTrialRecord {
    let mut trial_cfg = config.clone();
    trial_cfg.seed = config.seed.wrapping_add((seq_id as u64) * 0x1000);
    let mut substrate = build_circuit(trial_cfg);

    let mut distractor_leakage_events = 0usize;
    let mut mutex_violation_ticks = 0usize;
    let initial_spikes = substrate.total_spikes;
    let initial_steps = substrate.total_steps;

    // 1. Present sequence tokens (L tokens, each T_token = 16 ticks)
    for token in &seq.tokens {
        let is_distractor = matches!(token, Token::Distractor);

        for t in 0..16 {
            let mut inputs = IngressInputs::default();
            if t == 0 {
                match token {
                    Token::KeyValue { key, val } => {
                        let actual_key =
                            if substrate.config.condition == Condition::BaselineUnindexed {
                                0
                            } else {
                                *key
                            };
                        inputs.keys[actual_key] = 1.0;
                        inputs.vals[*val] = 1.0;
                    }
                    Token::Distractor => {
                        inputs.distractor = 1.0;
                    }
                }
            }

            substrate.step(inputs, false);

            if is_distractor {
                let mem_spikes = substrate.count_memory_spikes();
                if mem_spikes > 16 {
                    distractor_leakage_events += 1;
                }
            }

            // Check settled ticks (t in [4, 15]) for mutual exclusivity violations
            if t >= 4 {
                let mutex_count = substrate.check_mutex_violations();
                if mutex_count > 0 {
                    mutex_violation_ticks += 1;
                }
            }
        }
    }

    // 2. Present Query Token Q(K_j) and monitor dual-rail readout ports
    let mut out0_spikes = 0usize;
    let mut out1_spikes = 0usize;

    let actual_query_key = if substrate.config.condition == Condition::BaselineUnindexed {
        0
    } else {
        seq.queried_key
    };

    for t in 0..16 {
        let mut q_inputs = IngressInputs::default();
        // Query pulse active for duration of ring period P_ring = 4 ticks
        if t < 4 {
            q_inputs.queries[actual_query_key] = 1.0;
        }

        let (s0, s1) = substrate.step(q_inputs, true);
        if s0 > 0.0 {
            out0_spikes += 1;
        }
        if s1 > 0.0 {
            out1_spikes += 1;
        }
    }

    // 3. Dual-rail output decoding
    let predicted_value = if out1_spikes > out0_spikes {
        1
    } else if out0_spikes > out1_spikes {
        0
    } else {
        // True tie (collision or omission): fair coin flip
        if substrate.rng.bernoulli(0.5) { 1 } else { 0 }
    };

    let correct = predicted_value == seq.target_val;

    let delta_spikes = substrate.total_spikes - initial_spikes;
    let delta_steps = substrate.total_steps - initial_steps;
    let mean_firing_density = if delta_steps > 0 {
        delta_spikes as f64 / (substrate.nodes.len() as f64 * delta_steps as f64)
    } else {
        0.0
    };

    SequenceTrialRecord {
        seed: substrate.config.seed,
        condition: substrate.config.condition.as_str().to_string(),
        seq_length: seq.seq_length,
        noise_rate: substrate.config.noise_rate,
        seq_id,
        queried_key: seq.queried_key,
        target_value: seq.target_val,
        predicted_value,
        correct,
        distractor_leakage_events,
        mutex_violation_ticks,
        mean_firing_density,
    }
}

/// Execute a complete run of M sequences for one condition/noise/length/seed cell.
pub fn execute_run(
    config: &ExperimentConfig,
    dataset: &[EvaluationSequence],
) -> (RunTelemetryRecord, Vec<SequenceTrialRecord>) {
    let mut trials = Vec::with_capacity(dataset.len());

    let mut correct_count = 0usize;
    let mut total_leakage = 0usize;
    let mut total_mutex_violations = 0usize;
    let mut total_distractor_ticks = 0usize;
    let mut total_settled_ticks = 0usize;
    let mut density_sum = 0.0;

    for (seq_id, seq) in dataset.iter().enumerate() {
        let trial = evaluate_sequence_trial(config, seq, seq_id + 1);
        if trial.correct {
            correct_count += 1;
        }
        total_leakage += trial.distractor_leakage_events;
        total_mutex_violations += trial.mutex_violation_ticks;

        let num_distractors = seq.seq_length.saturating_sub(16);
        total_distractor_ticks += num_distractors * 16;
        total_settled_ticks += seq.seq_length * 12;
        density_sum += trial.mean_firing_density;

        trials.push(trial);
    }

    let n = dataset.len();
    let accuracy = if n > 0 {
        correct_count as f64 / n as f64
    } else {
        0.0
    };
    let ber = 1.0 - accuracy;
    let leakage_rate = if total_distractor_ticks > 0 {
        total_leakage as f64 / total_distractor_ticks as f64
    } else {
        0.0
    };
    let mutex_rate = if total_settled_ticks > 0 {
        total_mutex_violations as f64 / total_settled_ticks as f64
    } else {
        0.0
    };
    let mean_density = if n > 0 { density_sum / n as f64 } else { 0.0 };

    let homeostasis_stable = (0.01..=0.25).contains(&mean_density);

    let metrics = RunMetrics {
        accuracy_retrieval: accuracy,
        ber,
        distractor_leakage_rate: leakage_rate,
        mutex_violation_rate: mutex_rate,
        mean_firing_density: mean_density,
        total_sequences: n,
        correct_sequences: correct_count,
        distractor_leakage_events: total_leakage,
        distractor_ticks: total_distractor_ticks,
        mutex_violations: total_mutex_violations,
        settled_ticks: total_settled_ticks,
        total_spikes: (density_sum * 272.0 * (config.seq_length * 16 + 16) as f64).round() as usize,
        total_steps: n * (config.seq_length * 16 + 16),
    };

    let record = RunTelemetryRecord {
        run_id: config.run_id.clone(),
        seed: config.seed,
        condition: config.condition.as_str().to_string(),
        seq_length: config.seq_length,
        noise_rate: config.noise_rate,
        metrics,
        homeostasis_stable,
    };

    (record, trials)
}

/// Execute full factorial sweep across conditions, noise levels, sequence lengths, and seeds.
pub fn execute_factorial_sweep(
    conditions: &[Condition],
    noise_rates: &[f64],
    seq_lengths: &[usize],
    seeds: &[u64],
    sequences_per_run: usize,
    output_dir: &Path,
) -> EvaluationSummaryManifest {
    create_dir_all(output_dir).expect("Failed to create output telemetry directory");

    let num_cpus = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    println!(
        "Starting EXP-2026-011a factorial sweep: {} conditions, {} noise rates, {} lengths, {} seeds across {} threads",
        conditions.len(),
        noise_rates.len(),
        seq_lengths.len(),
        seeds.len(),
        num_cpus
    );

    // Build task combinations
    struct SweepTask {
        cond: Condition,
        noise: f64,
        len: usize,
        seed: u64,
    }

    let mut tasks = Vec::new();
    for &cond in conditions {
        for &noise in noise_rates {
            for &len in seq_lengths {
                for &seed in seeds {
                    tasks.push(SweepTask {
                        cond,
                        noise,
                        len,
                        seed,
                    });
                }
            }
        }
    }

    let chunk_size = tasks.len().div_ceil(num_cpus);
    let chunks: Vec<Vec<SweepTask>> = tasks
        .chunks(chunk_size)
        .map(|chunk| {
            chunk
                .iter()
                .map(|t| SweepTask {
                    cond: t.cond,
                    noise: t.noise,
                    len: t.len,
                    seed: t.seed,
                })
                .collect()
        })
        .collect();

    let results = thread::scope(|s| {
        let mut handles = Vec::new();
        for chunk in chunks {
            handles.push(s.spawn(move || {
                let mut run_records = Vec::new();
                let mut trial_records = Vec::new();

                for task in chunk {
                    let cfg = ExperimentConfig::new(task.cond, task.noise, task.len, task.seed);
                    let mut rng = FastRng::seed_from_u64(task.seed.wrapping_add(0x011A_DA7A));
                    let dataset = generate_dataset(task.len, sequences_per_run, &mut rng);
                    let (rec, trials) = execute_run(&cfg, &dataset);
                    run_records.push(rec);
                    trial_records.extend(trials);
                }

                (run_records, trial_records)
            }));
        }

        let mut all_runs = Vec::new();
        let mut all_trials = Vec::new();
        for h in handles {
            let (runs, trials) = h.join().expect("Worker thread panicked");
            all_runs.extend(runs);
            all_trials.extend(trials);
        }
        (all_runs, all_trials)
    });

    let (all_runs, all_trials) = results;

    println!(
        "Completed {} total runs and {} trial sequences.",
        all_runs.len(),
        all_trials.len()
    );

    // Write trials.csv
    let trials_csv_path = output_dir.join("trials.csv");
    let trials_file = File::create(&trials_csv_path).expect("Failed to create trials.csv");
    let mut writer = BufWriter::new(trials_file);
    writeln!(
        writer,
        "seed,condition,seq_length,noise_rate,seq_id,queried_key,target_value,predicted_value,correct,distractor_leakage_events,mutex_violation_ticks,mean_firing_density"
    )
    .expect("Failed to write CSV header");

    for t in &all_trials {
        writeln!(
            writer,
            "{},{},{},{:.2},{},{},{},{},{},{},{},{:.6}",
            t.seed,
            t.condition,
            t.seq_length,
            t.noise_rate,
            t.seq_id,
            t.queried_key,
            t.target_value,
            t.predicted_value,
            if t.correct { 1 } else { 0 },
            t.distractor_leakage_events,
            t.mutex_violation_ticks,
            t.mean_firing_density
        )
        .expect("Failed to write CSV row");
    }
    writer.flush().expect("Failed to flush trials.csv");

    // Aggregate condition statistics
    let mut cond_summaries = BTreeMap::new();

    for &cond in conditions {
        let cond_runs: Vec<&RunTelemetryRecord> = all_runs
            .iter()
            .filter(|r| r.condition == cond.as_str())
            .collect();

        if cond_runs.is_empty() {
            continue;
        }

        let overall_acc = sample_mean(
            &cond_runs
                .iter()
                .map(|r| r.metrics.accuracy_retrieval)
                .collect::<Vec<_>>(),
        );

        let mut horizon_breakdown = BTreeMap::new();
        for &len in seq_lengths {
            let len_runs: Vec<f64> = cond_runs
                .iter()
                .filter(|r| r.seq_length == len && (r.noise_rate - 0.0).abs() < 1e-4)
                .map(|r| r.metrics.accuracy_retrieval)
                .collect();
            horizon_breakdown.insert(len.to_string(), sample_mean(&len_runs));
        }

        let mut noise_sweep = BTreeMap::new();
        for &noise in noise_rates {
            let n_runs: Vec<&RunTelemetryRecord> = cond_runs
                .iter()
                .filter(|r| (r.noise_rate - noise).abs() < 1e-4)
                .copied()
                .collect();
            let acc = sample_mean(
                &n_runs
                    .iter()
                    .map(|r| r.metrics.accuracy_retrieval)
                    .collect::<Vec<_>>(),
            );
            let ber = sample_mean(&n_runs.iter().map(|r| r.metrics.ber).collect::<Vec<_>>());
            noise_sweep.insert(
                format!("{:.2}", noise),
                NoiseSweepEntry { accuracy: acc, ber },
            );
        }

        let distractor_leakage = sample_mean(
            &cond_runs
                .iter()
                .map(|r| r.metrics.distractor_leakage_rate)
                .collect::<Vec<_>>(),
        );

        let mutex_rate = sample_mean(
            &cond_runs
                .iter()
                .map(|r| r.metrics.mutex_violation_rate)
                .collect::<Vec<_>>(),
        );

        let mean_density = sample_mean(
            &cond_runs
                .iter()
                .map(|r| r.metrics.mean_firing_density)
                .collect::<Vec<_>>(),
        );

        cond_summaries.insert(
            cond.as_str().to_string(),
            ConditionSummary {
                overall_accuracy: overall_acc,
                horizon_breakdown,
                distractor_leakage_rate: distractor_leakage,
                noise_sweep,
                mutex_violation_rate: mutex_rate,
                mean_firing_density: mean_density,
                welch_t_stat_vs_unindexed: None,
                welch_p_val_vs_unindexed: None,
                cohens_d_vs_unindexed: None,
            },
        );
    }

    // Welch's t-test comparing active_associative vs baseline_unindexed under clean baseline (noise = 0.00)
    let active_baseline_accs: Vec<f64> = all_runs
        .iter()
        .filter(|r| r.condition == "active_associative" && (r.noise_rate - 0.0).abs() < 1e-4)
        .map(|r| r.metrics.accuracy_retrieval)
        .collect();

    let unindexed_baseline_accs: Vec<f64> = all_runs
        .iter()
        .filter(|r| r.condition == "baseline_unindexed" && (r.noise_rate - 0.0).abs() < 1e-4)
        .map(|r| r.metrics.accuracy_retrieval)
        .collect();

    let (t_stat, p_val, d_val) = welch_t_test(&active_baseline_accs, &unindexed_baseline_accs);

    if let Some(active_summary) = cond_summaries.get_mut("active_associative") {
        active_summary.welch_t_stat_vs_unindexed = Some(t_stat);
        active_summary.welch_p_val_vs_unindexed = Some(p_val);
        active_summary.welch_d_vs_unindexed(d_val);
    }

    // Evaluate 6 Gates
    let gate1_acc = cond_summaries
        .get("active_associative")
        .and_then(|s| s.noise_sweep.get("0.00"))
        .map(|e| e.accuracy)
        .unwrap_or(0.0);
    let gate1_passed = gate1_acc >= 0.990;

    let gate2_long_retention = cond_summaries
        .get("active_associative")
        .map(|s| {
            let acc256 = s.horizon_breakdown.get("256").copied().unwrap_or(0.0);
            let acc512 = s.horizon_breakdown.get("512").copied().unwrap_or(0.0);
            (acc256 + acc512) / 2.0
        })
        .unwrap_or(0.0);
    let gate2_passed = gate2_long_retention >= 0.990;

    let gate3_leakage = cond_summaries
        .get("active_associative")
        .map(|s| s.distractor_leakage_rate)
        .unwrap_or(1.0);
    let gate3_passed = gate3_leakage <= 0.010;

    let gate4_acc005 = cond_summaries
        .get("active_associative")
        .and_then(|s| s.noise_sweep.get("0.05"))
        .map(|e| e.accuracy)
        .unwrap_or(0.0);
    let gate4_ber005 = cond_summaries
        .get("active_associative")
        .and_then(|s| s.noise_sweep.get("0.05"))
        .map(|e| e.ber)
        .unwrap_or(1.0);
    let gate4_passed = gate4_acc005 >= 0.900 && gate4_ber005 <= 0.050;

    let gate5_passed = p_val < 1e-6 && d_val >= 2.0;

    let active_runs: Vec<&RunTelemetryRecord> = all_runs
        .iter()
        .filter(|r| r.condition == "active_associative")
        .collect();
    let total_active_runs = active_runs.len();
    let stable_active_runs = active_runs.iter().filter(|r| r.homeostasis_stable).count();
    let within_envelope_pct = if total_active_runs > 0 {
        stable_active_runs as f64 / total_active_runs as f64
    } else {
        0.0
    };
    let gate6_passed = within_envelope_pct >= 0.990;

    let all_passed = gate1_passed
        && gate2_passed
        && gate3_passed
        && gate4_passed
        && gate5_passed
        && gate6_passed;

    let manifest = EvaluationSummaryManifest {
        experiment_id: "EXP-2026-011a".to_string(),
        timestamp: "2026-09-07T22:00:00Z".to_string(),
        conditions: cond_summaries,
        gate_evaluations: GateEvaluations {
            gate_1_retrieval_accuracy: Gate1Evaluation {
                passed: gate1_passed,
                value: gate1_acc,
                threshold: 0.990,
            },
            gate_2_long_horizon_retention: Gate2Evaluation {
                passed: gate2_passed,
                value: gate2_long_retention,
                threshold: 0.990,
            },
            gate_3_distractor_immunity: Gate3Evaluation {
                passed: gate3_passed,
                value: gate3_leakage,
                threshold: 0.010,
            },
            gate_4_noise_resilience: Gate4Evaluation {
                passed: gate4_passed,
                accuracy_at_005: gate4_acc005,
                ber_at_005: gate4_ber005,
                threshold_accuracy: 0.900,
                threshold_ber: 0.050,
            },
            gate_5_associative_advantage: Gate5Evaluation {
                passed: gate5_passed,
                p_value: p_val,
                cohens_d: d_val,
                threshold_p: 1e-6,
                threshold_d: 2.0,
            },
            gate_6_homeostatic_stability: Gate6Evaluation {
                passed: gate6_passed,
                within_envelope_pct,
                threshold: 0.990,
            },
        },
        overall_verdict: if all_passed {
            "SUPPORTED".to_string()
        } else {
            "REFUTED".to_string()
        },
    };

    // Serialize summary.json
    let summary_json_path = output_dir.join("summary.json");
    let summary_file = File::create(&summary_json_path).expect("Failed to create summary.json");
    serde_json::to_writer_pretty(summary_file, &manifest).expect("Failed to write summary.json");

    manifest
}

impl ConditionSummary {
    fn welch_d_vs_unindexed(&mut self, d: f64) {
        self.cohens_d_vs_unindexed = Some(d);
    }
}
