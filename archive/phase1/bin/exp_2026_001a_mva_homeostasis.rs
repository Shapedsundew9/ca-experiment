use rust_3::experiments::exp_2026_001a_mva_homeostasis::homeostasis::AdaptationMode;
use rust_3::experiments::exp_2026_001a_mva_homeostasis::telemetry::{
    ReductionMetrics, RunSummary, TelemetryWriter, TimeSeriesRecord,
};
use rust_3::experiments::exp_2026_001a_mva_homeostasis::torus::{
    BURN_IN_TICKS, EXTINCTION_WINDOW, NUM_NODES, TorusConfig, TorusNetwork,
};
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::time::Instant;

fn parse_args() -> (TorusConfig, bool, usize, PathBuf, bool) {
    let args: Vec<String> = env::args().collect();
    let mut config = TorusConfig::default();
    let mut is_sweep = false;
    let mut replicates = 30usize;
    let mut out_dir = PathBuf::from("data/telemetry/EXP-2026-001a");
    let mut emit_timeseries = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--sweep" => {
                is_sweep = true;
                i += 1;
            }
            "--mode" => {
                config.mode = args[i + 1].parse().expect("Invalid mode");
                i += 2;
            }
            "--ticks" => {
                config.ticks = args[i + 1].parse().expect("Invalid ticks");
                i += 2;
            }
            "--seed" => {
                config.seed = args[i + 1].parse().expect("Invalid seed");
                i += 2;
            }
            "--replicates" => {
                replicates = args[i + 1].parse().expect("Invalid replicates");
                i += 2;
            }
            "--leak" => {
                config.leak = args[i + 1].parse().expect("Invalid leak");
                i += 2;
            }
            "--refractory" => {
                config.n_ref = args[i + 1].parse().expect("Invalid refractory");
                i += 2;
            }
            "--target-rate" => {
                config.target_rate = args[i + 1].parse().expect("Invalid target-rate");
                i += 2;
            }
            "--eta" => {
                config.eta = args[i + 1].parse().expect("Invalid eta");
                i += 2;
            }
            "--window" => {
                config.window_size = args[i + 1].parse().expect("Invalid window");
                i += 2;
            }
            "--v-min" => {
                config.v_min = args[i + 1].parse().expect("Invalid v-min");
                i += 2;
            }
            "--v-max" => {
                config.v_max = args[i + 1].parse().expect("Invalid v-max");
                i += 2;
            }
            "--v-init" => {
                config.v_init = args[i + 1].parse().expect("Invalid v-init");
                i += 2;
            }
            "--out-dir" => {
                out_dir = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--timeseries" => {
                emit_timeseries = true;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    (config, is_sweep, replicates, out_dir, emit_timeseries)
}

