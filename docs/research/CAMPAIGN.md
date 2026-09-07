# Scientific Research Campaign: Multi-Scale Self-Organizing Computational Substrate

- **Campaign Identifier**: CAMPAIGN-2026-MVA-SUBSTRATE
- **Current Status**: ACTIVE (Tier 3: Milestone 3.1 Pushdown Memory Protocol Design & Execution)
- **Roadmap Reference**: `docs/vision.md` (Milestone Tiers)
- **Active Milestone**: Milestone 3.1: Pushdown Memory (Dyck Languages)
- **Target Paradigm**: Autopoietic, thermodynamically bounded computational substrate via discrete bit-stream cellular automata and homeostatic self-organized criticality
- **Parent Lineage**: `src/experiments/exp_2026_009a_mva_finite_state_automata/` (Tag: `exp/EXP-2026-009a-01`)
- **Last Updated**: 2026-09-07 21:15:00 UTC

---

## 1. Autonomous Exploration & Checkpoint Guardrails

- **Autonomous Checkpoint Horizon**: 5 cycles  <!-- SINGLE POINT OF CONFIGURATION: change to 3, 5, 10, etc. based on oversight budget -->
- **Current Burst Progress**: Cycle 4 of 5 (Milestone 3.1 Protocol Design & Dispatch)
- **Milestone Cumulative Cycles**: 0 cycles completed on Milestone 3.1 (Cycle 1 in progress)
- **Campaign Cumulative Cycles**: 9 cycles completed across all milestones (Cycle 10 in progress)
- **Branch Depth Limit**: Max 2 consecutive runs on a single mechanism/branch
- **Current Branch Depth**: Run 1 of 2
- **Active Hypothesis / Mechanism**: Milestone 3.1: Pushdown Memory via Cascaded Dynamical Stack Cells for Context-Free Dyck Languages. Realizing a spatial/recurrent cellular pushdown store that implements LIFO stack dynamics (PUSH / POP operations) using cascaded bistable resonant latch units, bidirectional depth steering, and top-of-stack coincidence gating in excitable MVA cellular automata under ADR-0001. Target tasks: Dyck-1 (single-bracket balanced parentheses verification) and Dyck-2 (multi-bracket nested matching `([{}])`) evaluating generalization to nesting depths $2\times$ deeper than calibration depth (e.g. depth $D \in [1, 8]$) with zero classification error and robust under channel noise.
- **Escalation Triggers**: Stop and request operator input ONLY on:
  1. *Multi-path ambiguity* (competing hypotheses with no clear theoretical winner)
  2. *2-strike paradigm stall* (2 distinct ideas fail consecutively to show signal)
  3. *Checkpoint horizon reached* (Current Burst Progress == Autonomous Checkpoint Horizon)
  4. *Repo-level boundary modification* (modifications outside isolated experiment packages, module roots, or test harnesses)
- **Graveyard of Discarded Ideas (Autopsy Log)**:
  - *Instantaneous 16-node readout for $\tau > 4$ on $4 \times 4$ Torus*: Falsified in `EXP-2026-003a`. Fixed isotropic torus edges cannot sustain reverberation beyond graph diameter ($D=4$) due to wavefront collision and refractory annihilation. Resolved in `EXP-2026-004a` via Hebbian symmetry breaking (+463% noise resilience).
  - *Passive Leaky Transmission Cable*: Extinguished exponentially ($V(x) \propto (1-\lambda)^x$) by $D \ge 10$ ($V(30) \approx 0.042 \ll \theta$), resulting in $\text{BER} \approx 0.15$ and complete signal extinction in `EXP-2026-006a`. Resolved by active all-or-none somatic regeneration.
  - *Unshielded Bidirectional Physical Coupling ($N_{\text{ref}} = 0$)*: Falsified in `EXP-2026-006a`. Retrograde back-coupling causes continuous standing-wave ring reverberations ($\text{BER} = 0.2140$, inter-branch crosstalk $\chi_{1\to 2} = 0.1000$). Resolved via refractory diode shielding ($N_{\text{ref}} = 2$).
  - *Uncompensated Delay Cascades*: Falsified in `EXP-2026-007a`. Path latency disparities cause multi-input coincidence failure ($\Delta \tau = 3$, accuracy collapsed to 0.3750). Resolved via meander delay equalization tracks ($\Delta \tau = 0$).
  - *Unshielded 4-Way Planar Wire Crossing*: Falsified in `EXP-2026-007a`. Crossing intersection merges signals, generating massive crosstalk ($\chi_{\text{cross}} = 0.5000$). Resolved via refractory-shielded planar bridge ($\chi_{\text{cross}} = 0.0000$).
  - *Open-Loop Pulse Storage ($W_{\text{fb}} = 0$)*: Falsified in `EXP-2026-008a`. Open-loop feedforward topologies possess only quiescent ground attractors; pulses dissipate after traversal ($t=4$), destroying stored state ($\text{Accuracy} = 0.5000$). Resolved via closed-loop recurrent resonance ($W_{\text{fb}} = 1.00$).
  - *Unshielded Recurrent Feedback Ring ($N_{\text{ref}} = 0$)*: Falsified in `EXP-2026-008a`. Symmetrical retrograde conduction causes standing-wave collision and complete failure of reset hyperpolarization ($\text{Fidelity} = 0.5000$). Resolved via refractory diode shielding ($N_{\text{ref}} = 2$).

