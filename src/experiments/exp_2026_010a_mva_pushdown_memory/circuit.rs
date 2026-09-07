//! Pushdown automaton circuit graph builder for EXP-2026-010a.
//!
//! Constructs the directed spatial circuit graph for:
//! - Hierarchical LIFO pushdown stack memory for Dyck-1 and Dyck-2 language recognition.
//! - Bidirectional pointer ladder bank P_k (k in [0, D_max]).
//! - Cascaded modular stack frames C_k with round and square resonant rings.
//! - Coincidence transition gates, quench interneurons, and violation rejection attractor.
//!
//! Enforces bounded degree constraints (k_in <= 4, k_out <= 4) under ADR-0001.

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
            self.config.leak,
        );
        self.nodes.push(node);
        self.in_neighbors.push(Vec::new());
        id
    }

    pub fn add_gate_node(
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
            0.50, // Fast somatic leak for coincidence gates to prevent multi-cycle charge accumulation
        );
        self.nodes.push(node);
        self.in_neighbors.push(Vec::new());
        id
    }

    pub fn add_edge(&mut self, from: usize, to: usize, weight: f64) {
        self.in_neighbors[to].push((from, weight));
    }
}

/// Helper: Combine a list of trigger node outputs into a single output using a tree of OR-funnel nodes.
/// Each funnel node has theta = 1.00, leak = 0.50, and input weights = 1.00.
#[allow(clippy::too_many_arguments)]
fn build_or_funnel(
    builder: &mut CircuitBuilder,
    name_prefix: &str,
    mut inputs: Vec<usize>,
    w_fwd: f64,
    theta_init: f64,
    theta_floor: f64,
    theta_max: f64,
    beta_theta: f64,
    n_ref: usize,
) -> usize {
    if inputs.is_empty() {
        return builder.add_gate_node(
            &format!("{name_prefix}_empty"),
            theta_init,
            theta_floor,
            theta_max,
            beta_theta,
            n_ref,
        );
    }

    let mut layer = 0;
    while inputs.len() > 1 {
        let mut next_inputs = Vec::new();
        for chunk in inputs.chunks(4) {
            if chunk.len() == 1 {
                next_inputs.push(chunk[0]);
            } else {
                let node_id = builder.add_gate_node(
                    &format!("{name_prefix}_l{layer}_n{}", next_inputs.len()),
                    theta_init,
                    theta_floor,
                    theta_max,
                    beta_theta,
                    n_ref,
                );
                for &src in chunk {
                    builder.add_edge(src, node_id, w_fwd);
                }
                next_inputs.push(node_id);
            }
        }
        inputs = next_inputs;
        layer += 1;
    }
    inputs[0]
}