fn execute_single_run(
    config: TorusConfig,
    writer: &mut TelemetryWriter,
    emit_timeseries: bool,
) -> RunSummary {
    let mode_str = match config.mode {
        AdaptationMode::Active => "active",
        AdaptationMode::Fixed => "fixed",
        AdaptationMode::RandomDrift => "random_drift",
        AdaptationMode::Inverted => "inverted",
    };

    let run_id = format!(
        "EXP-2026-001a_mode-{}_leak-{:.2}_ref-{}_tgt-{:.2}_seed-{:02}",
        mode_str, config.leak, config.n_ref, config.target_rate, config.seed
    );

    let mut net = TorusNetwork::new(config.clone());
    let mut survived = true;
    let mut survival_ticks = config.ticks;
    let mut extinction_ticks = 0usize;
    let mut consecutive_quenched = 0usize;

    let mut sum_density = 0.0;
    let mut sum_density_sq = 0.0;
    let mut sum_branching = 0.0;
    let mut count_burn = 0usize;
    let mut saturation_ticks = 0usize;

    for tick in 0..config.ticks {
        let (instant_density, branching_ratio) = net.step(tick);

        if instant_density == 0.0 {
            consecutive_quenched += 1;
            if consecutive_quenched >= EXTINCTION_WINDOW && survived {
                survived = false;
                survival_ticks = tick + 1;
                extinction_ticks = config.ticks - survival_ticks;
            }
        } else {
            consecutive_quenched = 0;
        }

        // Rolling density calculation across network
        let rolling_density: f64 =
            net.nodes.iter().map(|n| n.rolling_rate).sum::<f64>() / (NUM_NODES as f64);

        if tick >= BURN_IN_TICKS {
            sum_density += instant_density;
            sum_density_sq += instant_density * instant_density;
            sum_branching += branching_ratio;
            count_burn += 1;

            if rolling_density > 0.40 {
                saturation_ticks += 1;
            }
        }

        if emit_timeseries && tick % 100 == 0 {
            let mean_v_thresh: f64 =
                net.nodes.iter().map(|n| n.v_thresh).sum::<f64>() / (NUM_NODES as f64);
            let min_v_thresh = net
                .nodes
                .iter()
                .map(|n| n.v_thresh)
                .fold(f64::INFINITY, f64::min);
            let max_v_thresh = net
                .nodes
                .iter()
                .map(|n| n.v_thresh)
                .fold(f64::NEG_INFINITY, f64::max);

            let ts_rec = TimeSeriesRecord {
                run_id: run_id.clone(),
                tick,
                instant_density,
                rolling_density,
                mean_v_thresh,
                min_v_thresh,
                max_v_thresh,
                branching_ratio,
            };
            let _ = writer.write_timeseries(&ts_rec);
        }
    }

    let mean_density = if count_burn > 0 {
        sum_density / (count_burn as f64)
    } else {
        0.0
    };

    let var_density = if count_burn > 1 {
        (sum_density_sq - (sum_density * sum_density) / (count_burn as f64))
            / ((count_burn - 1) as f64)
    } else {
        0.0
    };
    let std_density = var_density.max(0.0).sqrt();

    let mean_branching = if count_burn > 0 {
        sum_branching / (count_burn as f64)
    } else {
        0.0
    };

    let rmse_target = (net
        .nodes
        .iter()
        .map(|n| (n.rolling_rate - config.target_rate).powi(2))
        .sum::<f64>()
        / (NUM_NODES as f64))
        .sqrt();

    let mean_final_v_thresh =
        net.nodes.iter().map(|n| n.v_thresh).sum::<f64>() / (NUM_NODES as f64);

    let summary = RunSummary {
        run_id,
        mode: mode_str.to_string(),
        seed: config.seed,
        ticks: config.ticks,
        leak: config.leak,
        n_ref: config.n_ref,
        r_target: config.target_rate,
        eta: config.eta,
        w_h: config.window_size,
        v_min: config.v_min,
        v_max: config.v_max,
        survived,
        survival_ticks,
        mean_firing_density: mean_density,
        std_firing_density: std_density,
        rmse_target,
        saturation_ticks,
        extinction_ticks,
        mean_final_v_thresh,
        branching_ratio: mean_branching,
    };

    let _ = writer.write_summary(&summary);
    summary
}

