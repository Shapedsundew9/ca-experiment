//! Discrete homeostatic Leaky Integrate-and-Fire (LIF) node model.
//!
//! Models an excitable somatic element with passive subthreshold leak, absolute refractory
//! quenching, all-or-none spike emission, and dynamic homeostatic threshold adaptation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitNode {
    pub id: usize,
    pub name: String,
    pub v: f64,
    pub leak: f64,
    pub refractory_counter: usize,
    pub n_ref: usize,
    pub rolling_rate: f64,
    pub theta: f64,
    pub theta_floor: f64,
    pub theta_max: f64,
    pub beta_theta: f64,
    pub spike: f64,
}

impl CircuitNode {
    /// Instantiate an excitable circuit node with explicit homeostatic parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: usize,
        name: String,
        theta_init: f64,
        theta_floor: f64,
        theta_max: f64,
        beta_theta: f64,
        n_ref: usize,
        rho_target: f64,
        leak: f64,
    ) -> Self {
        Self {
            id,
            name,
            v: 0.0,
            leak,
            refractory_counter: 0,
            n_ref,
            rolling_rate: rho_target,
            theta: theta_init,
            theta_floor,
            theta_max,
            beta_theta,
            spike: 0.0,
        }
    }

    /// Check if the node is currently within its absolute refractory period.
    #[inline]
    pub fn is_refractory(&self) -> bool {
        self.refractory_counter > 0
    }

    /// Reset node state (voltage, spike, and refractory counter) to resting conditions.
    pub fn reset_state(&mut self) {
        self.v = 0.0;
        self.spike = 0.0;
        self.refractory_counter = 0;
    }
}
