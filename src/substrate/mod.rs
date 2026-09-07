//! Shared Computational Substrate Foundation.
//!
//! Provides the invariant cellular automata and excitable neural substrate primitives:
//! - [`rng`]: Deterministic [`FastRng`] based on Xoshiro256++.
//! - [`node`]: Discrete homeostatic Leaky Integrate-and-Fire [`CircuitNode`].
//! - [`config`]: Common [`SubstrateConfig`] parameter specifications.
//! - [`network`]: Directed graph [`NetworkSubstrate`] with synaptic propagation and noise.
//! - [`stats`]: Statistical hypothesis testing (`welch_t_test`, `sample_mean`, `sample_std`, `cohen_d`).
//! - [`builder`]: Declarative [`CircuitBuilder`] for network topology assembly.
//! - [`primitives`]: Reusable circuit building blocks (transmission tracks, logic gates, resonant rings).

pub mod builder;
pub mod config;
pub mod network;
pub mod node;
pub mod primitives;
pub mod rng;
pub mod stats;

pub use builder::CircuitBuilder;
pub use config::SubstrateConfig;
pub use network::NetworkSubstrate;
pub use node::CircuitNode;
pub use rng::FastRng;

pub mod prelude {
    pub use super::builder::CircuitBuilder;
    pub use super::config::SubstrateConfig;
    pub use super::network::NetworkSubstrate;
    pub use super::node::CircuitNode;
    pub use super::primitives::*;
    pub use super::rng::FastRng;
    pub use super::stats::*;
}
