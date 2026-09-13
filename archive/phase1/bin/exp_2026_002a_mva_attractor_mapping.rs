//! Binary CLI Entry Point for EXP-2026-002a Attractor Mapping Experiment.

use rust_3::experiments::exp_2026_002a_mva_attractor_mapping::runner::{
    RunResult, execute_single_run, reduce_sweep_metrics, save_ndjson_telemetry,
};
use rust_3::experiments::exp_2026_002a_mva_attractor_mapping::types::{
    ExperimentConfig, HomeostaticMode, InputPatternId,
};
use std::env;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut out_dir = PathBuf::from("data/telemetry/EXP-2026-002a");
    let mut seeds = 30usize;
    let mut t_drive = 32usize;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output-dir" | "--out-dir" => {
                out_dir = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--seeds" => {
                seeds = args[i + 1].parse().expect("Invalid seeds");
                i += 2;
            }
            "--t-drive" => {
                t_drive = args[i + 1].parse().expect("Invalid t-drive");
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    println!("============================================================");
    println!("  EXP-2026-002a Factorial Sweep: Attractor Mapping & Separation");
    println!("============================================================");
    println!("Output directory: {}", out_dir.display());

    let patterns = InputPatternId::ALL;
    let relax_horizons = [200usize, 500usize, 1000usize];
    let noise_levels = [0.00, 0.01, 0.05];
    let modes = [HomeostaticMode::Active, HomeostaticMode::Fixed];

    let total_runs =
        patterns.len() * relax_horizons.len() * noise_levels.len() * modes.len() * seeds;
    println!("Total planned runs: {}", total_runs);

    let t0 = Instant::now();
    let mut results: Vec<RunResult> = Vec::with_capacity(total_runs);
    let mut completed = 0usize;

    for &pattern in &patterns {
        for &t_relax in &relax_horizons {
            for &noise_rate in &noise_levels {
                for &mode in &modes {
                    for s in 1..=(seeds as u64) {
                        let config = ExperimentConfig {
                            pattern,
                            t_drive,
                            t_relax,
                            noise_rate,
                            mode,
                            seed: s,
                            n_ref: 2,
                            leak: 0.05,
                            r_target: 0.12,
                            eta: if mode == HomeostaticMode::Active {
                                0.02
                            } else {
                                0.00
                            },
                            v_init: 1.15,
                            v_min: 0.5,
                            v_max: 3.5,
                        };

                        let res = execute_single_run(config);
                        results.push(res);
                        completed += 1;

                        if completed.is_multiple_of(300) || completed == total_runs {
                            let pct = (completed as f64 / total_runs as f64) * 100.0;
                            println!(
                                "Progress: {:4}/{} ({:5.1}%) | Elapsed: {:.1?} | Pattern: {}",
                                completed,
                                total_runs,
                                pct,
                                t0.elapsed(),
                                pattern.name()
                            );
                        }
                    }
                }
            }
        }
    }

    println!("[+] Saving NDJSON telemetry...");
    save_ndjson_telemetry(&results, &out_dir).expect("Failed to save NDJSON telemetry");

    println!("[+] Reducing sweep metrics...");
    let metrics = reduce_sweep_metrics(&results, &out_dir);

    println!("\n============================================================");
    println!("  PRE-REGISTERED PASS/FAIL GATE EVALUATION (CRITERIA 1 - 5)");
    println!("============================================================");
    println!(
        "CRITERION 1 (Attractor Separation >= 0.20):     {} (Mean D = {:.4})",
        if metrics.criterion1_separation_pass {
            "PASS"
        } else {
            "FAIL"
        },
        metrics.mean_inter_pattern_distance
    );
    println!(
        "CRITERION 2 (Basin Consistency < 0.05):         {} (Mean Intra D = {:.4})",
        if metrics.criterion2_basin_consistency_pass {
            "PASS"
        } else {
            "FAIL"
        },
        metrics.mean_intra_pattern_distance
    );
    println!(
        "CRITERION 3 (Noise Robustness < 0.08 at eps=0.01): {} (Mean Perturbed D = {:.4})",
        if metrics.criterion3_noise_robustness_pass {
            "PASS"
        } else {
            "FAIL"
        },
        metrics.mean_noise_perturbed_distance
    );
    println!(
        "CRITERION 4 (Limit Cycle Conv >= 95%):          {} (Cycle Rate = {:.1}%)",
        if metrics.criterion4_limit_cycle_pass {
            "PASS"
        } else {
            "FAIL"
        },
        metrics.cycle_detection_rate_pct
    );
    println!(
        "CRITERION 5 (Homeostatic Advantage SNR >= 2x):  {} (Active SNR={:.2}, Fixed SNR={:.2})",
        if metrics.criterion5_homeostatic_advantage_pass {
            "PASS"
        } else {
            "FAIL"
        },
        metrics.snr_active,
        metrics.snr_fixed
    );
    println!("------------------------------------------------------------");

    if metrics.overall_verdict {
        println!("OVERALL VERDICT: HYPOTHESIS HYP-2026-002 SUPPORTED [HIGH CONFIDENCE]");
    } else {
        println!("OVERALL VERDICT: HYPOTHESIS HYP-2026-002 REFUTED / INCONCLUSIVE");
    }
    println!("Total Execution Duration: {:.2?}", t0.elapsed());
}
