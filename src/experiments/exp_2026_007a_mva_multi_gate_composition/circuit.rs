//! 1-bit Full Adder circuit graph builder for EXP-2026-007a.
//!
//! Constructs the directed spatial circuit graph with excitable LIF nodes,
//! logic primitives (AND, OR, XOR), meander delay equalization tracks,
//! and refractory-shielded / unshielded planar wire crossing.

use super::config::{Condition, ExperimentConfig};
use super::substrate::{CircuitNode, CircuitSubstrate};

pub struct CircuitBuilder {
    config: ExperimentConfig,
    nodes: Vec<CircuitNode>,
    in_neighbors: Vec<Vec<(usize, f64)>>,
}

impl CircuitBuilder {
    pub fn new(config: ExperimentConfig) -> Self {
        Self {
            config,
            nodes: Vec::new(),
            in_neighbors: Vec::new(),
        }
    }

    pub fn add_node(
        &mut self,
        name: &str,
        theta_init: f64,
        theta_floor: f64,
        theta_max: f64,
        beta_theta: f64,
        n_ref: usize,
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
        );
        self.nodes.push(node);
        self.in_neighbors.push(Vec::new());
        id
    }

    pub fn add_edge(&mut self, from: usize, to: usize, weight: f64) {
        self.in_neighbors[to].push((from, weight));
    }
}

