//! Experiment EXP-2026-009a: Finite State Automata and Regular Language Recognition via Coupled Resonant Attractor Basins.

pub mod circuit;
pub mod config;
pub mod metrics;
pub mod runner;
pub mod substrate;

pub use circuit::{build_circuit, build_parity_circuit, build_regex_circuit};
pub use config::{AutomatonTask, Condition, ExperimentConfig};
pub use metrics::{RunMetrics, sample_mean, sample_std, welch_t_test};
pub use runner::{
    EvaluationSequence, EvaluationSummaryManifest, RunTelemetryRecord, SequenceTrialRecord,
    evaluate_sequence_trial, execute_factorial_sweep, execute_run, generate_parity_dataset,
    generate_regex_dataset,
};
pub use substrate::{CircuitNode, CircuitSubstrate, FastRng};
