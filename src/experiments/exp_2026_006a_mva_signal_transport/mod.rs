//! EXP-2026-006a: Directed Regenerative Transmission Tracks and Branching Fan-Out.
//!
//! Milestone 1.1: Signal Transport & Fan-Out (Tier 1: Spatial Routing & Compositionality).

pub mod config;
pub mod metrics;
pub mod runner;
pub mod signals;
pub mod substrate;

pub use config::{Condition, ExperimentConfig, SignalPattern};
pub use metrics::{RunMetrics, welch_t_test};
pub use runner::{build_experiment_plan, execute_single_run, run_experiment_sweep};
pub use signals::{FastRng, generate_input_stream, generate_perturbation_stream};
pub use substrate::{TransportNode, TransportSubstrate, TransportTopology};
