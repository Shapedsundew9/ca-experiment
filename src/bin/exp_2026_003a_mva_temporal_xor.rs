//! Binary CLI Entry Point for EXP-2026-003a Temporal XOR Benchmark.

use rust_3::experiments::exp_2026_003a_mva_temporal_xor::config::ExperimentalCondition;
use rust_3::experiments::exp_2026_003a_mva_temporal_xor::{SweepConfig, execute_full_sweep};
use std::env;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut out_dir = PathBuf::from("data/telemetry/EXP-2026-003a");
    let mut num_seeds = 30usize;
    let mut delays: Vec<usize> = vec![3, 5, 7, 10];
    let mut densities: Vec<f64> = vec![0.10, 0.20, 0.50];
    let mut train_ticks = 2000usize;
    let mut test_ticks = 1000usize;
    let mut washout_ticks = 200usize;
    let mut ridge_alpha = 0.01f64;

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
            "--delays" => {
                delays = args[i + 1]
                    .split(',')
                    .map(|s| s.trim().parse().expect("Invalid delay in list"))
                    .collect();
                i += 2;
            }
            "--densities" => {
                densities = args[i + 1]
                    .split(',')
                    .map(|s| s.trim().parse().expect("Invalid density in list"))
                    .collect();
                i += 2;
            }
            "--train-ticks" => {
                train_ticks = args[i + 1].parse().expect("Invalid train ticks");
                i += 2;
            }
            "--test-ticks" => {
                test_ticks = args[i + 1].parse().expect("Invalid test ticks");
                i += 2;
            }
            "--washout-ticks" => {
                washout_ticks = args[i + 1].parse().expect("Invalid washout ticks");
                i += 2;
            }
            "--ridge-alpha" => {
                ridge_alpha = args[i + 1].parse().expect("Invalid ridge alpha");
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    println!("============================================================");
    println!("  EXP-2026-003a Factorial Sweep: Non-Linear Temporal XOR    ");
    println!("============================================================");
    println!("Output directory: {}", out_dir.display());

    let seeds: Vec<u64> = (42..(42 + num_seeds as u64)).collect();
    let conditions = ExperimentalCondition::ALL.to_vec();

    let total_runs = conditions.len() * delays.len() * densities.len() * seeds.len();
    println!(
        "Planned factor space: {} conditions x {} delays x {} densities x {} seeds = {} runs",
        conditions.len(),
        delays.len(),
        densities.len(),
        seeds.len(),
        total_runs
    );

    let sweep = SweepConfig {
        seeds,
        delays,
        densities,
        conditions,
        ridge_alpha,
        washout_ticks,
        train_ticks,
        test_ticks,
        output_dir: out_dir.display().to_string(),
    };

    let t0 = Instant::now();
    let (_records, summary) = execute_full_sweep(&sweep);
    let elapsed = t0.elapsed();

    // Also mirror to results/exp_2026_003a/ if distinct
    let mirror_dir = PathBuf::from("results/exp_2026_003a");
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
        "Tau=5 Active XOR Mean Accuracy: {:.4} (95% CI: [{:.4}, {:.4}])",
        summary.gate_evaluation.tau_5_active_xor_mean_accuracy,
        summary.gate_evaluation.tau_5_active_xor_ci_95[0],
        summary.gate_evaluation.tau_5_active_xor_ci_95[1],
    );
    println!(
        "P-value vs Memoryless: {:e} (Passed: {})",
        summary.gate_evaluation.p_value_vs_memoryless,
        summary.gate_evaluation.memoryless_separation_passed
    );
    println!(
        "P-value vs Direct Linear: {:e} (Passed: {})",
        summary.gate_evaluation.p_value_vs_direct_linear,
        summary.gate_evaluation.direct_linear_separation_passed
    );
    println!(
        "P-value vs Static Threshold: {:e} (Passed: {})",
        summary.gate_evaluation.p_value_vs_static_thresh,
        summary.gate_evaluation.static_thresh_advantage_passed
    );
    println!(
        "Overall Hypothesis Verdict: {}",
        summary.gate_evaluation.overall_hypothesis_verdict
    );
    println!("============================================================");
}
