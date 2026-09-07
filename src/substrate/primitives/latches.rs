//! Bistable resonant latching, recurrent memory rings, and reset inhibition.

use crate::substrate::builder::CircuitBuilder;

/// Build a directed resonant 4-node attractor ring:
/// `R_0 -> R_1 -> R_2 -> R_3 -> R_0` with unidirectional recurrent feedback.
///
/// Returns an array of the 4 ring node indices `[r0, r1, r2, r3]`.
pub fn build_resonant_ring(
    builder: &mut CircuitBuilder,
    prefix: &str,
    theta: f64,
    n_ref: usize,
    forward_weight: f64,
    feedback_weight: f64,
) -> [usize; 4] {
    let r0 = builder.add_node(
        &format!("{prefix}_r0"),
        theta,
        builder.config.theta_floor,
        builder.config.theta_max,
        builder.config.beta_theta,
        n_ref,
        builder.config.rho_target,
        builder.config.leak,
    );
    let r1 = builder.add_node(
        &format!("{prefix}_r1"),
        theta,
        builder.config.theta_floor,
        builder.config.theta_max,
        builder.config.beta_theta,
        n_ref,
        builder.config.rho_target,
        builder.config.leak,
    );
    let r2 = builder.add_node(
        &format!("{prefix}_r2"),
        theta,
        builder.config.theta_floor,
        builder.config.theta_max,
        builder.config.beta_theta,
        n_ref,
        builder.config.rho_target,
        builder.config.leak,
    );
    let r3 = builder.add_node(
        &format!("{prefix}_r3"),
        theta,
        builder.config.theta_floor,
        builder.config.theta_max,
        builder.config.beta_theta,
        n_ref,
        builder.config.rho_target,
        builder.config.leak,
    );

    builder.add_edge(r0, r1, forward_weight);
    builder.add_edge(r1, r2, forward_weight);
    builder.add_edge(r2, r3, forward_weight);
    builder.add_edge(r3, r0, feedback_weight);

    [r0, r1, r2, r3]
}

/// Attach an inhibitory interneuron that quenches activity across all nodes in a resonant ring.
pub fn attach_ring_inhibition(
    builder: &mut CircuitBuilder,
    inhibitory_node: usize,
    ring_nodes: &[usize; 4],
    inhibitory_weight: f64,
) {
    for &r_idx in ring_nodes {
        builder.add_edge(inhibitory_node, r_idx, inhibitory_weight);
    }
}
