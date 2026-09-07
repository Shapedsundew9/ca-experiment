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
use super::substrate::{CircuitNode, CircuitSubstrate};

pub struct CircuitBuilder {
    config: ExperimentConfig,
    pub nodes: Vec<CircuitNode>,
    pub in_neighbors: Vec<Vec<(usize, f64)>>,
}

impl CircuitBuilder {
    pub fn new(config: ExperimentConfig) -> Self {
        Self {
            config,
            nodes: Vec::new(),
            in_neighbors: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_node(
        &mut self,
        name: &str,
        theta_init: f64,
        theta_floor: f64,
        theta_max: f64,
        beta_theta: f64,
        n_ref: usize,
        leak: f64,
    ) -> usize {
        let id = self.nodes.len();
        let node = CircuitNode::new(
            id,
            name.to_string(),
            theta_init,
            theta_floor,
            theta_max,
            beta_theta,
            n_ref,
            self.config.rho_target,
            leak,
        );
        self.nodes.push(node);
        self.in_neighbors.push(Vec::new());
        id
    }

    pub fn add_edge(&mut self, from: usize, to: usize, weight: f64) {
        if to < self.in_neighbors.len() {
            self.in_neighbors[to].push((from, weight));
        }
    }
}

#[allow(clippy::needless_range_loop)]
pub fn build_circuit(config: ExperimentConfig) -> CircuitSubstrate {
    let mut builder = CircuitBuilder::new(config.clone());
    let cond = config.condition;

    let n_ref_std = config.n_ref;
    let beta_theta_std = config.beta_theta;
    let theta_floor_std = config.theta_floor;
    let theta_max_std = config.theta_max;

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
    // -------------------------------------------------------------------------
    let mut slot_rings = Vec::with_capacity(16);

    for i in 0..16 {
        // Ring 0 (stores bit 0)
        let r0_0 = builder.add_node(
            &format!("m{i}_r0_0"),
            ring_theta_init,
            theta_floor_std,
            theta_max_std,
            beta_theta_std,
            n_ref_std,
            config.leak,
        );
        let r0_1 = builder.add_node(
            &format!("m{i}_r0_1"),
            ring_theta_init,
            theta_floor_std,
            theta_max_std,
            beta_theta_std,
            n_ref_std,
            config.leak,
        );
        let r0_2 = builder.add_node(
            &format!("m{i}_r0_2"),
            ring_theta_init,
            theta_floor_std,
            theta_max_std,
            beta_theta_std,
            n_ref_std,
            config.leak,
        );
        let r0_3 = builder.add_node(
            &format!("m{i}_r0_3"),
            ring_theta_init,
            theta_floor_std,
            theta_max_std,
            beta_theta_std,
            n_ref_std,
            config.leak,
        );

        // Forward ring loop 0
        builder.add_edge(r0_0, r0_1, w_fwd);
        builder.add_edge(r0_1, r0_2, w_fwd);
        builder.add_edge(r0_2, r0_3, w_fwd);
        builder.add_edge(r0_3, r0_0, w_fb);

        // Retrograde reflections in unshielded ablation
        if cond == Condition::AblationUnshieldedRetrieval {
            builder.add_edge(r0_1, r0_0, w_fwd);
            builder.add_edge(r0_2, r0_1, w_fwd);
            builder.add_edge(r0_3, r0_2, w_fwd);
            builder.add_edge(r0_0, r0_3, w_fwd);
        }

        // Ring 1 (stores bit 1)
        let r1_0 = builder.add_node(
            &format!("m{i}_r1_0"),
            ring_theta_init,
            theta_floor_std,
            theta_max_std,
            beta_theta_std,
            n_ref_std,
            config.leak,
        );
        let r1_1 = builder.add_node(
            &format!("m{i}_r1_1"),
            ring_theta_init,
            theta_floor_std,
            theta_max_std,
            beta_theta_std,
            n_ref_std,
            config.leak,
        );
        let r1_2 = builder.add_node(
            &format!("m{i}_r1_2"),
            ring_theta_init,
            theta_floor_std,
            theta_max_std,
            beta_theta_std,
            n_ref_std,
            config.leak,
        );
        let r1_3 = builder.add_node(
            &format!("m{i}_r1_3"),
            ring_theta_init,
            theta_floor_std,
            theta_max_std,
            beta_theta_std,
            n_ref_std,
            config.leak,
        );

        // Forward ring loop 1
        builder.add_edge(r1_0, r1_1, w_fwd);
        builder.add_edge(r1_1, r1_2, w_fwd);
        builder.add_edge(r1_2, r1_3, w_fwd);
        builder.add_edge(r1_3, r1_0, w_fb);

        if cond == Condition::AblationUnshieldedRetrieval {
            builder.add_edge(r1_1, r1_0, w_fwd);
            builder.add_edge(r1_2, r1_1, w_fwd);
            builder.add_edge(r1_3, r1_2, w_fwd);
            builder.add_edge(r1_0, r1_3, w_fwd);
        }

        slot_rings.push(([r0_0, r0_1, r0_2, r0_3], [r1_0, r1_1, r1_2, r1_3]));
    }

    // -------------------------------------------------------------------------
    // 2. Write Coincidence Gates & Quench Interneurons (16 x 4 = 64 nodes)
    // -------------------------------------------------------------------------
    let mut write_gates = Vec::with_capacity(16);
    let mut quench_nodes = Vec::with_capacity(16);

    for i in 0..16 {
        // Write Gate 0
        let gw0 = builder.add_node(
            &format!("g_write_{i}_0"),
            gate_theta_init,
            gate_theta_floor,
            theta_max_std,
            beta_theta_std,
            n_ref_std,
            0.85, // Fast leak prevents multi-cycle accumulation on solitary inputs
        );

        // Write Gate 1
        let gw1 = builder.add_node(
            &format!("g_write_{i}_1"),
            gate_theta_init,
            gate_theta_floor,
            theta_max_std,
            beta_theta_std,
            n_ref_std,
            0.85,
        );

        // Quench Interneuron 0 (quenches Ring 0 when Bit 1 is written)
        let inh0 = builder.add_node(
            &format!("v_inh_{i}_0"),
            ring_theta_init,
            theta_floor_std,
            theta_max_std,
            beta_theta_std,
            n_ref_std,
            0.50,
        );

        // Quench Interneuron 1 (quenches Ring 1 when Bit 0 is written)
        let inh1 = builder.add_node(
            &format!("v_inh_{i}_1"),
            ring_theta_init,
            theta_floor_std,
            theta_max_std,
            beta_theta_std,
            n_ref_std,
            0.50,
        );

        // Gate 0 drives Ring 0, node 0 and Quench Interneuron 1
        builder.add_edge(gw0, slot_rings[i].0[0], w_fwd);
        if cond == Condition::AblationUnshieldedRetrieval {
            builder.add_edge(gw0, slot_rings[i].1[0], w_fwd);
        } else {
            builder.add_edge(gw0, inh1, w_fwd);
        }

        // Gate 1 drives Ring 1, node 0 and Quench Interneuron 0
        builder.add_edge(gw1, slot_rings[i].1[0], w_fwd);
        if cond == Condition::AblationUnshieldedRetrieval {
            builder.add_edge(gw1, slot_rings[i].0[0], w_fwd);
        } else {
            builder.add_edge(gw1, inh0, w_fwd);
        }

        // Quench hyperpolarization (active diode shielding preserves mutual exclusivity)
        if cond != Condition::AblationUnshieldedRetrieval {
            for &r0_idx in &slot_rings[i].0 {
                builder.add_edge(inh0, r0_idx, w_inh);
            }

            for &r1_idx in &slot_rings[i].1 {
                builder.add_edge(inh1, r1_idx, w_inh);
            }
        }

        write_gates.push((gw0, gw1));
        quench_nodes.push((inh0, inh1));
    }

    // -------------------------------------------------------------------------
    // 3. Query Interrogation Gating Stage (16 x 2 = 32 nodes)
    // -------------------------------------------------------------------------
    let mut query_gates = Vec::with_capacity(16);

    for j in 0..16 {
        // Query Gate 0: evaluates conjunction of query line and Ring 0, node 0
        let gq0 = builder.add_node(
            &format!("g_query_{j}_0"),
            gate_theta_init,
            gate_theta_floor,
            theta_max_std,
            beta_theta_std,
            n_ref_std,
            0.85,
        );
        builder.add_edge(slot_rings[j].0[0], gq0, w_sense);

        // Query Gate 1: evaluates conjunction of query line and Ring 1, node 0
        let gq1 = builder.add_node(
            &format!("g_query_{j}_1"),
            gate_theta_init,
            gate_theta_floor,
            theta_max_std,
            beta_theta_std,
            n_ref_std,
            0.85,
        );
        builder.add_edge(slot_rings[j].1[0], gq1, w_sense);

        query_gates.push((gq0, gq1));
    }

    // -------------------------------------------------------------------------
    // 4. Streaming Ingress Demultiplexing Ports (16 + 2 + 1 + 16 = 35 nodes)
    // -------------------------------------------------------------------------
    let mut key_ports = [0usize; 16];
    for i in 0..16 {
        let kp = builder.add_node(
            &format!("v_k_{i}"),
            ring_theta_init,
            theta_floor_std,
            theta_max_std,
            beta_theta_std,
            0, // Ingress sensory ports have n_ref = 0 to track external input trains
            0.50,
        );
        key_ports[i] = kp;

        // Key line drives Write Gate 0 and Write Gate 1
        builder.add_edge(kp, write_gates[i].0, w_key);
        builder.add_edge(kp, write_gates[i].1, w_key);
    }

    // Value lines v_val_0 and v_val_1
    let val_0 = builder.add_node(
        "v_val_0",
        ring_theta_init,
        theta_floor_std,
        theta_max_std,
        beta_theta_std,
        0,
        0.50,
    );
    let val_1 = builder.add_node(
        "v_val_1",
        ring_theta_init,
        theta_floor_std,
        theta_max_std,
        beta_theta_std,
        0,
        0.50,
    );
    let val_ports = [val_0, val_1];

    for i in 0..16 {
        builder.add_edge(val_0, write_gates[i].0, w_val);
        builder.add_edge(val_1, write_gates[i].1, w_val);
    }

    // Distractor line v_dist
    let dist_port = builder.add_node(
        "v_dist",
        ring_theta_init,
        theta_floor_std,
        theta_max_std,
        beta_theta_std,
        0,
        0.50,
    );

    // Parasitic coupling from distractor line to write gates
    for i in 0..16 {
        builder.add_edge(dist_port, write_gates[i].0, w_dist);
        builder.add_edge(dist_port, write_gates[i].1, w_dist);
    }

    // Query lines v_q_0 .. v_q_15
    let mut query_ports = [0usize; 16];
    for j in 0..16 {
        let qp = builder.add_node(
            &format!("v_q_{j}"),
            ring_theta_init,
            theta_floor_std,
            theta_max_std,
            beta_theta_std,
            0,
            0.50,
        );
        query_ports[j] = qp;

        // Query line drives Query Gate 0 and Query Gate 1 for slot j
        builder.add_edge(qp, query_gates[j].0, w_query);
        builder.add_edge(qp, query_gates[j].1, w_query);
    }

    // -------------------------------------------------------------------------
    // 5. Shared Dual-Rail Readout Reduction Trees (5 + 5 = 10 nodes)
    // -------------------------------------------------------------------------
    // Rail 0: 4 Hubs pooling 4 query gates each, driving terminal v_out_0
    let mut hubs_0 = [0usize; 4];
    for h in 0..4 {
        let hub = builder.add_node(
            &format!("hub_0_{h}"),
            ring_theta_init,
            theta_floor_std,
            theta_max_std,
            beta_theta_std,
            n_ref_std,
            0.50,
        );
        hubs_0[h] = hub;
        for m in 0..4 {
            let slot_idx = h * 4 + m;
            builder.add_edge(query_gates[slot_idx].0, hub, w_fwd);
        }
    }
    let v_out_0 = builder.add_node(
        "v_out_0",
        ring_theta_init,
        theta_floor_std,
        theta_max_std,
        beta_theta_std,
        n_ref_std,
        0.50,
    );
    for &hub in &hubs_0 {
        builder.add_edge(hub, v_out_0, w_fwd);
    }

    // Rail 1: 4 Hubs pooling 4 query gates each, driving terminal v_out_1
    let mut hubs_1 = [0usize; 4];
    for h in 0..4 {
        let hub = builder.add_node(
            &format!("hub_1_{h}"),
            ring_theta_init,
            theta_floor_std,
            theta_max_std,
            beta_theta_std,
            n_ref_std,
            0.50,
        );
        hubs_1[h] = hub;
        for m in 0..4 {
            let slot_idx = h * 4 + m;
            builder.add_edge(query_gates[slot_idx].1, hub, w_fwd);
        }
    }
    let v_out_1 = builder.add_node(
        "v_out_1",
        ring_theta_init,
        theta_floor_std,
        theta_max_std,
        beta_theta_std,
        n_ref_std,
        0.50,
    );
    for &hub in &hubs_1 {
        builder.add_edge(hub, v_out_1, w_fwd);
    }

    // -------------------------------------------------------------------------
    // 6. Distractor Dissipation Sink (3 nodes)
    // -------------------------------------------------------------------------
    let sink_0 = builder.add_node(
        "sink_0",
        ring_theta_init,
        theta_floor_std,
        theta_max_std,
        beta_theta_std,
        n_ref_std,
        config.leak,
    );
    let sink_1 = builder.add_node(
        "sink_1",
        ring_theta_init,
        theta_floor_std,
        theta_max_std,
        beta_theta_std,
        n_ref_std,
        config.leak,
    );
    let sink_2 = builder.add_node(
        "sink_2",
        ring_theta_init,
        theta_floor_std,
        theta_max_std,
        beta_theta_std,
        n_ref_std,
        config.leak,
    );

    builder.add_edge(dist_port, sink_0, w_dist);
    builder.add_edge(sink_0, sink_1, w_dist);
    builder.add_edge(sink_1, sink_2, w_dist);
    let sink_nodes = [sink_0, sink_1, sink_2];

    assert_eq!(
        builder.nodes.len(),
        272,
        "Circuit graph must contain exactly 272 excitable nodes"
    );

    // Verify bounded degree constraints on internal nodes
    for (i, in_edges) in builder.in_neighbors.iter().enumerate() {
        assert!(
            in_edges.len() <= 4,
            "Node {} ({}) exceeds max fan-in k_in <= 4 (got {})",
            i,
            builder.nodes[i].name,
            in_edges.len()
        );
    }

    CircuitSubstrate::new(
        config,
        builder.nodes,
        builder.in_neighbors,
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
        (hubs_0, hubs_1),
    )
}