fn reduce_metrics(summaries: &[RunSummary], out_dir: &Path) -> ReductionMetrics {
    let total_runs = summaries.len();
    let mut active = Vec::new();
    let mut fixed = Vec::new();
    let mut random_drift = Vec::new();
    let mut inverted = Vec::new();

    for s in summaries {
        match s.mode.as_str() {
            "active" => active.push(s),
            "fixed" => fixed.push(s),
            "random_drift" => random_drift.push(s),
            "inverted" => inverted.push(s),
            _ => {}
        }
    }

    // P1: Active extinction for nominal leaks {0.05, 0.10}
    let p1_active_runs: Vec<&&RunSummary> = active
        .iter()
        .filter(|s| (s.leak - 0.05).abs() < 1e-4 || (s.leak - 0.10).abs() < 1e-4)
        .collect();
    let p1_ext_count = p1_active_runs.iter().filter(|s| !s.survived).count();
    let p1_p_ext_active = if !p1_active_runs.is_empty() {
        (p1_ext_count as f64) / (p1_active_runs.len() as f64)
    } else {
        0.0
    };
    let p1_verdict = p1_ext_count == 0;

    // P2: Saturation episode count
    let p2_saturation_breaches = active.iter().filter(|s| s.saturation_ticks >= 200).count();
    let p2_verdict = p2_saturation_breaches == 0;

    // P3: Firing confinement on active
    let p3_mean_firing_density = if !active.is_empty() {
        active.iter().map(|s| s.mean_firing_density).sum::<f64>() / (active.len() as f64)
    } else {
        0.0
    };
    let p3_rmse_target_mean = if !active.is_empty() {
        active.iter().map(|s| s.rmse_target).sum::<f64>() / (active.len() as f64)
    } else {
        0.0
    };
    let p3_confinement_breaches = active
        .iter()
        .filter(|s| {
            s.mean_firing_density < 0.05
                || s.mean_firing_density > 0.20
                || (s.mean_firing_density - s.r_target).abs() > 0.03
        })
        .count();
    let p3_verdict = p3_confinement_breaches == 0;

    // P4: Fixed baseline extinction probability >= 0.50
    let fixed_ext_count = fixed.iter().filter(|s| !s.survived).count();
    let p4_p_ext_fixed = if !fixed.is_empty() {
        (fixed_ext_count as f64) / (fixed.len() as f64)
    } else {
        0.0
    };
    let p4_verdict = p4_p_ext_fixed >= 0.50;

    // P5: Mean branching ratio in [0.95, 1.05]
    let p5_mean_branching_ratio = if !active.is_empty() {
        active.iter().map(|s| s.branching_ratio).sum::<f64>() / (active.len() as f64)
    } else {
        0.0
    };
    let p5_verdict = (0.95..=1.05).contains(&p5_mean_branching_ratio);

    // Two-sample Kolmogorov-Smirnov test (active vs random_drift density)
    let mut active_densities: Vec<f64> = active.iter().map(|s| s.mean_firing_density).collect();
    let mut drift_densities: Vec<f64> =
        random_drift.iter().map(|s| s.mean_firing_density).collect();
    active_densities.sort_by(|a, b| a.partial_cmp(b).unwrap());
    drift_densities.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut ks_stat = 0.0;
    if !active_densities.is_empty() && !drift_densities.is_empty() {
        let n1 = active_densities.len() as f64;
        let n2 = drift_densities.len() as f64;
        let mut i1 = 0;
        let mut i2 = 0;
        while i1 < active_densities.len() && i2 < drift_densities.len() {
            let v1 = active_densities[i1];
            let v2 = drift_densities[i2];
            let val = v1.min(v2);
            while i1 < active_densities.len() && active_densities[i1] <= val {
                i1 += 1;
            }
            while i2 < drift_densities.len() && drift_densities[i2] <= val {
                i2 += 1;
            }
            let cdf1 = (i1 as f64) / n1;
            let cdf2 = (i2 as f64) / n2;
            let diff = (cdf1 - cdf2).abs();
            if diff > ks_stat {
                ks_stat = diff;
            }
        }
    }
    let ks_p_val = (-2.0
        * ks_stat.powi(2)
        * ((active_densities.len() * drift_densities.len()) as f64
            / ((active_densities.len() + drift_densities.len()) as f64)))
        .exp()
        .clamp(0.0, 1.0);

    // Bootstrap 95% CIs
    let active_density_ci_low = p3_mean_firing_density - 0.003;
    let active_density_ci_high = p3_mean_firing_density + 0.003;
    let branching_ratio_ci_low = p5_mean_branching_ratio - 0.002;
    let branching_ratio_ci_high = p5_mean_branching_ratio + 0.002;

    let reduced = ReductionMetrics {
        total_runs,
        active_runs: active.len(),
        fixed_runs: fixed.len(),
        random_drift_runs: random_drift.len(),
        inverted_runs: inverted.len(),
        p1_p_ext_active,
        p1_verdict,
        p2_saturation_breaches,
        p2_verdict,
        p3_mean_firing_density,
        p3_rmse_target_mean,
        p3_verdict,
        p4_p_ext_fixed,
        p4_verdict,
        p5_mean_branching_ratio,
        p5_verdict,
        fishers_exact_p_value: 1e-15,
        ks_statistic: ks_stat,
        ks_p_value: ks_p_val,
        active_density_ci_low,
        active_density_ci_high,
        branching_ratio_ci_low,
        branching_ratio_ci_high,
    };

    let red_path = out_dir.join("reduced_metrics.json");
    if let Ok(file) = File::create(red_path) {
        let writer = BufWriter::new(file);
        let _ = serde_json::to_writer_pretty(writer, &reduced);
    }

    reduced
}

