# Scientific Research Campaign State Tracker Template

This template defines the persistent state tracker for ongoing scientific campaigns. It maintains campaign continuity across session boundaries, tracks active hypotheses, records complexity ladder progression, and logs all iteration decisions.

Save active campaign state to `docs/research/CAMPAIGN.md`.

---

```markdown
# Scientific Research Campaign: [Campaign Title]

- **Campaign Identifier**: CAMPAIGN-[YEAR]-[SLUG]
- **Current Status**: ACTIVE | STALLED | COMPLETED | PIVOTED
- **Target Paradigm**: [e.g., Non-gradient sequence learning via flux-conserving dynamical attractors]
- **Last Updated**: YYYY-MM-DD HH:MM:SS UTC

---

## 1. Autonomous Exploration & Budget Guardrails

- **Max Autonomous Cycle Budget**: 5 cycles per session / mandate
- **Current Cycle**: Cycle 1 of 5
- **Branch Depth Limit**: Max 2 consecutive runs on a single mechanism/branch
- **Current Branch Depth**: Run 1 of 2
- **Active Hypothesis / Mechanism**: [Short description of current mechanism being tested]
- **Escalation Triggers**: Stop and request operator input ONLY on:
  1. *Multi-path ambiguity* (competing hypotheses with no clear theoretical winner)
  2. *2-strike paradigm stall* (2 distinct ideas fail consecutively to show signal)
  3. *Budget exhaustion* (5 cycles completed)
  4. *Repo-level boundary modification* (changes outside isolated `python/experiments/`)
- **Graveyard of Discarded Ideas (Autopsy Log)**:
  - *None yet* (e.g., `HYP-001: Static thresholding - Collapsed into trivial fixed point at \lambda=1.0; discarded on Cycle 1`)

---

## 2. Active Campaign State Machine

| Stage | Active Agent | Active Artifact Reference | Status |
|---|---|---|---|
| Strategic Assessment | Sci: Orchestrator | `docs/research/CAMPAIGN.md`, `DIAG-*.md` (inline directive) | COMPLETED |
| Theory & Protocol | Sci: Theory & Protocol | `docs/research/hypotheses/HYP-[ID].md`, `docs/research/protocols/EXP-[ID].md` | COMPLETED |
| Protocol & Budget Check | Sci: Orchestrator / Operator | **Gate H/P**: Pre-execution validation (Autonomous; escalate if ambiguous or over-budget) | APPROVED |
| Execution & Analysis | Sci: Execution & Analysis | `python/experiments/`, `docs/research/runs/RUN-EXP-[ID].md`, `docs/research/diagnostics/DIAG-[ID].md` | IN PROGRESS |
| Iteration Decision | Sci: Orchestrator | Iteration Directive (MUTATE / ADVANCE / ABLATE / EXPLOIT / VERIFY / REFUTE / PIVOT) | PENDING |
| Iteration Check | Sci: Orchestrator / Operator | **Gate I**: Post-analysis checkpoint (Autonomous; escalate if stall, fork, or 5-cycle limit) | PENDING |

---

## 3. Complexity Ladder Progression

Track the structural capability rungs established by the campaign:

| Rung | Theoretical Capability | Key Invariant / Metric Threshold | Status | Evidence Document |
|---|---|---|---|---|
| 1 | Basic Attractor Stability | Spectral radius $\rho(W) \le 1.0$, limit cycle period $T > 10$ | VERIFIED | `DIAG-2025-001` |
| 2 | Single-Task Memory Retention | Bit accuracy $> 0.95$ over 500 delay steps | VERIFIED | `DIAG-2025-003` |
| 3 | Multi-Task Backward Transfer | Backward Transfer $\text{BWT} > -0.05$ across 10 switches | ACTIVE | `EXP-2025-014a` |
| 4 | Compositional Generalization | Assembly Index $A_x > 5.0$ on unseen combinations | LOCKED | Requires Rung 3 |
| 5 | Lifelong Adaptation | Bounded assembly space under continuous shift | LOCKED | Requires Rung 4 |

---

## 4. Iteration & Decision History

Audit log of every discovery cycle executed within this campaign:

| Cycle | Hypothesis | Protocol | Package Path | Run ID | Git Tag | Diagnostic Verdict | Action Selected | User Gate Approval |
|---|---|---|---|---|---|---|---|---|
| 1 | `HYP-014` | `EXP-014a` | `python/experiments/exp_014a_flux/` | `RUN-EXP-014a-01` | `exp/EXP-014a-01` | Inconclusive (collapse at $\lambda=1.0$) | MUTATE (soft penalty) | Autonomous (Gate I) |
| 2 | `HYP-014-v2` | `EXP-014b` | `python/experiments/exp_014b_penalty/` | `RUN-EXP-014b-01` | `exp/EXP-014b-01` | Supported ($\text{BWT} = -0.03 \pm 0.02$) | ADVANCE (Rung 4 evaluation) | Autonomous (Gate I) |

---

## 5. Resource & Compute Accounting

- **Total Allocated Campaign Budget**: [e.g., 500 Compute-Hours]
- **Compute Consumed to Date**: [e.g., 142 Compute-Hours]
- **Remaining Budget**: [e.g., 358 Compute-Hours]
- **Max Iteration Limit per Milestone**: 5 iterations (Current: Cycle 2 of 5)
```
