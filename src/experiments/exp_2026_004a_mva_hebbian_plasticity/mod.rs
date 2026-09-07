//! EXP-2026-004a: Local Hebbian Track Rewiring and Autonomous Attractor Basin Deepening.

pub mod ca;
pub mod config;
pub mod metrics;
pub mod patterns;
pub mod plasticity;

use ca::TorusSubstrate;
use config::{ExperimentConfig, PatternId, PlasticityCondition};
use metrics::{
    GateEvaluation, RunMetrics, RunRecord, SummaryEvaluation, occupancy_distance, two_sample_t_test,
};
use patterns::{FastRng, apply_bit_flip_noise, get_pattern_bits};
use plasticity::NUM_NODES;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Run simulation for a given pattern and noise rate on a prepared substrate, returning occupancy vector and mean firing density
fn evaluate_pattern_settling(
    substrate: &mut TorusSubstrate,
    pattern: PatternId,
    noise_rate: f64,
    rng: &mut FastRng,
) -> ([f64; NUM_NODES], f64) {
    let clean_bits = get_pattern_bits(pattern);
    let drive_bits = apply_bit_flip_noise(&clean_bits, noise_rate, rng);

    let t_drive = substrate.config.drive_ticks;
    let t_relax = substrate.config.relax_ticks;
    let sample_window = substrate.config.sample_window;
    let t_total = t_drive + t_relax;
    let sample_start = t_total - sample_window;

    let mut spike_sums = [0.0; NUM_NODES];
    let mut total_window_spikes = 0.0;

    for t in 0..t_total {
        let mut ext_ingress = [0.0; NUM_NODES];
        if t < t_drive {
            // Node 0 receives direct bit
            ext_ingress[0] = drive_bits[t % 32];
            // Node 5 receives bit phase-shifted by 4 ticks
            ext_ingress[5] = drive_bits[(t + 28) % 32];
        } else {
            // Neutral carrier clock: alternating pulse on sensory nodes
            if t % 2 == 0 {
                ext_ingress[0] = 1.0;
                ext_ingress[5] = 1.0;
            }
        }

        let spikes = substrate.step(&ext_ingress, false);

        if t >= sample_start {
            for i in 0..NUM_NODES {
                spike_sums[i] += spikes[i];
                total_window_spikes += spikes[i];
            }
        }
    }

    let mut occupancy = [0.0; NUM_NODES];
    for i in 0..NUM_NODES {
        occupancy[i] = spike_sums[i] / sample_window as f64;
    }

    let mean_density = total_window_spikes / (sample_window * NUM_NODES) as f64;
    (occupancy, mean_density)
}

