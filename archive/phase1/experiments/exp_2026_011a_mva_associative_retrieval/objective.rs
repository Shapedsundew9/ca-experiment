//! Pre-registered calibration objective for EXP-2026-011a.
//!
//! Adapter only: it overlays candidate parameters onto [`ExperimentConfig`] via serde and
//! scores the existing trial loop. No scientific logic lives here.
//!
//! The loss encodes **every** acceptance assertion in `tests/test_exp_2026_011a.rs`, not only
//! the primary retrieval assertion. Each comparative assertion becomes a smooth shortfall term
//! that is exactly zero once the required ordering holds by its pre-registered margin and
//! grows linearly as the margin is approached or inverted, so the optimizer sees a gradient
//! towards the ordering rather than a pass/fail cliff.

use serde::Serialize;
use serde_json::{Value, json};

use super::config::{Condition, ExperimentConfig};
use super::runner::{
    EvaluationSequence, RunTelemetryRecord, SequenceTrialRecord, execute_run, generate_dataset,
};
use super::substrate::FastRng;
use crate::substrate::search::{Evaluation, Objective, ParamSpec, SearchSpace};

/// Separates the long-horizon dataset stream from the L = 64 stream drawn for the same seed.
const LONG_HORIZON_SALT: u64 = 0x9E37_79B9_7F4A_7C15;

/// The six constants hand-tuned during the original EXP-2026-011a session, bracketed
/// generously around their hand-tuned values.
pub fn search_space() -> SearchSpace {
    SearchSpace::new(vec![
        ParamSpec::continuous("theta_gate_init", 0.60, 1.80).with_init(1.10),
        ParamSpec::continuous("beta_theta", 0.0, 0.20).with_init(0.05),
        ParamSpec::continuous("w_inh", -3.00, -0.25).with_init(-1.50),
        ParamSpec::continuous("leak", 0.005, 0.30).with_init(0.05),
        ParamSpec::integer("n_ref", 0, 8).with_init(2.0),
        ParamSpec::integer("t_readout", 4, 32).with_init(16.0),
    ])
}

/// The hand-tuned values currently baked into [`ExperimentConfig::new`], read back from the
/// constructor rather than restated, so the baseline cannot drift from the shipped config.
pub fn hand_tuned_params() -> Value {
    let cfg = ExperimentConfig::new(Condition::ActiveAssociative, 0.0, 64, 0);
    json!({
        "theta_gate_init": cfg.theta_gate_init,
        "beta_theta": cfg.beta_theta,
        "w_inh": cfg.w_inh,
        "leak": cfg.leak,
        "n_ref": cfg.n_ref,
        "t_readout": cfg.t_readout,
    })
}

/// Overlay named raw parameters onto a base config through its existing serde representation.
fn overlay(base: &ExperimentConfig, params: &Value) -> ExperimentConfig {
    let mut doc = serde_json::to_value(base).expect("ExperimentConfig is serialisable");
    if let (Some(target), Some(source)) = (doc.as_object_mut(), params.as_object()) {
        for (key, value) in source {
            if target.contains_key(key) {
                target.insert(key.clone(), value.clone());
            }
        }
    }
    serde_json::from_value(doc).expect("overlaid ExperimentConfig is well-typed")
}

/// Overlay candidate parameters, then re-impose the constant that *defines* each ablation.
///
/// Without this, a candidate `beta_theta` would be written into the fixed-theta ablation and a
/// candidate `n_ref` into the unshielded ablation, collapsing both controls onto the active
/// condition and making every comparative term vacuous.
pub fn overlay_for_condition(
    condition: Condition,
    base: &ExperimentConfig,
    params: &Value,
) -> ExperimentConfig {
    let mut cfg = overlay(base, params);
    match condition {
        Condition::AblationFixedTheta => cfg.beta_theta = 0.0,
        Condition::AblationUnshieldedRetrieval => cfg.n_ref = 0,
        _ => {}
    }
    cfg
}

/// Pre-registered ordering margins, one per comparative assertion.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Margins {
    /// Active clean accuracy must exceed unindexed accuracy by this much
    /// (`test_unindexed_baseline_collapses`: unindexed < 0.70 against a perfect active run).
    pub unindexed_accuracy: f64,
    /// Active clean accuracy must exceed the unshielded ablation by this much.
    pub unshielded_accuracy: f64,
    /// The unshielded ablation must retain at least this mutex violation rate
    /// (`test_unshielded_ablation_fails` asserts a strictly positive rate).
    pub unshielded_mutex_rate: f64,
    /// Active noisy accuracy must exceed fixed-theta noisy accuracy by this much
    /// (`test_noise_percolation_shielding`, comparative half).
    pub shielding: f64,
}

