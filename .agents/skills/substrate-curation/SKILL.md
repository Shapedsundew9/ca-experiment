---
name: substrate-curation
description: >-
  Operational procedure for scientific code curation. Use for shared
  infrastructure enablement, post-acceptance factoring, behavioral equivalence
  checks, artifact standardization, or automated provenance work.
---

# Scientific Substrate Curation

Apply `.agents/skills/research-lifecycle/SKILL.md` first. That contract defines
ownership and routing; this skill defines how the curator performs assigned
engineering work.

## Invariants

- Preserve the approved hypothesis, parameters, factor grid, acceptance
  criteria, telemetry schema, and observed behavior.
- Establish an executable baseline before refactoring accepted work.
- Prefer the smallest stable shared interface that resolves demonstrated reuse
  or duplication. Do not build for hypothetical future experiments.
- Keep domain-specific mechanisms in their experiment package.
- Use repository-provided formatting, validation, and provenance automation.
- Stop and return to the orchestrator if equivalence cannot be demonstrated.

## Mode A: Infrastructure Enablement

Use this mode only for an orchestrator-issued `INFRASTRUCTURE_GAP`.

1. Read the gap report and only the shared interfaces named in scope.
2. Restate the minimum behavioral contract and a check that can falsify it.
3. Implement the smallest reusable capability needed by the blocked operation.
4. Add focused contract tests and run them.
5. Document the public interface and return its paths, usage constraints, and
   validation evidence to the orchestrator.

Do not implement the experiment, select parameters, reinterpret the protocol, or
perform unrelated repository cleanup in this mode.

## Mode B: Post-Acceptance Curation

1. **Baseline**: run the scoped experiment checks and capture deterministic
   outputs or comparison tolerances before editing.
2. **Inventory**: review `Curation Notes`, locate actual duplication, and
   distinguish shared apparatus from experiment-specific mechanism code.
3. **Factor**: promote only demonstrated reusable behavior; add focused tests for
   every new shared interface.
4. **Tidy**: update the experiment to consume shared interfaces and remove code
   proven redundant.
5. **Verify**: rerun baseline comparisons and all checks required by the work
   package. Treat unexplained output movement as a failure.
6. **Standardize**: apply repository formatting and presentation conventions
   using existing automation.
7. **Record**: perform only the provenance operations authorized by the work
   package, using repository automation where available.

## Factoring Decision

Promote code when at least one condition is true:

- It duplicates behavior already present in a shared module or another active
  package.
- The same substantial structure occurs at least three times.
- It implements a generic concern such as execution scheduling, serialization,
  statistics, reduction, validation, or rendering infrastructure.
- An approved infrastructure-gap contract requires the capability.

Apply the promotion conditions before the keep-local test. A generic concern is
the reusable implementation itself, not a thin experiment-specific call-site
adapter around an existing shared interface. Keep code local when no promotion
condition applies and it expresses the experiment's novel mechanism, has one
caller, or is a bounded adapter under the lifecycle contract. A size threshold
is a review signal, not sufficient evidence for abstraction by itself.

## Required Report

- Mode and scoped objective.
- Baseline command and result, when applicable.
- Shared interfaces added or changed.
- Duplication removed and local code intentionally retained.
- Post-change checks and equivalence evidence.
- Unresolved blockers and protected artifacts.
- Formatting and provenance status, when included in scope.

## Anti-Patterns

- Refactoring before recording a baseline.
- Altering scientific behavior to make factoring easier.
- Replacing a small local adapter with a speculative framework.
- Combining infrastructure enablement with experiment implementation.
- Repeating manual formatting or provenance loops when automation exists.
- Claiming equivalence solely because tests compile.
