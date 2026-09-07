//! Factorial sweep execution engine and telemetry serializer for EXP-2026-009a.

use super::circuit::build_circuit;
use super::config::{AutomatonTask, Condition, ExperimentConfig};
use super::metrics::{RunMetrics, sample_mean, sample_std, welch_t_test};
use super::substrate::FastRng;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{File, create_dir_all};
use std::io::{BufWriter, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceTrialRecord {
    pub seed: u64,
    pub condition: String,
    pub task: String,
    pub seq_length: usize,
    pub noise_rate: f64,
    pub seq_id: usize,
    pub input_tokens: String,
    pub target_label: usize,
    pub predicted_label: usize,
    pub correct: bool,
    pub transitions_correct: f64,
    pub crosstalk_ticks: usize,
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
    pub fidelity_mean: f64,
    pub fidelity_std: f64,
    pub crosstalk_mean: f64,
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

/// A synthetic evaluation sequence with tokens and target classification label.
#[derive(Debug, Clone)]
pub struct EvaluationSequence {
    pub tokens: Vec<usize>,
    pub target_label: usize,
    pub expected_states: Vec<usize>, // state after each token
}

/// Generate balanced dataset of M sequences for Parity DFA
pub fn generate_parity_dataset(
    seed: u64,
    seq_length: usize,
    m_sequences: usize,
) -> Vec<EvaluationSequence> {
    let mut rng = FastRng::seed_from_u64(seed.wrapping_add(seq_length as u64 * 1000 + 0xBA_2026));
    let mut dataset = Vec::with_capacity(m_sequences);

    for _ in 0..m_sequences {
        let mut tokens = Vec::with_capacity(seq_length);
        let mut states = Vec::with_capacity(seq_length);
        let mut curr_state = 0; // q_even = 0, q_odd = 1

        for _ in 0..seq_length {
            let tok = if rng.bernoulli(0.50) { 1 } else { 0 };
            tokens.push(tok);
            if tok == 1 {
                curr_state = 1 - curr_state;
            }
            states.push(curr_state);
        }

        dataset.push(EvaluationSequence {
            tokens,
            target_label: curr_state,
            expected_states: states,
        });
    }

    dataset
}

/// Generate balanced dataset of M sequences for Regex (10)+1 DFA
pub fn generate_regex_dataset(
    seed: u64,
    seq_length: usize,
    m_sequences: usize,
) -> Vec<EvaluationSequence> {
    let mut rng = FastRng::seed_from_u64(seed.wrapping_add(seq_length as u64 * 2000 + 0xDE_2026));
    let mut dataset = Vec::with_capacity(m_sequences);

    let n_pos = m_sequences / 2;
    let n_neg = m_sequences - n_pos;

    // Helper: compute expected DFA states for regex (10)+1
    // states: 0=q0, 1=q1, 2=q2, 3=q3, 4=trap
    let dfa_transition = |state: usize, tok: usize| -> usize {
        match (state, tok) {
            (0, 1) => 1,
            (0, 0) => 4,
            (1, 0) => 2,
            (1, 1) => 4,
            (2, 1) => 3,
            (2, 0) => 4,
            (3, 0) => 2,
            (3, 1) => 4,
            (4, _) => 4,
            _ => 4,
        }
    };

    // 1. Positive class sequences: matching (10)^k 1
    // If seq_length is even, positive strings have length seq_length - 1 (odd)
    let pos_len = if seq_length % 2 == 1 {
        seq_length
    } else {
        seq_length - 1
    };
    let k_pairs = (pos_len - 1) / 2;

    for _ in 0..n_pos {
        let mut tokens = Vec::with_capacity(pos_len);
        let mut states = Vec::with_capacity(pos_len);
        let mut curr_state = 0;

        for _ in 0..k_pairs {
            tokens.push(1);
            curr_state = dfa_transition(curr_state, 1);
            states.push(curr_state);

            tokens.push(0);
            curr_state = dfa_transition(curr_state, 0);
            states.push(curr_state);
        }
        tokens.push(1);
        curr_state = dfa_transition(curr_state, 1);
        states.push(curr_state);

        dataset.push(EvaluationSequence {
            tokens,
            target_label: 1, // q3 is accepting
            expected_states: states,
        });
    }

    // 2. Negative class sequences: corruptions or even-length alternating strings
    for i in 0..n_neg {
        let mut tokens = Vec::with_capacity(seq_length);
        let mut states = Vec::with_capacity(seq_length);
        let mut curr_state = 0;

        match i % 4 {
            0 => {
                // Alternating string of length seq_length ending in 0: (10)^(seq_length/2) -> ends in q2 (not accepting)
                let num_pairs = seq_length / 2;
                for _ in 0..num_pairs {
                    tokens.push(1);
                    curr_state = dfa_transition(curr_state, 1);
                    states.push(curr_state);

                    tokens.push(0);
                    curr_state = dfa_transition(curr_state, 0);
                    states.push(curr_state);
                }
                while tokens.len() < seq_length {
                    tokens.push(0);
                    curr_state = dfa_transition(curr_state, 0);
                    states.push(curr_state);
                }
            }
            1 => {
                // Starts with 0 -> immediately enters trap state 4
                tokens.push(0);
                curr_state = dfa_transition(curr_state, 0);
                states.push(curr_state);

                for _ in 1..seq_length {
                    let tok = if rng.bernoulli(0.5) { 1 } else { 0 };
                    tokens.push(tok);
                    curr_state = dfa_transition(curr_state, tok);
                    states.push(curr_state);
                }
            }
            2 => {
                // Double 1 corruption (e.g. 1 1 ...) -> enters trap
                tokens.push(1);
                curr_state = dfa_transition(curr_state, 1);
                states.push(curr_state);

                tokens.push(1);
                curr_state = dfa_transition(curr_state, 1);
                states.push(curr_state);

                for _ in 2..seq_length {
                    let tok = if rng.bernoulli(0.5) { 1 } else { 0 };
                    tokens.push(tok);
                    curr_state = dfa_transition(curr_state, tok);
                    states.push(curr_state);
                }
            }
            _ => {
                // Random bitstring, check if matches (10)+1; if so, flip last bit to 0 to make negative
                let mut tok_list = Vec::with_capacity(seq_length);
                let mut st_list = Vec::with_capacity(seq_length);
                let mut st = 0;
                for _ in 0..seq_length {
                    let tok = if rng.bernoulli(0.5) { 1 } else { 0 };
                    tok_list.push(tok);
                    st = dfa_transition(st, tok);
                    st_list.push(st);
                }
                if st == 3 {
                    // Accidentally matched! Change last token to 0
                    let last_idx = seq_length - 1;
                    tok_list[last_idx] = 0;
                    st_list[last_idx] = 2; // q3 on 0 goes to q2
                }
                tokens = tok_list;
                states = st_list;
            }
        }

        let final_state = *states.last().unwrap_or(&4);
        let target_label = if final_state == 3 { 1 } else { 0 };

        dataset.push(EvaluationSequence {
            tokens,
            target_label,
            expected_states: states,
        });
    }

    dataset
}

/// Execute a single sequence through the simulated substrate
pub fn evaluate_sequence_trial(
    config: &ExperimentConfig,
    seq: &EvaluationSequence,
    seq_id: usize,
) -> SequenceTrialRecord {
    let t_token = config.t_token; // 16 ticks
    let t_warmup = config.t_warmup; // 16 ticks

    // Handle BaselineMemoryless directly
    if config.condition == Condition::BaselineMemoryless {
        let last_token = *seq.tokens.last().unwrap_or(&0);
        let predicted_label = match config.task {
            AutomatonTask::ParityDfa => last_token,
            AutomatonTask::RegexDfa => {
                if last_token == 1 {
                    1
                } else {
                    0
                }
            }
        };
        let correct = predicted_label == seq.target_label;
        let tok_str: String = seq
            .tokens
            .iter()
            .map(|&t| (b'0' + t as u8) as char)
            .collect();

        return SequenceTrialRecord {
            seed: config.seed,
            condition: config.condition.as_str().to_string(),
            task: config.task.as_str().to_string(),
            seq_length: seq.tokens.len(),
            noise_rate: config.noise_rate,
            seq_id,
            input_tokens: tok_str,
            target_label: seq.target_label,
            predicted_label,
            correct,
            transitions_correct: if correct { 1.0 } else { 0.5 },
            crosstalk_ticks: 0,
            mean_firing_density: 0.05,
        };
    }

    let mut circuit = build_circuit(config.clone());

    // Initial state seeding: seed solitary pulse into Ring 0 (q0 / q_even)
    circuit.force_spike(circuit.ring_nodes[0][0]);

    // Warmup phase: let Ring 0 circulate to establish clean limit cycle
    for _ in 0..t_warmup {
        circuit.step((0.0, 0.0), false);
    }

    let mut crosstalk_ticks = 0;
    let mut transitions_correct = 0;
    let num_tokens = seq.tokens.len();

    // Stream tokens sequentially through time
    for (token_idx, &tok) in seq.tokens.iter().enumerate() {
        let expected_dfa_state = seq.expected_states[token_idx];

        // Tick 0 of token aperture: inject token pulse
        let token_input = if tok == 0 { (1.0, 0.0) } else { (0.0, 1.0) };
        circuit.step(token_input, true);

        // Ticks 1..t_token: relaxation and circulation
        let mut ring_spikes_in_window = vec![0usize; circuit.ring_nodes.len()];

        for tick in 1..t_token {
            circuit.step((0.0, 0.0), true);

            // In settled phase (ticks 4..15): monitor ring mutual exclusivity
            if tick >= 4 {
                let active_rings = circuit.active_ring_count();
                if active_rings > 1 {
                    crosstalk_ticks += 1;
                }
            }

            // In settled window [4..8]: accumulate spikes per ring to identify active ring
            if (4..8).contains(&tick) {
                for (r_idx, ring) in circuit.ring_nodes.iter().enumerate() {
                    for &node_idx in ring {
                        if circuit.nodes[node_idx].spike > 0.0 {
                            ring_spikes_in_window[r_idx] += 1;
                        }
                    }
                }
            }
        }

        // Determine which ring was settled active
        let observed_state = {
            let active_list: Vec<usize> = ring_spikes_in_window
                .iter()
                .enumerate()
                .filter(|&(_, &spikes)| spikes > 0)
                .map(|(r_idx, _)| r_idx)
                .collect();
            if active_list.len() == 1 {
                active_list[0]
            } else if active_list.is_empty() {
                4 // trap / quiescent silence
            } else {
                99 // crosstalk violation
            }
        };

        if observed_state == expected_dfa_state {
            transitions_correct += 1;
        }
    }

    // Readout Phase: Probe v_accept over 4 clock ticks
    let mut accept_spikes = 0;
    for _ in 0..4 {
        let spike = circuit.step((0.0, 0.0), true);
        if spike > 0.0 {
            accept_spikes += 1;
        }
    }

    let predicted_label = if accept_spikes > 0 { 1 } else { 0 };
    let correct = predicted_label == seq.target_label;
    let mean_firing_density = circuit.evaluation_firing_density();

    let tok_str: String = seq
        .tokens
        .iter()
        .map(|&t| (b'0' + t as u8) as char)
        .collect();

    SequenceTrialRecord {
        seed: config.seed,
        condition: config.condition.as_str().to_string(),
        task: config.task.as_str().to_string(),
        seq_length: seq.tokens.len(),
        noise_rate: config.noise_rate,
        seq_id,
        input_tokens: tok_str,
        target_label: seq.target_label,
        predicted_label,
        correct,
        transitions_correct: transitions_correct as f64 / num_tokens as f64,
        crosstalk_ticks,
        mean_firing_density,
    }
}

/// Execute a full run (e.g. 50 sequences for a given seed, condition, task, length, noise)
pub fn execute_run(
    config: &ExperimentConfig,
    m_sequences: usize,
) -> (RunTelemetryRecord, Vec<SequenceTrialRecord>) {
    let dataset = match config.task {
        AutomatonTask::ParityDfa => {
            generate_parity_dataset(config.seed, config.seq_length, m_sequences)
        }
        AutomatonTask::RegexDfa => {
            generate_regex_dataset(config.seed, config.seq_length, m_sequences)
        }
    };

    let mut trials = Vec::with_capacity(dataset.len());
    let mut sequences_correct = 0;
    let mut transitions_total = 0;
    let mut transitions_correct = 0;
    let mut crosstalk_ticks = 0;
    let mut settled_ticks = 0;
    let mut firing_densities = Vec::with_capacity(dataset.len());

    for (seq_id, seq) in dataset.iter().enumerate() {
        let trial = evaluate_sequence_trial(config, seq, seq_id + 1);
        if trial.correct {
            sequences_correct += 1;
        }
        transitions_total += seq.tokens.len();
        transitions_correct +=
            (trial.transitions_correct * seq.tokens.len() as f64).round() as usize;
        crosstalk_ticks += trial.crosstalk_ticks;
        // In each token aperture, settled ticks = (t_token - 4) = 12 ticks
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
    let fidelity_trans = if transitions_total > 0 {
        transitions_correct as f64 / transitions_total as f64
    } else {
        0.0
    };
    let crosstalk_violation_rate = if settled_ticks > 0 {
        crosstalk_ticks as f64 / settled_ticks as f64
    } else {
        0.0
    };
    let ber_read = 1.0 - accuracy_seq;
    let mean_firing_density = sample_mean(&firing_densities);

    let homeostasis_stable = (0.01..=0.25).contains(&mean_firing_density);

    let metrics = RunMetrics {
        accuracy_seq,
        fidelity_trans,
        crosstalk_violation_rate,
        ber_read,
        mean_firing_density,
        sequences_total,
        sequences_correct,
        transitions_total,
        transitions_correct,
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

/// Execute complete factorial sweep and serialize telemetry files
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

    println!(
        "Initiating EXP-2026-009a Factorial Sweep: {} conditions, {} tasks, {} noise rates, {} lengths, {} seeds",
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
                for &len in seq_lengths {
                    let cell_key =
                        format!("{}:{}:{}:{:.2}", cond.as_str(), task.as_str(), len, noise);
                    let mut cell_metrics = Vec::with_capacity(num_seeds);

                    for seed in 1..=(num_seeds as u64) {
                        let mut cfg = ExperimentConfig::new(cond, task, noise, seed);
                        cfg.seq_length = len;
                        cfg.output_dir = output_dir.to_string_lossy().to_string();

                        let (run_rec, trials) = execute_run(&cfg, m_sequences);

                        for trial in &trials {
                            let json_line = serde_json::to_string(trial)
                                .expect("Failed to serialize trial record");
                            writeln!(trials_writer, "{}", json_line)
                                .expect("Failed to write to trials.jsonl");
                        }

                        cell_metrics.push(run_rec.metrics.clone());
                        all_run_records.push(run_rec);
                    }

                    cell_records.insert(cell_key, cell_metrics);
                    completed_cells += 1;
                    if completed_cells % 10 == 0 || completed_cells == total_cells {
                        print!(
                            "\rProgress: [{completed_cells}/{total_cells}] parameter cells completed."
                        );
                        let _ = std::io::stdout().flush();
                    }
                }
            }
        }
    }
    println!();
    trials_writer.flush().expect("Failed to flush trials.jsonl");

    // ----------------------------------------------------
    // Compute Summaries and Gates
    // ----------------------------------------------------
    let mut conditions_summary = BTreeMap::new();
    for (cell_key, metrics_vec) in &cell_records {
        let accs: Vec<f64> = metrics_vec.iter().map(|m| m.accuracy_seq).collect();
        let fids: Vec<f64> = metrics_vec.iter().map(|m| m.fidelity_trans).collect();
        let x_talks: Vec<f64> = metrics_vec
            .iter()
            .map(|m| m.crosstalk_violation_rate)
            .collect();
        let bers: Vec<f64> = metrics_vec.iter().map(|m| m.ber_read).collect();
        let dens: Vec<f64> = metrics_vec.iter().map(|m| m.mean_firing_density).collect();

        let acc_mean = sample_mean(&accs);
        let acc_std = sample_std(&accs, acc_mean);
        let fid_mean = sample_mean(&fids);
        let fid_std = sample_std(&fids, fid_mean);
        let x_talk_mean = sample_mean(&x_talks);
        let ber_mean = sample_mean(&bers);
        let ber_std = sample_std(&bers, ber_mean);
        let den_mean = sample_mean(&dens);

        conditions_summary.insert(
            cell_key.clone(),
            ConditionCellSummary {
                accuracy_mean: acc_mean,
                accuracy_std: acc_std,
                fidelity_mean: fid_mean,
                fidelity_std: fid_std,
                crosstalk_mean: x_talk_mean,
                ber_mean,
                ber_std,
                firing_density_mean: den_mean,
            },
        );
    }

    // Gate 1: Sequence Classification Accuracy at eps = 0.00 for active_dfa across all lengths and tasks
    let mut gate1_accs = Vec::new();
    for (cell_key, metrics_vec) in &cell_records {
        if cell_key.starts_with("active_dfa:") && cell_key.ends_with(":0.00") {
            for m in metrics_vec {
                gate1_accs.push(m.accuracy_seq);
            }
        }
    }
    let gate1_observed = sample_mean(&gate1_accs);
    let gate1_pass = gate1_observed >= 0.9999;

    // Gate 2: State Transition Reliability at eps = 0.00
    let mut gate2_fids = Vec::new();
    for (cell_key, metrics_vec) in &cell_records {
        if cell_key.starts_with("active_dfa:") && cell_key.ends_with(":0.00") {
            for m in metrics_vec {
                gate2_fids.push(m.fidelity_trans);
            }
        }
    }
    let gate2_observed = sample_mean(&gate2_fids);
    let gate2_pass = gate2_observed >= 0.9999;

    // Gate 3: Attractor Mutual Exclusivity at eps = 0.00
    let mut gate3_xtalks = Vec::new();
    for (cell_key, metrics_vec) in &cell_records {
        if cell_key.starts_with("active_dfa:") && cell_key.ends_with(":0.00") {
            for m in metrics_vec {
                gate3_xtalks.push(m.crosstalk_violation_rate);
            }
        }
    }
    let gate3_observed = sample_mean(&gate3_xtalks);
    let gate3_pass = gate3_observed <= 0.0001;

    // Gate 4: Critical Noise Resilience at eps = 0.05
    let mut gate4_accs = Vec::new();
    let mut gate4_bers = Vec::new();
    for (cell_key, metrics_vec) in &cell_records {
        if cell_key.starts_with("active_dfa:") && cell_key.ends_with(":0.05") {
            for m in metrics_vec {
                gate4_accs.push(m.accuracy_seq);
                gate4_bers.push(m.ber_read);
            }
        }
    }
    let gate4_acc_obs = sample_mean(&gate4_accs);
    let gate4_ber_obs = sample_mean(&gate4_bers);
    let gate4_pass = gate4_acc_obs >= 0.950 && gate4_ber_obs <= 0.025;

    // Gate 5: Recurrent Advantage vs Feedforward Loss at eps = 0.00
    let mut act_clean_accs = Vec::new();
    let mut ff_clean_accs = Vec::new();
    for (cell_key, metrics_vec) in &cell_records {
        if cell_key.starts_with("active_dfa:") && cell_key.ends_with(":0.00") {
            for m in metrics_vec {
                act_clean_accs.push(m.accuracy_seq);
            }
        }
        if cell_key.starts_with("baseline_feedforward_loss:") && cell_key.ends_with(":0.00") {
            for m in metrics_vec {
                ff_clean_accs.push(m.accuracy_seq);
            }
        }
    }
    let (t_g5, p_g5, d_g5) = welch_t_test(&act_clean_accs, &ff_clean_accs);
    let gate5_pass = p_g5 < 1e-6 && d_g5 >= 2.0;

    // Gate 6: Homeostatic Density Stability
    let mut homeo_compliant = 0;
    let mut total_active_runs = 0;
    for (cell_key, metrics_vec) in &cell_records {
        if cell_key.starts_with("active_dfa:") {
            for m in metrics_vec {
                total_active_runs += 1;
                if (0.01..=0.25).contains(&m.mean_firing_density) {
                    homeo_compliant += 1;
                }
            }
        }
    }
    let gate6_observed = if total_active_runs > 0 {
        homeo_compliant as f64 / total_active_runs as f64
    } else {
        0.0
    };
    let gate6_pass = gate6_observed >= 0.990;

    let mut gates = BTreeMap::new();
    gates.insert(
        "gate_1_sequence_classification".to_string(),
        GateResult {
            metric: "sequence_classification_accuracy".to_string(),
            target: 1.000,
            observed: gate1_observed,
            passed: gate1_pass,
        },
    );
    gates.insert(
        "gate_2_transition_fidelity".to_string(),
        GateResult {
            metric: "state_transition_fidelity".to_string(),
            target: 1.000,
            observed: gate2_observed,
            passed: gate2_pass,
        },
    );
    gates.insert(
        "gate_3_mutual_exclusivity".to_string(),
        GateResult {
            metric: "crosstalk_violation_rate".to_string(),
            target: 0.000,
            observed: gate3_observed,
            passed: gate3_pass,
        },
    );
    gates.insert(
        "gate_4_noise_resilience".to_string(),
        GateResult {
            metric: "accuracy_under_critical_noise".to_string(),
            target: 0.950,
            observed: gate4_acc_obs,
            passed: gate4_pass,
        },
    );
    gates.insert(
        "gate_5_recurrent_advantage".to_string(),
        GateResult {
            metric: "welch_p_value".to_string(),
            target: 1e-6,
            observed: p_g5,
            passed: gate5_pass,
        },
    );
    gates.insert(
        "gate_6_homeostatic_stability".to_string(),
        GateResult {
            metric: "firing_density_compliance_rate".to_string(),
            target: 0.990,
            observed: gate6_observed,
            passed: gate6_pass,
        },
    );

    let mut statistical_tests = BTreeMap::new();
    statistical_tests.insert(
        "active_vs_feedforward_loss_clean".to_string(),
        StatisticalTestResult {
            t_stat: t_g5,
            p_value: p_g5,
            cohen_d: d_g5,
            significant: gate5_pass,
        },
    );

    // Active vs Fixed Theta at eps = 0.05
    let mut fixed_noise_accs = Vec::new();
    for (cell_key, metrics_vec) in &cell_records {
        if cell_key.starts_with("ablation_fixed_theta:") && cell_key.ends_with(":0.05") {
            for m in metrics_vec {
                fixed_noise_accs.push(m.accuracy_seq);
            }
        }
    }
    let (t_g4, p_g4, d_g4) = welch_t_test(&gate4_accs, &fixed_noise_accs);
    statistical_tests.insert(
        "active_vs_fixed_theta_noise_0_05".to_string(),
        StatisticalTestResult {
            t_stat: t_g4,
            p_value: p_g4,
            cohen_d: d_g4,
            significant: p_g4 < 1e-4,
        },
    );

    let all_passed =
        gate1_pass && gate2_pass && gate3_pass && gate4_pass && gate5_pass && gate6_pass;
    let overall_verdict = if all_passed {
        "SUPPORTED".to_string()
    } else {
        "REFUTED".to_string()
    };

    let manifest = EvaluationSummaryManifest {
        protocol_id: "EXP-2026-009a".to_string(),
        hypothesis_id: "HYP-2026-009".to_string(),
        timestamp_utc: "2026-09-07T20:30:00Z".to_string(),
        git_commit: "HEAD".to_string(),
        total_runs: all_run_records.len(),
        total_sequences: all_run_records.len() * m_sequences,
        gates,
        conditions: conditions_summary,
        statistical_tests,
        overall_verdict,
    };

    let summary_file = File::create(&summary_path).expect("Failed to create summary.json");
    let mut summary_writer = BufWriter::new(summary_file);
    serde_json::to_writer_pretty(&mut summary_writer, &manifest)
        .expect("Failed to serialize summary.json");
    summary_writer
        .flush()
        .expect("Failed to flush summary.json");

    println!(
        "EXP-2026-009a Sweep Complete. Summary written to {}",
        summary_path.display()
    );
    manifest
}
