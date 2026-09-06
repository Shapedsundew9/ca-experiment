# Architectural Decision Records (ADRs)

This directory contains Architectural Decision Records (ADRs) authored by [`Spec: ADR Generator`](../../.github/agents/spec-adr-generator.agent.md).

## ADR Life-Cycle Statuses

- **`Proposed`**: The ADR has been drafted with trade-off analysis and options, but has not yet received human sign-off at Gate 1 or Gate 2.
- **`Accepted`**: The user has signed off on this architectural decision at a formal decision gate.
- **`Rejected`**: The architectural option was considered and explicitly rejected.
- **`Superseded`**: A subsequent ADR replaces this decision (must link to `superseded_by`).
- **`Deprecated`**: The decision is no longer in effect.

## ADR Index

| ID | Title | Status | Date | Primary Driver | Superseded By |
| :--- | :--- | :--- | :--- | :--- | :--- |
| *(None recorded yet)* | | | | | |

## Authoring Guidelines

- ADRs are formatted according to [`docs/templates/adr-template.md`](../templates/adr-template.md).
- Filenames follow the sequential 4-digit scheme: `adr-NNNN-[title-slug].md`.
- Formal derived requirements from accepted ADRs are placed in `docs/requirements/architecture/` (`REQ-T1-*`) or `docs/requirements/system/` (`REQ-T2-*`).
