# Scientific Research Campaign State Tracker Template

This template defines the persistent state tracker for ongoing scientific campaigns. It maintains campaign continuity across session boundaries, tracks active hypotheses, records complexity ladder progression, and logs all iteration decisions.

Save active campaign state to `docs/research/CAMPAIGN.md`.

---

```markdown
# Scientific Research Campaign: [Campaign Title]

- **Campaign Identifier**: CAMPAIGN-[YEAR]-[SLUG]
- **Current Status**: ACTIVE | STALLED | COMPLETED | PIVOTED
- **Roadmap Reference**: `docs/vision.md` (Milestone Tiers)
- **Active Milestone**: [e.g., Milestone 1.1: Signal Transport & Fan-Out]
- **Target Paradigm**: [e.g., Autopoietic, thermodynamically bounded computational substrate]
- **Parent Lineage**: [e.g., src/experiments/exp_yyyy_nnna_[slug]/ or tag]
- **Last Updated**: YYYY-MM-DD HH:MM:SS UTC

---

## 1. Autonomous Exploration & Checkpoint Guardrails

- **Autonomous Checkpoint Horizon**: 5 cycles  <!-- SINGLE POINT OF CONFIGURATION: adjust based on oversight/budget preference -->
- **Current Burst Progress**: Cycle 0 of 5 (Awaiting Burst Start)
- **Milestone Cumulative Cycles**: 0 cycles completed on active milestone
- **Campaign Cumulative Cycles**: 0 cycles completed across all milestones
- **Branch Depth Limit**: Max 2 consecutive runs on a single mechanism/branch
- **Current Branch Depth**: Run 0 of 2
- **Active Hypothesis / Mechanism**: [Short description of current mechanism being tested]
- **Escalation Triggers**: Stop and request operator input ONLY on:
  1. *Multi-path ambiguity* (competing hypotheses with no clear theoretical winner)
  2. *2-strike paradigm stall* (2 distinct ideas fail consecutively to show signal)
  3. *Checkpoint horizon reached* (Current Burst Progress == Autonomous Checkpoint Horizon)
  4. *Repo-level boundary modification* (modifications outside isolated experiment packages or authorized module roots)
- **Graveyard of Discarded Ideas (Autopsy Log)**:
  - *None yet* (e.g., `HYP-001: Static thresholding - Collapsed into trivial fixed point at \lambda=1.0; discarded on Cycle 1`)

---

## 2. Active Campaign State Machine

| Stage | Active Agent | Active Artifact Reference | Status |
| :--- | :--- | :--- | :--- |
| Strategic Assessment | Sci: Orchestrator | `docs/research/CAMPAIGN.md`, `DIAG-*.md` (inline directive) | COMPLETED |
| Theory & Protocol | Sci: Theory & Protocol | `docs/research/hypotheses/HYP-[ID].md`, `docs/research/protocols/EXP-[ID].md` | COMPLETED |
| Protocol & Budget Check | Sci: Orchestrator / Operator | **Gate H/P**: Pre-execution validation (Autonomous; escalate if ambiguous) | APPROVED |
| Execution & Analysis | Sci: Execution & Analysis | Experiment package, `docs/research/runs/RUN-EXP-[ID].md`, `docs/research/diagnostics/DIAG-[ID].md` | IN PROGRESS |
| Iteration Decision | Sci: Orchestrator | Iteration Directive (MUTATE / ADVANCE / ABLATE / EXPLOIT / VERIFY / REFUTE / PIVOT) | PENDING |
| Iteration Check | Sci: Orchestrator / Operator | **Gate I**: Post-analysis checkpoint (Autonomous; escalate if stall, fork, or checkpoint horizon) | PENDING |

---

## 3. Capability Ladder & Milestone Progression (from docs/vision.md)

Track structural capability rungs and milestone gates established by the campaign:

| Tier | Milestone | Description & Target Invariant | Status | Cumulative Cycles | Evidence Document |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Tier 0** | Phase 0 MVA (Rungs 1–5) | Firing density, attractors, XOR, Hebbian, continual learning | VERIFIED | 5 | DIAG-2026-001a through 005a |
| **Tier 1** | 1.1 Signal Transport & Fan-Out | 1-to-2 Buffer over $D \ge 30$ cells, 100% transmission fidelity | ACTIVE | 0 | Pending |
| **Tier 1** | 1.2 Multi-Gate Composition | 1-bit Full Adder / 2-bit Multiplier, zero crosstalk | LOCKED | - | Requires 1.1 |
| **Tier 2** | 2.1 Bistable Latching | Dynamic bit retention over $\Delta t \ge 10^3$ steps | LOCKED | - | Requires Tier 1 |
| **Tier 2** | 2.2 Finite State Automata | Regular expression DFA streaming recognition | LOCKED | - | Requires 2.1 |
| **Tier 3** | 3.1 Pushdown Memory | Dyck-1 / Dyck-2 balanced parentheses recognition | LOCKED | - | Requires Tier 2 |
| **Tier 3** | 3.2 Associative Retrieval | Key-Value variable binding retrieval ($N \ge 16$) | LOCKED | - | Requires 3.1 |

---

## 4. Iteration & Decision History

Audit log of every discovery cycle executed within this campaign:

| Cycle | Hypothesis | Protocol | Package Path | Run ID | Git Tag | Diagnostic Verdict | Action Selected | Operator Gate |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| 1 | `HYP-014` | `EXP-014a` | `src/experiments/exp_014a_flux/` | `RUN-EXP-014a-01` | `exp/EXP-014a-01` | Inconclusive (collapse at $\lambda=1.0$) | MUTATE (soft penalty) | Autonomous (Gate I) |

---

## 5. Resource & Compute Accounting

- **Total Allocated Compute Budget**: [e.g., 500 Compute-Hours]
- **Compute Consumed to Date**: [e.g., 0.12 Compute-Hours]
- **Remaining Compute Budget**: [e.g., 499.88 Compute-Hours]
- **Autonomous Checkpoint Horizon**: 5 cycles per burst (Current: Cycle 0 of 5)
```
