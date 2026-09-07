//! Binary entry point for EXP-2026-011a: Associative Key-Value Retrieval ("Needle in a Haystack").

use std::env;
use std::path::PathBuf;
use std::time::Instant;

use rust_3::experiments::exp_2026_011a_mva_associative_retrieval::*;

fn print_help() {
    println!(
        r#"EXP-2026-011a: Associative Key-Value Retrieval ("Needle in a Haystack")
Milestone 3.2: Associative Key-Value Retrieval (Tier 3: Hierarchical Structure & Working Memory)

USAGE:
    exp_2026_011a_mva_associative_retrieval [OPTIONS]

OPTIONS:
    --help                    Display this help menu
    --output-dir <DIR>        Output directory for telemetry (default: data/telemetry/EXP-2026-011a)
    --condition <COND>        Filter condition (active_associative, baseline_unindexed, ablation_unshielded_retrieval, ablation_fixed_theta, or "all")
    --length <L>              Filter sequence length (64, 256, 512, or "all")
    --context <L>             Alias for --length
    --noise <EPS>             Filter noise rate (0.00, 0.01, 0.05, or "all")
    --seeds <N>               Number of seeds per condition (default: 30)
    --sequences-per-run <M>   Number of sequences per run (default: 50)
"#
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut output_dir = PathBuf::from("data/telemetry/EXP-2026-011a");
    let mut num_seeds = 30;
    let mut m_sequences = 50;
    let mut filter_cond: Option<String> = None;
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
            "--noise" | "--noise-rate" => {
                if i + 1 < args.len() {
                    filter_noise = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--length" | "--len" | "--context" => {
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
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        },
    };

    let noise_rates: Vec<f64> = match filter_noise.as_deref() {
        Some("all") | None => vec![0.00, 0.01, 0.05],
        Some("0.00") | Some("0") => vec![0.00],
        Some("0.01") => vec![0.01],
        Some("0.05") => vec![0.05],
        Some(s) => match s.parse::<f64>() {
            Ok(v) => vec![v],
            Err(_) => {
                eprintln!("Invalid noise rate: {s}");
                std::process::exit(1);
            }
        },
    };

    let seq_lengths: Vec<usize> = match filter_len.as_deref() {
        Some("all") | None => vec![64, 256, 512],
        Some("64") => vec![64],
        Some("256") => vec![256],
        Some("512") => vec![512],
        Some(s) => match s.parse::<usize>() {
            Ok(v) => vec![v],
            Err(_) => {
                eprintln!("Invalid sequence length: {s}");
                std::process::exit(1);
            }
        },
    };

    let seeds: Vec<u64> = (1..=num_seeds as u64).collect();

    println!("================================================================================");
    println!("EXP-2026-011a: Associative Key-Value Retrieval Execution");
    println!(
        "Conditions:    {:?}",
        conditions.iter().map(|c| c.as_str()).collect::<Vec<_>>()
    );
    println!("Noise rates:   {:?}", noise_rates);
    println!("Lengths:       {:?}", seq_lengths);
    println!("Seeds:         {} (1..={})", seeds.len(), num_seeds);
    println!("Sequences/run: {}", m_sequences);
    println!("Output dir:    {}", output_dir.display());
    println!("================================================================================");

    let start_time = Instant::now();
    let manifest = execute_factorial_sweep(
        &conditions,
        &noise_rates,
        &seq_lengths,
        &seeds,
        m_sequences,
        &output_dir,
    );
    let duration = start_time.elapsed();

    println!("================================================================================");
    println!("Execution completed in {:.2?}", duration);
    println!("Overall Verdict: {}", manifest.overall_verdict);
    println!(
        "Gate 1 (Retrieval Accuracy >= 0.990):   {} (Observed: {:.4})",
        if manifest.gate_evaluations.gate_1_retrieval_accuracy.passed {
            "PASS"
        } else {
            "FAIL"
        },
        manifest.gate_evaluations.gate_1_retrieval_accuracy.value
    );
    println!(
        "Gate 2 (Long Horizon Retention >= 0.990): {} (Observed: {:.4})",
        if manifest
            .gate_evaluations
            .gate_2_long_horizon_retention
            .passed
        {
            "PASS"
        } else {
            "FAIL"
        },
        manifest
            .gate_evaluations
            .gate_2_long_horizon_retention
            .value
    );
    println!(
        "Gate 3 (Distractor Immunity <= 0.010):   {} (Observed: {:.6})",
        if manifest.gate_evaluations.gate_3_distractor_immunity.passed {
            "PASS"
        } else {
            "FAIL"
        },
        manifest.gate_evaluations.gate_3_distractor_immunity.value
    );
    println!(
        "Gate 4 (Noise Resilience eps=0.05):     {} (Acc: {:.4}, BER: {:.4})",
        if manifest.gate_evaluations.gate_4_noise_resilience.passed {
            "PASS"
        } else {
            "FAIL"
        },
        manifest
            .gate_evaluations
            .gate_4_noise_resilience
            .accuracy_at_005,
        manifest.gate_evaluations.gate_4_noise_resilience.ber_at_005
    );
    println!(
        "Gate 5 (Associative Advantage p<1e-6, d>=2.0): {} (p={:.2e}, d={:.2})",
        if manifest
            .gate_evaluations
            .gate_5_associative_advantage
            .passed
        {
            "PASS"
        } else {
            "FAIL"
        },
        manifest
            .gate_evaluations
            .gate_5_associative_advantage
            .p_value,
        manifest
            .gate_evaluations
            .gate_5_associative_advantage
            .cohens_d
    );
    println!(
        "Gate 6 (Homeostatic Envelope >= 0.99):   {} (Pct: {:.4})",
        if manifest
            .gate_evaluations
            .gate_6_homeostatic_stability
            .passed
        {
            "PASS"
        } else {
            "FAIL"
        },
        manifest
            .gate_evaluations
            .gate_6_homeostatic_stability
            .within_envelope_pct
    );
    println!("================================================================================");
}