---

## 2. Active Campaign State Machine

| Stage | Active Agent | Active Artifact Reference | Status |
| :--- | :--- | :--- | :--- |
| Strategic Assessment | Sci: Orchestrator | `docs/research/CAMPAIGN.md`, `docs/vision.md`, `docs/architecture/adr-0001-substrate-connectivity-topology.md` | COMPLETED |
| Theory & Protocol | Sci: Theory & Protocol | `docs/research/hypotheses/HYP-2026-010.md`, `docs/research/protocols/EXP-2026-010a.md` | IN_PROGRESS |
| Protocol & Budget Check | Sci: Orchestrator / Operator | **Gate H/P**: Pre-execution validation | PENDING |
| Execution & Analysis | Sci: Execution & Analysis | Rust implementation, Run Manifest, Diagnostic Report | PENDING |
| Iteration Decision | Sci: Orchestrator | Iteration Directive (`DIAG-2026-010a.md`) | PENDING |
| Iteration Check | Sci: Orchestrator / Operator | **Gate I**: Post-analysis checkpoint | PENDING |

---

## 3. Capability Ladder & Milestone Progression (from docs/vision.md)

| Tier | Milestone | Description & Target Invariant | Status | Cumulative Cycles | Evidence Document |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Tier 0** | Phase 0 MVA (Rungs 1–5) | Firing density, attractors, XOR, Hebbian, continual learning | VERIFIED | 5 | [`DIAG-2026-001a`](file:///workspaces/ca-experiment/docs/research/diagnostics/DIAG-2026-001a.md) through [`005a`](file:///workspaces/ca-experiment/docs/research/diagnostics/DIAG-2026-005a.md) |
| **Tier 1** | 1.1 Signal Transport & Fan-Out | 1-to-2 Buffer over $D \ge 30$ cells, 100% transmission fidelity | VERIFIED | 1 | [`DIAG-2026-006a`](file:///workspaces/ca-experiment/docs/research/diagnostics/DIAG-2026-006a.md) |
| **Tier 1** | 1.2 Multi-Gate Composition | 1-bit Full Adder / 2-bit Multiplier, zero crosstalk | VERIFIED | 1 | [`DIAG-2026-007a`](file:///workspaces/ca-experiment/docs/research/diagnostics/DIAG-2026-007a.md) |
| **Tier 2** | 2.1 Bistable Latching | Dynamic bit retention over $\Delta t \ge 10^3$ steps | VERIFIED | 1 | [`DIAG-2026-008a`](file:///workspaces/ca-experiment/docs/research/diagnostics/DIAG-2026-008a.md) |
| **Tier 2** | 2.2 Finite State Automata | Regular expression DFA streaming recognition | VERIFIED | 1 | [`DIAG-2026-009a`](file:///workspaces/ca-experiment/docs/research/diagnostics/DIAG-2026-009a.md) |
| **Tier 3** | 3.1 Pushdown Memory | Dyck-1 / Dyck-2 balanced parentheses recognition | IN_PROGRESS | 0 | Inception & Protocol Design |
| **Tier 3** | 3.2 Associative Retrieval | Key-Value variable binding retrieval ($N \ge 16$) | LOCKED | - | Requires 3.1 |

---

## 4. Iteration & Decision History

| Cycle | Hypothesis | Protocol | Package Path | Run ID | Git Tag | Diagnostic Verdict | Action Selected | User Gate Approval |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| 1 | `HYP-2026-001` | `EXP-2026-001a` | `src/experiments/exp_2026_001a_mva_homeostasis/` | `RUN-EXP-2026-001a-01` | `exp/EXP-2026-001a-01` | Supported ($N_{\text{ref}}=2$, $P_{\text{ext}}=0$) | ADVANCE (Rung 2 evaluation) | Autonomous (Gate I) |
| 2 | `HYP-2026-002` | `EXP-2026-002a` | `src/experiments/exp_2026_002a_mva_attractor_mapping/` | `RUN-EXP-2026-002a-01` | `exp/EXP-2026-002a-01` | Supported ($p < 10^{-12}$, SNR=3.63) | ADVANCE (Rung 3 evaluation) | Autonomous (Gate I) |
| 3 | `HYP-2026-003` | `EXP-2026-003a` | `src/experiments/exp_2026_003a_mva_temporal_xor/` | `RUN-EXP-2026-003a-01` | `exp/EXP-2026-003a-01` | Falsified ($>0.95$ gate), Supported (dynamics) | ADVANCE (Rung 4 evaluation) | Autonomous (Gate I) |
| 4 | `HYP-2026-004` | `EXP-2026-004a` | `src/experiments/exp_2026_004a_mva_hebbian_plasticity/` | `RUN-EXP-2026-004a-01` | `exp/EXP-2026-004a-01` | Supported (+463% noise resilience, $p < 10^{-32}$) | ADVANCE (Rung 5 evaluation) | Autonomous (Gate I) |
| 5 | `HYP-2026-005` | `EXP-2026-005a` | `src/experiments/exp_2026_005a_mva_continual_learning/` | `RUN-EXP-2026-005a-01` | `exp/EXP-2026-005a-01` | Supported (All 6 Gates Pass) | ADVANCE (Tier 1 evaluation) | Autonomous (Gate I) |
| 6 | `HYP-2026-006` | `EXP-2026-006a` | `src/experiments/exp_2026_006a_mva_signal_transport/` | `RUN-EXP-2026-006a-01` | `exp/EXP-2026-006a-01` | Supported (All 6 Gates Pass) | VERIFY_COMPLETE (Advance to Milestone 1.2) | Autonomous (Gate I) |
| 7 | `HYP-2026-007` | `EXP-2026-007a` | `src/experiments/exp_2026_007a_mva_multi_gate_composition/` | `RUN-EXP-2026-007a-01` | `exp/EXP-2026-007a-01` | Supported (All 6 Gates Pass) | VERIFY_COMPLETE (Tier 1 Complete; Advance to Tier 2) | Autonomous (Gate I) |
| 8 | `HYP-2026-008` | `EXP-2026-008a` | `src/experiments/exp_2026_008a_mva_bistable_latching/` | `RUN-EXP-2026-008a-01` | `exp/EXP-2026-008a-01` | Supported (All 6 Gates Pass) | VERIFY_COMPLETE (Milestone 2.1 Complete; Advance to Milestone 2.2) | Autonomous (Gate I) |
| 9 | `HYP-2026-009` | `EXP-2026-009a` | `src/experiments/exp_2026_009a_mva_finite_state_automata/` | `RUN-EXP-2026-009a-01` | `exp/EXP-2026-009a-01` | Supported (All 6 Gates Pass) | VERIFY_COMPLETE (Milestone 2.2 Complete; Advance to Tier 3) | Autonomous (Gate I) |

---

## 5. Resource & Compute Accounting

- **Total Allocated Compute Budget**: 100 Compute-Hours
- **Compute Consumed to Date**: < 0.02 Compute-Hours
- **Remaining Compute Budget**: ~100 Compute-Hours
- **Autonomous Checkpoint Horizon**: 5 cycles per burst (Current Burst: Cycle 4 of 5 in progress)

---

## 6. Architectural Redirection Notice (Post-Milestone 1.2)

- **Decision Record**: `docs/architecture/adr-0001-substrate-connectivity-topology.md` (Accepted, 2026-09-07).
- **Redirection**: The strict 2D planar, nearest-neighbor-only embedding used through Milestone 1.2 was a self-imposed topology choice, not a substrate requirement, and manufactured the planar wire-crossing/crosstalk problem solved in `EXP-2026-007a`. Future protocol design (starting with Milestone 2.1) may use higher-dimensional embeddings and/or bounded-degree long-range shortcut edges, provided locality and a fixed connection-degree budget are preserved (no unconstrained, zero-cost, all-to-all routing).
- **Re-Verification Impact**: **None.** Tier 0 and Tier 1 (Milestones 1.1, 1.2) results stand as-is and require no re-proof: the Full Adder composability result was demonstrated under the strictest-case topology (2D planar, nearest-neighbor only) and holds a fortiori under any relaxation of that constraint. No rows in Section 3 or Section 4 above are altered by this decision.
- **Forward Guidance**: The `HYP-2026-008` protocol for Milestone 2.1 (Bistable Latching) should assume the relaxed topology from ADR-0001 rather than defaulting back to strict 2D planarity, and is not required to reproduce planar-crossing shielding machinery unless a specific hypothesis calls for it.
