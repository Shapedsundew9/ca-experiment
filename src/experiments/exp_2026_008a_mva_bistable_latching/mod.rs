//! EXP-2026-008a: Bistable Resonant Latching and Nondestructive Dynamic Bit Storage.
//!
//! Milestone 2.1 in Tier 2: Temporal Dynamics & State Retention.
//! Implements a bistable resonant latch with recurrent feedback, refractory diode shielding,
//! coordinated hyperpolarizing reset inhibition, and nondestructive sense tap readout.

pub mod circuit;
pub mod config;
pub mod metrics;
pub mod runner;
pub mod substrate;

pub use circuit::build_bistable_latch_circuit;
pub use config::{Condition, EvaluationSuite, ExperimentConfig};
pub use metrics::{RunMetrics, sample_mean, sample_std, welch_t_test};
pub use runner::{
    EvaluationSummaryManifest, RunTelemetryRecord, build_experiment_plan, execute_single_run,
    run_experiment_sweep,
};
pub use substrate::{CircuitNode, CircuitSubstrate, FastRng};
