//! Binary CLI Entry Point for EXP-2026-004a Hebbian Plasticity Experiment.

use rust_3::experiments::exp_2026_004a_mva_hebbian_plasticity::config::{
    PatternId, PlasticityCondition,
};
use rust_3::experiments::exp_2026_004a_mva_hebbian_plasticity::{SweepConfig, execute_full_sweep};
use std::env;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut out_dir = PathBuf::from("data/telemetry/EXP-2026-004a");
    let mut num_seeds = 30usize;
    let mut plasticity_rates = vec![0.01f64];
    let mut exposure_ticks = vec![1000usize];
    let mut noise_levels = vec![0.00f64, 0.01f64, 0.05f64];
    let mut patterns = PatternId::ALL.to_vec();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output-dir" | "--out-dir" => {
                out_dir = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--seeds" => {
                num_seeds = args[i + 1].parse().expect("Invalid seeds");
                i += 2;
            }
            "--plasticity-rates" => {
                plasticity_rates = args[i + 1]
                    .split(',')
                    .map(|s| s.trim().parse().expect("Invalid rate in list"))
                    .collect();
                i += 2;
            }
            "--exposure-ticks" => {
                exposure_ticks = args[i + 1]
                    .split(',')
                    .map(|s| s.trim().parse().expect("Invalid exposure in list"))
                    .collect();
                i += 2;
            }
            "--noise-levels" => {
                noise_levels = args[i + 1]
                    .split(',')
                    .map(|s| s.trim().parse().expect("Invalid noise in list"))
                    .collect();
                i += 2;
            }
            "--patterns" => {
                patterns = args[i + 1]
                    .split(',')
                    .map(|s| s.trim().parse().expect("Invalid pattern in list"))
                    .collect();
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    println!("============================================================");
    println!("  EXP-2026-004a Factorial Sweep: Hebbian Plasticity & Basins ");
    println!("============================================================");
    println!("Output directory: {}", out_dir.display());

    let seeds: Vec<u64> = (42..(42 + num_seeds as u64)).collect();
    let conditions = PlasticityCondition::ALL.to_vec();

    let total_runs = conditions.len() * patterns.len() * noise_levels.len() * seeds.len();
    println!(
        "Planned factor space: {} conditions x {} patterns x {} noise levels x {} seeds = {} runs",
        conditions.len(),
        patterns.len(),
        noise_levels.len(),
        seeds.len(),
        total_runs
    );

    let sweep = SweepConfig {
        seeds,
        plasticity_rates,
        exposure_ticks,
        noise_levels,
        patterns,
        conditions,
        output_dir: out_dir.display().to_string(),
    };

    let t0 = Instant::now();
    let (_records, summary) = execute_full_sweep(&sweep);
    let elapsed = t0.elapsed();

    // Also mirror to results/exp_2026_004a/ if distinct
    let mirror_dir = PathBuf::from("results/exp_2026_004a");
    if mirror_dir != out_dir {
        let _ = std::fs::create_dir_all(&mirror_dir);
        let _ = std::fs::copy(
            out_dir.join("run_results.jsonl"),
            mirror_dir.join("run_results.jsonl"),
        );
        let _ = std::fs::copy(
            out_dir.join("summary_evaluation.json"),
            mirror_dir.join("summary_evaluation.json"),
        );
    }

    println!("------------------------------------------------------------");
    println!("Sweep completed in {:.2?}", elapsed);
    println!("Total runs evaluated: {}", summary.total_runs);
    println!(
        "Relative Replay Reduction: {:.4} (Passed: {})",
        summary.gate_evaluation.mean_relative_replay_reduction,
        summary.gate_evaluation.basin_deepening_gate_passed
    );
    println!(
        "Relative Perturbation Gain (eps=0.05): {:.4} (Passed: {})",
        summary.gate_evaluation.mean_relative_perturbation_gain,
        summary.gate_evaluation.perturbation_resistance_gate_passed
    );
    println!(
        "P-value vs Static Baseline: {:e} (Passed: {})",
        summary.gate_evaluation.p_value_vs_static_baseline,
        summary.gate_evaluation.static_separation_passed
    );
    println!(
        "Pattern Specificity P-value: {:e} (Passed: {})",
        summary.gate_evaluation.pattern_specificity_p_value,
        summary.gate_evaluation.pattern_specificity_passed
    );
    println!(
        "P-value vs Anti-Hebbian: {:e} (Passed: {})",
        summary.gate_evaluation.p_value_vs_anti_hebbian,
        summary.gate_evaluation.anti_hebbian_separation_passed
    );
    println!(
        "Critical Density Invariant Passed: {}",
        summary.gate_evaluation.critical_density_all_passed
    );
    println!(
        "Overall Hypothesis Verdict: {}",
        summary.gate_evaluation.overall_hypothesis_verdict
    );
    println!("============================================================");
}