impl Default for Margins {
    fn default() -> Self {
        Self {
            unindexed_accuracy: 0.30,
            unshielded_accuracy: 0.10,
            unshielded_mutex_rate: 1e-3,
            shielding: 0.05,
        }
    }
}

/// Linear shortfall against a required margin: zero once `actual >= required`, one at
/// `actual == 0`, and continuing to grow as the ordering inverts.
fn shortfall(actual: f64, required: f64) -> f64 {
    ((required - actual) / required).max(0.0)
}

/// Scores associative retrieval against the full acceptance specification of
/// `tests/test_exp_2026_011a.rs`: clean retrieval, long-horizon distractor immunity,
/// accuracy under channel noise, mutual-exclusivity / leakage violations, and the three
/// between-condition orderings the tests assert.
pub struct RetrievalObjective {
    /// Sequence length of the primary workload.
    pub seq_length: usize,
    /// Sequences per seed for the primary active and fixed-theta runs.
    pub sequences_per_seed: usize,
    /// Sequence length of the long-horizon distractor-immunity workload.
    pub long_seq_length: usize,
    /// Sequences per seed at the long horizon.
    pub long_sequences_per_seed: usize,
    /// Sequences per seed for the unindexed and unshielded comparatives, which sit far from
    /// their margins and so need less resolution than the shielding comparison.
    pub comparative_sequences_per_seed: usize,
    /// Channel noise levels at which the noisy and shielding terms are evaluated.
    pub noise_rates: Vec<f64>,
    /// Required ordering margins.
    pub margins: Margins,
}

impl Default for RetrievalObjective {
    fn default() -> Self {
        Self {
            seq_length: 64,
            sequences_per_seed: 24,
            long_seq_length: 256,
            long_sequences_per_seed: 4,
            comparative_sequences_per_seed: 8,
            noise_rates: vec![0.05, 0.10],
            margins: Margins::default(),
        }
    }
}

impl RetrievalObjective {
    fn run(
        &self,
        params: &Value,
        condition: Condition,
        noise_rate: f64,
        seq_length: usize,
        seed: u64,
        dataset: &[EvaluationSequence],
    ) -> (RunTelemetryRecord, Vec<SequenceTrialRecord>) {
        let base = ExperimentConfig::new(condition, noise_rate, seq_length, seed);
        execute_run(&overlay_for_condition(condition, &base, params), dataset)
    }
}

/// Per-seed sums, averaged over the seed set before the loss is formed.
#[derive(Debug, Clone, Default)]
struct Totals {
    clean_acc: f64,
    long_acc: f64,
    violation_frac: f64,
    unindexed_acc: f64,
    unshielded_acc: f64,
    unshielded_mutex: f64,
    density: f64,
    active_noisy: Vec<f64>,
    fixed_noisy: Vec<f64>,
}

