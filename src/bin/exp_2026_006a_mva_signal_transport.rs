//! Binary entry point for EXP-2026-006a: Directed Regenerative Transmission Tracks and Branching Fan-Out.

use std::env;
use std::path::PathBuf;
use std::time::Instant;

use rust_3::experiments::exp_2026_006a_mva_signal_transport::*;

fn print_help() {
    println!(
        r#"EXP-2026-006a: Directed Regenerative Transmission Tracks and Branching Fan-Out

USAGE:
    exp_2026_006a_mva_signal_transport [OPTIONS]

OPTIONS:
    --help                 Display this help menu
    --output-dir <DIR>     Output directory for telemetry (default: data/telemetry/EXP-2026-006a)
    --seeds <N>            Number of seeds to evaluate per condition (default: 30)
    --t-warmup <TICKS>     Warmup settling window ticks (default: 50)
    --t-eval <TICKS>       Active evaluation window ticks (default: 200)
    --distance <D>         Filter/override distance D (cells)
    --jitter <J>           Filter/override jitter Delta L (cells)
    --noise-rate <EPS>     Filter/override background noise rate
    --condition <COND>     Filter/override condition (active_regenerative, baseline_passive, etc.)
    --pattern <PAT>        Filter/override signal pattern (single_impulse, alternating_clock, etc.)
    --fast-check           Execute fast sanity check (1 seed, small grid)
"#
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut output_dir = PathBuf::from("data/telemetry/EXP-2026-006a");
    let mut num_seeds = 30;
    let mut t_warmup: Option<usize> = None;
    let mut t_eval: Option<usize> = None;
    let mut filter_dist: Option<usize> = None;
    let mut filter_jitter: Option<usize> = None;
    let mut filter_noise: Option<f64> = None;
    let mut filter_cond: Option<String> = None;
    let mut filter_pattern: Option<String> = None;
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
            "--t-warmup" => {
                if i + 1 < args.len() {
                    t_warmup = Some(args[i + 1].parse().unwrap_or(50));
                    i += 1;
                }
            }
            "--t-eval" => {
                if i + 1 < args.len() {
                    t_eval = Some(args[i + 1].parse().unwrap_or(200));
                    i += 1;
                }
            }
            "--distance" => {
                if i + 1 < args.len() {
                    filter_dist = Some(args[i + 1].parse().unwrap_or(30));
                    i += 1;
                }
            }
            "--jitter" => {
                if i + 1 < args.len() {
                    filter_jitter = Some(args[i + 1].parse().unwrap_or(0));
                    i += 1;
                }
            }
            "--noise-rate" => {
                if i + 1 < args.len() {
                    filter_noise = Some(args[i + 1].parse().unwrap_or(0.0));
                    i += 1;
                }
            }
            "--condition" => {
                if i + 1 < args.len() {
                    filter_cond = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--pattern" => {
                if i + 1 < args.len() {
                    filter_pattern = Some(args[i + 1].clone());
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
    }

    println!("================================================================================");
    println!("EXP-2026-006a: Directed Regenerative Transmission Tracks & Branching Fan-Out");
    println!("Milestone 1.1: Signal Transport & Fan-Out (Tier 1 Inception)");
    println!("Output Directory: {}", output_dir.display());
    println!("Seeds per cell:   {num_seeds}");
    println!("================================================================================");

    let start_time = Instant::now();
    let mut plan = build_experiment_plan(num_seeds);

    // Apply any CLI overrides or filters
    if let Some(tw) = t_warmup {
        for cfg in plan.iter_mut() {
            cfg.t_warmup = tw;
        }
    }
    if let Some(te) = t_eval {
        for cfg in plan.iter_mut() {
            cfg.t_eval = te;
        }
    }

    if let Some(d) = filter_dist {
        plan.retain(|c| c.distance == d);
    }
    if let Some(j) = filter_jitter {
        plan.retain(|c| c.jitter == j);
    }
    if let Some(n) = filter_noise {
        plan.retain(|c| (c.noise_rate - n).abs() < 1e-5);
    }
    if let Some(ref cond_str) = filter_cond {
        plan.retain(|c| c.condition.as_str() == cond_str);
    }
    if let Some(ref pat_str) = filter_pattern {
        plan.retain(|c| c.pattern.as_str() == pat_str);
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
                println!("  - {:30}: {}", gate, verdict);
            }
            println!("\nStatistical Hypothesis Tests:");
            for (test_id, stat) in &summary.statistical_tests {
                println!(
                    "  - {:45}: t={:7.2}, p={:.2e}, d={:6.2} (sig: {})",
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
