//! Calibration driver: runs an [`Optimizer`] against an [`Objective`] under a budget and
//! reports a held-out validation score.

use rayon::prelude::*;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fmt;
use std::time::{Duration, Instant};

use super::objective::Objective;
use super::optimizer::Optimizer;
use super::space::SearchSpace;

/// Failure modes of a calibration run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchError {
    /// Calibration and validation seed sets intersect, so the held-out score is not held out.
    OverlappingSeeds,
    /// The search space has no coordinates.
    EmptySpace,
    /// The budget or optimizer schedule permitted no evaluations at all.
    NoEvaluations,
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OverlappingSeeds => {
                write!(f, "calibration and validation seed sets must be disjoint")
            }
            Self::EmptySpace => write!(f, "search space has zero dimensions"),
            Self::NoEvaluations => write!(f, "budget permitted no objective evaluations"),
        }
    }
}

impl std::error::Error for SearchError {}

/// Disjoint calibration and validation seed sets.
///
/// Disjointness is enforced at construction, so the overfitting guard is a property of the
/// type rather than of agent discipline.
#[derive(Debug, Clone, Serialize)]
pub struct SeedSplit {
    calibration: Vec<u64>,
    validation: Vec<u64>,
}

impl SeedSplit {
    /// Build a split, rejecting any overlap between the two seed sets.
    pub fn new(calibration: Vec<u64>, validation: Vec<u64>) -> Result<Self, SearchError> {
        let held_out: BTreeSet<u64> = validation.iter().copied().collect();
        if calibration.iter().any(|s| held_out.contains(s)) {
            return Err(SearchError::OverlappingSeeds);
        }
        Ok(Self {
            calibration,
            validation,
        })
    }

    /// Seeds the optimizer is allowed to see.
    pub fn calibration(&self) -> &[u64] {
        &self.calibration
    }

    /// Seeds reserved for the held-out score.
    pub fn validation(&self) -> &[u64] {
        &self.validation
    }
}

/// Hard stopping limits for a calibration run.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Budget {
    pub max_evals: usize,
    pub max_wall_clock: Duration,
    /// Stop once the best observed loss is at or below this value.
    pub target_loss: Option<f64>,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            max_evals: 10_000,
            max_wall_clock: Duration::from_secs(60),
            target_loss: None,
        }
    }
}

impl Budget {
    /// Budget capped only by evaluation count, retaining the 60 s default wall clock.
    pub fn with_max_evals(max_evals: usize) -> Self {
        Self {
            max_evals,
            ..Self::default()
        }
    }
}

/// Why a calibration run halted. Distinguishes convergence from budget exhaustion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// The best observed loss met `Budget::target_loss`.
    TargetReached,
    /// `Budget::max_evals` was reached.
    EvalBudget,
    /// `Budget::max_wall_clock` bound, counting the projected cost of the next batch.
    WallClock,
    /// The optimizer exhausted its own schedule.
    OptimizerDone,
}

/// One scored candidate in the search trace.
#[derive(Debug, Clone, Serialize)]
pub struct TraceEntry {
    pub params: Value,
    pub loss: f64,
}

/// Full provenance record of a calibration run, serialised to `calibration.json`.
#[derive(Debug, Clone, Serialize)]
pub struct CalibrationReport {
    pub space: SearchSpace,
    pub optimizer: String,
    pub budget: Budget,
    pub seeds: SeedSplit,
    pub evaluations: usize,
    pub stop_reason: StopReason,
    pub best_params: Value,
    pub best_calibration_loss: f64,
    pub validation_loss: f64,
    pub trace: Vec<TraceEntry>,
}

