//! Bistable resonant latch circuit graph builder for EXP-2026-008a.
//!
//! Constructs the directed spatial circuit graph with excitable LIF nodes,
//! Gated D-Latch control interface, resonant core with recurrent feedback,
//! refractory diode shielding, and nondestructive sense tap readout.

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

/// Build the complete Bistable Resonant Latch substrate graph according to the experimental condition.
pub fn build_bistable_latch_circuit(config: ExperimentConfig) -> CircuitSubstrate {
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

    let w_fb = match cond {
        Condition::BaselineFeedforwardLoss => 0.00,
        _ => w_fwd,
    };

    let n_ref_std = match cond {
        Condition::AblationUnshieldedFeedback => 0,
        _ => config.n_ref, // 2
    };

    let (and_theta_init, and_theta_floor) = match cond {
        Condition::AblationFixedTheta => (1.10, 1.00),
        _ => (1.10, 1.05),
    };

    // ----------------------------------------------------
    // Stage 1: Control Interface Ingress Ports (t = 0)
    // ----------------------------------------------------
    let in_d = builder.add_node(
        "in_d",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let in_we = builder.add_node(
        "in_we",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let in_s = builder.add_node(
        "in_s",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let in_r = builder.add_node(
        "in_r",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let in_re = builder.add_node(
        "in_re",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );

    // ----------------------------------------------------
    // Stage 1b: Gated Control Logic (AND / NOT-AND / Inh)
    // ----------------------------------------------------
    // Set Gate: D AND WE
    let gate_set = builder.add_node(
        "gate_set",
        and_theta_init,
        and_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(in_d, gate_set, 0.60);
    builder.add_edge(in_we, gate_set, 0.60);

    // Reset Gate: (NOT D) AND WE
    let gate_rst = builder.add_node(
        "gate_rst",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(in_we, gate_rst, w_fwd);
    builder.add_edge(in_d, gate_rst, -1.50);

    // Inhibitory Reset Interneuron: gate_rst OR direct in_r
    let v_inh = builder.add_node(
        "v_inh",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(gate_rst, v_inh, w_fwd);
    builder.add_edge(in_r, v_inh, w_fwd);

    // ----------------------------------------------------
    // Stage 2: Bistable Resonant Core Ring (L = 4)
    // Nodes: R0, R1, R2, R3
    // ----------------------------------------------------
    let r0 = builder.add_node(
        "r0",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let r1 = builder.add_node(
        "r1",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let r2 = builder.add_node(
        "r2",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let r3 = builder.add_node(
        "r3",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );

    // Forward loop edges: R0 -> R1 -> R2 -> R3
    builder.add_edge(r0, r1, w_fwd);
    builder.add_edge(r1, r2, w_fwd);
    builder.add_edge(r2, r3, w_fwd);

    // Recurrent feedback closure: R3 -> R0
    builder.add_edge(r3, r0, w_fb);

    // Ingress Set triggers into R0
    builder.add_edge(gate_set, r0, w_fwd);
    builder.add_edge(in_s, r0, w_fwd);

    // Coordinated hyperpolarizing reset: v_inh -> R_k (W_inh = -1.50)
    for &rk in &[r0, r1, r2, r3] {
        builder.add_edge(v_inh, rk, -1.50);
    }

    // Ablation 1 (Unshielded Feedback): Add retrograde reflections between adjacent ring nodes
    if cond == Condition::AblationUnshieldedFeedback {
        builder.add_edge(r1, r0, w_fwd);
        builder.add_edge(r2, r1, w_fwd);
        builder.add_edge(r3, r2, w_fwd);
        builder.add_edge(r0, r3, w_fwd);
    }

    // ----------------------------------------------------
    // Stage 3: Nondestructive Readout Stage
    // ----------------------------------------------------
    // Continuous Sense Tap buffer from R0
    let v_sense = builder.add_node(
        "v_sense",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(r0, v_sense, w_fwd);

    // Gated Read Port: v_sense AND in_re
    let v_read = builder.add_node(
        "v_read",
        and_theta_init,
        and_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(v_sense, v_read, 0.60);
    builder.add_edge(in_re, v_read, 0.60);

    CircuitSubstrate::new(
        config,
        builder.nodes,
        builder.in_neighbors,
        in_d,
        in_we,
        in_s,
        in_r,
        in_re,
        gate_set,
        gate_rst,
        v_inh,
        r0,
        r1,
        r2,
        r3,
        v_sense,
        v_read,
    )
}
