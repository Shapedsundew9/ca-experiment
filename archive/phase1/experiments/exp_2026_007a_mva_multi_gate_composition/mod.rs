//! EXP-2026-007a: Cascaded Boolean Logic Gate Composition and Planar Wire Crossing.
//!
//! Milestone 1.2 in Tier 1: Spatial Routing & Compositionality.
//! Implements a 1-bit Full Adder with delay equalization and refractory-shielded wire crossing.

pub mod circuit;
pub mod config;
pub mod metrics;
pub mod runner;
pub mod substrate;

pub use circuit::build_full_adder_circuit;
pub use config::{Condition, EvaluationMode, ExperimentConfig};
pub use metrics::{RunMetrics, expected_cout, expected_sum, sample_mean, sample_std, welch_t_test};
pub use runner::{
    EvaluationSummaryManifest, RunTelemetryRecord, build_experiment_plan, execute_single_run,
    run_experiment_sweep,
};
pub use substrate::{CircuitNode, CircuitSubstrate, FastRng};
