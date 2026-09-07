//! Regenerative signal transmission lines, delay equalizers, and fan-out tracks.

use crate::substrate::builder::CircuitBuilder;

/// Build a fan-out splitter: connects a single input node to multiple output lines.
pub fn build_fan_out(
    builder: &mut CircuitBuilder,
    input_node: usize,
    output_prefixes: &[&str],
    length: usize,
    theta: f64,
    n_ref: usize,
    weight: f64,
) -> Vec<usize> {
    let mut egress_nodes = Vec::with_capacity(output_prefixes.len());
    for prefix in output_prefixes {
        let (ingress, egress) =
            builder.add_transmission_track(prefix, length, theta, n_ref, weight);
        builder.add_edge(input_node, ingress, weight);
        egress_nodes.push(egress);
    }
    egress_nodes
}