/// Build the complete Full Adder substrate graph according to the experimental condition.
pub fn build_full_adder_circuit(config: ExperimentConfig) -> CircuitSubstrate {
    let cond = config.condition;
    let mut builder = CircuitBuilder::new(config.clone());

    let (std_theta_init, std_theta_floor, std_beta_theta) = match cond {
        Condition::ActiveComposed => (1.05, 1.02, 0.05),
        Condition::AblationFixedTheta => (1.00, 0.50, 0.00),
        Condition::BaselineUncompensatedDelay => (1.05, 1.02, 0.05),
        Condition::AblationUnshieldedCrossing => (1.05, 1.02, 0.05),
    };

    let w_fwd = match cond {
        Condition::AblationFixedTheta => 1.00,
        _ => 1.10,
    };

    let (and_w, and_theta) = match cond {
        Condition::AblationFixedTheta => (0.60, 1.10),
        _ => (0.65, 1.15),
    };

    let n_ref_std = config.n_ref; // 2

    // ----------------------------------------------------
    // Stage 1: Sensory Ingress & Fan-Out (t = 0..1)
    // ----------------------------------------------------
    let in_a = builder.add_node(
        "in_a",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let in_b = builder.add_node(
        "in_b",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let in_c = builder.add_node(
        "in_c",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );

    // 1-to-2 Fan-Out Buffers for A and B (fires at t = 1)
    let fork_a = builder.add_node(
        "fork_a",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let fork_b = builder.add_node(
        "fork_b",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(in_a, fork_a, w_fwd);
    builder.add_edge(in_b, fork_b, w_fwd);

    // ----------------------------------------------------
    // Stage 2: Half Adder 1 (HA1: t = 1..3)
    // XOR1 = A ^ B (latency 2 ticks, fires at t = 3)
    // AND1 = A & B (latency 1 tick, fires at t = 2)
    // Meander Delay Y1 delays AND1 by 1 tick (fires at t = 3)
    // ----------------------------------------------------
    let xor1_ha = builder.add_node(
        "xor1_ha",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let xor1_hb = builder.add_node(
        "xor1_hb",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let xor1_out = builder.add_node(
        "xor1_out",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );

    // XOR1 lateral inhibition dual-rail motif
    // xor1_ha: A and not B
    builder.add_edge(fork_a, xor1_ha, w_fwd);
    builder.add_edge(fork_b, xor1_ha, -1.50);
    // xor1_hb: B and not A
    builder.add_edge(fork_b, xor1_hb, w_fwd);
    builder.add_edge(fork_a, xor1_hb, -1.50);
    // xor1_out: OR of ha and hb (ready at t = 3)
    builder.add_edge(xor1_ha, xor1_out, w_fwd);
    builder.add_edge(xor1_hb, xor1_out, w_fwd);

    // AND1: Y1 = A & B (ready at t = 2)
    let and1_out = builder.add_node(
        "and1_out",
        and_theta,
        and_theta.min(std_theta_floor),
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(fork_a, and1_out, and_w);
    builder.add_edge(fork_b, and1_out, and_w);

    // ----------------------------------------------------
    // Meander Delay Tracks for HA1 and Cin Alignment (t = 1..3)
    // ----------------------------------------------------
    let delay_y1 = builder.add_node(
        "delay_y1",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(and1_out, delay_y1, w_fwd);

    // Cin delay track: 3 cells (fires at t=1, t=2, t=3)
    let delay_c1 = builder.add_node(
        "delay_c1",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let delay_c2 = builder.add_node(
        "delay_c2",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let delay_c3 = builder.add_node(
        "delay_c3",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(in_c, delay_c1, w_fwd);
    builder.add_edge(delay_c1, delay_c2, w_fwd);
    builder.add_edge(delay_c2, delay_c3, w_fwd);

    // ----------------------------------------------------
    // Stage 3: Planar Wire Crossing & Post-Crossing Fan-Out (t = 3..5)
    // Intersecting lines: Y1 (eastward) and Cin (northward)
    // ----------------------------------------------------
    let cross_y1 = builder.add_node(
        "cross_y1",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let cross_c = builder.add_node(
        "cross_c",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );

    let cross_shared = if cond == Condition::AblationUnshieldedCrossing {
        Some(builder.add_node(
            "cross_shared",
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            0, // N_ref = 0: unshielded collision node
        ))
    } else {
        None
    };

    // Buffer for X1 to align with HA2 inputs at t = 5
    let delay_x1 = builder.add_node(
        "delay_x1",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let fork_x1 = builder.add_node(
        "fork_x1",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(xor1_out, delay_x1, w_fwd);
    builder.add_edge(delay_x1, fork_x1, w_fwd);

    // Buffer for Cin after crossing (fork_c, fires at t = 5)
    let fork_c = builder.add_node(
        "fork_c",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );

    // Delay tracks for Y1 post-crossing (fires at t = 5, t = 6 to meet OR gate at t = 6)
    let delay_y1_post1 = builder.add_node(
        "delay_y1_post1",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let delay_y1_post2 = builder.add_node(
        "delay_y1_post2",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );

    // Wire crossing connections based on condition
    match cond {
        Condition::ActiveComposed | Condition::AblationFixedTheta => {
            // Shielded planar crossing: Y1 and Cin are completely decoupled
            builder.add_edge(delay_y1, cross_y1, w_fwd);
            builder.add_edge(delay_c3, cross_c, w_fwd);
            builder.add_edge(cross_y1, delay_y1_post1, w_fwd);
            builder.add_edge(cross_c, fork_c, w_fwd);
        }
        Condition::AblationUnshieldedCrossing => {
            // Unshielded 4-way collision node: Y1 and Cin merge into shared node with N_ref = 0
            let shared = cross_shared.unwrap();
            builder.add_edge(delay_y1, shared, w_fwd);
            builder.add_edge(delay_c3, shared, w_fwd);
            // Shared node fires into both downstream paths
            builder.add_edge(shared, delay_y1_post1, w_fwd);
            builder.add_edge(shared, fork_c, w_fwd);
            // 4-way unshielded node reflections
            builder.add_edge(shared, delay_y1, w_fwd);
            builder.add_edge(shared, delay_c3, w_fwd);
        }
        Condition::BaselineUncompensatedDelay => {
            // Meander delay tracks are omitted: Cin arrives out-of-phase (Delta tau = 3 ticks)
            // Cin bypasses delay_c1 and delay_c2, arriving at fork_c 3 ticks early
            builder.add_edge(in_c, fork_c, w_fwd);
            // Y1 delay tracks omitted: and1_out connects directly without delay_y1_post2
            builder.add_edge(and1_out, cross_y1, w_fwd);
            builder.add_edge(cross_y1, delay_y1_post1, w_fwd);
        }
    }

    builder.add_edge(delay_y1_post1, delay_y1_post2, w_fwd);

    // ----------------------------------------------------
    // Stage 4: Half Adder 2 (HA2: t = 5..7)
    // Operands: fork_x1 (X1) and fork_c (Cin)
    // XOR2: Sum = X1 ^ Cin (ready at t = 7)
    // AND2: Y2 = X1 & Cin (ready at t = 6)
    // ----------------------------------------------------
    let xor2_ha = builder.add_node(
        "xor2_ha",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let xor2_hb = builder.add_node(
        "xor2_hb",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let sum_out = builder.add_node(
        "sum_out",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );

    // XOR2 dual-rail lateral inhibition motif
    // xor2_ha: X1 and not Cin
    builder.add_edge(fork_x1, xor2_ha, w_fwd);
    builder.add_edge(fork_c, xor2_ha, -1.50);
    // xor2_hb: Cin and not X1
    builder.add_edge(fork_c, xor2_hb, w_fwd);
    builder.add_edge(fork_x1, xor2_hb, -1.50);
    // sum_out: ready at t = 7
    builder.add_edge(xor2_ha, sum_out, w_fwd);
    builder.add_edge(xor2_hb, sum_out, w_fwd);

    // AND2: Y2 = X1 & Cin (ready at t = 6)
    let and2_out = builder.add_node(
        "and2_out",
        and_theta,
        and_theta.min(std_theta_floor),
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(fork_x1, and2_out, and_w);
    builder.add_edge(fork_c, and2_out, and_w);

    // ----------------------------------------------------
    // Stage 5: Carry-Out Synthesis (OR Gate: t = 6..7)
    // Cout = Y1 | Y2 (ready at t = 7)
    // Operands: delay_y1_post2 (Y1 at t = 6) and and2_out (Y2 at t = 6)
    // ----------------------------------------------------
    let cout_out = builder.add_node(
        "cout_out",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(delay_y1_post2, cout_out, w_fwd);
    builder.add_edge(and2_out, cout_out, w_fwd);

    // For uncompensated baseline: if Y1 delay post tracks were omitted, also wire early Y1 to cout_out
    if cond == Condition::BaselineUncompensatedDelay {
        builder.add_edge(delay_y1_post1, cout_out, w_fwd);
    }

    let cin_post_cross_idx = fork_c;
    let crosstalk_inject_idx = delay_y1;

    CircuitSubstrate::new(
        config,
        builder.nodes,
        builder.in_neighbors,
        in_a,
        in_b,
        in_c,
        crosstalk_inject_idx,
        sum_out,
        cout_out,
        cin_post_cross_idx,
    )
}
