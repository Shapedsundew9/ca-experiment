# Scientific Research Campaign: Multi-Scale Self-Organizing Computational Substrate

- **Campaign Identifier**: CAMPAIGN-2026-MVA-SUBSTRATE
- **Current Status**: ACTIVE (Targeting Tier 1, Milestone 1.1)
- **Roadmap Reference**: `docs/vision.md` (Milestone Tiers)
- **Active Milestone**: Milestone 1.1: Signal Transport & Fan-Out
- **Target Paradigm**: Autopoietic, thermodynamically bounded computational substrate via discrete bit-stream cellular automata and homeostatic self-organized criticality
- **Parent Lineage**: `src/experiments/exp_2026_005a_mva_continual_learning/` (Tag: `exp/EXP-2026-005a-01`)
- **Last Updated**: 2026-09-07 17:00:00 UTC

---

## 1. Autonomous Exploration & Checkpoint Guardrails

- **Autonomous Checkpoint Horizon**: 5 cycles  <!-- SINGLE POINT OF CONFIGURATION: change to 3, 5, 10, etc. based on oversight budget -->
- **Current Burst Progress**: Cycle 1 of 5 (Milestone 1.1 Inception: Theory & Protocol Formulation)
- **Milestone Cumulative Cycles**: 0 cycles completed on Milestone 1.1 (Cycle 1 active)
- **Campaign Cumulative Cycles**: 5 cycles completed across all milestones
- **Branch Depth Limit**: Max 2 consecutive runs on a single mechanism/branch
- **Current Branch Depth**: Run 1 of 2
- **Active Hypothesis / Mechanism**: Milestone 1.1: Signal Transport & Fan-Out (Tier 1). Directed regenerative transmission tracks with refractory diode shielding and branching fan-out nodes under homeostatic threshold regulation for zero-attenuation propagation ($D \ge 30$) and 1-to-2 duplication under background noise and spatial jitter.
- **Escalation Triggers**: Stop and request operator input ONLY on:
  1. *Multi-path ambiguity* (competing hypotheses with no clear theoretical winner)
  2. *2-strike paradigm stall* (2 distinct ideas fail consecutively to show signal)
  3. *Checkpoint horizon reached* (Current Burst Progress == Autonomous Checkpoint Horizon)
  4. *Repo-level boundary modification* (modifications outside isolated experiment packages, module roots, or test harnesses)
- **Graveyard of Discarded Ideas (Autopsy Log)**:
  - *Instantaneous 16-node readout for $\tau > 4$ on $4 \times 4$ Torus*: Falsified in `EXP-2026-003a`. Fixed isotropic torus edges cannot sustain reverberation beyond graph diameter ($D=4$) due to wavefront collision and refractory annihilation. Resolved in `EXP-2026-004a` via Hebbian symmetry breaking (+463% noise resilience).

---

## 2. Active Campaign State Machine

| Stage | Active Agent | Active Artifact Reference | Status |
| :--- | :--- | :--- | :--- |
| Strategic Assessment | Sci: Orchestrator | `docs/research/CAMPAIGN.md`, `docs/vision.md` | COMPLETED |
| Theory & Protocol | Sci: Theory & Protocol | `docs/research/hypotheses/HYP-2026-006.md`, `docs/research/protocols/EXP-2026-006a.md` | COMPLETED |
| Protocol & Budget Check | Sci: Orchestrator / Operator | **Gate H/P**: Pre-execution validation | APPROVED |
| Execution & Analysis | Sci: Execution & Analysis | Rust implementation, Run Manifest, Diagnostic Report | IN_PROGRESS |
| Iteration Decision | Sci: Orchestrator | Iteration Directive | PENDING |
| Iteration Check | Sci: Orchestrator / Operator | **Gate I**: Post-analysis checkpoint | PENDING |

---

## 3. Capability Ladder & Milestone Progression (from docs/vision.md)

