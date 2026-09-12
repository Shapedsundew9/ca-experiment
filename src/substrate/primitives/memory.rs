//! High-level excitable memory arrays and addressing structures.
//!
//! Provides modular circuit assembly for:
//! - Addressable dual-rail associative memory slots with active quench shielding
//! - Hierarchical readout reduction trees
//! - Pushdown stack pointer ladders and symbol storage frames

use crate::substrate::builder::CircuitBuilder;
use crate::substrate::primitives::latches::{
    add_retrograde_coupling, attach_ring_inhibition, build_resonant_ring,
};

/// An addressable associative memory slot containing two resonant attractor rings,
/// write coincidence gates, quench interneurons, and query interrogation gates.
#[derive(Debug, Clone, Copy)]
pub struct AssociativeSlot {
    pub ring_0: [usize; 4],
    pub ring_1: [usize; 4],
    pub write_gate_0: usize,
    pub write_gate_1: usize,
    pub quench_0: usize,
    pub quench_1: usize,
    pub query_gate_0: usize,
    pub query_gate_1: usize,
}

/// Parameters for constructing an addressable associative slot.
#[derive(Debug, Clone, Copy)]
pub struct AssociativeSlotParams {
    pub slot_idx: usize,
    pub ring_theta: f64,
    pub gate_theta: f64,
    pub gate_theta_floor: f64,
    pub n_ref: usize,
    pub ring_leak: f64,
    pub w_fwd: f64,
    pub w_fb: f64,
    pub w_inh: f64,
    pub w_sense: f64,
    pub unshielded_ablation: bool,
}

/// Build a 1-bit addressable dual-rail resonant memory slot.
///
/// Under normal operation (`unshielded_ablation = false`), active diode shielding
/// routes write pulses to quench the opposite rail, enforcing strict mutual exclusivity.
/// Under ablation, retrograde reflections and cross-talk are allowed to propagate.
pub fn build_associative_slot(
    builder: &mut CircuitBuilder,
    params: AssociativeSlotParams,
) -> AssociativeSlot {
    let i = params.slot_idx;
    let beta = builder.config.beta_theta;
    let theta_max = builder.config.theta_max;
    let theta_floor = builder.config.theta_floor;
    let rho = builder.config.rho_target;

    // 1. Dual resonant rings (Ring 0 stores bit 0, Ring 1 stores bit 1)
    let ring_0 = build_resonant_ring(
        builder,
        &format!("m{i}_r0"),
        params.ring_theta,
        params.n_ref,
        params.w_fwd,
        params.w_fb,
    );
    let ring_1 = build_resonant_ring(
        builder,
        &format!("m{i}_r1"),
        params.ring_theta,
        params.n_ref,
        params.w_fwd,
        params.w_fb,
    );

    if params.unshielded_ablation {
        add_retrograde_coupling(builder, &ring_0, params.w_fwd);
        add_retrograde_coupling(builder, &ring_1, params.w_fwd);
    }

    // 2. Coincidence Write Gates (high leak prevents multi-cycle accumulation)
    let gw0 = builder.add_node(
        &format!("g_write_{i}_0"),
        params.gate_theta,
        params.gate_theta_floor,
        theta_max,
        beta,
        params.n_ref,
        rho,
        0.85,
    );
    let gw1 = builder.add_node(
        &format!("g_write_{i}_1"),
        params.gate_theta,
        params.gate_theta_floor,
        theta_max,
        beta,
        params.n_ref,
        rho,
        0.85,
    );

    // 3. Quench Interneurons (hyperpolarize opposing ring upon new write)
    let inh0 = builder.add_node(
        &format!("v_inh_{i}_0"),
        params.ring_theta,
        theta_floor,
        theta_max,
        beta,
        params.n_ref,
        rho,
        0.50,
    );
    let inh1 = builder.add_node(
        &format!("v_inh_{i}_1"),
        params.ring_theta,
        theta_floor,
        theta_max,
        beta,
        params.n_ref,
        rho,
        0.50,
    );

    // Gate 0 drives Ring 0 (bit 0); Gate 1 drives Ring 1 (bit 1)
    builder.add_edge(gw0, ring_0[0], params.w_fwd);
    builder.add_edge(gw1, ring_1[0], params.w_fwd);

    if params.unshielded_ablation {
        // Unshielded reflections cross-talk directly into opposite ring
        builder.add_edge(gw0, ring_1[0], params.w_fwd);
        builder.add_edge(gw1, ring_0[0], params.w_fwd);
    } else {
        // Active shielding drives quench interneurons
        builder.add_edge(gw0, inh1, params.w_fwd);
        builder.add_edge(gw1, inh0, params.w_fwd);

        attach_ring_inhibition(builder, inh0, &ring_0, params.w_inh);
        attach_ring_inhibition(builder, inh1, &ring_1, params.w_inh);
    }

    // 4. Query Interrogation Gates (evaluate conjunction of query line and ring state)
    let gq0 = builder.add_node(
        &format!("g_query_{i}_0"),
        params.gate_theta,
        params.gate_theta_floor,
        theta_max,
        beta,
        params.n_ref,
        rho,
        0.85,
    );
    let gq1 = builder.add_node(
        &format!("g_query_{i}_1"),
        params.gate_theta,
        params.gate_theta_floor,
        theta_max,
        beta,
        params.n_ref,
        rho,
        0.85,
    );

    builder.add_edge(ring_0[0], gq0, params.w_sense);
    builder.add_edge(ring_1[0], gq1, params.w_sense);

    AssociativeSlot {
        ring_0,
        ring_1,
        write_gate_0: gw0,
        write_gate_1: gw1,
        quench_0: inh0,
        quench_1: inh1,
        query_gate_0: gq0,
        query_gate_1: gq1,
    }
}

