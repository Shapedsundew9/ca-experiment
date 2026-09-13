//! Mechanical parameter calibration: search spaces, objectives, optimizers, and the driver.
//!
//! - [`space`]: [`SearchSpace`] encode/decode between named raw values and `[0,1]^n`.
//! - [`objective`]: [`Objective`] contract taking seeds as an explicit parameter.
//! - [`optimizer`]: ask/tell [`Optimizer`] implementations ([`RandomSearch`],
//!   [`DifferentialEvolution`]).
//! - [`calibrate`]: budgeted driver enforcing a disjoint calibration/validation seed split.

pub mod calibrate;
pub mod objective;
pub mod optimizer;
pub mod space;

pub use calibrate::{
    Budget, CalibrationReport, SearchError, SeedSplit, StopReason, TraceEntry, calibrate,
};
pub use objective::{Evaluation, Objective};
pub use optimizer::{DifferentialEvolution, Optimizer, RandomSearch};
pub use space::{ParamKind, ParamSpec, SearchSpace};