impl Objective for RetrievalObjective {
    fn evaluate(&self, params: &Value, seeds: &[u64]) -> Evaluation {
        let k = self.noise_rates.len().max(1) as f64;
        let mut t = Totals {
            active_noisy: vec![0.0; self.noise_rates.len()],
            fixed_noisy: vec![0.0; self.noise_rates.len()],
            ..Totals::default()
        };

        for &seed in seeds {
            let mut rng = FastRng::seed_from_u64(seed);
            let dataset = generate_dataset(self.seq_length, self.sequences_per_seed, &mut rng);
            let comparative = &dataset[..self.comparative_sequences_per_seed.min(dataset.len())];

            let mut long_rng = FastRng::seed_from_u64(seed ^ LONG_HORIZON_SALT);
            let long_dataset = generate_dataset(
                self.long_seq_length,
                self.long_sequences_per_seed,
                &mut long_rng,
            );

            let (clean, clean_trials) = self.run(
                params,
                Condition::ActiveAssociative,
                0.0,
                self.seq_length,
                seed,
                &dataset,
            );
            let (long, long_trials) = self.run(
                params,
                Condition::ActiveAssociative,
                0.0,
                self.long_seq_length,
                seed,
                &long_dataset,
            );
            let (unindexed, _) = self.run(
                params,
                Condition::BaselineUnindexed,
                0.0,
                self.seq_length,
                seed,
                comparative,
            );
            let (unshielded, _) = self.run(
                params,
                Condition::AblationUnshieldedRetrieval,
                0.0,
                self.seq_length,
                seed,
                comparative,
            );

            for (i, &eps) in self.noise_rates.iter().enumerate() {
                let (active, _) = self.run(
                    params,
                    Condition::ActiveAssociative,
                    eps,
                    self.seq_length,
                    seed,
                    &dataset,
                );
                let (fixed, _) = self.run(
                    params,
                    Condition::AblationFixedTheta,
                    eps,
                    self.seq_length,
                    seed,
                    &dataset,
                );
                t.active_noisy[i] += active.metrics.accuracy_retrieval;
                t.fixed_noisy[i] += fixed.metrics.accuracy_retrieval;
            }

            let violating = clean_trials
                .iter()
                .chain(long_trials.iter())
                .filter(|r| r.mutex_violation_ticks > 0 || r.distractor_leakage_events > 0)
                .count();
            let scored = (clean_trials.len() + long_trials.len()).max(1);

            t.clean_acc += clean.metrics.accuracy_retrieval;
            t.long_acc += long.metrics.accuracy_retrieval;
            t.violation_frac += violating as f64 / scored as f64;
            t.unindexed_acc += unindexed.metrics.accuracy_retrieval;
            t.unshielded_acc += unshielded.metrics.accuracy_retrieval;
            t.unshielded_mutex += unshielded.metrics.mutex_violation_rate;
            t.density += clean.metrics.mean_firing_density;
        }

        let n = seeds.len().max(1) as f64;
        let clean_acc = t.clean_acc / n;
        let long_acc = t.long_acc / n;
        let violation_frac = t.violation_frac / n;
        let unindexed_acc = t.unindexed_acc / n;
        let unshielded_acc = t.unshielded_acc / n;
        let unshielded_mutex = t.unshielded_mutex / n;
        let active_noisy: Vec<f64> = t.active_noisy.iter().map(|v| v / n).collect();
        let fixed_noisy: Vec<f64> = t.fixed_noisy.iter().map(|v| v / n).collect();

        // One term per acceptance assertion.
        // test_clean_baseline_retrieval: every clean L = 64 trial correct.
        let clean_term = 1.0 - clean_acc;
        // test_distractor_immunity_long_horizon: every clean L = 256 trial correct.
        let long_horizon_term = 1.0 - long_acc;
        // test_noise_percolation_shielding, primary half: active accuracy under noise.
        let noisy_term = active_noisy.iter().map(|a| 1.0 - a).sum::<f64>() / k;
        // Both clean tests: zero mutex violation ticks and zero distractor leakage events.
        let violation_term = violation_frac;
        // test_unindexed_baseline_collapses: active must beat the unindexed baseline.
        let unindexed_term = shortfall(clean_acc - unindexed_acc, self.margins.unindexed_accuracy);
        // test_unshielded_ablation_fails: active must beat the unshielded ablation, which must
        // itself still exhibit the mutex violations that define the ablation.
        let unshielded_term =
            shortfall(clean_acc - unshielded_acc, self.margins.unshielded_accuracy);
        let unshielded_mutex_term = shortfall(unshielded_mutex, self.margins.unshielded_mutex_rate);
        // test_noise_percolation_shielding, comparative half: the term omitted in WP-5a.
        let shielding_term = active_noisy
            .iter()
            .zip(fixed_noisy.iter())
            .map(|(a, f)| shortfall(a - f, self.margins.shielding))
            .sum::<f64>()
            / k;

        let loss = clean_term
            + long_horizon_term
            + noisy_term
            + violation_term
            + unindexed_term
            + unshielded_term
            + unshielded_mutex_term
            + shielding_term;

        let mut eval = Evaluation::new(loss)
            .with_metric("accuracy_clean", clean_acc)
            .with_metric("accuracy_long_horizon", long_acc)
            .with_metric("accuracy_unindexed", unindexed_acc)
            .with_metric("accuracy_unshielded", unshielded_acc)
            .with_metric("mutex_violation_rate_unshielded", unshielded_mutex)
            .with_metric("violation_trial_fraction", violation_frac)
            .with_metric("mean_firing_density_clean", t.density / n)
            .with_metric("term_clean", clean_term)
            .with_metric("term_long_horizon", long_horizon_term)
            .with_metric("term_noisy", noisy_term)
            .with_metric("term_violation", violation_term)
            .with_metric("term_order_unindexed", unindexed_term)
            .with_metric("term_order_unshielded", unshielded_term)
            .with_metric("term_unshielded_mutex", unshielded_mutex_term)
            .with_metric("term_order_shielding", shielding_term);

        for (i, &eps) in self.noise_rates.iter().enumerate() {
            let tag = format!("eps{:03}", (eps * 1000.0).round() as i64);
            eval = eval
                .with_metric(&format!("accuracy_active_{tag}"), active_noisy[i])
                .with_metric(&format!("accuracy_fixed_theta_{tag}"), fixed_noisy[i])
                .with_metric(
                    &format!("shielding_margin_{tag}"),
                    active_noisy[i] - fixed_noisy[i],
                );
        }
        eval
    }
}

