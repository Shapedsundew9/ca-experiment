# Scientific Research Campaign: Multi-Scale Self-Organizing Computational Substrate

- **Campaign Identifier**: CAMPAIGN-2026-MVA-SUBSTRATE
- **Current Status**: ACTIVE
- **Target Paradigm**: Autopoietic, thermodynamically bounded computational substrate via discrete bit-stream cellular automata and homeostatic self-organized criticality
- **Last Updated**: 2026-09-07 00:35:00 UTC

---

## 1. Autonomous Exploration & Budget Guardrails

- **Max Autonomous Cycle Budget**: 5 cycles per session / mandate
- **Current Cycle**: Cycle 1 of 5
- **Branch Depth Limit**: Max 2 consecutive runs on a single mechanism/branch
- **Current Branch Depth**: Run 1 of 2
- **Active Hypothesis / Mechanism**: Local homeostatic threshold regulation ($V_{\text{thresh}}$ adaptation) to stabilize firing density within critical regime ($5\% - 20\%$) on a 16-node 2D torus substrate without extinction or runaway saturation across $10^5$ ticks.
- **Escalation Triggers**: Stop and request operator input ONLY on:
  1. *Multi-path ambiguity* (competing hypotheses with no clear theoretical winner)
  2. *2-strike paradigm stall* (2 distinct ideas fail consecutively to show signal)
  3. *Budget exhaustion* (5 cycles completed)
  4. *Repo-level boundary modification* (changes outside isolated Rust experiment packages/modules)
- **Graveyard of Discarded Ideas (Autopsy Log)**:
  - *None yet*

---

## 2. Active Campaign State Machine

| Stage | Active Agent | Active Artifact Reference | Status |
| --- | --- | --- | --- |
| Strategic Assessment | Sci: Orchestrator | `docs/research/CAMPAIGN.md` | COMPLETED |
| Theory & Protocol | Sci: Theory & Protocol | `docs/research/hypotheses/HYP-2026-001.md`, `docs/research/protocols/EXP-2026-001a.md` | COMPLETED |
| Protocol & Budget Check | Sci: Orchestrator / Operator | **Gate H/P**: Pre-execution validation | APPROVED |
| Execution & Analysis | Sci: Execution & Analysis | Rust implementation, `docs/research/runs/RUN-EXP-2026-001a-01.md`, `docs/research/diagnostics/DIAG-2026-001a.md` | IN PROGRESS |
| Iteration Decision | Sci: Orchestrator | Iteration Directive | PENDING |
| Iteration Check | Sci: Orchestrator / Operator | **Gate I**: Post-analysis checkpoint | PENDING |

---

## 3. Complexity Ladder Progression

| Rung | Theoretical Capability | Key Invariant / Metric Threshold | Status | Evidence Document |
| --- | --- | --- | --- | --- |
| 1 | Homeostatic Critical Firing Density (Sprint 0) | Mean firing density $\bar{\rho} \in [0.05, 0.20]$, 0 extinctions, 0 saturations across $10^5$ ticks | ACTIVE | `EXP-2026-001a` |
| 2 | Attractor Separation & Limit Cycles (Sprint 1) | Hamming distance $D(S_A, S_B) > 0$ for distinct input drives $A \neq B$ | LOCKED | Requires Rung 1 |
| 3 | Non-linear Temporal Mixing (Sprint 2) | Delayed XOR accuracy $> 0.95$ for delay $\tau \ge 5$ ticks | LOCKED | Requires Rung 2 |
| 4 | Local Structural Plasticity (Sprint 3) | Local Hebbian track rewiring autonomously deepens attractor basins | LOCKED | Requires Rung 3 |
| 5 | Lifelong Adaptation & Thermodynamic Scaling | Continual BWT $\ge 0$, bounded topological state retention, Landauer efficiency | LOCKED | Requires Rung 4 |

---

## 4. Iteration & Decision History

| Cycle | Hypothesis       | Protocol        | Package Path                                      | Run ID  | Git Tag | Diagnostic Verdict | Action Selected | User Gate Approval    |
| :---- | :--------------- | :-------------- | :------------------------------------------------ | :------ | :------ | :----------------- | :-------------- | :-------------------- |
| 1     | `HYP-2026-001`   | `EXP-2026-001a` | `src/experiments/exp_2026_001a_mva_homeostasis/`  | Pending | Pending | Pending            | Pending         | Autonomous (Gate H/P) |

---

## 5. Resource & Compute Accounting

- **Total Allocated Campaign Budget**: 500 Compute-Hours / 5 Iteration Cycles
- **Compute Consumed to Date**: 0 Compute-Hours
- **Remaining Budget**: 500 Compute-Hours / 5 Iteration Cycles
- **Max Iteration Limit per Milestone**: 5 iterations (Current: Cycle 1 of 5)