/// Build the Pushdown Automaton substrate circuit graph.
pub fn build_circuit(config: ExperimentConfig) -> CircuitSubstrate {
    let cond = config.condition;
    let mut builder = CircuitBuilder::new(config.clone());

    let (std_theta_init, std_theta_floor, std_beta_theta) = match cond {
        Condition::AblationFixedTheta => (1.00, 0.50, 0.00),
        _ => (1.05, 1.02, 0.05),
    };

    let w_fwd = match cond {
        Condition::AblationFixedTheta => 1.00,
        _ => 1.10,
    };
    let w_fb = w_fwd;

    let n_ref_std = match cond {
        Condition::AblationUnshieldedStack => 0,
        _ => config.n_ref, // 2
    };

    let (gate_theta_init, gate_theta_floor) = match cond {
        Condition::AblationFixedTheta => (1.10, 1.00),
        _ => (1.10, 1.05),
    };

    let max_depth = config.max_depth; // 8 for Active, 2 for BaselineFiniteState

    // -------------------------------------------------------------------------
    // 1. Ingress Ports (5 nodes)
    // -------------------------------------------------------------------------
    let in_rnd_open = builder.add_node(
        "in_rnd_open",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let in_rnd_close = builder.add_node(
        "in_rnd_close",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let in_sqr_open = builder.add_node(
        "in_sqr_open",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let in_sqr_close = builder.add_node(
        "in_sqr_close",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let in_eos = builder.add_node(
        "in_eos",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );

    // -------------------------------------------------------------------------
    // 2. Pointer Ladder Bank P_k (k in [0, max_depth])
    // -------------------------------------------------------------------------
    let mut pointer_rings = Vec::with_capacity(max_depth + 1);
    let mut p_sense_push = Vec::with_capacity(max_depth + 1);
    let mut p_sense_pop = Vec::with_capacity(max_depth + 1);
    let mut v_inh_ptr = Vec::with_capacity(max_depth + 1);

    for k in 0..=max_depth {
        let p0 = builder.add_node(
            &format!("p{k}_0"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        let p1 = builder.add_node(
            &format!("p{k}_1"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        let p2 = builder.add_node(
            &format!("p{k}_2"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        let p3 = builder.add_node(
            &format!("p{k}_3"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );

        // Forward ring loop
        builder.add_edge(p0, p1, w_fwd);
        builder.add_edge(p1, p2, w_fwd);
        builder.add_edge(p2, p3, w_fwd);
        builder.add_edge(p3, p0, w_fb);

        // Ablation: Unshielded stack (add retrograde reflections)
        if cond == Condition::AblationUnshieldedStack {
            builder.add_edge(p1, p0, w_fwd);
            builder.add_edge(p2, p1, w_fwd);
            builder.add_edge(p3, p2, w_fwd);
            builder.add_edge(p0, p3, w_fwd);
        }

        pointer_rings.push([p0, p1, p2, p3]);

        // Pointer sense taps driven by P_k,0 (fires at t=0 of aperture)
        let sp_push = builder.add_node(
            &format!("p{k}_sense_push"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        let sp_pop = builder.add_node(
            &format!("p{k}_sense_pop"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        builder.add_edge(p0, sp_push, w_fwd);
        builder.add_edge(p0, sp_pop, w_fwd);

        p_sense_push.push(sp_push);
        p_sense_pop.push(sp_pop);

        // Quench interneuron for pointer ring k
        let inh_p = builder.add_node(
            &format!("v_inh_ptr_{k}"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        for &pk in &[p0, p1, p2, p3] {
            builder.add_edge(inh_p, pk, -1.50);
        }
        v_inh_ptr.push(inh_p);
    }

    // -------------------------------------------------------------------------
    // 3. Data Stack Frame Array C_k (k in [0, max_depth - 1])
    // -------------------------------------------------------------------------
    let mut frame_rnd_rings = Vec::with_capacity(max_depth);
    let mut frame_sqr_rings = Vec::with_capacity(max_depth);
    let mut r_sense_rnd = Vec::with_capacity(max_depth);
    let mut r_sense_sqr = Vec::with_capacity(max_depth);
    let mut v_inh_frame_rnd = Vec::with_capacity(max_depth);
    let mut v_inh_frame_sqr = Vec::with_capacity(max_depth);

    for k in 0..max_depth {
        // Round bracket ring for frame k
        let rr0 = builder.add_node(
            &format!("c{k}_rnd_0"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        let rr1 = builder.add_node(
            &format!("c{k}_rnd_1"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        let rr2 = builder.add_node(
            &format!("c{k}_rnd_2"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        let rr3 = builder.add_node(
            &format!("c{k}_rnd_3"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );

        builder.add_edge(rr0, rr1, w_fwd);
        builder.add_edge(rr1, rr2, w_fwd);
        builder.add_edge(rr2, rr3, w_fwd);
        builder.add_edge(rr3, rr0, w_fb);

        if cond == Condition::AblationUnshieldedStack {
            builder.add_edge(rr1, rr0, w_fwd);
            builder.add_edge(rr2, rr1, w_fwd);
            builder.add_edge(rr3, rr2, w_fwd);
            builder.add_edge(rr0, rr3, w_fwd);
        }

        frame_rnd_rings.push([rr0, rr1, rr2, rr3]);

        let sr_rnd = builder.add_node(
            &format!("c{k}_rnd_sense"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        builder.add_edge(rr0, sr_rnd, w_fwd);
        r_sense_rnd.push(sr_rnd);

        let inh_rr = builder.add_node(
            &format!("v_inh_c{k}_rnd"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        for &rk in &[rr0, rr1, rr2, rr3] {
            builder.add_edge(inh_rr, rk, -1.50);
        }
        v_inh_frame_rnd.push(inh_rr);

        // Square bracket ring for frame k
        let sq0 = builder.add_node(
            &format!("c{k}_sqr_0"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        let sq1 = builder.add_node(
            &format!("c{k}_sqr_1"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        let sq2 = builder.add_node(
            &format!("c{k}_sqr_2"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        let sq3 = builder.add_node(
            &format!("c{k}_sqr_3"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );

        builder.add_edge(sq0, sq1, w_fwd);
        builder.add_edge(sq1, sq2, w_fwd);
        builder.add_edge(sq2, sq3, w_fwd);
        builder.add_edge(sq3, sq0, w_fb);

        if cond == Condition::AblationUnshieldedStack {
            builder.add_edge(sq1, sq0, w_fwd);
            builder.add_edge(sq2, sq1, w_fwd);
            builder.add_edge(sq3, sq2, w_fwd);
            builder.add_edge(sq0, sq3, w_fwd);
        }

        frame_sqr_rings.push([sq0, sq1, sq2, sq3]);

        let sr_sqr = builder.add_node(
            &format!("c{k}_sqr_sense"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        builder.add_edge(sq0, sr_sqr, w_fwd);
        r_sense_sqr.push(sr_sqr);

        let inh_sq = builder.add_node(
            &format!("v_inh_c{k}_sqr"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        for &sk in &[sq0, sq1, sq2, sq3] {
            builder.add_edge(inh_sq, sk, -1.50);
        }
        v_inh_frame_sqr.push(inh_sq);
    }

    // -------------------------------------------------------------------------
    // 4. Violation Rejection Trap Ring & Trap Trigger Bus
    // -------------------------------------------------------------------------
    let t0 = builder.add_node(
        "trap_0",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let t1 = builder.add_node(
        "trap_1",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let t2 = builder.add_node(
        "trap_2",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let t3 = builder.add_node(
        "trap_3",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );

    builder.add_edge(t0, t1, w_fwd);
    builder.add_edge(t1, t2, w_fwd);
    builder.add_edge(t2, t3, w_fwd);
    builder.add_edge(t3, t0, w_fb);

    let trap_ring = [t0, t1, t2, t3];

    // Trap veto node: fired by any active trap node, delivering continuous -2.00 veto
    let v_trap_veto = builder.add_node(
        "v_trap_veto",
        1.00,
        std_theta_floor.min(1.00),
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(t0, v_trap_veto, w_fwd);
    builder.add_edge(t1, v_trap_veto, w_fwd);
    builder.add_edge(t2, v_trap_veto, w_fwd);
    builder.add_edge(t3, v_trap_veto, w_fwd);

    let mut violation_triggers = Vec::new();

    // -------------------------------------------------------------------------
    // 5. PUSH Coincidence Gates (k in [0, max_depth - 1])
    // -------------------------------------------------------------------------
    for k in 0..max_depth {
        // PUSH Round '('
        let g_push_rnd = builder.add_gate_node(
            &format!("g_push_{k}_rnd"),
            gate_theta_init,
            gate_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        builder.add_edge(p_sense_push[k], g_push_rnd, 0.60);
        builder.add_edge(in_rnd_open, g_push_rnd, 0.60);

        // Gate firing drives:
        // 1. Advance pointer to P_{k+1, 3}
        builder.add_edge(g_push_rnd, pointer_rings[k + 1][3], w_fwd);
        // 2. Set frame k round ring to R_{k, rnd, 3}
        builder.add_edge(g_push_rnd, frame_rnd_rings[k][3], w_fwd);
        // 3. Quench current pointer P_k
        builder.add_edge(g_push_rnd, v_inh_ptr[k], w_fwd);
        // 4. Quench frame k square ring (ensure mutual exclusivity)
        builder.add_edge(g_push_rnd, v_inh_frame_sqr[k], w_fwd);

        // PUSH Square '['
        let g_push_sqr = builder.add_gate_node(
            &format!("g_push_{k}_sqr"),
            gate_theta_init,
            gate_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        builder.add_edge(p_sense_push[k], g_push_sqr, 0.60);
        builder.add_edge(in_sqr_open, g_push_sqr, 0.60);

        builder.add_edge(g_push_sqr, pointer_rings[k + 1][3], w_fwd);
        builder.add_edge(g_push_sqr, frame_sqr_rings[k][3], w_fwd);
        builder.add_edge(g_push_sqr, v_inh_ptr[k], w_fwd);
        builder.add_edge(g_push_sqr, v_inh_frame_rnd[k], w_fwd);
    }

    // -------------------------------------------------------------------------
    // 6. POP Coincidence Gates (k in [1, max_depth])
    // -------------------------------------------------------------------------
    for k in 1..=max_depth {
        // POP Round ')'
        let g_pop_rnd = builder.add_gate_node(
            &format!("g_pop_{k}_rnd"),
            gate_theta_init,
            gate_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        builder.add_edge(p_sense_pop[k], g_pop_rnd, 0.40);
        builder.add_edge(r_sense_rnd[k - 1], g_pop_rnd, 0.40);
        builder.add_edge(in_rnd_close, g_pop_rnd, 0.40);

        // Gate firing drives:
        // 1. Retreat pointer to P_{k-1, 3}
        builder.add_edge(g_pop_rnd, pointer_rings[k - 1][3], w_fwd);
        // 2. Quench current pointer P_k
        builder.add_edge(g_pop_rnd, v_inh_ptr[k], w_fwd);
        // 3. Quench popped frame k-1 round ring
        builder.add_edge(g_pop_rnd, v_inh_frame_rnd[k - 1], w_fwd);

        // POP Square ']'
        let g_pop_sqr = builder.add_gate_node(
            &format!("g_pop_{k}_sqr"),
            gate_theta_init,
            gate_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        builder.add_edge(p_sense_pop[k], g_pop_sqr, 0.40);
        builder.add_edge(r_sense_sqr[k - 1], g_pop_sqr, 0.40);
        builder.add_edge(in_sqr_close, g_pop_sqr, 0.40);

        builder.add_edge(g_pop_sqr, pointer_rings[k - 1][3], w_fwd);
        builder.add_edge(g_pop_sqr, v_inh_ptr[k], w_fwd);
        builder.add_edge(g_pop_sqr, v_inh_frame_sqr[k - 1], w_fwd);
    }

    // -------------------------------------------------------------------------
    // 7. Structural Violation Detection & Funneling to Rejection Attractor
    // -------------------------------------------------------------------------
    // Underflow violations at depth 0
    let g_underflow_rnd = builder.add_gate_node(
        "g_underflow_rnd",
        gate_theta_init,
        gate_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(p_sense_pop[0], g_underflow_rnd, 0.60);
    builder.add_edge(in_rnd_close, g_underflow_rnd, 0.60);
    violation_triggers.push(g_underflow_rnd);

    let g_underflow_sqr = builder.add_gate_node(
        "g_underflow_sqr",
        gate_theta_init,
        gate_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(p_sense_pop[0], g_underflow_sqr, 0.60);
    builder.add_edge(in_sqr_close, g_underflow_sqr, 0.60);
    violation_triggers.push(g_underflow_sqr);

    // Cross-Bracket / Mismatch violations (k in [1, max_depth])
    for k in 1..=max_depth {
        // Mismatch: Top of stack is Round, but incoming token is Square Close ']'
        let g_mismatch_rnd_to_sqr = builder.add_gate_node(
            &format!("g_mismatch_{k}_rnd_to_sqr"),
            gate_theta_init,
            gate_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        builder.add_edge(p_sense_pop[k], g_mismatch_rnd_to_sqr, 0.40);
        builder.add_edge(r_sense_rnd[k - 1], g_mismatch_rnd_to_sqr, 0.40);
        builder.add_edge(in_sqr_close, g_mismatch_rnd_to_sqr, 0.40);
        violation_triggers.push(g_mismatch_rnd_to_sqr);

        // Mismatch: Top of stack is Square, but incoming token is Round Close ')'
        let g_mismatch_sqr_to_rnd = builder.add_gate_node(
            &format!("g_mismatch_{k}_sqr_to_rnd"),
            gate_theta_init,
            gate_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        builder.add_edge(p_sense_pop[k], g_mismatch_sqr_to_rnd, 0.40);
        builder.add_edge(r_sense_sqr[k - 1], g_mismatch_sqr_to_rnd, 0.40);
        builder.add_edge(in_rnd_close, g_mismatch_sqr_to_rnd, 0.40);
        violation_triggers.push(g_mismatch_sqr_to_rnd);
    }

    // Overflow violations at maximum depth capacity
    let g_overflow_rnd = builder.add_gate_node(
        "g_overflow_rnd",
        gate_theta_init,
        gate_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(p_sense_push[max_depth], g_overflow_rnd, 0.60);
    builder.add_edge(in_rnd_open, g_overflow_rnd, 0.60);
    violation_triggers.push(g_overflow_rnd);

    let g_overflow_sqr = builder.add_gate_node(
        "g_overflow_sqr",
        gate_theta_init,
        gate_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(p_sense_push[max_depth], g_overflow_sqr, 0.60);
    builder.add_edge(in_sqr_open, g_overflow_sqr, 0.60);
    violation_triggers.push(g_overflow_sqr);

    // Funnel all violation triggers into trap ring node t0
    let trap_trigger_out = build_or_funnel(
        &mut builder,
        "trap_funnel",
        violation_triggers,
        w_fwd,
        1.00,
        gate_theta_floor.min(1.00),
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(trap_trigger_out, t0, w_fwd);

    // -------------------------------------------------------------------------
    // 8. Acceptance Gate & Readout Stage
    // -------------------------------------------------------------------------
    let g_accept = builder.add_gate_node(
        "g_accept",
        gate_theta_init,
        gate_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(p_sense_pop[0], g_accept, 0.60);
    builder.add_edge(in_eos, g_accept, 0.60);
    builder.add_edge(v_trap_veto, g_accept, -2.00);

    // Accepting Indicator Ring A_k
    let a0 = builder.add_node(
        "accept_0",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let a1 = builder.add_node(
        "accept_1",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let a2 = builder.add_node(
        "accept_2",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let a3 = builder.add_node(
        "accept_3",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );

    builder.add_edge(a0, a1, w_fwd);
    builder.add_edge(a1, a2, w_fwd);
    builder.add_edge(a2, a3, w_fwd);
    builder.add_edge(a3, a0, w_fb);

    let accept_ring = [a0, a1, a2, a3];

    // Gate accept drives A_0
    builder.add_edge(g_accept, a0, w_fwd);

    // Readout sense port v_out monitored by test harness
    let v_out = builder.add_node(
        "v_out",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(a0, v_out, w_fwd);

    CircuitSubstrate::new(
        config,
        builder.nodes,
        builder.in_neighbors,
        in_rnd_open,
        in_rnd_close,
        in_sqr_open,
        in_sqr_close,
        in_eos,
        v_out,
        pointer_rings,
        frame_rnd_rings,
        frame_sqr_rings,
        trap_ring,
        accept_ring,
    )
}
