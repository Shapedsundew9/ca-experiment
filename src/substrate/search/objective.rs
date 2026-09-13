//! Scalar objective contract evaluated by the calibration driver.

use serde::Serialize;
use std::collections::BTreeMap;

/// Result of scoring one candidate parameter vector over a seed set.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Evaluation {
    /// Scalar quantity to minimise.
    pub loss: f64,
    /// Auxiliary metrics retained for provenance; never used for selection.
    pub metrics: BTreeMap<String, f64>,
}

impl Evaluation {
    /// Build an evaluation carrying only a loss.
    pub fn new(loss: f64) -> Self {
        Self {
            loss,
            metrics: BTreeMap::new(),
        }
    }

    /// Attach a named auxiliary metric.
    pub fn with_metric(mut self, name: &str, value: f64) -> Self {
        self.metrics.insert(name.to_string(), value);
        self
    }
}

/// A pre-registered scoring function over decoded parameters.
///
/// `seeds` is a parameter rather than state of the implementor; this is what lets the
/// calibration driver enforce a calibration/validation split by construction.
pub trait Objective: Sync {
    /// Score `params` over the supplied seed set.
    fn evaluate(&self, params: &serde_json::Value, seeds: &[u64]) -> Evaluation;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct SeedSum;

    impl Objective for SeedSum {
        fn evaluate(&self, params: &serde_json::Value, seeds: &[u64]) -> Evaluation {
            let scale = params["scale"].as_f64().unwrap_or(0.0);
            let total: f64 = seeds.iter().map(|s| *s as f64).sum();
            Evaluation::new(scale * total).with_metric("n_seeds", seeds.len() as f64)
        }
    }

    #[test]
    fn objective_is_usable_behind_a_trait_object() {
        let obj: &dyn Objective = &SeedSum;
        let eval = obj.evaluate(&json!({ "scale": 2.0 }), &[1, 2, 3]);
        assert!((eval.loss - 12.0).abs() < 1e-12);
        assert_eq!(eval.metrics["n_seeds"], 3.0);
    }
}