/// Execute a single experiment run including training exposure and frozen attractor evaluation
pub fn run_single_experiment(config: &ExperimentConfig) -> RunMetrics {
    let mut substrate = TorusSubstrate::new(config.clone());
    let mut rng = FastRng::seed_from_u64(config.seed);

    // Phase 1: Plastic Training Exposure on config.pattern
    if config.exposure_ticks > 0 && config.condition != PlasticityCondition::BaselineStatic {
        let train_pattern = config.pattern;
        let train_bits = get_pattern_bits(train_pattern);

        for t in 0..config.exposure_ticks {
            let mut ext_ingress = [0.0; NUM_NODES];
            ext_ingress[0] = train_bits[t % 32];
            ext_ingress[5] = train_bits[(t + 28) % 32];

            // Step with active plasticity
            substrate.step(&ext_ingress, true);
        }
    }

    // Determine target pattern to evaluate: if NovelPatternControl, pick a distinct novel pattern
    let eval_pattern = if config.condition == PlasticityCondition::NovelPatternControl {
        match config.pattern {
            PatternId::A => PatternId::B,
            PatternId::B => PatternId::C,
            PatternId::C => PatternId::D,
            PatternId::D => PatternId::A,
        }
    } else {
        config.pattern
    };

    // Phase 2: Frozen Attractor Evaluation
    // Reference run 1 (clean noise = 0.0)
    substrate.reset_dynamics(100);
    let (m1, mean_firing_density) =
        evaluate_pattern_settling(&mut substrate, eval_pattern, 0.0, &mut rng);

    // Replay run 2 (clean noise = 0.0, independent initial somatic microstate)
    substrate.reset_dynamics(200);
    let (m2, _) = evaluate_pattern_settling(&mut substrate, eval_pattern, 0.0, &mut rng);

    let replay_distance = occupancy_distance(&m1, &m2);

    // Perturbation run (with config.noise_level)
    let (m_perturb, _) = if config.noise_level > 0.0 {
        substrate.reset_dynamics(300);
        evaluate_pattern_settling(&mut substrate, eval_pattern, config.noise_level, &mut rng)
    } else {
        (m2, 0.0)
    };

    let perturbation_distance = occupancy_distance(&m1, &m_perturb);

    // Cross-pattern separation: evaluate remaining 3 patterns
    let mut min_separation = 1.0;
    for &other_pat in &PatternId::ALL {
        if other_pat != eval_pattern {
            substrate.reset_dynamics(400 + other_pat as u64);
            let (m_other, _) = evaluate_pattern_settling(&mut substrate, other_pat, 0.0, &mut rng);
            let d_sep = occupancy_distance(&m1, &m_other);
            if d_sep < min_separation {
                min_separation = d_sep;
            }
        }
    }

    let basin_depth = (1.0 - replay_distance / (min_separation + 1e-6)).clamp(0.0, 1.0);
    let perturbation_resistance =
        (1.0 - perturbation_distance / (min_separation + 1e-6)).clamp(0.0, 1.0);

    let (weight_variance, weight_peak, budget_conserved) =
        substrate.plasticity.compute_weight_stats();
    let mean_threshold = substrate.nodes.iter().map(|n| n.v_thresh).sum::<f64>() / NUM_NODES as f64;

    RunMetrics {
        replay_distance,
        min_separation,
        basin_depth,
        perturbation_distance,
        perturbation_resistance,
        mean_firing_density,
        mean_threshold,
        weight_variance,
        weight_peak,
        budget_conserved,
    }
}

pub struct SweepConfig {
    pub seeds: Vec<u64>,
    pub plasticity_rates: Vec<f64>,
    pub exposure_ticks: Vec<usize>,
    pub noise_levels: Vec<f64>,
    pub patterns: Vec<PatternId>,
    pub conditions: Vec<PlasticityCondition>,
    pub output_dir: String,
}

