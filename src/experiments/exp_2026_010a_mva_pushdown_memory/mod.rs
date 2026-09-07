//! Experiment EXP-2026-010a: Pushdown Memory and Context-Free Dyck Language Recognition via Cascaded Dynamical Pushdown Stack Cells.

pub mod circuit;
pub mod config;
pub mod metrics;
pub mod runner;
pub mod substrate;

pub use circuit::{CircuitBuilder, build_circuit};
pub use config::{AutomatonTask, Condition, ExperimentConfig};
pub use metrics::{RunMetrics, sample_mean, sample_std, welch_t_test};
pub use runner::{
    EvaluationSequence, EvaluationSummaryManifest, RunTelemetryRecord, SequenceTrialRecord, Token,
    evaluate_sequence_trial, execute_factorial_sweep, execute_run, generate_depth_dataset,
    generate_dyck1_dataset, generate_dyck2_dataset,
};
pub use substrate::{CircuitNode, CircuitSubstrate, FastRng, IngressInputs};
