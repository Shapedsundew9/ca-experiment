//! Circuit graph builder for EXP-2026-011a: Associative Key-Value Retrieval ("Needle in a Haystack").
//!
//! Constructs the 272-node directed excitable circuit graph under ADR-0001:
//! - 16 Addressable dual-resonant memory slots (128 nodes)
//! - Coincidence write gates and quench interneurons (64 nodes)
//! - Coincidence query interrogation gates (32 nodes)
//! - Streaming ingress demultiplexing ports (35 nodes)
//! - Dual-rail readout reduction trees (10 nodes)
//! - Distractor dissipation sink (3 nodes)
//!
//! Total nodes: exactly 272 nodes.
//! Strict degree compliance: k_in <= 4, k_out <= 4 across all internal graph nodes.

use super::config::{Condition, ExperimentConfig};
use super::substrate::CircuitSubstrate;
pub use crate::substrate::builder::CircuitBuilder;
use crate::substrate::config::SubstrateConfig;
use crate::substrate::primitives::memory::{
    AssociativeSlotParams, build_associative_slot, build_readout_tree,
};

pub fn build_circuit(config: ExperimentConfig) -> CircuitSubstrate {
    let cond = config.condition;

    let sub_cfg = SubstrateConfig {
        seed: config.seed.wrapping_add(0xCA_2026_011A),
        noise_rate: config.noise_rate,
        noise_mean: 0.40,
        noise_std: 0.03,
        leak: config.leak,
        n_ref: config.n_ref,
        alpha_rho: config.alpha_rho,
        rho_target: config.rho_target,
        beta_theta: config.beta_theta,
        theta_min: config.theta_floor,
        theta_max: config.theta_max,
        theta_floor: config.theta_floor,
        theta_init: config.theta_init,
    };

    let mut builder = CircuitBuilder::new(sub_cfg);

    let n_ref_std = config.n_ref;
    let ring_theta_init = config.theta_init;
    let gate_theta_init = config.theta_gate_init;
    let gate_theta_floor = match cond {
        Condition::AblationFixedTheta => 1.00,
        _ => 1.10,
    };

    let w_fwd = config.w_fwd;
    let w_fb = config.w_fb;
    let w_inh = config.w_inh;
    let w_key = config.w_key;
    let w_val = config.w_val;
    let w_query = config.w_query;
    let w_sense = config.w_sense;
    let w_dist = config.w_dist;

    // -------------------------------------------------------------------------
    // 1. Addressable Memory Array (16 slots x 8 nodes = 128 nodes)
    //    Write Coincidence Gates & Quench Interneurons (16 x 4 = 64 nodes)
    //    Query Interrogation Gating Stage (16 x 2 = 32 nodes)
    // -------------------------------------------------------------------------
    let unshielded = cond == Condition::AblationUnshieldedRetrieval;
    let mut slot_rings = Vec::with_capacity(16);
    let mut write_gates = Vec::with_capacity(16);
    let mut quench_nodes = Vec::with_capacity(16);
    let mut query_gates = Vec::with_capacity(16);

    for i in 0..16 {
        let slot = build_associative_slot(
            &mut builder,
            AssociativeSlotParams {
                slot_idx: i,
                ring_theta: ring_theta_init,
                gate_theta: gate_theta_init,
                gate_theta_floor,
                n_ref: n_ref_std,
                ring_leak: config.leak,
                w_fwd,
                w_fb,
                w_inh,
                w_sense,
                unshielded_ablation: unshielded,
            },
        );

        slot_rings.push((slot.ring_0, slot.ring_1));
        write_gates.push((slot.write_gate_0, slot.write_gate_1));
        quench_nodes.push((slot.quench_0, slot.quench_1));
        query_gates.push((slot.query_gate_0, slot.query_gate_1));
    }

    // -------------------------------------------------------------------------
    // 2. Streaming Ingress Demultiplexing Ports (16 + 2 + 1 + 16 = 35 nodes)
    // -------------------------------------------------------------------------
    let mut key_ports = [0usize; 16];
    for i in 0..16 {
        let kp = builder.add_node_simple(&format!("v_k_{i}"), ring_theta_init, 0, 0.50);
        key_ports[i] = kp;
        builder.add_edge(kp, write_gates[i].0, w_key);
        builder.add_edge(kp, write_gates[i].1, w_key);
    }

    let val_0 = builder.add_node_simple("v_val_0", ring_theta_init, 0, 0.50);
    let val_1 = builder.add_node_simple("v_val_1", ring_theta_init, 0, 0.50);
    let val_ports = [val_0, val_1];

    for &(wg0, wg1) in &write_gates {
        builder.add_edge(val_0, wg0, w_val);
        builder.add_edge(val_1, wg1, w_val);
    }

    let dist_port = builder.add_node_simple("v_dist", ring_theta_init, 0, 0.50);
    for &(wg0, wg1) in &write_gates {
        builder.add_edge(dist_port, wg0, w_dist);
        builder.add_edge(dist_port, wg1, w_dist);
    }

    let mut query_ports = [0usize; 16];
    for j in 0..16 {
        let qp = builder.add_node_simple(&format!("v_q_{j}"), ring_theta_init, 0, 0.50);
        query_ports[j] = qp;
        builder.add_edge(qp, query_gates[j].0, w_query);
        builder.add_edge(qp, query_gates[j].1, w_query);
    }

    // -------------------------------------------------------------------------
    // 3. Shared Dual-Rail Readout Reduction Trees (5 + 5 = 10 nodes)
    // -------------------------------------------------------------------------
    let query_gates_0: Vec<usize> = query_gates.iter().map(|g| g.0).collect();
    let query_gates_1: Vec<usize> = query_gates.iter().map(|g| g.1).collect();

    let (hubs_0, v_out_0) = build_readout_tree(
        &mut builder,
        "hub_0",
        "v_out_0",
        &query_gates_0,
        4,
        ring_theta_init,
        n_ref_std,
        0.50,
        w_fwd,
    );

    let (hubs_1, v_out_1) = build_readout_tree(
        &mut builder,
        "hub_1",
        "v_out_1",
        &query_gates_1,
        4,
        ring_theta_init,
        n_ref_std,
        0.50,
        w_fwd,
    );

    let hubs_0_arr: [usize; 4] = hubs_0.try_into().expect("Hubs 0 must have length 4");
    let hubs_1_arr: [usize; 4] = hubs_1.try_into().expect("Hubs 1 must have length 4");

    // -------------------------------------------------------------------------
    // 4. Distractor Dissipation Sink (3 nodes)
    // -------------------------------------------------------------------------
    let sink_0 = builder.add_node_simple("sink_0", ring_theta_init, n_ref_std, config.leak);
    let sink_1 = builder.add_node_simple("sink_1", ring_theta_init, n_ref_std, config.leak);
    let sink_2 = builder.add_node_simple("sink_2", ring_theta_init, n_ref_std, config.leak);

    builder.add_edge(dist_port, sink_0, w_dist);
    builder.add_edge(sink_0, sink_1, w_dist);
    builder.add_edge(sink_1, sink_2, w_dist);
    let sink_nodes = [sink_0, sink_1, sink_2];

    assert_eq!(
        builder.nodes.len(),
        272,
        "Circuit graph must contain exactly 272 excitable nodes"
    );

    builder
        .verify_degree_invariants(4)
        .expect("Degree invariant violated");

    let net = builder.build();

    CircuitSubstrate::new(
        config,
        net,
        key_ports,
        val_ports,
        dist_port,
        query_ports,
        v_out_0,
        v_out_1,
        slot_rings,
        write_gates,
        quench_nodes,
        query_gates,
        sink_nodes,
        (hubs_0_arr, hubs_1_arr),
    )
}
