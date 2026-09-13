//! Binary entry point for EXP-2026-005a: Continual Multi-Pattern Learning & Metaplasticity.

use std::env;
use std::path::PathBuf;
use std::time::Instant;

use rust_3::experiments::exp_2026_005a_mva_continual_learning::*;

fn print_help() {
    println!(
        r#"EXP-2026-005a: Lifelong Adaptation and Continual Multi-Pattern Learning

USAGE:
    exp_2026_005a_mva_continual_learning [OPTIONS]

OPTIONS:
    --help                 Display this help menu
    --output-dir <DIR>     Output directory for telemetry (default: data/telemetry/EXP-2026-005a)
    --seeds <N>            Number of seeds to evaluate (default: 30)
    --t-train <TICKS>      Single t_train override (default: evaluates both 500 and 1000)
    --fast-check           Fast smoke test (1 seed, 500 ticks)
    --threads <N>          Worker threads (default: available parallelism)
"#
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut output_dir = PathBuf::from("data/telemetry/EXP-2026-005a");
    let mut seeds = 30;
    let mut t_train_values = vec![500, 1000];
    let mut fast_check = false;
    let mut num_threads = std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4);

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
                    seeds = args[i + 1].parse().unwrap_or(30);
                    i += 1;
                }
            }
            "--t-train" => {
                if i + 1 < args.len() {
                    let val: usize = args[i + 1].parse().unwrap_or(1000);
                    t_train_values = vec![val];
                    i += 1;
                }
            }
            "--fast-check" => {
                fast_check = true;
            }
            "--threads" => {
                if i + 1 < args.len() {
                    num_threads = args[i + 1].parse().unwrap_or(num_threads);
                    i += 1;
                }
            }
            other => {
                eprintln!("Unknown argument: {}", other);
                print_help();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    if fast_check {
        seeds = 1;
        t_train_values = vec![500];
    }

    println!("===============================================================");
    println!("  EXP-2026-005a: Continual Multi-Pattern Learning Evaluation  ");
    println!("===============================================================");
    println!("  Output Directory : {}", output_dir.display());
    println!("  Seeds per Cell   : {}", seeds);
    println!("  T_train settings : {:?}", t_train_values);
    println!("  Worker Threads   : {}", num_threads);

    let configs = generate_factorial_configs(seeds, &t_train_values);
    let total_runs = configs.len();
    println!("  Total Runs       : {}", total_runs);
    println!("---------------------------------------------------------------");

    let start = Instant::now();
    let (results, manifest) = run_factorial_sweep(configs, num_threads);
    let elapsed = start.elapsed();

    println!("Completed {} runs in {:.2?}", total_runs, elapsed);
    println!("---------------------------------------------------------------");
    println!("Results Summary:");
    for (key, summary) in &manifest.conditions {
        println!(
            "  [{:<24}] BWT: {:+.4} +/- {:.4} | AR: {:.4} +/- {:.4} | D_sep: {:.4} | Forgetting: {:.4}",
            key,
            summary.bwt_mean,
            summary.bwt_std,
            summary.ar_mean,
            summary.ar_std,
            summary.d_sep_mean,
            summary.forgetting_mean
        );
    }
    println!("---------------------------------------------------------------");
    println!("Statistical Tests:");
    for (name, test) in &manifest.statistical_tests {
        println!(
            "  [{}] t = {:.2}, p = {:.2e}, Cohen's d = {:.2} (sig: {})",
            name, test.t_stat, test.p_value, test.cohen_d, test.significant
        );
    }
    println!("---------------------------------------------------------------");
    println!("Pre-registered Gate Verdicts:");
    for (gate, verd) in &manifest.gate_verdicts {
        println!("  {:<22} : {}", gate, verd);
    }
    println!("Overall Verdict: {}", manifest.overall_verdict);
    println!("---------------------------------------------------------------");

    if let Err(e) = emit_telemetry(&output_dir, &results, &manifest) {
        eprintln!("Failed to write telemetry: {}", e);
        std::process::exit(1);
    }
    println!("Telemetry successfully written to {}", output_dir.display());
}
