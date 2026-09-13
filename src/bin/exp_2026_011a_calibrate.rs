//! Mechanical calibration of EXP-2026-011a against the pre-registered retrieval objective.
//!
//! Runs `DifferentialEvolution` under the default 60 s budget with a pre-registered
//! `target_loss`, then `RandomSearch` at an equal evaluation count as the honest baseline,
//! replays the integration-test assertions at each parameter set, and sweeps the two
//! parameters implicated in the noise-shielding claim.

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use rayon::prelude::*;
use serde_json::{Value, json};

use rust_3::experiments::exp_2026_011a_mva_associative_retrieval::{
    AssertionOutcome, RetrievalObjective, check_acceptance_assertions, hand_tuned_params,
    search_space,
};
use rust_3::substrate::search::{
    Budget, DifferentialEvolution, Evaluation, Objective, RandomSearch, SeedSplit, calibrate,
};

const OUTPUT_DIR: &str = "data/telemetry/EXP-2026-011a";
const POP_SIZE: usize = 24;
const DE_SEED: u64 = 0x0011_A0DE;
const RS_SEED: u64 = 0x0011_A0B5;

/// Pre-registered stopping threshold. Reaching it requires every ordering and violation term
/// to be exactly zero and the mean accuracy shortfall under noise to be at most 0.05, i.e. a
/// mean noisy accuracy of 0.95 against the 0.90 the acceptance test demands.
const TARGET_LOSS: f64 = 0.05;

/// `beta_theta` grid for the shielding diagnostic: the full searched range, refined near zero
/// where the WP-5a optimum landed.
const BETA_THETA_GRID: [f64; 12] = [
    0.0, 0.001, 0.0025, 0.005, 0.01, 0.02, 0.04, 0.06, 0.08, 0.10, 0.14, 0.20,
];

/// `t_readout` grid for the competing-explanation diagnostic, spanning the searched range.
const T_READOUT_GRID: [i64; 8] = [4, 6, 7, 8, 12, 16, 24, 32];

fn print_eval(label: &str, eval: &Evaluation) {
    println!("  {label}: loss = {:.6}", eval.loss);
    for (name, value) in &eval.metrics {
        println!("      {name} = {value:.6}");
    }
}

fn print_assertions(label: &str, outcomes: &[AssertionOutcome]) {
    let passed = outcomes.iter().filter(|o| o.passed).count();
    println!("  {label}: {passed}/{} assertions pass", outcomes.len());
    for o in outcomes {
        println!(
            "      [{}] {:<40} {}",
            if o.passed { "PASS" } else { "FAIL" },
            o.name,
            o.detail
        );
    }
}

fn load_params(path: &Path) -> Option<Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

