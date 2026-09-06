# Architectural Decision Record (ADR) Template

This template defines the standard record for documenting architectural fork points, evaluating trade-offs, and recording decisions. Authored by `Spec: ADR Generator`.

Save completed ADRs to `docs/architecture/adr-NNNN-[title-slug].md` and update the registry in `docs/architecture/README.md`.

---

````markdown
---
title: "ADR-NNNN: [Decision Title]"
status: "Proposed"  # Proposed | Accepted | Rejected | Superseded | Deprecated
date: "YYYY-MM-DD"
authors: "[Author Names/Roles]"
tags: ["architecture", "decision"]
supersedes: ""
superseded_by: ""
---

# ADR-NNNN: [Decision Title]

## Status

**Proposed** | Accepted | Rejected | Superseded | Deprecated

> *Note: ADRs are created in `Proposed` status by `Spec: ADR Generator`. They transition to `Accepted` only upon explicit operator sign-off at Gate 1 or Gate 2.*

## Context & Problem Statement

[Describe the context, business drivers, and technical forces requiring this decision. Explain the problem, constraints, and forces at play.]

## Decision Drivers

- [Driver 1 - e.g., p99 latency budget < 200ms under 1,000 concurrent connections]
- [Driver 2 - e.g., Zero external C-library runtime dependencies]
- [Driver 3 - e.g., Local-first file persistence for single-user developer workflow]

## Considered Options

- **Option 1**: [Description]
- **Option 2**: [Description]
- **Option 3**: [Description]

## Decision Outcome

**Chosen Option**: [Option Name]

### Rationale

[Detailed technical justification explaining why this option was selected over the alternatives, referencing how it satisfies the decision drivers.]

### Comparison Matrix

| Criteria | Option 1 | Option 2 (Chosen) | Option 3 |
|---|---|---|---|
| [Criterion 1] | ⚠️ [Note] | ✅ [Advantage] | ❌ [Disadvantage] |
| [Criterion 2] | ❌ [Disadvantage] | ✅ [Advantage] | ⚠️ [Note] |
| [Criterion 3] | ✅ [Advantage] | ✅ [Advantage] | ❌ [Disadvantage] |

### Rejected Alternatives & Trade-Offs

- **[Option 1]**: Rejected because [specific technical reason/constraint violation].
- **[Option 3]**: Rejected because [specific technical reason/constraint violation].

## Consequences

### Positive

- [Beneficial outcome, performance improvement, maintainability gain]
- [Alignment with architectural invariants and goals]

### Negative & Risks

- [Trade-off, introduced complexity, technical debt]
- [Operational risks or limitations to monitor]

## Implementation Notes

- [Key architectural patterns or libraries to use]
- [Configuration parameters and recommended defaults]
- [Migration or rollout steps if applicable]

## Downstream Requirements & Entity Impact

- **Affected System Elements**: `COMP-[NAME]` (if applicable)
- **Technology Profile**: `TECH-[NAME]` (if applicable)
- **Formal Requirements**: Derived formal requirements (`REQ-T1-*` / `REQ-T2-*`) authored in `docs/requirements/`.

## References

- Upstream requirements / objectives: `[Reference if applicable]`
- Related ADRs: `[adr-XXXX-...]`
- External standards & documentation: `[Links]`
````