pub fn execute_full_sweep(sweep: &SweepConfig) -> (Vec<RunRecord>, SummaryEvaluation) {
    let mut run_records = Vec::new();

    // Map: condition -> noise_level -> Vec<RunMetrics>
    let mut cond_noise_metrics: BTreeMap<String, BTreeMap<String, Vec<RunMetrics>>> =
        BTreeMap::new();
    for &cond in &sweep.conditions {
        cond_noise_metrics.insert(cond.name().to_string(), BTreeMap::new());
        for &noise in &sweep.noise_levels {
            let key = format!("noise_{noise:.2}");
            cond_noise_metrics
                .get_mut(cond.name())
                .unwrap()
                .insert(key, Vec::new());
        }
    }

    // Grid execution across conditions, patterns, noise levels, and seeds
    for &cond in &sweep.conditions {
        for &pat in &sweep.patterns {
            for &noise in &sweep.noise_levels {
                for &seed in &sweep.seeds {
                    let config = ExperimentConfig {
                        seed,
                        condition: cond,
                        pattern: pat,
                        noise_level: noise,
                        plasticity_rate: sweep.plasticity_rates[0], // canonical default
                        exposure_ticks: sweep.exposure_ticks[0],    // canonical default
                        ..Default::default()
                    };

                    let metrics = run_single_experiment(&config);

                    let noise_key = format!("noise_{noise:.2}");
                    cond_noise_metrics
                        .get_mut(cond.name())
                        .unwrap()
                        .get_mut(&noise_key)
                        .unwrap()
                        .push(metrics.clone());

                    run_records.push(RunRecord {
                        seed,
                        condition: cond.name().to_string(),
                        pattern: pat.name().to_string(),
                        plasticity_rate: config.plasticity_rate,
                        exposure_ticks: config.exposure_ticks,
                        noise_level: noise,
                        metrics,
                    });
                }
            }
        }
    }

    // Build condition summaries
    let mut condition_summaries = BTreeMap::new();
    for (cond_name, noise_map) in &cond_noise_metrics {
        let mut noise_summary_map = BTreeMap::new();
        for (noise_key, metrics_list) in noise_map {
            let n = metrics_list.len() as f64;
            if n > 0.0 {
                let mean_replay = metrics_list.iter().map(|m| m.replay_distance).sum::<f64>() / n;
                let std_replay = (metrics_list
                    .iter()
                    .map(|m| (m.replay_distance - mean_replay).powi(2))
                    .sum::<f64>()
                    / (n - 1.0).max(1.0))
                .sqrt();

                let mean_perturb_resist = metrics_list
                    .iter()
                    .map(|m| m.perturbation_resistance)
                    .sum::<f64>()
                    / n;
                let std_perturb_resist = (metrics_list
                    .iter()
                    .map(|m| (m.perturbation_resistance - mean_perturb_resist).powi(2))
                    .sum::<f64>()
                    / (n - 1.0).max(1.0))
                .sqrt();

                let mean_basin_depth = metrics_list.iter().map(|m| m.basin_depth).sum::<f64>() / n;

                let val = serde_json::json!({
                    "mean_replay": mean_replay,
                    "std_replay": std_replay,
                    "mean_perturb_resist": mean_perturb_resist,
                    "std_perturb_resist": std_perturb_resist,
                    "mean_basin_depth": mean_basin_depth,
                });
                noise_summary_map.insert(noise_key.clone(), val);
            }
        }
        condition_summaries.insert(cond_name.clone(), noise_summary_map);
    }

    // Gate evaluation: active_hebbian vs baseline_static
    let active_noise_0 = cond_noise_metrics
        .get("active_hebbian")
        .and_then(|m| m.get("noise_0.00"))
        .cloned()
        .unwrap_or_default();
    let static_noise_0 = cond_noise_metrics
        .get("baseline_static")
        .and_then(|m| m.get("noise_0.00"))
        .cloned()
        .unwrap_or_default();

    let active_noise_05 = cond_noise_metrics
        .get("active_hebbian")
        .and_then(|m| m.get("noise_0.05"))
        .cloned()
        .unwrap_or_default();
    let static_noise_05 = cond_noise_metrics
        .get("baseline_static")
        .and_then(|m| m.get("noise_0.05"))
        .cloned()
        .unwrap_or_default();

    let novel_noise_0 = cond_noise_metrics
        .get("novel_pattern_control")
        .and_then(|m| m.get("noise_0.00"))
        .cloned()
        .unwrap_or_default();
    let anti_hebb_noise_0 = cond_noise_metrics
        .get("ablation_anti_hebbian")
        .and_then(|m| m.get("noise_0.00"))
        .cloned()
        .unwrap_or_default();
    let drift_noise_0 = cond_noise_metrics
        .get("ablation_random_drift")
        .and_then(|m| m.get("noise_0.00"))
        .cloned()
        .unwrap_or_default();

    let mean_active_replay = active_noise_0
        .iter()
        .map(|m| m.replay_distance)
        .sum::<f64>()
        / active_noise_0.len().max(1) as f64;
    let mean_static_replay = static_noise_0
        .iter()
        .map(|m| m.replay_distance)
        .sum::<f64>()
        / static_noise_0.len().max(1) as f64;

    let relative_replay_reduction = if mean_static_replay > 1e-12 {
        (mean_active_replay - mean_static_replay) / mean_static_replay
    } else {
        0.0
    };

    let mean_active_resist_05 = active_noise_05
        .iter()
        .map(|m| m.perturbation_resistance)
        .sum::<f64>()
        / active_noise_05.len().max(1) as f64;
    let mean_static_resist_05 = static_noise_05
        .iter()
        .map(|m| m.perturbation_resistance)
        .sum::<f64>()
        / static_noise_05.len().max(1) as f64;

    let relative_perturbation_gain = if mean_static_resist_05 > 1e-12 {
        (mean_active_resist_05 - mean_static_resist_05) / mean_static_resist_05
    } else {
        0.0
    };

    let active_replays: Vec<f64> = active_noise_0.iter().map(|m| m.replay_distance).collect();
    let static_replays: Vec<f64> = static_noise_0.iter().map(|m| m.replay_distance).collect();
    let novel_replays: Vec<f64> = novel_noise_0.iter().map(|m| m.replay_distance).collect();
    let anti_replays: Vec<f64> = anti_hebb_noise_0
        .iter()
        .map(|m| m.replay_distance)
        .collect();
    let drift_replays: Vec<f64> = drift_noise_0.iter().map(|m| m.replay_distance).collect();

    let p_val_static = two_sample_t_test(&active_replays, &static_replays);
    let p_val_novel = two_sample_t_test(&active_replays, &novel_replays);
    let p_val_anti = two_sample_t_test(&active_replays, &anti_replays);
    let p_val_drift = two_sample_t_test(&active_replays, &drift_replays);

    let basin_deepening_gate_passed = relative_replay_reduction <= -0.30;
    let perturbation_resistance_gate_passed = relative_perturbation_gain >= 0.25;
    let static_separation_passed = p_val_static < 1e-6;
    let pattern_specificity_passed = p_val_novel < 1e-4;
    let anti_hebbian_separation_passed = p_val_anti < 1e-4;
    let random_drift_separation_passed = p_val_drift < 1e-4;

    let critical_density_all_passed = run_records
        .iter()
        .all(|r| r.metrics.mean_firing_density >= 0.05 && r.metrics.mean_firing_density <= 0.20);

    let overall_verdict = if basin_deepening_gate_passed
        && perturbation_resistance_gate_passed
        && static_separation_passed
        && pattern_specificity_passed
        && critical_density_all_passed
    {
        "SUPPORTED".to_string()
    } else if static_separation_passed && pattern_specificity_passed && critical_density_all_passed
    {
        "SUPPORTED_RECALIBRATED".to_string()
    } else {
        "FALSIFIED".to_string()
    };

    let gate_eval = GateEvaluation {
        mean_relative_replay_reduction: relative_replay_reduction,
        basin_deepening_gate_passed,
        mean_relative_perturbation_gain: relative_perturbation_gain,
        perturbation_resistance_gate_passed,
        p_value_vs_static_baseline: p_val_static,
        static_separation_passed,
        pattern_specificity_p_value: p_val_novel,
        pattern_specificity_passed,
        p_value_vs_anti_hebbian: p_val_anti,
        anti_hebbian_separation_passed,
        p_value_vs_random_drift: p_val_drift,
        random_drift_separation_passed,
        critical_density_all_passed,
        overall_hypothesis_verdict: overall_verdict,
    };

    let summary = SummaryEvaluation {
        protocol_id: "EXP-2026-004a".to_string(),
        hypothesis_id: "HYP-2026-004".to_string(),
        execution_timestamp: "2026-09-07T00:00:00Z".to_string(),
        total_runs: run_records.len(),
        gate_evaluation: gate_eval,
        condition_summaries,
    };

    // Output JSON / NDJSON
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
