//! EXP-2026-002a: Attractor Mapping and Input Separation Protocol
//!
//! Substrate simulation package implementing limit cycle detection, attractor separation,
//! and basin consistency evaluation across distinct temporal input bit-trains.

pub mod attractor;
pub mod patterns;
pub mod runner;
pub mod substrate;
pub mod types;

pub use attractor::{
    composite_distance, detect_period, hamming_distance, occupancy_distance, orbit_distance,
};
pub use patterns::{apply_noise, generate_pattern};
pub use runner::{
    RunResult, SweepMetrics, execute_single_run, reduce_sweep_metrics, save_ndjson_telemetry,
};
pub use substrate::TorusSubstrate;
pub use types::{
    ExperimentConfig, HomeostaticMode, InputPatternId, RunSummaryRecord, SnapshotRecord,
};
