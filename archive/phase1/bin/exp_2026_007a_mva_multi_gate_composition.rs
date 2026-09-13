//! Binary entry point for EXP-2026-007a: Cascaded Boolean Logic Gate Composition and Planar Wire Crossing.

use std::env;
use std::path::PathBuf;
use std::time::Instant;

use rust_3::experiments::exp_2026_007a_mva_multi_gate_composition::*;

fn print_help() {
    println!(
        r#"EXP-2026-007a: Cascaded Boolean Logic Gate Composition and Planar Wire Crossing
Milestone 1.2: Multi-Gate Composition & Wire Crossing (Tier 1: Spatial Routing & Compositionality)

USAGE:
    exp_2026_007a_mva_multi_gate_composition [OPTIONS]

OPTIONS:
    --help                    Display this help menu
    --output-dir <DIR>        Output directory for telemetry (default: data/telemetry/EXP-2026-007a)
    --seeds <N>               Number of seeds to evaluate per condition (default: 30)
    --epochs-per-state <N>    Number of repetition epochs per Boolean state (default: 50)
    --t-warmup <TICKS>        Warmup settling window ticks (default: 50)
    --condition <COND>        Filter condition (active_composed, baseline_uncompensated_delay, etc.)
    --noise-rate <EPS>        Filter background noise rate (0.00, 0.01, 0.05)
    --fast-check              Execute quick smoke test (1 seed, 5 epochs, 20 warmup)
"#
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut output_dir = PathBuf::from("data/telemetry/EXP-2026-007a");
    let mut num_seeds = 30;
    let mut epochs_per_state: Option<usize> = None;
    let mut t_warmup: Option<usize> = None;
    let mut filter_cond: Option<String> = None;
    let mut filter_noise: Option<f64> = None;
    let mut fast_check = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_help();
                return;
            }
            "--output-dir" => {
                if i + 1 < args.len() {
                    output_dir = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            "--seeds" => {
                if i + 1 < args.len() {
                    num_seeds = args[i + 1].parse().unwrap_or(30);
                    i += 1;
                }
            }
            "--epochs-per-state" => {
                if i + 1 < args.len() {
                    epochs_per_state = Some(args[i + 1].parse().unwrap_or(50));
                    i += 1;
                }
            }
            "--t-warmup" => {
                if i + 1 < args.len() {
                    t_warmup = Some(args[i + 1].parse().unwrap_or(50));
                    i += 1;
                }
            }
            "--condition" => {
                if i + 1 < args.len() {
                    filter_cond = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--noise-rate" => {
                if i + 1 < args.len() {
                    filter_noise = Some(args[i + 1].parse().unwrap_or(0.0));
                    i += 1;
                }
            }
            "--fast-check" => {
                fast_check = true;
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
            }
        }
        i += 1;
    }

    if fast_check {
        num_seeds = 1;
        if epochs_per_state.is_none() {
            epochs_per_state = Some(5);
        }
        if t_warmup.is_none() {
            t_warmup = Some(20);
        }
    }

    println!("================================================================================");
    println!("EXP-2026-007a: Cascaded Boolean Logic Gate Composition & Planar Wire Crossing");
    println!("Milestone 1.2: Multi-Gate Composition & Wire Crossing (1-Bit Full Adder)");
    println!("Output Directory: {}", output_dir.display());
    println!("Seeds per cell:   {num_seeds}");
    println!("================================================================================");

    let start_time = Instant::now();
    let mut plan = build_experiment_plan(num_seeds);

    // Apply any CLI overrides or filters
    if let Some(eps) = epochs_per_state {
        for cfg in plan.iter_mut() {
            cfg.epochs_per_state = eps;
        }
    }
    if let Some(tw) = t_warmup {
        for cfg in plan.iter_mut() {
            cfg.t_warmup = tw;
        }
    }
    if let Some(ref cond_str) = filter_cond {
        plan.retain(|c| c.condition.as_str() == cond_str);
    }
    if let Some(n) = filter_noise {
        plan.retain(|c| (c.noise_rate - n).abs() < 1e-5);
    }

    println!("Scheduled runs:   {}", plan.len());

    match run_experiment_sweep(&plan, &output_dir) {
        Ok(summary) => {
            let duration = start_time.elapsed();
            println!("\nSweep completed successfully in {:.2?}", duration);
            println!("Total runs evaluated: {}", summary.total_runs);
            println!("Overall Hypothesis Verdict: {}", summary.overall_verdict);
            println!("\nPre-Registered Gate Verdicts:");
            for (gate, verdict) in &summary.gate_verdicts {
                println!("  - {:35}: {}", gate, verdict);
            }
            println!("\nStatistical Hypothesis Tests:");
            for (test_id, stat) in &summary.statistical_tests {
                println!(
                    "  - {:40}: t={:7.2}, p={:.2e}, d={:6.2} (sig: {})",
                    test_id, stat.t_stat, stat.p_value, stat.cohen_d, stat.significant
                );
            }
        }
        Err(e) => {
            eprintln!("\nExecution failed: {e}");
            std::process::exit(1);
        }
    }
}
