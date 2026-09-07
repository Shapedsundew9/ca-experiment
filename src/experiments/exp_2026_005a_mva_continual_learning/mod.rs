//! Experiment EXP-2026-005a: Lifelong Adaptation and Continual Multi-Pattern Learning.
//!
//! Investigates local activity-dependent synaptic metaplasticity on a 16-node 2D Torus
//! Minimal Viable Automaton (MVA) substrate under a sequential curriculum (A -> B -> C).

pub mod config;
pub mod curriculum;
pub mod metrics;
pub mod runner;
pub mod substrate;

pub use config::{Condition, ExperimentConfig};
pub use curriculum::{FastRng, NUM_NODES, PatternId};
pub use metrics::{
    compute_subspace_overlap, hamming_dist, measure_basin_depth, probe_occupancy, welch_t_test,
};
pub use runner::{
    ConditionSummary, RunMetricsRecord, RunTelemetryRecord, SnapshotBasins, Snapshots,
    StatTestSummary, SummaryManifest, emit_telemetry, generate_factorial_configs,
    run_factorial_sweep, run_single_experiment,
};
pub use substrate::{TorusNode, TorusSubstrate};
