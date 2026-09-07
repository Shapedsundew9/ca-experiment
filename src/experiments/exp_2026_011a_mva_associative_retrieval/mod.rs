//! Experiment EXP-2026-011a: Associative Key-Value Retrieval ("Needle in a Haystack") via Resonant Attractor Arrays.

pub mod circuit;
pub mod config;
pub mod metrics;
pub mod runner;
pub mod substrate;

pub use circuit::{CircuitBuilder, build_circuit};
pub use config::{Condition, ExperimentConfig};
pub use metrics::{RunMetrics, sample_mean, sample_std, welch_t_test};
pub use runner::{
    EvaluationSequence, EvaluationSummaryManifest, RunTelemetryRecord, SequenceTrialRecord, Token,
    evaluate_sequence_trial, execute_factorial_sweep, execute_run, generate_dataset,
};
pub use substrate::{CircuitNode, CircuitSubstrate, FastRng, IngressInputs};
