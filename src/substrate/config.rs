//! General configuration for substrate simulations and dynamical parameters.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstrateConfig {
    pub seed: u64,
    pub noise_rate: f64,
    pub leak: f64,
    pub n_ref: usize,
    pub alpha_rho: f64,
    pub rho_target: f64,
    pub beta_theta: f64,
    pub theta_min: f64,
    pub theta_max: f64,
    pub theta_floor: f64,
    pub theta_init: f64,
}

impl Default for SubstrateConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            noise_rate: 0.0,
            leak: 0.10,
            n_ref: 2,
            alpha_rho: 0.02,
            rho_target: 0.10,
            beta_theta: 0.05,
            theta_min: 0.50,
            theta_max: 2.50,
            theta_floor: 1.02,
            theta_init: 1.05,
        }
    }
}
