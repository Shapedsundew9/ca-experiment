//! Binary entry point for EXP-2026-010a: Pushdown Memory and Context-Free Dyck Language Recognition.

use std::env;
use std::path::PathBuf;
use std::time::Instant;

use rust_3::experiments::exp_2026_010a_mva_pushdown_memory::*;

fn print_help() {
    println!(
        r#"EXP-2026-010a: Pushdown Memory and Context-Free Dyck Language Recognition
Milestone 3.1: Pushdown Memory (Dyck Languages) (Tier 3: Hierarchical Structure & Working Memory)

USAGE:
    exp_2026_010a_mva_pushdown_memory [OPTIONS]

OPTIONS:
    --help                    Display this help menu
    --output-dir <DIR>        Output directory for telemetry (default: data/telemetry/EXP-2026-010a)
    --condition <COND>        Filter condition (active_pda, baseline_finite_state, ablation_unshielded_stack, ablation_fixed_theta, or "all")
    --task <TASK>             Filter language task (dyck_1, dyck_2, depth_generalization, or "all")
    --noise <EPS>             Filter noise rate (0.00, 0.01, 0.05, or "all")
    --seq-length <L>          Filter sequence length (8, 16, 24, 32, or "all")
    --seeds <N>               Number of seeds per condition (default: 30)
    --sequences-per-run <M>   Number of sequences per run (default: 50)
"#
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut output_dir = PathBuf::from("data/telemetry/EXP-2026-010a");
    let mut num_seeds = 30;
    let mut m_sequences = 50;
    let mut filter_cond: Option<String> = None;
    let mut filter_task: Option<String> = None;
    let mut filter_noise: Option<String> = None;
    let mut filter_len: Option<String> = None;

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
            "--condition" => {
                if i + 1 < args.len() {
                    filter_cond = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--task" => {
                if i + 1 < args.len() {
                    filter_task = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--noise" | "--noise-rate" => {
                if i + 1 < args.len() {
                    filter_noise = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--seq-length" | "--len" => {
                if i + 1 < args.len() {
                    filter_len = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--seeds" => {
                if i + 1 < args.len() {
                    num_seeds = args[i + 1].parse().unwrap_or(30);
                    i += 1;
                }
            }
            "--sequences-per-run" => {
                if i + 1 < args.len() {
                    m_sequences = args[i + 1].parse().unwrap_or(50);
                    i += 1;
                }
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                print_help();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let conditions: Vec<Condition> = match filter_cond.as_deref() {
        Some("all") | None => Condition::ALL.to_vec(),
        Some(s) => match s.parse::<Condition>() {
            Ok(c) => vec![c],
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        },
    };

    let tasks: Vec<AutomatonTask> = match filter_task.as_deref() {
        Some("all") | None => AutomatonTask::ALL.to_vec(),
        Some(s) => match s.parse::<AutomatonTask>() {
            Ok(t) => vec![t],
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        },
    };

    let noise_rates: Vec<f64> = match filter_noise.as_deref() {
        Some("all") | None => vec![0.00, 0.01, 0.05],
        Some(s) => vec![s.parse::<f64>().expect("Invalid noise rate")],
    };

    let seq_lengths: Vec<usize> = match filter_len.as_deref() {
        Some("all") | None => vec![8, 16, 24, 32],
        Some(s) => vec![s.parse::<usize>().expect("Invalid sequence length")],
    };

    let start_time = Instant::now();

    let manifest = execute_factorial_sweep(
        &output_dir,
        &conditions,
        &tasks,
        &noise_rates,
        &seq_lengths,
        num_seeds,
        m_sequences,
    );

    let duration = start_time.elapsed();
    println!(
        "\n======================================================\n\
         EXP-2026-010a Sweep Complete\n\
         Total Runs: {}\n\
         Total Sequences: {}\n\
         Wall-Clock Duration: {:.2?}\n\
         Overall Verdict: {}\n\
         ======================================================",
        manifest.total_runs, manifest.total_sequences, duration, manifest.overall_verdict
    );

    for (name, gate) in &manifest.gates {
        println!(
            "  - {}: observed={:.4}, target={:.4} => [{}]",
            name,
            gate.observed,
            gate.target,
            if gate.passed { "PASS" } else { "FAIL" }
        );
    }
}
