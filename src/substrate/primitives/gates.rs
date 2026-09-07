//! Discrete excitable logic gates: Coincidence (AND), Inverter (NOT), OR, and XOR.

use crate::substrate::builder::CircuitBuilder;

/// Build an excitable 2-input coincidence gate (AND gate).
///
/// Both input lines must arrive within the coincidence window to cross the somatic threshold.
pub fn build_and_gate(
    builder: &mut CircuitBuilder,
    name: &str,
    in1: usize,
    in2: usize,
    theta: f64,
    n_ref: usize,
    weight: f64,
) -> usize {
    let gate_node = builder.add_node(
        name,
        theta,
        builder.config.theta_floor,
        builder.config.theta_max,
        builder.config.beta_theta,
        n_ref,
        builder.config.rho_target,
        builder.config.leak,
    );
    builder.add_edge(in1, gate_node, weight);
    builder.add_edge(in2, gate_node, weight);
    gate_node
}

/// Build an excitable OR gate.
///
/// Either input line crossing threshold causes the gate node to fire.
pub fn build_or_gate(
    builder: &mut CircuitBuilder,
    name: &str,
    in1: usize,
    in2: usize,
    theta: f64,
    n_ref: usize,
    weight: f64,
) -> usize {
    let gate_node = builder.add_node(
        name,
        theta,
        builder.config.theta_floor,
        builder.config.theta_max,
        builder.config.beta_theta,
        n_ref,
        builder.config.rho_target,
        builder.config.leak,
    );
    builder.add_edge(in1, gate_node, weight);
    builder.add_edge(in2, gate_node, weight);
    gate_node
}

/// Build an inhibitory inverter (NOT gate).
///
/// A tonic driver or bias line is inhibited by the input line with a negative synaptic weight.
#[allow(clippy::too_many_arguments)]
pub fn build_inverter(
    builder: &mut CircuitBuilder,
    name: &str,
    input_node: usize,
    bias_node: usize,
    theta: f64,
    n_ref: usize,
    inhibitory_weight: f64,
    excitatory_weight: f64,
) -> usize {
    let gate_node = builder.add_node(
        name,
        theta,
        builder.config.theta_floor,
        builder.config.theta_max,
        builder.config.beta_theta,
        n_ref,
        builder.config.rho_target,
        builder.config.leak,
    );
    builder.add_edge(bias_node, gate_node, excitatory_weight);
    builder.add_edge(input_node, gate_node, inhibitory_weight);
    gate_node
}
