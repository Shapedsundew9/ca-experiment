# Specification Roadmap Template

This template defines the persistent state tracker for requirements and architecture refinement across sessions. Authored and maintained by `Spec: Orchestrator`.

Save active roadmap state to `docs/requirements/ROADMAP.md`.

---

````markdown
# Specification Roadmap: [Project or Feature Name]

**Status**: Planning | In Progress | Approved | Superseded
**Lead Architect**: `Spec: Orchestrator`
**Created**: YYYY-MM-DD
**Last Updated**: YYYY-MM-DD

---

## 1. Abstraction Tier Progression

| Stage | Abstraction Level | Primary Subagents | Deliverable Targets | Status | Gate Sign-Off |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Tier 0** | Domain Invariants & Goals | `Spec: PRD`, `Spec: PM`, `Spec: Arch Reviewer`, `Spec: Specification` | `docs/product/*-prd.md`, `docs/requirements/product/REQ-T0-*.md` | Planned | 🔒 Gate 0: [Pending] |
| **UX Track** | User Journeys & Ergonomics | `Spec: UX Designer`, `Spec: PM` | `docs/architecture/ux-*.md` | [Planned / Skipped] | 🔒 Gate UX: [Pending] |
| **Tier 1** | Logical Architecture & Contracts | `Spec: Arch Reviewer`, `Spec: API Architect`, `Spec: ADR Generator` | `docs/architecture/adr-*.md`, `docs/requirements/architecture/REQ-T1-*.md` | Planned | 🔒 Gate 1: [Pending] |
| **NFRs** | Quality Attributes & Budgets | `Spec: Architecture Reviewer` | Performance, memory, and security bounds | Planned | N/A (Audited in T1) |
| **Tier 2** | Tech Realization Profiles | `Spec: Specification`, `Spec: Arch Reviewer` | `docs/requirements/system/REQ-T2-*.md` | Planned | 🔒 Gate 2: [Pending] |

---

## 2. Gate Verification Sign-Off Ledger

| Gate | Scope Verified | Decision | Timestamp | Operator Sign-Off Notes |
| :--- | :--- | :--- | :--- | :--- |
| **Gate 0** | PRD problem statement, business KPIs, non-goals, and tech-agnostic domain invariants. UX stage requirement evaluated. | [Pending / Approved / Rejected] | YYYY-MM-DD HH:MM | [Operator comments & UX decision] |
| **Gate UX** | User journey maps, CLI interaction flows, edge/error states, and accessibility constraints. | [Pending / Approved / Skipped] | YYYY-MM-DD HH:MM | [Operator comments] |
| **Gate 1** | Component boundaries, OpenAPI/IDL schemas, resilience budgets, and ADR promotion to `Accepted`. | [Pending / Approved / Rejected] | YYYY-MM-DD HH:MM | [Operator comments & ADR promotions] |
| **Gate 2** | Tier 2 tech stack bindings, complete traceability matrix, test verification recipes, final handoff to Code track. | [Pending / Approved / Rejected] | YYYY-MM-DD HH:MM | [Operator final approval] |

---

## 3. Registered Requirement Sets

### Tier 0: Domain Invariants (`docs/requirements/product/`)

- [ ] `REQ-T0-[DOMAIN]-001`: [Title]
- [ ] `REQ-T0-[DOMAIN]-002`: [Title]

### Tier 1: Logical Contracts (`docs/requirements/architecture/`)

- [ ] `REQ-T1-[DOMAIN]-001`: [Title]
- [ ] `REQ-T1-[DOMAIN]-002`: [Title]

### Tier 2: Technology Realization (`docs/requirements/system/`)

- [ ] `REQ-T2-[DOMAIN]-001`: [Title]
- [ ] `REQ-T2-[DOMAIN]-002`: [Title]

---

## 4. Architectural Decision Records (ADRs)

| ADR ID | Title | Status | Date | Decision Summary |
| :--- | :--- | :--- | :--- | :--- |
| `ADR-0001` | [Decision Title] | Proposed | YYYY-MM-DD | [One-line summary] |

---

## 5. Audit Reports (`docs/requirements/audits/`)

| Report ID | Tier Audited | Auditing Subagent | Verdict | Date | Key Findings |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `AUDIT-T0-001` | Tier 0 | `Spec: PM` / `Spec: Arch Reviewer` | PASS | YYYY-MM-DD | [Summary of audit] |

---

## 6. Open Decision Forks & Unresolved Ambiguities

- **Fork 1**: [Description of trade-off, options considered, and pending operator decision].
````
