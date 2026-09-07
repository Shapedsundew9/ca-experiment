//! EXP-2026-001a: Empirical Validation of Homeostatic Firing Regulation on 16-Node Torus Automata
//!
//! Substrate simulation package implementing recurrent 2D toroidal cellular automata
//! with local homeostatic threshold adaptation.

pub mod homeostasis;
pub mod node;
pub mod telemetry;
pub mod torus;

pub use homeostasis::AdaptationMode;
pub use node::MicroNode;
pub use telemetry::{ReductionMetrics, RunSummary, TimeSeriesRecord};
pub use torus::{TorusConfig, TorusNetwork};
