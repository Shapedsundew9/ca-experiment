# Formal Requirement (r9ts) Template

This template defines the standard specification format for formal requirements authored by `Spec: Specification`.

Save requirements to `docs/requirements/[product|architecture|system]/REQ-TX-[DOMAIN]-[SEQ].md`.

---

````markdown
---
id: REQ-TX-DOMAIN-001
title: "Short Descriptive Title"
tier: 0 # 0 (Domain Invariant) | 1 (Logical Contract) | 2 (Tech Realization)
binding: shall # shall (mandatory) | should (recommended) | may (discretionary)
category: functional # functional | performance | interface | security | constraint
priority: high # critical | high | medium | low
verification_method: [test] # test | analysis | demonstration | inspection
status: draft # draft | approved | implemented | verified | rejected
is_derived: false
traces_to: [] # Upstream objective or invariant IDs (e.g., [OBJ-001])
refines: [] # Parent requirement IDs if refining an upstream tier (e.g., [REQ-T0-AUTH-001])
allocated_to: [] # Component or module IDs (e.g., [COMP-AUTH])
---

# REQ-TX-DOMAIN-001: Short Descriptive Title

## Statement

[The system SHALL / When <event>, the system SHALL / While <state>, the system SHALL...]

> *Note: Use EARS syntax with NASA modal verbs (`SHALL` for mandatory, `SHOULD` for recommended, `MAY` for discretionary). Avoid vague terms such as 'robust', 'fast', 'user-friendly', or 'efficient'.*

## Rationale

[Explain why this requirement exists, the business or technical need it fulfills, and the problem it prevents.]

## Verification Criteria

- [ ] **Method**: [test | analysis | demonstration | inspection]
- [ ] **Procedure**: [Clear, unambiguous verification steps or automated test criteria]
- [ ] **Pass Condition**: [Measurable threshold, assertion, or expected output proving satisfaction]
````
