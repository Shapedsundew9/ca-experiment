//! Factorial sweep execution engine and telemetry serializer for EXP-2026-010a.

use super::circuit::build_circuit;
use super::config::{AutomatonTask, Condition, ExperimentConfig};
use super::metrics::{RunMetrics, sample_mean, sample_std, welch_t_test};
use super::substrate::{FastRng, IngressInputs};

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::fs::{File, create_dir_all};
use std::io::{BufWriter, Write};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    OpenRnd,  // '('
    CloseRnd, // ')'
    OpenSqr,  // '['
    CloseSqr, // ']'
}

impl Token {
    pub fn as_char(&self) -> char {
        match self {
            Token::OpenRnd => '(',
            Token::CloseRnd => ')',
            Token::OpenSqr => '[',
            Token::CloseSqr => ']',
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvaluationSequence {
    pub tokens: Vec<Token>,
    pub target_label: usize, // 1 = accept, 0 = reject
    pub violation_type: String,
    pub nesting_depth: usize,
}

impl fmt::Display for EvaluationSequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = self.tokens.iter().map(|t| t.as_char()).collect();
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceTrialRecord {
    pub seed: u64,
    pub condition: String,
    pub task: String,
    pub seq_length: usize,
    pub nesting_depth: usize,
    pub noise_rate: f64,
    pub seq_id: usize,
    pub input_string: String,
    pub target_label: usize,
    pub predicted_label: usize,
    pub correct: bool,
    pub violation_type: String,
    pub pointer_crosstalk_ticks: usize,
    pub mean_firing_density: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunTelemetryRecord {
    pub run_id: String,
    pub seed: u64,
    pub condition: String,
    pub task: String,
    pub seq_length: usize,
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
pub struct GateResult {
    pub metric: String,
    pub target: f64,
    pub observed: f64,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionCellSummary {
    pub accuracy_mean: f64,
    pub accuracy_std: f64,
    pub fidelity_reject_mean: f64,
    pub fidelity_reject_std: f64,
    pub pointer_crosstalk_mean: f64,
    pub ber_mean: f64,
    pub ber_std: f64,
    pub firing_density_mean: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationSummaryManifest {
    pub protocol_id: String,
    pub hypothesis_id: String,
    pub timestamp_utc: String,
    pub git_commit: String,
    pub total_runs: usize,
    pub total_sequences: usize,
    pub gates: BTreeMap<String, GateResult>,
    pub conditions: BTreeMap<String, ConditionCellSummary>,
    pub statistical_tests: BTreeMap<String, StatisticalTestResult>,
    pub overall_verdict: String,
}

// -----------------------------------------------------------------------------
// Dataset Generators
// -----------------------------------------------------------------------------

/// Generate balanced dataset of M sequences for Dyck-1 language verification.
pub fn generate_dyck1_dataset(
    seed: u64,
    seq_length: usize,
    m_sequences: usize,
) -> Vec<EvaluationSequence> {
    let mut rng = FastRng::seed_from_u64(seed.wrapping_add(seq_length as u64 * 1000 + 0xD1_2026));
    let mut dataset = Vec::with_capacity(m_sequences);

    let n_valid = m_sequences / 2;
    let n_invalid = m_sequences - n_valid;

    // 1. Valid Sequences (y* = 1)
    for _ in 0..n_valid {
        let mut tokens = Vec::with_capacity(seq_length);
        let mut depth = 0usize;
        let mut max_depth = 0usize;

        for i in 0..seq_length {
            let remaining = seq_length - i;
            let can_open = remaining > depth && depth < 8;
            let can_close = depth > 0;

            let open = if can_open && can_close {
                rng.bernoulli(0.50)
            } else {
                can_open
            };

            if open {
                tokens.push(Token::OpenRnd);
                depth += 1;
                max_depth = max_depth.max(depth);
            } else {
                tokens.push(Token::CloseRnd);
                depth = depth.saturating_sub(1);
            }
        }

        dataset.push(EvaluationSequence {
            tokens,
            target_label: 1,
            violation_type: "none".to_string(),
            nesting_depth: max_depth,
        });
    }

    // 2. Invalid Sequences (y* = 0)
    for i in 0..n_invalid {
        let v_type = match i % 3 {
            0 => "underflow",
            1 => "unclosed",
            _ => "mismatch", // excess close
        };

        let mut tokens = Vec::with_capacity(seq_length);
        let mut depth = 0usize;
        let mut max_depth = 0usize;

        match v_type {
            "underflow" => {
                // Starts with ')' or premature underflow
                tokens.push(Token::CloseRnd);
                for _ in 1..seq_length {
                    tokens.push(if rng.bernoulli(0.50) {
                        Token::OpenRnd
                    } else {
                        Token::CloseRnd
                    });
                }
                max_depth = 1;
            }
            "unclosed" => {
                // Excess open brackets: more '(' than ')'
                for i in 0..seq_length {
                    let tok = if i < (seq_length / 2 + 2) || rng.bernoulli(0.50) {
                        Token::OpenRnd
                    } else {
                        Token::CloseRnd
                    };
                    if tok == Token::OpenRnd {
                        depth += 1;
                        max_depth = max_depth.max(depth);
                    }
                    tokens.push(tok);
                }
            }
            _ => {
                // Excess closing brackets: starts balanced then closed too many times
                tokens.push(Token::OpenRnd);
                tokens.push(Token::CloseRnd);
                tokens.push(Token::CloseRnd); // underflow / excess close
                for _ in 3..seq_length {
                    tokens.push(if rng.bernoulli(0.50) {
                        Token::OpenRnd
                    } else {
                        Token::CloseRnd
                    });
                }
                max_depth = 1;
            }
        }

        dataset.push(EvaluationSequence {
            tokens,
            target_label: 0,
            violation_type: v_type.to_string(),
            nesting_depth: max_depth,
        });
    }

    dataset
}

/// Generate balanced dataset of M sequences for Dyck-2 language verification.
pub fn generate_dyck2_dataset(
    seed: u64,
    seq_length: usize,
    m_sequences: usize,
) -> Vec<EvaluationSequence> {
    let mut rng = FastRng::seed_from_u64(seed.wrapping_add(seq_length as u64 * 2000 + 0xD2_2026));
    let mut dataset = Vec::with_capacity(m_sequences);

    let n_valid = m_sequences / 2;
    let n_invalid = m_sequences - n_valid;

    // 1. Valid Sequences (y* = 1)
    for _ in 0..n_valid {
        let mut tokens = Vec::with_capacity(seq_length);
        let mut stack = Vec::with_capacity(seq_length);
        let mut max_depth = 0usize;

        for i in 0..seq_length {
            let remaining = seq_length - i;
            let can_open = remaining > stack.len() && stack.len() < 8;
            let can_close = !stack.is_empty();

            let open = if can_open && can_close {
                rng.bernoulli(0.50)
            } else {
                can_open
            };

            if open {
                let is_sqr = rng.bernoulli(0.50);
                if is_sqr {
                    tokens.push(Token::OpenSqr);
                    stack.push(Token::CloseSqr);
                } else {
                    tokens.push(Token::OpenRnd);
                    stack.push(Token::CloseRnd);
                }
                max_depth = max_depth.max(stack.len());
            } else {
                let closing = stack.pop().unwrap_or(Token::CloseRnd);
                tokens.push(closing);
            }
        }

        dataset.push(EvaluationSequence {
            tokens,
            target_label: 1,
            violation_type: "none".to_string(),
            nesting_depth: max_depth,
        });
    }

    // 2. Invalid Sequences (y* = 0)
    for i in 0..n_invalid {
        let v_type = match i % 4 {
            0 => "cross_bracket",
            1 => "mismatch",
            2 => "underflow",
            _ => "unclosed",
        };

        let mut tokens = Vec::with_capacity(seq_length);

        let max_depth = match v_type {
            "cross_bracket" => {
                // Non-LIFO nesting: e.g. ([)]
                tokens.push(Token::OpenRnd);
                tokens.push(Token::OpenSqr);
                tokens.push(Token::CloseRnd); // cross-bracket mismatch! Top of stack is sqr, token is close rnd
                tokens.push(Token::CloseSqr);
                for _ in 4..seq_length {
                    tokens.push(if rng.bernoulli(0.50) {
                        Token::OpenRnd
                    } else {
                        Token::CloseRnd
                    });
                }
                2
            }
            "mismatch" => {
                // Mismatched closure: [)
                tokens.push(Token::OpenSqr);
                tokens.push(Token::CloseRnd); // mismatch!
                for _ in 2..seq_length {
                    tokens.push(if rng.bernoulli(0.50) {
                        Token::OpenRnd
                    } else {
                        Token::CloseRnd
                    });
                }
                1
            }
            "underflow" => {
                // Prefix underflow: starts with ']'
                tokens.push(Token::CloseSqr);
                for _ in 1..seq_length {
                    tokens.push(if rng.bernoulli(0.50) {
                        Token::OpenSqr
                    } else {
                        Token::CloseSqr
                    });
                }
                1
            }
            _ => {
                // Unclosed open bracket at end
                tokens.push(Token::OpenRnd);
                tokens.push(Token::OpenSqr);
                tokens.push(Token::CloseSqr);
                // leaves OpenRnd unclosed
                for _ in 3..seq_length {
                    tokens.push(Token::OpenRnd);
                }
                2
            }
        };

        dataset.push(EvaluationSequence {
            tokens,
            target_label: 0,
            violation_type: v_type.to_string(),
            nesting_depth: max_depth,
        });
    }

    dataset
}

/// Generate depth generalization dataset parameterized by maximum nesting depth D in [1, 8].
pub fn generate_depth_dataset(
    seed: u64,
    depth: usize,
    m_sequences: usize,
) -> Vec<EvaluationSequence> {
    let mut rng = FastRng::seed_from_u64(seed.wrapping_add(depth as u64 * 3000 + 0xDE_2026));
    let mut dataset = Vec::with_capacity(m_sequences);

    let n_valid = m_sequences / 2;
    let n_invalid = m_sequences - n_valid;

    // 1. Valid Sequences of exact depth D
    for _ in 0..n_valid {
        let mut tokens = Vec::with_capacity(depth * 2);
        let mut stack = Vec::with_capacity(depth);

        // Push D brackets
        for _ in 0..depth {
            if rng.bernoulli(0.50) {
                tokens.push(Token::OpenRnd);
                stack.push(Token::CloseRnd);
            } else {
                tokens.push(Token::OpenSqr);
                stack.push(Token::CloseSqr);
            }
        }
        // Pop D brackets in LIFO order
        while let Some(tok) = stack.pop() {
            tokens.push(tok);
        }

        dataset.push(EvaluationSequence {
            tokens,
            target_label: 1,
            violation_type: "none".to_string(),
            nesting_depth: depth,
        });
    }

    // 2. Invalid Sequences of depth D
    for i in 0..n_invalid {
        let v_type = if i % 2 == 0 { "mismatch" } else { "unclosed" };
        let mut tokens = Vec::with_capacity(depth * 2);
        let mut stack = Vec::with_capacity(depth);

        for _ in 0..depth {
            if rng.bernoulli(0.50) {
                tokens.push(Token::OpenRnd);
                stack.push(Token::CloseRnd);
            } else {
                tokens.push(Token::OpenSqr);
                stack.push(Token::CloseSqr);
            }
        }

        if v_type == "mismatch" {
            // Flip the first closing token to create a mismatch
            if let Some(tok) = stack.pop() {
                let corrupt = match tok {
                    Token::CloseRnd => Token::CloseSqr,
                    Token::CloseSqr => Token::CloseRnd,
                    _ => Token::CloseRnd,
                };
                tokens.push(corrupt);
            }
            while let Some(tok) = stack.pop() {
                tokens.push(tok);
            }
        } else {
            // Unclosed: leave last bracket unclosed by replacing close with open
            while stack.len() > 1 {
                tokens.push(stack.pop().unwrap());
            }
            tokens.push(Token::OpenRnd); // unclosed!
        }

        dataset.push(EvaluationSequence {
            tokens,
            target_label: 0,
            violation_type: v_type.to_string(),
            nesting_depth: depth,
        });
    }

    dataset
}

// -----------------------------------------------------------------------------
// Trial & Run Execution
// -----------------------------------------------------------------------------

/// Evaluate a single sequence through the simulated excitable substrate.
pub fn evaluate_sequence_trial(
    config: &ExperimentConfig,
    seq: &EvaluationSequence,
    seq_id: usize,
) -> SequenceTrialRecord {
    let t_token = config.t_token; // 16 ticks
    let t_warmup = config.t_warmup; // 16 ticks

    let mut circuit = build_circuit(config.clone());

    // Initial state seeding: seed solitary circulating pulse into Pointer P0
    circuit.force_spike(circuit.pointer_rings[0][0]);

    // Warmup phase: let Pointer P0 circulate to establish clean limit cycle
    for _ in 0..t_warmup {
        circuit.step(IngressInputs::default(), false);
    }

    let mut pointer_crosstalk_ticks = 0;

    // Stream tokens sequentially through time
    for &tok in &seq.tokens {
        // Tick 0 of token aperture: inject sensory pulse into designated ingress port
        let mut ingress = IngressInputs::default();
        match tok {
            Token::OpenRnd => ingress.open_rnd = 1.0,
            Token::CloseRnd => ingress.close_rnd = 1.0,
            Token::OpenSqr => ingress.open_sqr = 1.0,
            Token::CloseSqr => ingress.close_sqr = 1.0,
        }
        circuit.step(ingress, true);

        // Ticks 1..t_token: relaxation and circulation
        for tick in 1..t_token {
            circuit.step(IngressInputs::default(), true);

            // In settled phase (ticks 4..15): monitor pointer ladder mutual exclusivity
            if tick >= 4 {
                let active_ptr = circuit.active_pointer_count();
                if active_ptr != 1 {
                    pointer_crosstalk_ticks += 1;
                }
            }
        }
    }

    // End-of-Stream (EOS) Resolution: Deliver probe pulse v_eos at t_eos
    circuit.step(
        IngressInputs {
            eos: 1.0,
            ..Default::default()
        },
        true,
    );

    // Readout Phase: Probe v_out over 6 clock ticks
    let mut accept_spikes = 0;
    for _ in 0..6 {
        let spike = circuit.step(IngressInputs::default(), true);
        if spike > 0.0 {
            accept_spikes += 1;
        }
    }

    let predicted_label = if accept_spikes > 0 { 1 } else { 0 };
    let correct = predicted_label == seq.target_label;
    let mean_firing_density = circuit.evaluation_firing_density();

    SequenceTrialRecord {
        seed: config.seed,
        condition: config.condition.as_str().to_string(),
        task: config.task.as_str().to_string(),
        seq_length: seq.tokens.len(),
        nesting_depth: seq.nesting_depth,
        noise_rate: config.noise_rate,
        seq_id,
        input_string: seq.to_string(),
        target_label: seq.target_label,
        predicted_label,
        correct,
        violation_type: seq.violation_type.clone(),
        pointer_crosstalk_ticks,
        mean_firing_density,
    }
}

/// Execute a single factorial cell run across a dataset of sequences.
pub fn execute_run(
    config: &ExperimentConfig,
    dataset: &[EvaluationSequence],
) -> (RunTelemetryRecord, Vec<SequenceTrialRecord>) {
    let mut trials = Vec::with_capacity(dataset.len());
    let mut sequences_correct = 0;
    let mut invalid_total = 0;
    let mut invalid_rejected = 0;
    let mut underflow_total = 0;
    let mut underflow_rejected = 0;
    let mut mismatch_total = 0;
    let mut mismatch_rejected = 0;
    let mut unclosed_total = 0;
    let mut unclosed_rejected = 0;
    let mut crosstalk_ticks = 0;
    let mut settled_ticks = 0;
    let mut firing_densities = Vec::with_capacity(dataset.len());

    for (seq_id, seq) in dataset.iter().enumerate() {
        let trial = evaluate_sequence_trial(config, seq, seq_id + 1);
        if trial.correct {
            sequences_correct += 1;
        }

        if seq.target_label == 0 {
            invalid_total += 1;
            if trial.predicted_label == 0 {
                invalid_rejected += 1;
            }

            match seq.violation_type.as_str() {
                "underflow" => {
                    underflow_total += 1;
                    if trial.predicted_label == 0 {
                        underflow_rejected += 1;
                    }
                }
                "mismatch" | "cross_bracket" => {
                    mismatch_total += 1;
                    if trial.predicted_label == 0 {
                        mismatch_rejected += 1;
                    }
                }
                "unclosed" => {
                    unclosed_total += 1;
                    if trial.predicted_label == 0 {
                        unclosed_rejected += 1;
                    }
                }
                _ => {}
            }
        }

        crosstalk_ticks += trial.pointer_crosstalk_ticks;
        settled_ticks += seq.tokens.len() * (config.t_token.saturating_sub(4));
        firing_densities.push(trial.mean_firing_density);
        trials.push(trial);
    }

    let sequences_total = dataset.len();
    let accuracy_seq = if sequences_total > 0 {
        sequences_correct as f64 / sequences_total as f64
    } else {
        0.0
    };

    let fidelity_reject = if invalid_total > 0 {
        invalid_rejected as f64 / invalid_total as f64
    } else {
        1.0
    };

    let fidelity_underflow = if underflow_total > 0 {
        underflow_rejected as f64 / underflow_total as f64
    } else {
        1.0
    };

    let fidelity_mismatch = if mismatch_total > 0 {
        mismatch_rejected as f64 / mismatch_total as f64
    } else {
        1.0
    };

    let fidelity_unclosed = if unclosed_total > 0 {
        unclosed_rejected as f64 / unclosed_total as f64
    } else {
        1.0
    };

    let pointer_crosstalk_rate = if settled_ticks > 0 {
        crosstalk_ticks as f64 / settled_ticks as f64
    } else {
        0.0
    };

    let ber = 1.0 - accuracy_seq;
    let mean_firing_density = sample_mean(&firing_densities);
    let homeostasis_stable = (0.01..=0.25).contains(&mean_firing_density);

    let metrics = RunMetrics {
        accuracy_seq,
        fidelity_reject,
        fidelity_underflow,
        fidelity_mismatch,
        fidelity_unclosed,
        pointer_crosstalk_rate,
        ber,
        mean_firing_density,
        sequences_total,
        sequences_correct,
        invalid_total,
        invalid_rejected,
        underflow_total,
        underflow_rejected,
        mismatch_total,
        mismatch_rejected,
        unclosed_total,
        unclosed_rejected,
        crosstalk_ticks,
        settled_ticks,
    };

    let record = RunTelemetryRecord {
        run_id: config.run_id.clone(),
        seed: config.seed,
        condition: config.condition.as_str().to_string(),
        task: config.task.as_str().to_string(),
        seq_length: config.seq_length,
        noise_rate: config.noise_rate,
        metrics,
        homeostasis_stable,
    };

    (record, trials)
}

/// Execute complete factorial sweep and serialize telemetry files.
#[allow(clippy::too_many_arguments)]
pub fn execute_factorial_sweep(
    output_dir: &Path,
    conditions: &[Condition],
    tasks: &[AutomatonTask],
    noise_rates: &[f64],
    seq_lengths: &[usize],
    num_seeds: usize,
    m_sequences: usize,
) -> EvaluationSummaryManifest {
    create_dir_all(output_dir).expect("Failed to create output directory");

    let trials_path = output_dir.join("trials.jsonl");
    let summary_path = output_dir.join("summary.json");

    let trials_file = File::create(&trials_path).expect("Failed to create trials.jsonl");
    let mut trials_writer = BufWriter::new(trials_file);

    let mut all_run_records = Vec::new();
    let mut cell_records: BTreeMap<String, Vec<RunMetrics>> = BTreeMap::new();
    let mut active_d_ge_3_acc = Vec::new();
    let mut fsm_d_ge_3_acc = Vec::new();
    let mut active_noise_05_acc = Vec::new();
    let mut fixed_noise_05_acc = Vec::new();
    let mut depth_gen_accuracies = Vec::new();

    println!(
        "Initiating EXP-2026-010a Factorial Sweep: {} conditions, {} tasks, {} noise rates, {} lengths, {} seeds",
        conditions.len(),
        tasks.len(),
        noise_rates.len(),
        seq_lengths.len(),
        num_seeds
    );

    let total_cells = conditions.len() * tasks.len() * noise_rates.len() * seq_lengths.len();
    let mut completed_cells = 0;

    for &cond in conditions {
        for &task in tasks {
            for &noise in noise_rates {
                for &seq_len in seq_lengths {
                    if task == AutomatonTask::DepthGeneralization
                        && seq_lengths.len() > 1
                        && seq_len != 16
                    {
                        continue;
                    }
                    completed_cells += 1;
                    print!(
                        "[{completed_cells}/{total_cells}] Sweep Cell: cond={}, task={}, noise={:.2}, len={} ... ",
                        cond.as_str(),
                        task.as_str(),
                        noise,
                        seq_len
                    );

                    let cell_key =
                        format!("{}:{}:{}:{seq_len}", cond.as_str(), task.as_str(), noise);

                    for seed in 1..=num_seeds as u64 {
                        let mut cfg = ExperimentConfig::new(cond, task, noise, seed);
                        cfg.seq_length = seq_len;

                        let dataset = match task {
                            AutomatonTask::Dyck1 => {
                                generate_dyck1_dataset(seed, seq_len, m_sequences)
                            }
                            AutomatonTask::Dyck2 => {
                                generate_dyck2_dataset(seed, seq_len, m_sequences)
                            }
                            AutomatonTask::DepthGeneralization => {
                                // Depth generalization evaluates parametric depths 1..8
                                let mut combined = Vec::new();
                                for d in 1..=8 {
                                    combined.extend(generate_depth_dataset(seed, d, 20));
                                }
                                combined
                            }
                        };

                        let (run_rec, trials) = execute_run(&cfg, &dataset);

                        for trial in &trials {
                            let line = serde_json::to_string(trial)
                                .expect("Failed to serialize trial record");
                            writeln!(trials_writer, "{line}")
                                .expect("Failed to write to trials.jsonl");
                        }

                        // Collect Gate 5 comparisons (active_pda vs baseline_finite_state on D >= 3)
                        if noise == 0.00 {
                            if cond == Condition::ActivePda {
                                let d_ge_3_correct = trials
                                    .iter()
                                    .filter(|t| t.nesting_depth >= 3)
                                    .filter(|t| t.correct)
                                    .count();
                                let d_ge_3_total =
                                    trials.iter().filter(|t| t.nesting_depth >= 3).count();
                                if d_ge_3_total > 0 {
                                    active_d_ge_3_acc
                                        .push(d_ge_3_correct as f64 / d_ge_3_total as f64);
                                }
                            } else if cond == Condition::BaselineFiniteState {
                                let d_ge_3_correct = trials
                                    .iter()
                                    .filter(|t| t.nesting_depth >= 3)
                                    .filter(|t| t.correct)
                                    .count();
                                let d_ge_3_total =
                                    trials.iter().filter(|t| t.nesting_depth >= 3).count();
                                if d_ge_3_total > 0 {
                                    fsm_d_ge_3_acc
                                        .push(d_ge_3_correct as f64 / d_ge_3_total as f64);
                                }
                            }
                        }

                        // Collect Gate 4 comparisons (active_pda vs ablation_fixed_theta at noise 0.05)
                        if noise == 0.05 {
                            if cond == Condition::ActivePda {
                                active_noise_05_acc.push(run_rec.metrics.accuracy_seq);
                            } else if cond == Condition::AblationFixedTheta {
                                fixed_noise_05_acc.push(run_rec.metrics.accuracy_seq);
                            }
                        }

                        // Collect Gate 2 depth generalization accuracies (Active PDA on depth_generalization)
                        if cond == Condition::ActivePda
                            && task == AutomatonTask::DepthGeneralization
                            && noise == 0.00
                        {
                            depth_gen_accuracies.push(run_rec.metrics.accuracy_seq);
                        }

                        cell_records
                            .entry(cell_key.clone())
                            .or_default()
                            .push(run_rec.metrics.clone());
                        all_run_records.push(run_rec);
                    }
                    println!("done");
                }
            }
        }
    }

    trials_writer.flush().expect("Failed to flush trials.jsonl");

    // Aggregate condition summaries
    let mut conditions_summary = BTreeMap::new();
    for (key, metrics_list) in &cell_records {
        let acc_vals: Vec<f64> = metrics_list.iter().map(|m| m.accuracy_seq).collect();
        let fid_vals: Vec<f64> = metrics_list.iter().map(|m| m.fidelity_reject).collect();
        let cross_vals: Vec<f64> = metrics_list
            .iter()
            .map(|m| m.pointer_crosstalk_rate)
            .collect();
        let ber_vals: Vec<f64> = metrics_list.iter().map(|m| m.ber).collect();
        let rho_vals: Vec<f64> = metrics_list.iter().map(|m| m.mean_firing_density).collect();

        let acc_mean = sample_mean(&acc_vals);
        let acc_std = sample_std(&acc_vals);
        let fid_mean = sample_mean(&fid_vals);
        let fid_std = sample_std(&fid_vals);
        let ber_mean = sample_mean(&ber_vals);
        let ber_std = sample_std(&ber_vals);

        conditions_summary.insert(
            key.clone(),
            ConditionCellSummary {
                accuracy_mean: acc_mean,
                accuracy_std: acc_std,
                fidelity_reject_mean: fid_mean,
                fidelity_reject_std: fid_std,
                pointer_crosstalk_mean: sample_mean(&cross_vals),
                ber_mean,
                ber_std,
                firing_density_mean: sample_mean(&rho_vals),
            },
        );
    }

    // Evaluate Gates
    // Gate 1: Sequence classification accuracy for Active PDA at eps = 0.00
    let gate1_accs: Vec<f64> = all_run_records
        .iter()
        .filter(|r| {
            r.condition == "active_pda" && r.noise_rate == 0.00 && r.task != "depth_generalization"
        })
        .map(|r| r.metrics.accuracy_seq)
        .collect();
    let gate1_observed = sample_mean(&gate1_accs);
    let gate1_passed = gate1_observed >= 1.000;

    // Gate 2: Depth Generalization accuracy D in [1, 8]
    let gate2_observed = sample_mean(&depth_gen_accuracies);
    let gate2_passed = gate2_observed >= 0.950;

    // Gate 3: Error Rejection Fidelity at eps = 0.00
    let gate3_fids: Vec<f64> = all_run_records
        .iter()
        .filter(|r| r.condition == "active_pda" && r.noise_rate == 0.00)
        .map(|r| r.metrics.fidelity_reject)
        .collect();
    let gate3_observed = sample_mean(&gate3_fids);
    let gate3_passed = gate3_observed >= 1.000;

    // Gate 4: Noise resilience at critical noise eps = 0.05
    let gate4_observed = sample_mean(&active_noise_05_acc);
    let gate4_passed = gate4_observed >= 0.900;

    // Gate 5: Pushdown advantage over finite-state baseline on D >= 3
    let (t_stat_g5, p_val_g5, cohen_d_g5) = welch_t_test(&active_d_ge_3_acc, &fsm_d_ge_3_acc);
    let gate5_passed = p_val_g5 < 1e-6 && cohen_d_g5 >= 2.0;

    // Gate 6: Homeostatic density stability on Active PDA runs
    let active_runs: Vec<&RunTelemetryRecord> = all_run_records
        .iter()
        .filter(|r| r.condition == "active_pda")
        .collect();
    let homeostatic_runs = active_runs.iter().filter(|r| r.homeostasis_stable).count();
    let gate6_observed = if !active_runs.is_empty() {
        homeostatic_runs as f64 / active_runs.len() as f64
    } else {
        0.0
    };
    let gate6_passed = gate6_observed >= 0.990;

    let mut gates = BTreeMap::new();
    gates.insert(
        "gate_1_dyck_sequence_accuracy".to_string(),
        GateResult {
            metric: "sequence_classification_accuracy".to_string(),
            target: 1.00,
            observed: gate1_observed,
            passed: gate1_passed,
        },
    );
    gates.insert(
        "gate_2_depth_generalization".to_string(),
        GateResult {
            metric: "depth_generalization_accuracy_d1_to_d8".to_string(),
            target: 0.95,
            observed: gate2_observed,
            passed: gate2_passed,
        },
    );
    gates.insert(
        "gate_3_error_rejection_fidelity".to_string(),
        GateResult {
            metric: "violation_rejection_fidelity".to_string(),
            target: 1.00,
            observed: gate3_observed,
            passed: gate3_passed,
        },
    );
    gates.insert(
        "gate_4_noise_resilience".to_string(),
        GateResult {
            metric: "accuracy_at_critical_noise_0_05".to_string(),
            target: 0.90,
            observed: gate4_observed,
            passed: gate4_passed,
        },
    );
    gates.insert(
        "gate_5_pushdown_advantage".to_string(),
        GateResult {
            metric: "welch_t_p_value_and_cohens_d".to_string(),
            target: 2.00,
            observed: cohen_d_g5,
            passed: gate5_passed,
        },
    );
    gates.insert(
        "gate_6_homeostatic_stability".to_string(),
        GateResult {
            metric: "fraction_runs_within_firing_bounds".to_string(),
            target: 0.99,
            observed: gate6_observed,
            passed: gate6_passed,
        },
    );

    let (t_stat_noise, p_val_noise, cohen_d_noise) =
        welch_t_test(&active_noise_05_acc, &fixed_noise_05_acc);

    let mut statistical_tests = BTreeMap::new();
    statistical_tests.insert(
        "pushdown_vs_finite_state_d_ge_3".to_string(),
        StatisticalTestResult {
            t_stat: t_stat_g5,
            p_value: p_val_g5,
            cohen_d: cohen_d_g5,
            significant: gate5_passed,
        },
    );
    statistical_tests.insert(
        "dynamic_vs_fixed_theta_noise_0_05".to_string(),
        StatisticalTestResult {
            t_stat: t_stat_noise,
            p_value: p_val_noise,
            cohen_d: cohen_d_noise,
            significant: p_val_noise < 1e-4,
        },
    );

    let overall_verdict = if gate1_passed
        && gate2_passed
        && gate3_passed
        && gate4_passed
        && gate5_passed
        && gate6_passed
    {
        "PASS"
    } else {
        "FAIL"
    };

    let manifest = EvaluationSummaryManifest {
        protocol_id: "EXP-2026-010a".to_string(),
        hypothesis_id: "HYP-2026-010".to_string(),
        timestamp_utc: "2026-09-07T22:00:00Z".to_string(),
        git_commit: "HEAD".to_string(),
        total_runs: all_run_records.len(),
        total_sequences: all_run_records
            .iter()
            .map(|r| r.metrics.sequences_total)
            .sum(),
        gates,
        conditions: conditions_summary,
        statistical_tests,
        overall_verdict: overall_verdict.to_string(),
    };

    let summary_file = File::create(&summary_path).expect("Failed to create summary.json");
    serde_json::to_writer_pretty(summary_file, &manifest).expect("Failed to write summary.json");

    manifest
}