/// Outcome of replaying one integration-test assertion at a candidate parameter set.
#[derive(Debug, Clone, Serialize)]
pub struct AssertionOutcome {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

fn outcome(name: &str, passed: bool, detail: String) -> AssertionOutcome {
    AssertionOutcome {
        name: name.to_string(),
        passed,
        detail,
    }
}

/// Replay the acceptance assertions of `tests/test_exp_2026_011a.rs` at `params`.
///
/// The test seeds (101, 202, 303, 404, 505) are disjoint from both the calibration and the
/// validation seed sets, so this is a third, fully held-out check. The topology invariants are
/// not replayed: none of the six searched constants can affect node or partition counts.
pub fn check_acceptance_assertions(params: &Value) -> Vec<AssertionOutcome> {
    let mut results = Vec::new();

    let cfg = overlay_for_condition(
        Condition::ActiveAssociative,
        &ExperimentConfig::new(Condition::ActiveAssociative, 0.0, 64, 101),
        params,
    );
    let mut rng = FastRng::seed_from_u64(101);
    let dataset = generate_dataset(64, 10, &mut rng);
    let (_, trials) = execute_run(&cfg, &dataset);
    let wrong = trials.iter().filter(|t| !t.correct).count();
    let violations = trials
        .iter()
        .filter(|t| t.mutex_violation_ticks > 0 || t.distractor_leakage_events > 0)
        .count();
    results.push(outcome(
        "test_clean_baseline_retrieval",
        wrong == 0 && violations == 0,
        format!(
            "{}/{} correct, {violations} trials with violations",
            trials.len() - wrong,
            trials.len()
        ),
    ));

    let cfg = overlay_for_condition(
        Condition::ActiveAssociative,
        &ExperimentConfig::new(Condition::ActiveAssociative, 0.0, 256, 202),
        params,
    );
    let mut rng = FastRng::seed_from_u64(202);
    let dataset = generate_dataset(256, 5, &mut rng);
    let (_, trials) = execute_run(&cfg, &dataset);
    let wrong = trials.iter().filter(|t| !t.correct).count();
    let violations = trials
        .iter()
        .filter(|t| t.mutex_violation_ticks > 0 || t.distractor_leakage_events > 0)
        .count();
    results.push(outcome(
        "test_distractor_immunity_long_horizon",
        wrong == 0 && violations == 0,
        format!(
            "{}/{} correct, {violations} trials with violations",
            trials.len() - wrong,
            trials.len()
        ),
    ));

    let cfg = overlay_for_condition(
        Condition::BaselineUnindexed,
        &ExperimentConfig::new(Condition::BaselineUnindexed, 0.0, 64, 303),
        params,
    );
    let mut rng = FastRng::seed_from_u64(303);
    let dataset = generate_dataset(64, 50, &mut rng);
    let (rec, _) = execute_run(&cfg, &dataset);
    results.push(outcome(
        "test_unindexed_baseline_collapses",
        rec.metrics.accuracy_retrieval < 0.70,
        format!("unindexed accuracy = {:.4}", rec.metrics.accuracy_retrieval),
    ));

    let cfg = overlay_for_condition(
        Condition::AblationUnshieldedRetrieval,
        &ExperimentConfig::new(Condition::AblationUnshieldedRetrieval, 0.0, 64, 404),
        params,
    );
    let mut rng = FastRng::seed_from_u64(404);
    let dataset = generate_dataset(64, 30, &mut rng);
    let (rec, _) = execute_run(&cfg, &dataset);
    results.push(outcome(
        "test_unshielded_ablation_fails",
        rec.metrics.mutex_violation_rate > 0.0,
        format!("mutex rate = {:.6}", rec.metrics.mutex_violation_rate),
    ));

    let mut rng = FastRng::seed_from_u64(505);
    let dataset = generate_dataset(64, 30, &mut rng);
    let (active, _) = execute_run(
        &overlay_for_condition(
            Condition::ActiveAssociative,
            &ExperimentConfig::new(Condition::ActiveAssociative, 0.05, 64, 505),
            params,
        ),
        &dataset,
    );
    let (fixed, _) = execute_run(
        &overlay_for_condition(
            Condition::AblationFixedTheta,
            &ExperimentConfig::new(Condition::AblationFixedTheta, 0.05, 64, 505),
            params,
        ),
        &dataset,
    );
    let a = active.metrics.accuracy_retrieval;
    let f = fixed.metrics.accuracy_retrieval;
    results.push(outcome(
        "test_noise_percolation_shielding",
        a >= 0.90 && f < a,
        format!(
            "active = {a:.4}, fixed_theta = {f:.4}, margin = {:.4}",
            a - f
        ),
    ));

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_replaces_only_named_fields() {
        let base = ExperimentConfig::new(Condition::ActiveAssociative, 0.0, 64, 7);
        let applied = overlay(
            &base,
            &json!({ "n_ref": 5, "w_inh": -0.75, "unknown_field": 1 }),
        );
        assert_eq!(applied.n_ref, 5);
        assert!((applied.w_inh + 0.75).abs() < 1e-12);
        assert_eq!(applied.seed, base.seed);
        assert_eq!(applied.condition, base.condition);
        assert_eq!(applied.t_readout, base.t_readout);
    }

    #[test]
    fn ablations_keep_their_defining_constant() {
        let params = json!({ "beta_theta": 0.17, "n_ref": 6, "t_readout": 9 });

        let fixed = overlay_for_condition(
            Condition::AblationFixedTheta,
            &ExperimentConfig::new(Condition::AblationFixedTheta, 0.05, 64, 1),
            &params,
        );
        assert_eq!(fixed.beta_theta, 0.0);
        assert_eq!(fixed.n_ref, 6);
        assert_eq!(fixed.t_readout, 9);

        let unshielded = overlay_for_condition(
            Condition::AblationUnshieldedRetrieval,
            &ExperimentConfig::new(Condition::AblationUnshieldedRetrieval, 0.0, 64, 1),
            &params,
        );
        assert_eq!(unshielded.n_ref, 0);
        assert!((unshielded.beta_theta - 0.17).abs() < 1e-12);

        let active = overlay_for_condition(
            Condition::ActiveAssociative,
            &ExperimentConfig::new(Condition::ActiveAssociative, 0.05, 64, 1),
            &params,
        );
        assert!((active.beta_theta - 0.17).abs() < 1e-12);
        assert_eq!(active.n_ref, 6);
    }

    #[test]
    fn shortfall_is_zero_at_the_margin_and_grows_past_inversion() {
        assert_eq!(shortfall(0.20, 0.10), 0.0);
        assert_eq!(shortfall(0.10, 0.10), 0.0);
        assert!((shortfall(0.05, 0.10) - 0.5).abs() < 1e-12);
        assert!((shortfall(0.0, 0.10) - 1.0).abs() < 1e-12);
        assert!(shortfall(-0.10, 0.10) > 1.9);
    }

    #[test]
    fn hand_tuned_params_round_trip_through_the_space() {
        let space = search_space();
        let hand = hand_tuned_params();
        let decoded = space.decode(&space.encode(&hand));
        assert_eq!(decoded["n_ref"], json!(2));
        assert_eq!(decoded["t_readout"], json!(16));
        assert!((decoded["w_inh"].as_f64().unwrap() + 1.50).abs() < 1e-9);
    }

    #[test]
    fn objective_reports_one_term_per_acceptance_assertion() {
        let objective = RetrievalObjective {
            sequences_per_seed: 4,
            long_sequences_per_seed: 1,
            comparative_sequences_per_seed: 4,
            noise_rates: vec![0.05],
            ..RetrievalObjective::default()
        };
        let eval = objective.evaluate(&hand_tuned_params(), &[11]);
        for term in [
            "term_clean",
            "term_long_horizon",
            "term_noisy",
            "term_violation",
            "term_order_unindexed",
            "term_order_unshielded",
            "term_unshielded_mutex",
            "term_order_shielding",
        ] {
            assert!(eval.metrics.contains_key(term), "missing {term}");
        }
        assert_eq!(eval.metrics["accuracy_clean"], 1.0);
        assert_eq!(eval.metrics["violation_trial_fraction"], 0.0);
        assert!(eval.metrics.contains_key("shielding_margin_eps050"));
    }
}