/// Score a parameter set on held-out seeds once per value of one named parameter.
fn sweep(
    objective: &RetrievalObjective,
    base: &Value,
    name: &str,
    grid: &[Value],
    seeds: &[u64],
) -> Vec<Value> {
    grid.par_iter()
        .map(|value| {
            let mut p = base.clone();
            p[name] = value.clone();
            let eval = objective.evaluate(&p, seeds);
            json!({ name: value, "loss": eval.loss, "metrics": eval.metrics })
        })
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let space = search_space();
    let objective = RetrievalObjective::default();
    let seeds = SeedSplit::new((1_000..1_008).collect(), (5_000..5_008).collect())?;
    let out = Path::new(OUTPUT_DIR);
    fs::create_dir_all(out)?;

    println!(
        "EXP-2026-011a calibration | dim = {} | calibration seeds = {} | validation seeds = {}",
        space.dim(),
        seeds.calibration().len(),
        seeds.validation().len()
    );
    println!(
        "workload/seed: {} sequences at L = {} (active clean, plus active and fixed-theta at eps {:?}), {} at L = {}, {} comparative sequences for unindexed and unshielded",
        objective.sequences_per_seed,
        objective.seq_length,
        objective.noise_rates,
        objective.long_sequences_per_seed,
        objective.long_seq_length,
        objective.comparative_sequences_per_seed
    );

    // --- Cost of a single evaluation, measured rather than assumed ------------------------
    let hand: Value = hand_tuned_params();
    let probe_started = Instant::now();
    let hand_calibration = objective.evaluate(&hand, seeds.calibration());
    let single_eval = probe_started.elapsed();
    println!(
        "single serial evaluation over {} calibration seeds = {:.3} s",
        seeds.calibration().len(),
        single_eval.as_secs_f64()
    );

    // --- Differential evolution under the 60 s budget -------------------------------------
    let budget = Budget {
        target_loss: Some(TARGET_LOSS),
        ..Budget::default()
    };
    let mut de = DifferentialEvolution::new(space.dim(), POP_SIZE, DE_SEED);
    let de_started = Instant::now();
    let de_report = calibrate(&space, &objective, &mut de, &seeds, budget)?;
    let de_wall = de_started.elapsed();

    // --- Random search at an equal evaluation count ---------------------------------------
    let mut rs = RandomSearch::new(space.dim(), de_report.evaluations, POP_SIZE, RS_SEED);
    let rs_budget = Budget {
        max_evals: de_report.evaluations,
        max_wall_clock: Duration::from_secs(600),
        target_loss: None,
    };
    let rs_started = Instant::now();
    let rs_report = calibrate(&space, &objective, &mut rs, &seeds, rs_budget)?;
    let rs_wall = rs_started.elapsed();

    // --- Secondary arm: the same search run to budget exhaustion --------------------------
    // `target_loss` halts the registered run after a single generation, which is too few
    // evaluations for a DE-versus-random comparison to mean anything. This arm spends the
    // whole 60 s so the two optimizers can be compared at a non-trivial evaluation count. It
    // is reported only; `params.json` stays with the registered run above.
    let full_budget = Budget {
        target_loss: None,
        ..Budget::default()
    };
    let mut de_full = DifferentialEvolution::new(space.dim(), POP_SIZE, DE_SEED);
    let de_full_started = Instant::now();
    let de_full_report = calibrate(&space, &objective, &mut de_full, &seeds, full_budget)?;
    let de_full_wall = de_full_started.elapsed();

    let mut rs_full = RandomSearch::new(space.dim(), de_full_report.evaluations, POP_SIZE, RS_SEED);
    let rs_full_budget = Budget {
        max_evals: de_full_report.evaluations,
        max_wall_clock: Duration::from_secs(600),
        target_loss: None,
    };
    let rs_full_started = Instant::now();
    let rs_full_report = calibrate(&space, &objective, &mut rs_full, &seeds, rs_full_budget)?;
    let rs_full_wall = rs_full_started.elapsed();

    fs::write(
        out.join("calibration.json"),
        serde_json::to_string_pretty(&de_report)?,
    )?;
    fs::write(
        out.join("calibration_random_search.json"),
        serde_json::to_string_pretty(&rs_report)?,
    )?;
    fs::write(
        out.join("calibration_full_budget.json"),
        serde_json::to_string_pretty(&de_full_report)?,
    )?;
    fs::write(
        out.join("calibration_random_search_full_budget.json"),
        serde_json::to_string_pretty(&rs_full_report)?,
    )?;
    fs::write(
        out.join("params.json"),
        serde_json::to_string_pretty(&de_report.best_params)?,
    )?;

    // `calibrate` spends one extra evaluation scoring the winner on held-out seeds; time an
    // equivalent one so the search itself can be compared against its 60 s budget.
    let holdout_started = Instant::now();
    let de_validation = objective.evaluate(&de_report.best_params, seeds.validation());
    let holdout_wall = holdout_started.elapsed();
    let search_wall = de_wall.saturating_sub(holdout_wall);

    println!("\n== Differential evolution ==");
    println!(
        "  evaluations = {}  stop_reason = {:?}  search wall = {:.2} s (total {:.2} s)",
        de_report.evaluations,
        de_report.stop_reason,
        search_wall.as_secs_f64(),
        de_wall.as_secs_f64()
    );
    println!(
        "  throughput = {:.2} evals/s",
        de_report.evaluations as f64 / search_wall.as_secs_f64()
    );
    println!(
        "  best calibration loss = {:.6}",
        de_report.best_calibration_loss
    );
    println!(
        "  held-out validation loss = {:.6}",
        de_report.validation_loss
    );
    print_eval("validation", &de_validation);

    println!("\n== Random search (equal evaluation count) ==");
    println!(
        "  evaluations = {}  stop_reason = {:?}  wall = {:.2} s",
        rs_report.evaluations,
        rs_report.stop_reason,
        rs_wall.as_secs_f64()
    );
    println!(
        "  best calibration loss = {:.6}",
        rs_report.best_calibration_loss
    );
    println!(
        "  held-out validation loss = {:.6}",
        rs_report.validation_loss
    );

    println!("\n== Full-budget arm (target_loss disabled, reported only) ==");
    println!(
        "  DE            evaluations = {:>4}  stop_reason = {:?}  wall = {:.2} s  calibration = {:.6}  held-out = {:.6}",
        de_full_report.evaluations,
        de_full_report.stop_reason,
        de_full_wall.as_secs_f64(),
        de_full_report.best_calibration_loss,
        de_full_report.validation_loss
    );
    println!(
        "  random search evaluations = {:>4}  stop_reason = {:?}  wall = {:.2} s  calibration = {:.6}  held-out = {:.6}",
        rs_full_report.evaluations,
        rs_full_report.stop_reason,
        rs_full_wall.as_secs_f64(),
        rs_full_report.best_calibration_loss,
        rs_full_report.validation_loss
    );
    println!(
        "  DE throughput = {:.2} evals/s",
        de_full_report.evaluations as f64 / de_full_wall.as_secs_f64()
    );
    for spec in space.specs() {
        println!(
            "    {:<18} {:>22}",
            spec.name,
            de_full_report.best_params[&spec.name].to_string()
        );
    }

    let hand_validation = objective.evaluate(&hand, seeds.validation());
    println!("\n== Hand-tuned baseline (same seeds) ==");
    print_eval("calibration", &hand_calibration);
    print_eval("validation", &hand_validation);

    let wp5a = load_params(&out.join("params_wp5a.json"));
    if let Some(p) = &wp5a {
        println!("\n== WP-5a parameters under the completed objective ==");
        print_eval("calibration", &objective.evaluate(p, seeds.calibration()));
        print_eval("validation", &objective.evaluate(p, seeds.validation()));
    }

    println!("\n== Parameters ==");
    println!(
        "  {:<18} {:>14} {:>14} {:>14}",
        "parameter", "hand-tuned", "WP-5a", "WP-5c"
    );
    for spec in space.specs() {
        println!(
            "  {:<18} {:>14} {:>14} {:>14}",
            spec.name,
            hand[&spec.name].to_string(),
            wp5a.as_ref()
                .map(|p| p[&spec.name].to_string())
                .unwrap_or_else(|| "-".to_string()),
            de_report.best_params[&spec.name].to_string()
        );
    }

    println!("\n== Integration-test assertions replayed at each parameter set ==");
    print_assertions("hand-tuned", &check_acceptance_assertions(&hand));
    if let Some(p) = &wp5a {
        print_assertions("WP-5a", &check_acceptance_assertions(p));
    }
    let de_assertions = check_acceptance_assertions(&de_report.best_params);
    print_assertions("WP-5c", &de_assertions);

    // --- Shielding diagnostic: does the mechanism claim depend on beta_theta? -------------
    let beta_grid: Vec<Value> = BETA_THETA_GRID.iter().map(|b| json!(b)).collect();
    let t_grid: Vec<Value> = T_READOUT_GRID.iter().map(|t| json!(t)).collect();

    let beta_rows = sweep(
        &objective,
        &de_report.best_params,
        "beta_theta",
        &beta_grid,
        seeds.validation(),
    );
    let beta_rows_hand = sweep(
        &objective,
        &hand,
        "beta_theta",
        &beta_grid,
        seeds.validation(),
    );
    let t_rows = sweep(
        &objective,
        &de_report.best_params,
        "t_readout",
        &t_grid,
        seeds.validation(),
    );

    let eps_tags: Vec<String> = objective
        .noise_rates
        .iter()
        .map(|e| format!("eps{:03}", (e * 1000.0).round() as i64))
        .collect();

    let header = |first: &str| {
        let mut line = format!("  {first:<12}");
        for tag in &eps_tags {
            line.push_str(&format!(
                " {:>14} {:>14} {:>14}",
                format!("active_{tag}"),
                format!("fixed_{tag}"),
                format!("margin_{tag}")
            ));
        }
        line.push_str(&format!(" {:>10} {:>10}", "clean", "loss"));
        line
    };
    let row = |first: String, entry: &Value| {
        let m = &entry["metrics"];
        let mut line = format!("  {first:<12}");
        for tag in &eps_tags {
            line.push_str(&format!(
                " {:>14.4} {:>14.4} {:>14.4}",
                m[format!("accuracy_active_{tag}")].as_f64().unwrap_or(0.0),
                m[format!("accuracy_fixed_theta_{tag}")]
                    .as_f64()
                    .unwrap_or(0.0),
                m[format!("shielding_margin_{tag}")].as_f64().unwrap_or(0.0)
            ));
        }
        line.push_str(&format!(
            " {:>10.4} {:>10.4}",
            m["accuracy_clean"].as_f64().unwrap_or(0.0),
            entry["loss"].as_f64().unwrap_or(0.0)
        ));
        line
    };
    let beta_label = |entry: &Value| format!("{:.4}", entry["beta_theta"].as_f64().unwrap_or(0.0));

    println!("\n== beta_theta sweep at the WP-5c optimum (held-out seeds) ==");
    println!("{}", header("beta_theta"));
    for entry in &beta_rows {
        println!("{}", row(beta_label(entry), entry));
    }

    println!("\n== beta_theta sweep at the hand-tuned parameters (held-out seeds) ==");
    println!("{}", header("beta_theta"));
    for entry in &beta_rows_hand {
        println!("{}", row(beta_label(entry), entry));
    }

    println!("\n== t_readout sweep at the WP-5c optimum (held-out seeds) ==");
    println!("{}", header("t_readout"));
    for entry in &t_rows {
        println!(
            "{}",
            row(
                format!("{}", entry["t_readout"].as_i64().unwrap_or(0)),
                entry
            )
        );
    }

    fs::write(
        out.join("shielding_sweeps.json"),
        serde_json::to_string_pretty(&json!({
            "wp5c_params": de_report.best_params,
            "hand_tuned_params": hand,
            "validation_seeds": seeds.validation(),
            "noise_rates": objective.noise_rates,
            "beta_theta_sweep_at_wp5c": beta_rows,
            "beta_theta_sweep_at_hand_tuned": beta_rows_hand,
            "t_readout_sweep_at_wp5c": t_rows,
        }))?,
    )?;

    let met = de_report.validation_loss <= hand_validation.loss && search_wall.as_secs_f64() < 60.0;
    println!(
        "\nACCEPTANCE: held-out loss {:.6} vs hand-tuned {:.6}, search wall {:.2} s -> {}",
        de_report.validation_loss,
        hand_validation.loss,
        search_wall.as_secs_f64(),
        if met { "PASS" } else { "FAIL" }
    );
    println!(
        "  assertions replayed at WP-5c params: {}/{} pass",
        de_assertions.iter().filter(|o| o.passed).count(),
        de_assertions.len()
    );

    Ok(())
}
