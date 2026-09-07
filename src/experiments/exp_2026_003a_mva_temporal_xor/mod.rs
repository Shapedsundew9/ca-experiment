//! EXP-2026-003a: Non-Linear Temporal XOR Benchmark on Homeostatic CA Substrates.

pub mod ca;
pub mod config;
pub mod dataset;
pub mod metrics;
pub mod readout;

use ca::{NUM_NODES, TorusSubstrate};
use config::{ExperimentConfig, ExperimentalCondition};
use dataset::TemporalStream;
use metrics::{
    ConditionDelaySummary, GateEvaluation, RunMetrics, RunRecord, SummaryEvaluation,
    two_sample_t_test,
};
use readout::OnlineRidgeAccumulator;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Execute a single experiment run conforming to config
#[allow(clippy::needless_range_loop)]
pub fn run_single_experiment(config: &ExperimentConfig) -> RunMetrics {
    let total_ticks = config.washout_ticks + config.train_ticks + config.test_ticks;
    let stream = TemporalStream::generate(config, total_ticks);

    if config.condition == ExperimentalCondition::BaselineLinearDirect {
        // Direct linear model on raw inputs: [x_t, x_{t-1}, ..., x_{t-tau}]
        let feat_dim = config.delay_tau + 1;
        let mut accum = OnlineRidgeAccumulator::new(feat_dim);

        // Training accumulation
        for t in config.washout_ticks..(config.washout_ticks + config.train_ticks) {
            let mut phi = vec![0.0; feat_dim];
            for lag in 0..=config.delay_tau {
                phi[lag] = if t >= lag {
                    stream.inputs[t - lag]
                } else {
                    0.0
                };
            }
            accum.update(&phi, stream.targets[t]);
        }

        let model = match accum.solve(config.ridge_alpha) {
            Some(m) => m,
            None => {
                return RunMetrics {
                    train_accuracy: 0.5,
                    test_accuracy: 0.5,
                    train_mse: 0.25,
                    test_mse: 0.25,
                    capacity_k_xor: 0.0,
                    capacity_m_tau: 0.0,
                    mean_firing_density: 0.0,
                    min_firing_density: 0.0,
                    max_firing_density: 0.0,
                    mean_threshold: 0.0,
                    weight_l2_norm: 0.0,
                };
            }
        };

        // Train evaluation
        let mut train_correct = 0;
        let mut train_se = 0.0;
        for t in config.washout_ticks..(config.washout_ticks + config.train_ticks) {
            let mut phi = vec![0.0; feat_dim];
            for lag in 0..=config.delay_tau {
                phi[lag] = if t >= lag {
                    stream.inputs[t - lag]
                } else {
                    0.0
                };
            }
            let pred = model.predict_binary(&phi);
            let act = model.predict_activation(&phi);
            if (pred - stream.targets[t]).abs() < 1e-6 {
                train_correct += 1;
            }
            train_se += (act - stream.targets[t]).powi(2);
        }
        let train_accuracy = train_correct as f64 / config.train_ticks as f64;
        let train_mse = train_se / config.train_ticks as f64;

        // Test evaluation
        let mut test_correct = 0;
        let mut test_se = 0.0;
        let mut sum_sq_tot = 0.0;
        let test_start = config.washout_ticks + config.train_ticks;
        let mean_test_target: f64 =
            stream.targets[test_start..total_ticks].iter().sum::<f64>() / config.test_ticks as f64;

        for t in test_start..total_ticks {
            let mut phi = vec![0.0; feat_dim];
            for lag in 0..=config.delay_tau {
                phi[lag] = if t >= lag {
                    stream.inputs[t - lag]
                } else {
                    0.0
                };
            }
            let pred = model.predict_binary(&phi);
            let act = model.predict_activation(&phi);
            if (pred - stream.targets[t]).abs() < 1e-6 {
                test_correct += 1;
            }
            test_se += (act - stream.targets[t]).powi(2);
            sum_sq_tot += (stream.targets[t] - mean_test_target).powi(2);
        }

        let test_accuracy = test_correct as f64 / config.test_ticks as f64;
        let test_mse = test_se / config.test_ticks as f64;
        let capacity_k_xor = if sum_sq_tot > 1e-12 {
            1.0 - test_se / sum_sq_tot
        } else {
            0.0
        };

        return RunMetrics {
            train_accuracy,
            test_accuracy,
            train_mse,
            test_mse,
            capacity_k_xor,
            capacity_m_tau: 0.0,
            mean_firing_density: 0.0,
            min_firing_density: 0.0,
            max_firing_density: 0.0,
            mean_threshold: 0.0,
            weight_l2_norm: model.l2_norm(),
        };
    }

    // Substrate simulation
    let mut substrate = TorusSubstrate::new(config.clone());
    let mut accum = OnlineRidgeAccumulator::new(NUM_NODES);

    let mut train_spikes = Vec::with_capacity(config.train_ticks);
    let mut test_spikes = Vec::with_capacity(config.test_ticks);

    let test_start = config.washout_ticks + config.train_ticks;
    let mut test_spike_counts = [0usize; NUM_NODES];
    let mut test_tick_densities = Vec::with_capacity(config.test_ticks);

    for t in 0..total_ticks {
        let spikes = substrate.step(stream.inputs[t]);

        if t >= config.washout_ticks && t < test_start {
            accum.update(&spikes, stream.targets[t]);
            train_spikes.push(spikes);
        } else if t >= test_start {
            test_spikes.push(spikes);
            let mut tick_spikes = 0usize;
            for (i, &s) in spikes.iter().enumerate() {
                if s > 0.5 {
                    test_spike_counts[i] += 1;
                    tick_spikes += 1;
                }
            }
            test_tick_densities.push(tick_spikes as f64 / NUM_NODES as f64);
        }
    }

    let model = match accum.solve(config.ridge_alpha) {
        Some(m) => m,
        None => {
            return RunMetrics {
                train_accuracy: 0.5,
                test_accuracy: 0.5,
                train_mse: 0.25,
                test_mse: 0.25,
                capacity_k_xor: 0.0,
                capacity_m_tau: 0.0,
                mean_firing_density: 0.0,
                min_firing_density: 0.0,
                max_firing_density: 0.0,
                mean_threshold: 0.0,
                weight_l2_norm: 0.0,
            };
        }
    };

    // Train evaluation
    let mut train_correct = 0;
    let mut train_se = 0.0;
    for (spikes, &target) in train_spikes
        .iter()
        .zip(stream.targets[config.washout_ticks..test_start].iter())
    {
        let pred = model.predict_binary(spikes);
        let act = model.predict_activation(spikes);
        if (pred - target).abs() < 1e-6 {
            train_correct += 1;
        }
        train_se += (act - target).powi(2);
    }
    let train_accuracy = train_correct as f64 / config.train_ticks as f64;
    let train_mse = train_se / config.train_ticks as f64;

    // Test evaluation
    let mut test_correct = 0;
    let mut test_se = 0.0;
    let mut sum_sq_tot = 0.0;
    let mean_test_target: f64 =
        stream.targets[test_start..total_ticks].iter().sum::<f64>() / config.test_ticks as f64;

    for (spikes, &target) in test_spikes
        .iter()
        .zip(stream.targets[test_start..total_ticks].iter())
    {
        let pred = model.predict_binary(spikes);
        let act = model.predict_activation(spikes);
        if (pred - target).abs() < 1e-6 {
            test_correct += 1;
        }
        test_se += (act - target).powi(2);
        sum_sq_tot += (target - mean_test_target).powi(2);
    }

    let test_accuracy = test_correct as f64 / config.test_ticks as f64;
    let test_mse = test_se / config.test_ticks as f64;
    let capacity_k_xor = if sum_sq_tot > 1e-12 {
        1.0 - test_se / sum_sq_tot
    } else {
        0.0
    };

    let mean_firing_density = test_tick_densities.iter().sum::<f64>() / config.test_ticks as f64;
    let min_firing_density = test_tick_densities.iter().copied().fold(1.0f64, f64::min);
    let max_firing_density = test_tick_densities.iter().copied().fold(0.0f64, f64::max);

    let mean_threshold: f64 =
        substrate.nodes.iter().map(|n| n.v_thresh).sum::<f64>() / NUM_NODES as f64;

    RunMetrics {
        train_accuracy,
        test_accuracy,
        train_mse,
        test_mse,
        capacity_k_xor,
        capacity_m_tau: if config.condition == ExperimentalCondition::ActiveIdentity {
            capacity_k_xor
        } else {
            0.0
        },
        mean_firing_density,
        min_firing_density,
        max_firing_density,
        mean_threshold,
        weight_l2_norm: model.l2_norm(),
    }
}