/// Run `opt` against `objective` over `space`, then score the winner on the held-out seeds.
///
/// The loop halts on whichever comes first: `target_loss` being met, the optimizer declaring
/// itself done, `max_evals`, or `max_wall_clock`. The wall-clock limit is a true cap: a batch
/// is dispatched only if the running mean batch duration fits inside the remaining time. Each
/// `ask` batch is evaluated in parallel across all available cores.
pub fn calibrate(
    space: &SearchSpace,
    objective: &dyn Objective,
    opt: &mut dyn Optimizer,
    seeds: &SeedSplit,
    budget: Budget,
) -> Result<CalibrationReport, SearchError> {
    if space.dim() == 0 {
        return Err(SearchError::EmptySpace);
    }

    let started = Instant::now();
    let mut evaluations = 0usize;
    let mut trace: Vec<TraceEntry> = Vec::new();
    let mut best: Option<(Value, f64)> = None;
    let mut batch_time = Duration::ZERO;
    let mut batches = 0u32;

    let stop_reason = loop {
        if matches!((budget.target_loss, best.as_ref()), (Some(t), Some((_, l))) if *l <= t) {
            break StopReason::TargetReached;
        }
        if evaluations >= budget.max_evals {
            break StopReason::EvalBudget;
        }
        let elapsed = started.elapsed();
        let mean_batch = batch_time.checked_div(batches).unwrap_or_default();
        if elapsed >= budget.max_wall_clock || elapsed + mean_batch > budget.max_wall_clock {
            break StopReason::WallClock;
        }
        if opt.is_done() {
            break StopReason::OptimizerDone;
        }

        let mut batch = opt.ask();
        if batch.is_empty() {
            break StopReason::OptimizerDone;
        }

        // A truncated batch breaks the positional ask/tell contract, so it is the last one.
        let remaining = budget.max_evals - evaluations;
        let truncated = batch.len() > remaining;
        batch.truncate(remaining);

        let batch_started = Instant::now();
        let scored: Vec<(Value, f64)> = batch
            .par_iter()
            .map(|x| {
                let params = space.decode(x);
                let loss = objective.evaluate(&params, seeds.calibration()).loss;
                (params, loss)
            })
            .collect();
        batch_time += batch_started.elapsed();
        batches += 1;

        evaluations += scored.len();
        for (params, loss) in &scored {
            if best.as_ref().is_none_or(|(_, b)| loss < b) {
                best = Some((params.clone(), *loss));
            }
            trace.push(TraceEntry {
                params: params.clone(),
                loss: *loss,
            });
        }

        if truncated {
            break StopReason::EvalBudget;
        }

        let feedback: Vec<(Vec<f64>, f64)> = batch
            .into_iter()
            .zip(scored.iter().map(|(_, loss)| *loss))
            .collect();
        opt.tell(&feedback);
    };

    let (best_params, best_calibration_loss) = best.ok_or(SearchError::NoEvaluations)?;
    let validation_loss = objective.evaluate(&best_params, seeds.validation()).loss;

    Ok(CalibrationReport {
        space: space.clone(),
        optimizer: opt.name().to_string(),
        budget,
        seeds: seeds.clone(),
        evaluations,
        stop_reason,
        best_params,
        best_calibration_loss,
        validation_loss,
        trace,
    })
}

#[cfg(test)]
mod tests {
    use super::super::objective::Evaluation;
    use super::super::optimizer::{DifferentialEvolution, RandomSearch};
    use super::super::space::ParamSpec;
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Quadratic bowl whose minimum drifts with the mean seed, so calibration and validation
    /// losses are distinguishable.
    struct Bowl {
        calls: AtomicUsize,
    }

    impl Bowl {
        fn new() -> Self {
            Self {
                calls: AtomicUsize::new(0),
            }
        }
    }

    impl Objective for Bowl {
        fn evaluate(&self, params: &Value, seeds: &[u64]) -> Evaluation {
            self.calls.fetch_add(1, Ordering::Relaxed);
            let x = params["x"].as_f64().unwrap_or(0.0);
            let offset = seeds.iter().map(|s| *s as f64).sum::<f64>() / seeds.len().max(1) as f64;
            Evaluation::new((x - 1.0).powi(2) + offset * 1e-3)
        }
    }

    fn bowl_space() -> SearchSpace {
        SearchSpace::new(vec![ParamSpec::continuous("x", -5.0, 5.0)])
    }

    /// Bowl whose cost scales with the seed count, making batch duration predictable.
    struct SlowBowl {
        per_seed: Duration,
    }

    impl Objective for SlowBowl {
        fn evaluate(&self, params: &Value, seeds: &[u64]) -> Evaluation {
            std::thread::sleep(self.per_seed * seeds.len() as u32);
            let x = params["x"].as_f64().unwrap_or(0.0);
            Evaluation::new((x - 1.0).powi(2))
        }
    }

    #[test]
    fn seed_split_rejects_intersecting_sets() {
        assert_eq!(
            SeedSplit::new(vec![1, 2, 3], vec![3, 4]).unwrap_err(),
            SearchError::OverlappingSeeds
        );
        let ok = SeedSplit::new(vec![1, 2, 3], vec![4, 5]).unwrap();
        assert_eq!(ok.calibration(), &[1, 2, 3]);
        assert_eq!(ok.validation(), &[4, 5]);
    }

    #[test]
    fn default_budget_caps_wall_clock_at_sixty_seconds() {
        assert_eq!(Budget::default().max_wall_clock, Duration::from_secs(60));
        assert_eq!(Budget::default().max_evals, 10_000);
        assert_eq!(Budget::default().target_loss, None);
    }

    #[test]
    fn calibrate_rejects_an_empty_space() {
        let seeds = SeedSplit::new(vec![1], vec![2]).unwrap();
        let mut opt = RandomSearch::new(0, 4, 2, 1);
        let err = calibrate(
            &SearchSpace::new(Vec::new()),
            &Bowl::new(),
            &mut opt,
            &seeds,
            Budget::default(),
        )
        .unwrap_err();
        assert_eq!(err, SearchError::EmptySpace);
    }

