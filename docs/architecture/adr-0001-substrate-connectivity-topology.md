---
title: "ADR-0001: Substrate Connectivity Topology Relaxation"
status: "Accepted"
date: "2026-09-07"
authors: "Operator, Spec: ADR Generator"
tags: ["architecture", "decision", "substrate", "topology"]
supersedes: ""
superseded_by: ""
---

<!-- markdownlint-disable-next-line MD025 -->
# ADR-0001: Substrate Connectivity Topology Relaxation

## Status

Accepted

## Context & Problem Statement

Milestone 1.2 (`HYP-2026-007`, `EXP-2026-007a`) required resolving "planar wire crossing" and inter-channel "crosstalk" between two signal paths sharing a lattice node, and had to introduce dedicated machinery (refractory-shielded crossing bridges, meander delay equalization) to solve it.

This problem is an artifact of a self-imposed design choice, not a requirement of a digital cellular automaton substrate: the substrate was embedded strictly in a 2D planar, nearest-neighbor-only lattice (see `HYP-2026-007.md`, "Topological Planarity" assumption). Nothing in `docs/vision.md` mandates a 2D planar embedding — the Black-Box Evaluator Interface ground rule explicitly states the team must "never constrain whether the substrate uses continuous fields, discrete cell states, asynchronous updates, or reservoir readouts." A 2D-planar-only topology forces artificial routing conflicts (wire crossings) that a richer connectivity model does not.

## Decision Drivers

- Avoid re-deriving software workarounds (multiplexing, shielded crossing bridges) for problems created by an arbitrary topology choice rather than by the substrate's actual computational or physical constraints.
- Preserve the campaign's North Star premise (`docs/vision.md`): a decentralized, local, thermodynamically bounded substrate — not an unconstrained global crossbar.
- Keep Tier 0/Tier 1 empirical results valid without requiring re-verification.

## Considered Options

- **Option 1: Fully arbitrary point-to-point connectivity.** Any node may form a direct, zero-cost, zero-delay edge to any other node, with no bound on degree or distance.
- **Option 2: Relax planarity, preserve locality/degree bound.** Allow higher-dimensional embeddings (3D+) and/or a small fixed number of bounded-degree long-range "small-world" shortcut edges per node, as already anticipated in `docs/vision.md`'s "Speed of Light Bottleneck" guidance. Nearest-neighbor locality and bounded fan-in/fan-out remain the default.
- **Option 3: Status quo.** Retain strict 2D planar, nearest-neighbor-only embedding for all future milestones.

## Decision Outcome

**Chosen Option**: Option 2 — Relax planarity, preserve locality/degree bound.

### Rationale

Option 1 eliminates wire-crossing problems entirely but also eliminates every property that makes this substrate scientifically distinct from a conventional boolean circuit or arbitrary graph (decentralization, wiring cost, physical realizability, the metrics in `docs/vision.md`'s Target Quantitative Metrics such as $\eta_{\text{thermo}}$). It would trivialize future milestones into a solved logic-synthesis problem rather than testing self-organizing substrate dynamics.

Option 2 removes the specific artifact (forced planar embedding) that caused the wire-crossing/crosstalk problem while keeping the locality and bounded-degree constraints that anchor the project's thermodynamic and decentralization claims. It is also already licensed by the existing roadmap language, requiring no change to the North Star.

### Comparison Matrix

| Criteria | Option 1 (Arbitrary) | Option 2 (Chosen) | Option 3 (Status Quo) |
| --- | --- | --- | --- |
| Eliminates wire-crossing/crosstalk problem | ✅ Fully | ✅ Fully | ❌ Requires shielding machinery |
| Preserves decentralization/locality premise | ❌ Discarded | ✅ Preserved | ✅ Preserved |
| Preserves thermodynamic/physical-realizability framing | ❌ Discarded | ✅ Preserved | ✅ Preserved |
| Avoids unnecessary engineering overhead (multiplexing, shielding) | ✅ Yes | ✅ Yes | ❌ No |
| Consistent with existing `docs/vision.md` guidance | ⚠️ Not addressed | ✅ Already anticipated | ✅ Default assumption |

### Rejected Alternatives & Trade-Offs

- **Option 1**: Rejected because unconstrained instant global connectivity removes the locality/decentralization property the campaign is designed to validate, reducing future milestones to conventional logic synthesis.
- **Option 3**: Rejected because it forces continued investment in software machinery (shielded crossings, meander delays) to solve a self-imposed problem with no scientific payoff.

## Consequences

### Positive

- Future substrate designs (Tier 2 onward) are not required to solve planar wire-crossing as a prerequisite for logic composition.
- Removes an unnecessary engineering burden (crossing bridges, crosstalk metrics) from future protocols.
- Fully compatible with the existing `docs/vision.md` Strategic Guidance on non-local communication mechanisms.

### Negative & Risks

- Higher-dimensional or small-world topologies introduce new design parameters (embedding dimension, shortcut degree/count) that must be specified per-protocol going forward.
- Care is needed to keep the bounded-degree/locality constraint explicit in future hypotheses, or the same drift toward Option 1 could recur by default.

## Implementation Notes

- No re-verification of Tier 0 or Tier 1 (Milestones 1.1, 1.2) results is required: the Full Adder composability result (`EXP-2026-007a`) was proven under the strictest-case topology (2D planar, nearest-neighbor only) and holds a fortiori under any relaxation of that constraint.
- Future protocol authors (starting with Milestone 2.1) should default to nearest-neighbor locality with an explicit, bounded degree budget, and may use 3D (or higher) embeddings or a small fixed number of long-range shortcut edges per node when non-local communication is required.
- Planar-crossing-specific mechanisms (refractory-shielded crossing bridge, `χ_cross` metric) from `EXP-2026-007a` are retained as valid historical evidence but are not mandatory patterns for future substrate designs.

## Downstream Requirements & Entity Impact

- **Affected System Elements**: Substrate topology definitions in future experiment packages (`src/experiments/exp_2026_00Xa_*`), starting with Milestone 2.1.
- **Formal Requirements**: None derived at this time; this ADR revises design guidance in `docs/vision.md` and `docs/research/CAMPAIGN.md` rather than introducing new formal requirements.

## References

- Upstream requirements / objectives: `docs/vision.md` (Milestone 1.2, Strategic Guidance §2 "Speed of Light Bottleneck")
- Related documents: `docs/research/hypotheses/HYP-2026-007.md`, `docs/research/protocols/EXP-2026-007a.md`, `docs/research/CAMPAIGN.md`