| Tier | Milestone | Description & Target Invariant | Status | Cumulative Cycles | Evidence Document |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Tier 0** | Phase 0 MVA (Rungs 1–5) | Firing density, attractors, XOR, Hebbian, continual learning | VERIFIED | 5 | [`DIAG-2026-001a`](file:///workspaces/ca-experiment/docs/research/diagnostics/DIAG-2026-001a.md) through [`005a`](file:///workspaces/ca-experiment/docs/research/diagnostics/DIAG-2026-005a.md) |
| **Tier 1** | 1.1 Signal Transport & Fan-Out | 1-to-2 Buffer over $D \ge 30$ cells, 100% transmission fidelity | ACTIVE | 0 | Pending |
| **Tier 1** | 1.2 Multi-Gate Composition | 1-bit Full Adder / 2-bit Multiplier, zero crosstalk | LOCKED | - | Requires 1.1 |
| **Tier 2** | 2.1 Bistable Latching | Dynamic bit retention over $\Delta t \ge 10^3$ steps | LOCKED | - | Requires Tier 1 |
| **Tier 2** | 2.2 Finite State Automata | Regular expression DFA streaming recognition | LOCKED | - | Requires 2.1 |
| **Tier 3** | 3.1 Pushdown Memory | Dyck-1 / Dyck-2 balanced parentheses recognition | LOCKED | - | Requires Tier 2 |
| **Tier 3** | 3.2 Associative Retrieval | Key-Value variable binding retrieval ($N \ge 16$) | LOCKED | - | Requires 3.1 |

---

## 4. Iteration & Decision History

| Cycle | Hypothesis | Protocol | Package Path | Run ID | Git Tag | Diagnostic Verdict | Action Selected | User Gate Approval |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| 1 | `HYP-2026-001` | `EXP-2026-001a` | `src/experiments/exp_2026_001a_mva_homeostasis/` | `RUN-EXP-2026-001a-01` | `exp/EXP-2026-001a-01` | Supported ($N_{\text{ref}}=2$, $P_{\text{ext}}=0$) | ADVANCE (Rung 2 evaluation) | Autonomous (Gate I) |
| 2 | `HYP-2026-002` | `EXP-2026-002a` | `src/experiments/exp_2026_002a_mva_attractor_mapping/` | `RUN-EXP-2026-002a-01` | `exp/EXP-2026-002a-01` | Supported ($p < 10^{-12}$, SNR=3.63) | ADVANCE (Rung 3 evaluation) | Autonomous (Gate I) |
| 3 | `HYP-2026-003` | `EXP-2026-003a` | `src/experiments/exp_2026_003a_mva_temporal_xor/` | `RUN-EXP-2026-003a-01` | `exp/EXP-2026-003a-01` | Falsified ($>0.95$ gate), Supported (dynamics) | ADVANCE (Rung 4 evaluation) | Autonomous (Gate I) |
| 4 | `HYP-2026-004` | `EXP-2026-004a` | `src/experiments/exp_2026_004a_mva_hebbian_plasticity/` | `RUN-EXP-2026-004a-01` | `exp/EXP-2026-004a-01` | Supported (+463% noise resilience, $p < 10^{-32}$) | ADVANCE (Rung 5 evaluation) | Autonomous (Gate I) |
| 5 | `HYP-2026-005` | `EXP-2026-005a` | `src/experiments/exp_2026_005a_mva_continual_learning/` | `RUN-EXP-2026-005a-01` | `exp/EXP-2026-005a-01` | Supported (All 6 Gates Pass) | ADVANCE (Tier 1 evaluation) | Autonomous (Gate I) |

---

## 5. Resource & Compute Accounting

- **Total Allocated Compute Budget**: 100 Compute-Hours
- **Compute Consumed to Date**: < 0.01 Compute-Hours
- **Remaining Compute Budget**: ~100 Compute-Hours
- **Autonomous Checkpoint Horizon**: 5 cycles per burst (Current Burst: Cycle 0 of 5)
