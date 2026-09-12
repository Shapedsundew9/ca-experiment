---
name: research-lifecycle
description: >-
  Canonical phase and handoff contract for depth-one scientific workflows. Use
  when orchestrating, designing, executing, or curating experiments; deciding
  between local scaffolding and shared infrastructure work; or preparing a
  cross-role work package.
---

# Research Lifecycle Contract

This skill is the normative contract for scientific workflow ownership. Agent
definitions state role-specific behavior; they must not restate or override this
contract.

## Delegation Topology

The supported topology is one level deep:

```text
operator -> orchestrator -> leaf agent
```

- The orchestrator is the only agent that dispatches leaf agents.
- Leaf agents never dispatch or hand work directly to another agent.
- Every phase transition returns to the orchestrator through workspace artifacts
  and a concise completion or blocker report.
- A leaf that is blocked records the blocker and stops. The orchestrator decides
  whether another specialist should run and later issues a fresh work package.

## Role Ownership

| Concern | Theory and protocol | Execution and analysis | Curation |
| :--- | :--- | :--- | :--- |
| Research direction | Formalize the assigned question | Do not change | Do not change |
| Hypotheses, controls, acceptance criteria | Own | Implement faithfully | Preserve |
| Novel experimental mechanism | Specify | Implement | Preserve |
| Existing shared components | Identify required capabilities | Reuse | Improve when authorized |
| Experiment-local adapters | Describe only when essential | May add when bounded below | Remove or retain intentionally |
| Shared abstractions and generic tooling | Record requirements | Do not design during execution | Own |
| Telemetry and diagnostic plots | Specify minimum evidence | Produce functional evidence | Standardize presentation |
| Repository-wide formatting and provenance | Do not perform | Defer | Own |

## Minimum Novel Delta

Before implementation, inspect only the shared components named in the work
package and the nearest relevant interfaces. Reuse suitable components and keep
the experiment package focused on the mechanism needed to test the hypothesis.
Do not copy an earlier experiment merely because its directory shape is
convenient.

### Bounded Local Scaffolding

Execution may add local scaffolding without another agent cycle only when all of
these conditions hold:

1. It is confined to one experiment and has one known caller.
2. It adapts data, configuration, or calls around existing behavior; it does not
   copy an algorithm, simulation kernel, statistical routine, runner, serializer,
   or visualization framework.
3. It does not change a shared public API or a shared-library directory.
4. It fits in one small helper or module and is expected to remain below roughly
   100 non-test lines. This is a review trigger, not a target or a license to
   compress code.
5. It is listed under `Curation Notes` in the run manifest.

When these conditions hold, finishing the experiment is cheaper than interrupting
it. Record any plausible reuse opportunity as `CURATION_CANDIDATE` and continue.

### Material Infrastructure Gap

Return `INFRASTRUCTURE_GAP` instead of building a local workaround when any of
these conditions holds:

- Correct execution requires copying behavior from a shared module or another
  experiment.
- The missing capability is a generic runner, serializer, statistical routine,
  data reducer, visualization framework, simulation kernel, or similar apparatus.
- The proposed workaround would modify shared APIs, create a second
  implementation of an existing concern, repeat substantial structure three or
  more times, or exceed the bounded-scaffolding review trigger.
- The current shared interface cannot represent the protocol without changing
  scientific behavior.

An execution leaf must not invoke a curator. It returns control to the
orchestrator with:

```text
STATUS: INFRASTRUCTURE_GAP
Blocked step: <specific operation>
Required capability: <behavior, not a preferred implementation>
Evidence: <existing interfaces inspected and why they are insufficient>
Minimum contract: <inputs, outputs, and invariants>
Preserved artifacts: <paths already created>
```

The orchestrator weighs the context-switch cost. It may narrow the experiment,
approve bounded local scaffolding that satisfies the rules above, or dispatch a
curator in infrastructure-enablement mode and then issue a fresh execution work
package with the resulting interface and preserved context.

## Phase Contracts

### Theory and Protocol

Produce the smallest complete falsifiable specification. Include required shared
capabilities and known gaps, but do not design repository-wide frameworks or
publication assets.

### Execution and Analysis

Implement the novel mechanism, focused tests, reproducible execution, reduced
telemetry, and a concise diagnostic verdict. Use functional plots only when they
are needed to interpret evidence. Leave formatting, generalization, and
provenance work to curation.

The draft run manifest must include:

```markdown
## Curation Notes
- Local scaffolding: [path and purpose, or none]
- CURATION_CANDIDATE: [reusable pattern, or none]
- Infrastructure gaps: [unresolved capability, or none]
```

### Curation

Curation has two orchestrator-dispatched modes:

1. **Infrastructure enablement**: add or improve a shared capability required by
   a blocked experiment, validate its contract, and return it to the orchestrator.
   Do not implement or tune the experiment itself.
2. **Post-acceptance curation**: establish a behavioral baseline, factor reusable
   code, remove duplication, standardize artifacts, run repository-level checks,
   and record provenance.

In both modes, preserve hypotheses, parameters, acceptance criteria, and observed
behavior. Any change to those belongs in a new science work package.

Dispatch post-acceptance curation when at least one of these conditions holds:

- `Curation Notes` contains local scaffolding or a `CURATION_CANDIDATE`.
- The accepted package contains known duplication or changed shared apparatus.
- Required artifact standardization or a provenance checkpoint remains due.

Skip the curator context switch only when none of those conditions holds, the
package already consumes suitable shared interfaces, and scoped validation is
complete. The orchestrator records the skip reason in campaign state.

## Completion Rule

A scientific cycle is complete only after the orchestrator has reviewed the
empirical result and applied the curation dispatch criteria. The operator is
needed only at an explicit campaign gate or escalation point, not for ordinary
leaf-to-leaf routing.