pub struct SweepConfig {
    pub seeds: Vec<u64>,
    pub delays: Vec<usize>,
    pub densities: Vec<f64>,
    pub conditions: Vec<ExperimentalCondition>,
    pub ridge_alpha: f64,
    pub washout_ticks: usize,
    pub train_ticks: usize,
    pub test_ticks: usize,
    pub output_dir: String,
}

pub fn execute_full_sweep(sweep: &SweepConfig) -> (Vec<RunRecord>, SummaryEvaluation) {
    let mut run_records = Vec::new();

    // Map: condition -> delay -> Vec<acc>
    let mut cond_delay_accs: BTreeMap<String, BTreeMap<usize, Vec<f64>>> = BTreeMap::new();

    for &cond in &sweep.conditions {
        cond_delay_accs.insert(cond.name().to_string(), BTreeMap::new());
        for &tau in &sweep.delays {
            cond_delay_accs
                .get_mut(cond.name())
                .unwrap()
                .insert(tau, Vec::new());
        }
    }

    for &cond in &sweep.conditions {
        for &tau in &sweep.delays {
            for &dens in &sweep.densities {
                for &seed in &sweep.seeds {
                    let config = ExperimentConfig {
                        seed,
                        condition: cond,
                        delay_tau: tau,
                        pulse_density: dens,
                        ridge_alpha: sweep.ridge_alpha,
                        washout_ticks: sweep.washout_ticks,
                        train_ticks: sweep.train_ticks,
                        test_ticks: sweep.test_ticks,
                        ..Default::default()
                    };

                    let metrics = run_single_experiment(&config);

                    cond_delay_accs
                        .get_mut(cond.name())
                        .unwrap()
                        .get_mut(&tau)
                        .unwrap()
                        .push(metrics.test_accuracy);

                    run_records.push(RunRecord {
                        seed,
                        condition: cond.name().to_string(),
                        delay_tau: tau,
                        pulse_density: dens,
                        ridge_alpha: sweep.ridge_alpha,
                        metrics,
                    });
                }
            }
        }
    }

    // Build condition summaries
    let mut condition_summaries = BTreeMap::new();
    for (cond_name, delays_map) in &cond_delay_accs {
        let mut delay_summary_map = BTreeMap::new();
        for (&tau, accs) in delays_map {
            let n = accs.len() as f64;
            let mean = if n > 0.0 {
                accs.iter().sum::<f64>() / n
            } else {
                0.0
            };
            let std = if n > 1.0 {
                (accs.iter().map(|a| (a - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt()
            } else {
                0.0
            };
            delay_summary_map.insert(
                format!("tau_{tau}"),
                ConditionDelaySummary {
                    mean_acc: mean,
                    std_acc: std,
                },
            );
        }
        condition_summaries.insert(cond_name.clone(), delay_summary_map);
    }

    // Gate evaluation at tau = 5
    let active_tau_5 = cond_delay_accs
        .get("active_xor")
        .and_then(|m| m.get(&5))
        .cloned()
        .unwrap_or_default();
    let memless_tau_5 = cond_delay_accs
        .get("ablation_memoryless")
        .and_then(|m| m.get(&5))
        .cloned()
        .unwrap_or_default();
    let direct_tau_5 = cond_delay_accs
        .get("baseline_linear_direct")
        .and_then(|m| m.get(&5))
        .cloned()
        .unwrap_or_default();
    let static_tau_5 = cond_delay_accs
        .get("ablation_static_thresh")
        .and_then(|m| m.get(&5))
        .cloned()
        .unwrap_or_default();

    let mean_active_5 = if !active_tau_5.is_empty() {
        active_tau_5.iter().sum::<f64>() / active_tau_5.len() as f64
    } else {
        0.0
    };
    let std_active_5 = if active_tau_5.len() > 1 {
        (active_tau_5
            .iter()
            .map(|a| (a - mean_active_5).powi(2))
            .sum::<f64>()
            / (active_tau_5.len() as f64 - 1.0))
            .sqrt()
    } else {
        0.0
    };
    let ci_margin = 1.96 * std_active_5 / (active_tau_5.len() as f64).sqrt().max(1.0);
    let tau_5_active_xor_ci_95 = [mean_active_5 - ci_margin, mean_active_5 + ci_margin];

    let p_memless = two_sample_t_test(&active_tau_5, &memless_tau_5);
    let p_direct = two_sample_t_test(&active_tau_5, &direct_tau_5);
    let p_static = two_sample_t_test(&active_tau_5, &static_tau_5);

    let tau_5_gate_passed = mean_active_5 > 0.950;
    let memoryless_separation_passed = p_memless < 1e-6;
    let direct_linear_separation_passed = p_direct < 1e-6;
    let static_thresh_advantage_passed = p_static < 1e-3;

    let overall_verdict =
        if tau_5_gate_passed && memoryless_separation_passed && direct_linear_separation_passed {
            "SUPPORTED".to_string()
        } else if memoryless_separation_passed && direct_linear_separation_passed {
            "SUPPORTED_RECALIBRATED".to_string()
        } else {
            "FALSIFIED".to_string()
        };

    let gate_eval = GateEvaluation {
        tau_5_active_xor_mean_accuracy: mean_active_5,
        tau_5_active_xor_ci_95,
        tau_5_gate_passed,
        p_value_vs_memoryless: p_memless,
        memoryless_separation_passed,
        p_value_vs_direct_linear: p_direct,
        direct_linear_separation_passed,
        p_value_vs_static_thresh: p_static,
        static_thresh_advantage_passed,
        overall_hypothesis_verdict: overall_verdict,
    };

    let summary = SummaryEvaluation {
        protocol_id: "EXP-2026-003a".to_string(),
        hypothesis_id: "HYP-2026-003".to_string(),
        execution_timestamp: "2026-09-07T00:00:00Z".to_string(),
        total_runs: run_records.len(),
        gate_evaluation: gate_eval,
        condition_summaries,
    };

    // Write output files
    let out_dir = Path::new(&sweep.output_dir);
    std::fs::create_dir_all(out_dir).expect("Failed to create output directory");

    let run_results_path = out_dir.join("run_results.jsonl");
    let file = File::create(&run_results_path).expect("Failed to create run_results.jsonl");
    let mut writer = BufWriter::new(file);
    for rec in &run_records {
        let json = serde_json::to_string(rec).expect("Failed to serialize record");
        writeln!(writer, "{}", json).expect("Failed to write record line");
    }
    writer.flush().expect("Failed to flush run_results.jsonl");

    let summary_path = out_dir.join("summary_evaluation.json");
    let sum_file = File::create(&summary_path).expect("Failed to create summary_evaluation.json");
    serde_json::to_writer_pretty(sum_file, &summary).expect("Failed to write summary json");

    (run_records, summary)
}
