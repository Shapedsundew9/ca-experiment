# Scientific Research Campaign: Multi-Scale Self-Organizing Computational Substrate

- **Campaign Identifier**: CAMPAIGN-2026-MVA-SUBSTRATE
- **Current Status**: ACTIVE
- **Target Paradigm**: Autopoietic, thermodynamically bounded computational substrate via discrete bit-stream cellular automata and homeostatic self-organized criticality
- **Last Updated**: 2026-09-07 00:35:00 UTC

---

## 1. Autonomous Exploration & Budget Guardrails

- **Max Autonomous Cycle Budget**: 5 cycles per session / mandate
- **Current Cycle**: Cycle 5 of 5
- **Branch Depth Limit**: Max 2 consecutive runs on a single mechanism/branch
- **Current Branch Depth**: Run 1 of 2
- **Active Hypothesis / Mechanism**: Sprint 4: Lifelong Adaptation & Continual Multi-Pattern Learning (Complexity Ladder Rung 5). Can the self-organizing substrate learn sequential patterns ($A \to B \to C$) continually without catastrophic forgetting, achieving non-negative Backward Transfer ($\text{BWT} \ge 0$) via orthogonal attractor subspace retention?
- **Escalation Triggers**: Stop and request operator input ONLY on:
  1. *Multi-path ambiguity* (competing hypotheses with no clear theoretical winner)
  2. *2-strike paradigm stall* (2 distinct ideas fail consecutively to show signal)
  3. *Budget exhaustion* (5 cycles completed)
  4. *Repo-level boundary modification* (changes outside isolated Rust experiment packages/modules)
- **Graveyard of Discarded Ideas (Autopsy Log)**:
  - *Instantaneous 16-node readout for $\tau > 4$ on $4 \times 4$ Torus*: Falsified in `EXP-2026-003a`. Fixed isotropic torus edges cannot sustain reverberation beyond graph diameter ($D=4$) due to wavefront collision and refractory annihilation. Resolved in `EXP-2026-004a` via Hebbian symmetry breaking (+463% noise resilience).

---

## 2. Active Campaign State Machine

| Stage | Active Agent | Active Artifact Reference | Status |
| :--- | :--- | :--- | :--- |
| Strategic Assessment | Sci: Orchestrator | `docs/research/CAMPAIGN.md` | COMPLETED |
| Theory & Protocol | Sci: Theory & Protocol | `docs/research/hypotheses/HYP-2026-005.md`, `docs/research/protocols/EXP-2026-005a.md` | IN PROGRESS |
| Protocol & Budget Check | Sci: Orchestrator / Operator | **Gate H/P**: Pre-execution validation | PENDING |
| Execution & Analysis | Sci: Execution & Analysis | Rust implementation, Run Manifest, Diagnostic Report | PENDING |
| Iteration Decision | Sci: Orchestrator | Iteration Directive | PENDING |
| Iteration Check | Sci: Orchestrator / Operator | **Gate I**: Post-analysis checkpoint | PENDING |

---

## 3. Complexity Ladder Progression

| Rung | Theoretical Capability | Key Invariant / Metric Threshold | Status | Evidence Document |
| :--- | :--- | :--- | :--- | :--- |
| 1 | Homeostatic Critical Firing Density (Sprint 0) | Mean firing density $\bar{\rho} \in [0.05, 0.20]$, 0 extinctions, 0 saturations across $10^5$ ticks ($N_{\text{ref}}=2$) | VERIFIED | [`DIAG-2026-001a`](file:///workspaces/ca-experiment/docs/research/diagnostics/DIAG-2026-001a.md) |
| 2 | Attractor Separation & Limit Cycles (Sprint 1) | High separation property $D(S_A, S_B) > 0$ ($p < 10^{-12}$), basin consistency $D(A, A') = 0.0115 < 0.05$ | VERIFIED | [`DIAG-2026-002a`](file:///workspaces/ca-experiment/docs/research/diagnostics/DIAG-2026-002a.md) |
| 3 | Non-linear Temporal Mixing (Sprint 2) | Non-linear XOR capacity $K_{\text{xor}} > 0$ ($p < 10^{-12}$), homeostatic advantage $+80\%$ ($K=0.1600$ vs $0.0943$) | VERIFIED (Ceiling $D=4$) | [`DIAG-2026-003a`](file:///workspaces/ca-experiment/docs/research/diagnostics/DIAG-2026-003a.md) |
| 4 | Local Structural Plasticity (Sprint 3) | Local Hebbian track rewiring breaks isotropic symmetry into resonant loops; $+463.6\%$ noise resilience ($p < 10^{-32}$) | VERIFIED | [`DIAG-2026-004a`](file:///workspaces/ca-experiment/docs/research/diagnostics/DIAG-2026-004a.md) |
| 5 | Lifelong Adaptation & Thermodynamic Scaling | Continual BWT $\ge 0$, bounded topological state retention, Landauer efficiency | ACTIVE | `EXP-2026-005a` |

---

## 4. Iteration & Decision History

| Cycle | Hypothesis | Protocol | Package Path | Run ID | Git Tag | Diagnostic Verdict | Action Selected | User Gate Approval |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| 1 | `HYP-2026-001` | `EXP-2026-001a` | `src/experiments/exp_2026_001a_mva_homeostasis/` | `RUN-EXP-2026-001a-01` | `exp/EXP-2026-001a-01` | Supported ($N_{\text{ref}}=2$, $P_{\text{ext}}=0$) | ADVANCE (Rung 2 evaluation) | Autonomous (Gate I) |
| 2 | `HYP-2026-002` | `EXP-2026-002a` | `src/experiments/exp_2026_002a_mva_attractor_mapping/` | `RUN-EXP-2026-002a-01` | `exp/EXP-2026-002a-01` | Supported ($p < 10^{-12}$, SNR=3.63) | ADVANCE (Rung 3 evaluation) | Autonomous (Gate I) |
| 3 | `HYP-2026-003` | `EXP-2026-003a` | `src/experiments/exp_2026_003a_mva_temporal_xor/` | `RUN-EXP-2026-003a-01` | `exp/EXP-2026-003a-01` | Falsified ($>0.95$ gate), Supported (dynamics) | ADVANCE (Rung 4 evaluation) | Autonomous (Gate I) |
| 4 | `HYP-2026-004` | `EXP-2026-004a` | `src/experiments/exp_2026_004a_mva_hebbian_plasticity/` | `RUN-EXP-2026-004a-01` | `exp/EXP-2026-004a-01` | Supported (+463% noise resilience, $p < 10^{-32}$) | ADVANCE (Rung 5 evaluation) | Autonomous (Gate I) |
| 5 | `HYP-2026-005` | `EXP-2026-005a` | `src/experiments/exp_2026_005a_mva_continual_learning/` | Pending | Pending | Pending | Pending | Autonomous (Gate H/P) |

---

## 5. Resource & Compute Accounting

- **Total Allocated Campaign Budget**: 500 Compute-Hours / 5 Iteration Cycles
- **Compute Consumed to Date**: 0 Compute-Hours
- **Remaining Budget**: 500 Compute-Hours / 5 Iteration Cycles
- **Max Iteration Limit per Milestone**: 5 iterations (Current: Cycle 1 of 5)