    #[test]
    fn calibrate_respects_max_evals() {
        let seeds = SeedSplit::new(vec![1, 2], vec![10, 11]).unwrap();
        let objective = Bowl::new();
        let mut opt = DifferentialEvolution::new(1, 10, 42).with_max_generations(1_000);
        let report = calibrate(
            &bowl_space(),
            &objective,
            &mut opt,
            &seeds,
            Budget::with_max_evals(25),
        )
        .unwrap();

        assert_eq!(report.evaluations, 25);
        assert_eq!(report.trace.len(), 25);
        assert_eq!(report.stop_reason, StopReason::EvalBudget);
        // 25 search evaluations plus the single held-out re-evaluation.
        assert_eq!(objective.calls.load(Ordering::Relaxed), 26);
    }

    #[test]
    fn calibrate_halts_once_the_target_loss_is_met() {
        let seeds = SeedSplit::new(vec![1, 2], vec![10, 11]).unwrap();
        let objective = Bowl::new();
        let mut opt = DifferentialEvolution::new(1, 8, 3).with_max_generations(1_000);
        let report = calibrate(
            &bowl_space(),
            &objective,
            &mut opt,
            &seeds,
            Budget {
                target_loss: Some(20.0),
                ..Budget::with_max_evals(10_000)
            },
        )
        .unwrap();

        assert_eq!(report.stop_reason, StopReason::TargetReached);
        // The first batch already clears the target, so the run stops one generation in.
        assert_eq!(report.evaluations, 8);
        assert!(report.best_calibration_loss <= 20.0);
    }

    #[test]
    fn calibrate_reports_a_held_out_score_for_the_best_candidate() {
        let seeds = SeedSplit::new(vec![0, 0], vec![100, 100]).unwrap();
        let mut opt = DifferentialEvolution::new(1, 12, 7).with_max_generations(60);
        let report = calibrate(
            &bowl_space(),
            &Bowl::new(),
            &mut opt,
            &seeds,
            Budget::with_max_evals(2_000),
        )
        .unwrap();

        assert_eq!(report.optimizer, "differential_evolution");
        assert!(report.best_calibration_loss < 1e-6);
        assert!((report.best_params["x"].as_f64().unwrap() - 1.0).abs() < 1e-3);
        // The validation seeds carry a larger offset, so the held-out score is strictly worse.
        assert!(report.validation_loss > report.best_calibration_loss);
        assert!((report.validation_loss - 0.1).abs() < 1e-3);
    }

    #[test]
    fn calibrate_halts_on_an_exhausted_optimizer() {
        let seeds = SeedSplit::new(vec![1], vec![2]).unwrap();
        let mut opt = RandomSearch::new(1, 12, 4, 3);
        let report = calibrate(
            &bowl_space(),
            &Bowl::new(),
            &mut opt,
            &seeds,
            Budget::with_max_evals(10_000),
        )
        .unwrap();
        assert_eq!(report.evaluations, 12);
        assert_eq!(report.optimizer, "random_search");
        assert_eq!(report.stop_reason, StopReason::OptimizerDone);
    }

    #[test]
    fn calibrate_halts_on_wall_clock() {
        let seeds = SeedSplit::new(vec![1], vec![2]).unwrap();
        let mut opt = DifferentialEvolution::new(1, 8, 11).with_max_generations(usize::MAX);
        let report = calibrate(
            &bowl_space(),
            &Bowl::new(),
            &mut opt,
            &seeds,
            Budget {
                max_evals: usize::MAX,
                max_wall_clock: Duration::from_millis(20),
                target_loss: None,
            },
        )
        .unwrap();
        assert!(report.evaluations > 0);
        assert_eq!(report.stop_reason, StopReason::WallClock);
    }

    #[test]
    fn calibrate_stops_before_a_batch_would_overrun_the_wall_clock() {
        let seeds = SeedSplit::new((0..8).collect(), vec![100]).unwrap();
        let mut opt = DifferentialEvolution::new(1, 4, 17).with_max_generations(usize::MAX);
        let max_wall_clock = Duration::from_millis(430);
        let started = Instant::now();
        let report = calibrate(
            &bowl_space(),
            &SlowBowl {
                per_seed: Duration::from_millis(5),
            },
            &mut opt,
            &seeds,
            Budget {
                max_evals: usize::MAX,
                max_wall_clock,
                target_loss: None,
            },
        )
        .unwrap();
        let elapsed = started.elapsed();

        assert_eq!(report.stop_reason, StopReason::WallClock);
        assert!(report.evaluations >= 4);
        assert!(
            elapsed <= max_wall_clock,
            "run took {elapsed:?}, exceeding the {max_wall_clock:?} cap"
        );
    }

    #[test]
    fn report_serialises() {
        let seeds = SeedSplit::new(vec![1], vec![2]).unwrap();
        let mut opt = RandomSearch::new(1, 4, 2, 5);
        let report = calibrate(
            &bowl_space(),
            &Bowl::new(),
            &mut opt,
            &seeds,
            Budget::with_max_evals(4),
        )
        .unwrap();
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["evaluations"], 4);
        assert_eq!(json["stop_reason"], "eval_budget");
        assert_eq!(json["seeds"]["validation"][0], 2);
        assert!(json["space"]["specs"].is_array());
    }
}