/// Construct a multi-hub readout reduction tree pooling multiple gate outputs into a terminal port.
///
/// Returns `(hub_node_ids, terminal_output_id)`.
#[allow(clippy::too_many_arguments)]
pub fn build_readout_tree(
    builder: &mut CircuitBuilder,
    rail_prefix: &str,
    out_name: &str,
    gate_outputs: &[usize],
    slots_per_hub: usize,
    theta: f64,
    n_ref: usize,
    leak: f64,
    w_fwd: f64,
) -> (Vec<usize>, usize) {
    let beta = builder.config.beta_theta;
    let theta_max = builder.config.theta_max;
    let theta_floor = builder.config.theta_floor;
    let rho = builder.config.rho_target;

    let num_hubs = gate_outputs.len().div_ceil(slots_per_hub);
    let mut hubs = Vec::with_capacity(num_hubs);

    for h in 0..num_hubs {
        let hub = builder.add_node(
            &format!("{rail_prefix}_{h}"),
            theta,
            theta_floor,
            theta_max,
            beta,
            n_ref,
            rho,
            leak,
        );
        hubs.push(hub);

        let start = h * slots_per_hub;
        let end = (start + slots_per_hub).min(gate_outputs.len());
        for &gate_idx in &gate_outputs[start..end] {
            builder.add_edge(gate_idx, hub, w_fwd);
        }
    }

    let terminal = builder.add_node(
        out_name,
        theta,
        theta_floor,
        theta_max,
        beta,
        n_ref,
        rho,
        leak,
    );

    for &hub in &hubs {
        builder.add_edge(hub, terminal, w_fwd);
    }

    (hubs, terminal)
}

/// Construct a cascaded pushdown pointer ladder of resonant rings.
///
/// Returns a vector of 4-node ring IDs for each depth level $k \in [0, \text{depth})$.
pub fn build_pointer_ladder(
    builder: &mut CircuitBuilder,
    depth: usize,
    prefix: &str,
    theta: f64,
    n_ref: usize,
    w_fwd: f64,
    w_fb: f64,
) -> Vec<[usize; 4]> {
    let mut rings = Vec::with_capacity(depth);
    for k in 0..depth {
        let ring =
            build_resonant_ring(builder, &format!("{prefix}_{k}"), theta, n_ref, w_fwd, w_fb);
        rings.push(ring);
    }
    rings
}
