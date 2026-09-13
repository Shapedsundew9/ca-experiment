//! Finite state automaton circuit graph builder for EXP-2026-009a.
//!
//! Constructs the directed spatial circuit graphs for:
//! 1. Parity DFA (2-state DFA tracking streaming bitstream Even/Odd parity)
//! 2. Regex (10)+1 DFA (4-state DFA recognizing the canonical regular expression)
//!
//! Enforces bounded degree constraints (k_in <= 4, k_out <= 4) under ADR-0001.

use super::config::{AutomatonTask, Condition, ExperimentConfig};
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

/// Build the Parity DFA circuit (2-state DFA).
pub fn build_parity_circuit(config: ExperimentConfig) -> CircuitSubstrate {
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
    // Stage 1: Ingress Ports
    // ----------------------------------------------------
    let in_sigma0 = builder.add_node(
        "in_sigma0",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let in_sigma1 = builder.add_node(
        "in_sigma1",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );

    // ----------------------------------------------------
    // Stage 2: Coupled Resonant Attractor Rings (L = 4)
    // Ring 0: q_even
    // Ring 1: q_odd
    // ----------------------------------------------------
    let r0_0 = builder.add_node(
        "r0_0",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let r0_1 = builder.add_node(
        "r0_1",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let r0_2 = builder.add_node(
        "r0_2",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let r0_3 = builder.add_node(
        "r0_3",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );

    builder.add_edge(r0_0, r0_1, w_fwd);
    builder.add_edge(r0_1, r0_2, w_fwd);
    builder.add_edge(r0_2, r0_3, w_fwd);
    builder.add_edge(r0_3, r0_0, w_fb);

    let r1_0 = builder.add_node(
        "r1_0",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let r1_1 = builder.add_node(
        "r1_1",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let r1_2 = builder.add_node(
        "r1_2",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let r1_3 = builder.add_node(
        "r1_3",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );

    builder.add_edge(r1_0, r1_1, w_fwd);
    builder.add_edge(r1_1, r1_2, w_fwd);
    builder.add_edge(r1_2, r1_3, w_fwd);
    builder.add_edge(r1_3, r1_0, w_fb);

    // Ablation 1 (Unshielded Feedback): Add retrograde reflections between adjacent ring nodes
    if cond == Condition::AblationUnshieldedFeedback {
        builder.add_edge(r0_1, r0_0, w_fwd);
        builder.add_edge(r0_2, r0_1, w_fwd);
        builder.add_edge(r0_3, r0_2, w_fwd);
        builder.add_edge(r0_0, r0_3, w_fwd);

        builder.add_edge(r1_1, r1_0, w_fwd);
        builder.add_edge(r1_2, r1_1, w_fwd);
        builder.add_edge(r1_3, r1_2, w_fwd);
        builder.add_edge(r1_0, r1_3, w_fwd);
    }

    // ----------------------------------------------------
    // Stage 3: Sense Tap Nodes
    // ----------------------------------------------------
    let v_sense0 = builder.add_node(
        "v_sense0",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let v_sense1 = builder.add_node(
        "v_sense1",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(r0_0, v_sense0, w_fwd);
    builder.add_edge(r1_0, v_sense1, w_fwd);

    // ----------------------------------------------------
    // Stage 4: Coincidence Transition Gates
    // ----------------------------------------------------
    // Gate 0 -> 1: triggered on token 1 when in q_even
    let g_0_to_1 = builder.add_gate_node(
        "g_0_to_1",
        and_theta_init,
        and_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(v_sense0, g_0_to_1, 0.60);
    builder.add_edge(in_sigma1, g_0_to_1, 0.60);

    // Gate 1 -> 0: triggered on token 1 when in q_odd
    let g_1_to_0 = builder.add_gate_node(
        "g_1_to_0",
        and_theta_init,
        and_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(v_sense1, g_1_to_0, 0.60);
    builder.add_edge(in_sigma1, g_1_to_0, 0.60);

    // ----------------------------------------------------
    // Stage 5: Target Ingress Coupling & Inhibitory Interneurons
    // ----------------------------------------------------
    // Target ring sets: gate pulse enters R_tgt,3 to achieve phase-aligned R_tgt,0 firing at t+4
    builder.add_edge(g_0_to_1, r1_3, w_fwd);
    builder.add_edge(g_1_to_0, r0_3, w_fwd);

    // Inhibitory Interneurons
    let v_inh0 = builder.add_node(
        "v_inh0",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let v_inh1 = builder.add_node(
        "v_inh1",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(g_0_to_1, v_inh0, w_fwd);
    builder.add_edge(g_1_to_0, v_inh1, w_fwd);

    // Reset inhibition delivered to origin ring nodes (W_inh = -1.50)
    for &rk in &[r0_0, r0_1, r0_2, r0_3] {
        builder.add_edge(v_inh0, rk, -1.50);
    }
    for &rk in &[r1_0, r1_1, r1_2, r1_3] {
        builder.add_edge(v_inh1, rk, -1.50);
    }

    // ----------------------------------------------------
    // Stage 6: Acceptance Readout Port
    // ----------------------------------------------------
    let v_accept = builder.add_node(
        "v_accept",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(v_sense1, v_accept, w_fwd);

    let ring_nodes = vec![[r0_0, r0_1, r0_2, r0_3], [r1_0, r1_1, r1_2, r1_3]];

    CircuitSubstrate::new(
        config,
        builder.nodes,
        builder.in_neighbors,
        in_sigma0,
        in_sigma1,
        v_accept,
        ring_nodes,
    )
}

/// Build the Regex (10)+1 DFA circuit (4-state DFA).
pub fn build_regex_circuit(config: ExperimentConfig) -> CircuitSubstrate {
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
    // Stage 1: Ingress Ports
    // ----------------------------------------------------
    let in_sigma0 = builder.add_node(
        "in_sigma0",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    let in_sigma1 = builder.add_node(
        "in_sigma1",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );

    // ----------------------------------------------------
    // Stage 2: 4 Resonant Attractor Rings (L = 4)
    // Ring 0: q0 (start)
    // Ring 1: q1 (matched "1")
    // Ring 2: q2 (matched "(10)^k")
    // Ring 3: q3 (accepting, matched "(10)^k 1")
    // ----------------------------------------------------
    let mut rings = Vec::new();
    for r in 0..4 {
        let r_0 = builder.add_node(
            &format!("r{r}_0"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        let r_1 = builder.add_node(
            &format!("r{r}_1"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        let r_2 = builder.add_node(
            &format!("r{r}_2"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        let r_3 = builder.add_node(
            &format!("r{r}_3"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );

        builder.add_edge(r_0, r_1, w_fwd);
        builder.add_edge(r_1, r_2, w_fwd);
        builder.add_edge(r_2, r_3, w_fwd);
        builder.add_edge(r_3, r_0, w_fb);

        if cond == Condition::AblationUnshieldedFeedback {
            builder.add_edge(r_1, r_0, w_fwd);
            builder.add_edge(r_2, r_1, w_fwd);
            builder.add_edge(r_3, r_2, w_fwd);
            builder.add_edge(r_0, r_3, w_fwd);
        }

        rings.push([r_0, r_1, r_2, r_3]);
    }

    // ----------------------------------------------------
    // Stage 3: Sense Tap Nodes
    // ----------------------------------------------------
    let mut senses = Vec::new();
    for (r, ring) in rings.iter().enumerate() {
        let vs = builder.add_node(
            &format!("v_sense{r}"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        builder.add_edge(ring[0], vs, w_fwd);
        senses.push(vs);
    }

    // ----------------------------------------------------
    // Stage 4: Inhibitory Interneurons
    // ----------------------------------------------------
    let mut inhs = Vec::new();
    for (r, ring) in rings.iter().enumerate() {
        let v_inh = builder.add_node(
            &format!("v_inh{r}"),
            std_theta_init,
            std_theta_floor,
            config.theta_max,
            std_beta_theta,
            n_ref_std,
        );
        for &rk in ring {
            builder.add_edge(v_inh, rk, -1.50);
        }
        inhs.push(v_inh);
    }

    // ----------------------------------------------------
    // Stage 5: Coincidence Transition Gates
    // ----------------------------------------------------
    // Transition 1: q0 --(1)--> q1
    let g_0_to_1 = builder.add_gate_node(
        "g_0_to_1",
        and_theta_init,
        and_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(senses[0], g_0_to_1, 0.60);
    builder.add_edge(in_sigma1, g_0_to_1, 0.60);
    builder.add_edge(g_0_to_1, rings[1][3], w_fwd); // Set R1
    builder.add_edge(g_0_to_1, inhs[0], w_fwd); // Reset R0

    // Trap 0: q0 --(0)--> trap
    let g_0_trap = builder.add_gate_node(
        "g_0_trap",
        and_theta_init,
        and_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(senses[0], g_0_trap, 0.60);
    builder.add_edge(in_sigma0, g_0_trap, 0.60);
    builder.add_edge(g_0_trap, inhs[0], w_fwd); // Reset R0 (no target ring set)

    // Transition 2: q1 --(0)--> q2
    let g_1_to_2 = builder.add_gate_node(
        "g_1_to_2",
        and_theta_init,
        and_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(senses[1], g_1_to_2, 0.60);
    builder.add_edge(in_sigma0, g_1_to_2, 0.60);
    builder.add_edge(g_1_to_2, rings[2][3], w_fwd); // Set R2
    builder.add_edge(g_1_to_2, inhs[1], w_fwd); // Reset R1

    // Trap 1: q1 --(1)--> trap
    let g_1_trap = builder.add_gate_node(
        "g_1_trap",
        and_theta_init,
        and_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(senses[1], g_1_trap, 0.60);
    builder.add_edge(in_sigma1, g_1_trap, 0.60);
    builder.add_edge(g_1_trap, inhs[1], w_fwd); // Reset R1

    // Transition 3: q2 --(1)--> q3 (Accepting)
    let g_2_to_3 = builder.add_gate_node(
        "g_2_to_3",
        and_theta_init,
        and_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(senses[2], g_2_to_3, 0.60);
    builder.add_edge(in_sigma1, g_2_to_3, 0.60);
    builder.add_edge(g_2_to_3, rings[3][3], w_fwd); // Set R3
    builder.add_edge(g_2_to_3, inhs[2], w_fwd); // Reset R2

    // Trap 2: q2 --(0)--> trap
    let g_2_trap = builder.add_gate_node(
        "g_2_trap",
        and_theta_init,
        and_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(senses[2], g_2_trap, 0.60);
    builder.add_edge(in_sigma0, g_2_trap, 0.60);
    builder.add_edge(g_2_trap, inhs[2], w_fwd); // Reset R2

    // Transition 4: q3 --(0)--> q2
    let g_3_to_2 = builder.add_gate_node(
        "g_3_to_2",
        and_theta_init,
        and_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(senses[3], g_3_to_2, 0.60);
    builder.add_edge(in_sigma0, g_3_to_2, 0.60);
    builder.add_edge(g_3_to_2, rings[2][3], w_fwd); // Set R2
    builder.add_edge(g_3_to_2, inhs[3], w_fwd); // Reset R3

    // Trap 3: q3 --(1)--> trap
    let g_3_trap = builder.add_gate_node(
        "g_3_trap",
        and_theta_init,
        and_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(senses[3], g_3_trap, 0.60);
    builder.add_edge(in_sigma1, g_3_trap, 0.60);
    builder.add_edge(g_3_trap, inhs[3], w_fwd); // Reset R3

    // ----------------------------------------------------
    // Stage 6: Acceptance Readout Port (tapped from Ring 3 / sense 3)
    // ----------------------------------------------------
    let v_accept = builder.add_node(
        "v_accept",
        std_theta_init,
        std_theta_floor,
        config.theta_max,
        std_beta_theta,
        n_ref_std,
    );
    builder.add_edge(senses[3], v_accept, w_fwd);

    CircuitSubstrate::new(
        config,
        builder.nodes,
        builder.in_neighbors,
        in_sigma0,
        in_sigma1,
        v_accept,
        rings,
    )
}

/// Convenience dispatcher to construct either Parity or Regex circuit
pub fn build_circuit(config: ExperimentConfig) -> CircuitSubstrate {
    match config.task {
        AutomatonTask::ParityDfa => build_parity_circuit(config),
        AutomatonTask::RegexDfa => build_regex_circuit(config),
    }
}