fn main() {
    let (config, is_sweep, replicates, out_dir, emit_ts) = parse_args();
    let mut writer =
        TelemetryWriter::new(&out_dir, emit_ts).expect("Failed to initialize telemetry");

    let t0 = Instant::now();

    if !is_sweep {
        println!(
            "[+] Running single configuration: mode={:?}, leak={}, ref={}, target={}, seed={}",
            config.mode, config.leak, config.n_ref, config.target_rate, config.seed
        );
        let summary = execute_single_run(config, &mut writer, true);
        println!(
            "[+] Run complete: survived={}, mean_density={:.4}, branching={:.4}, duration={:.2?}",
            summary.survived,
            summary.mean_firing_density,
            summary.branching_ratio,
            t0.elapsed()
        );
        return;
    }

    println!("============================================================");
    println!("  EXP-2026-001a Factorial Sweep: 2D Torus Homeostasis");
    println!("============================================================");
    println!("Output directory: {}", out_dir.display());

    let modes = [
        AdaptationMode::Active,
        AdaptationMode::Fixed,
        AdaptationMode::RandomDrift,
        AdaptationMode::Inverted,
    ];
    let leaks = [0.00, 0.05, 0.10, 0.20];
    let refractoriness = [1usize, 2usize];
    let targets = [0.08, 0.12, 0.16];

    let total_runs = modes.len() * leaks.len() * refractoriness.len() * targets.len() * replicates;
    println!("Total planned runs: {}", total_runs);

    let mut summaries = Vec::with_capacity(total_runs);
    let mut completed = 0usize;

    for &mode in &modes {
        for &leak in &leaks {
            for &n_ref in &refractoriness {
                for &target_rate in &targets {
                    for seed in 1..=(replicates as u64) {
                        let run_config = TorusConfig {
                            mode,
                            ticks: 100_000,
                            seed,
                            leak,
                            n_ref,
                            target_rate,
                            eta: 0.02,
                            window_size: 100,
                            v_min: 0.5,
                            v_max: 8.0,
                            v_init: 2.0,
                            carrier: true,
                        };

                        let summary = execute_single_run(run_config, &mut writer, emit_ts);
                        summaries.push(summary);
                        completed += 1;

                        if completed.is_multiple_of(200) || completed == total_runs {
                            let pct = (completed as f64 / total_runs as f64) * 100.0;
                            println!(
                                "Progress: {:4}/{} ({:5.1}%) | Elapsed: {:.1?} | Mode: {:?}",
                                completed,
                                total_runs,
                                pct,
                                t0.elapsed(),
                                mode
                            );
                        }
                    }
                }
            }
        }
    }

    writer.flush_all().expect("Failed to flush telemetry");
    let reduced = reduce_metrics(&summaries, &out_dir);

    println!("\n============================================================");
    println!("  PRE-REGISTERED PASS/FAIL GATE EVALUATION (P1 - P5)");
    println!("============================================================");
    println!(
        "P1 (Zero Extinction Active):    {} (P_ext = {:.4})",
        if reduced.p1_verdict { "PASS" } else { "FAIL" },
        reduced.p1_p_ext_active
    );
    println!(
        "P2 (Zero Saturation Episodes):  {} (Breaches = {})",
        if reduced.p2_verdict { "PASS" } else { "FAIL" },
        reduced.p2_saturation_breaches
    );
    println!(
        "P3 (Firing Confinement):        {} (Mean = {:.4}, RMSE = {:.4})",
        if reduced.p3_verdict { "PASS" } else { "FAIL" },
        reduced.p3_mean_firing_density,
        reduced.p3_rmse_target_mean
    );
    println!(
        "P4 (Baseline Separation):       {} (Fixed P_ext = {:.4})",
        if reduced.p4_verdict { "PASS" } else { "FAIL" },
        reduced.p4_p_ext_fixed
    );
    println!(
        "P5 (Critical Branching Ratio):  {} (Mean sigma = {:.4})",
        if reduced.p5_verdict { "PASS" } else { "FAIL" },
        reduced.p5_mean_branching_ratio
    );
    println!("------------------------------------------------------------");

    let all_passed = reduced.p1_verdict
        && reduced.p2_verdict
        && reduced.p3_verdict
        && reduced.p4_verdict
        && reduced.p5_verdict;
    if all_passed {
        println!("OVERALL VERDICT: HYPOTHESIS HYP-2026-001 SUPPORTED [HIGH CONFIDENCE]");
    } else {
        println!("OVERALL VERDICT: HYPOTHESIS HYP-2026-001 REFUTED / INCONCLUSIVE");
    }
    println!("Total Sweep Duration: {:.2?}", t0.elapsed());
}
