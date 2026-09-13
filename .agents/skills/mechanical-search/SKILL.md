---
name: mechanical-search
description: >-
  Mandatory procedure for choosing numeric constants. Use when a threshold,
  weight, delay, window, refractory count, or any other scalar needs a value
  that is not derived from theory, and whenever a test fails on a numeric
  tolerance.
---

# Mechanical Parameter Search

Hand-tuning a constant is a search with a population of one and no provenance.
This skill replaces it with `crate::substrate::search`, and states the point at
which hand-tuning becomes forbidden rather than merely discouraged.

## When This Skill Applies

Any time a numeric value is needed and theory does not supply it:

- Gate thresholds, homeostatic setpoints, adaptation rates, inhibitory weights.
- Propagation delays, evaluation windows, refractory ticks, quench durations.
- Any topological scalar exposed as a parameter: ring size, track length, slot
  count.

It does not apply to values fixed by the protocol, by a hypothesis, or by an
invariant. Those are not tunable; changing them is a scientific decision.

## The Tripwire

> **At most two numeric-literal edits against the same failing assertion. The
> third is forbidden.**

Countable, self-auditable, and checkable after the fact by grepping the diff for
repeated edits to the same constant. On reaching the limit:

1. Calibrate — define the space and run the search below; or
2. Stop and return an `INFRASTRUCTURE_GAP` report if the experiment cannot
   expose the constant as a search parameter.

Guessing a third value is a protocol violation even if the third guess passes.

## Interfaces

All types live in `crate::substrate::search` and are re-exported through
`crate::substrate::prelude`.

| Type | Role |
| :--- | :--- |
| `SearchSpace` / `ParamSpec` / `ParamKind` | Named parameters and their domains |
| `Objective` / `Evaluation` | Pre-registered scalar loss over a seed set |
| `DifferentialEvolution` / `RandomSearch` | Ask/tell optimizers |
| `SeedSplit` / `Budget` | Disjoint seed sets and hard stopping limits |
| `calibrate` / `CalibrationReport` | Driver and provenance record |

`ParamKind` has three variants — `Continuous { lo, hi }`, `Integer { lo, hi }`,
and `Categorical(Vec<String>)` — constructed through `ParamSpec::continuous`,
`ParamSpec::integer`, and `ParamSpec::categorical`. Integer and categorical
quantisation happens inside `SearchSpace::decode`, so discrete parameters need
no special handling.

## Invocation

```rust
use crate::substrate::search::{
    Budget, DifferentialEvolution, Evaluation, Objective, ParamSpec, SearchSpace,
    SeedSplit, calibrate,
};

/// Pre-registered objective. Committed before the search runs; never amended after.
struct RetrievalAccuracy;

impl Objective for RetrievalAccuracy {
    fn evaluate(&self, params: &serde_json::Value, seeds: &[u64]) -> Evaluation {
        let theta_gate = params["theta_gate"].as_f64().unwrap_or(0.0);
        let n_ref = params["n_ref"].as_i64().unwrap_or(1);
        // Existing experiment trial loop; seeds come from the driver, never from self.
        let accuracy = run_trials(theta_gate, n_ref, seeds);
        Evaluation::new(1.0 - accuracy).with_metric("accuracy", accuracy)
    }
}

let space = SearchSpace::new(vec![
    ParamSpec::continuous("theta_gate", 0.05, 0.95),
    ParamSpec::integer("n_ref", 1, 12),
]);

let seeds = SeedSplit::new((0..24).collect(), (1_000..1_024).collect())?;
let mut opt = DifferentialEvolution::new(space.dim(), 24, 0xC0FFEE);

let report = calibrate(&space, &RetrievalAccuracy, &mut opt, &seeds, Budget::default())?;

std::fs::write("params.json", serde_json::to_string_pretty(&report.best_params)?)?;
std::fs::write("calibration.json", serde_json::to_string_pretty(&report)?)?;
```

`DifferentialEvolution` is the default: it handles bounds naturally and descends
the plateau-heavy objectives that stall simplex methods. `RandomSearch` is the
honest baseline — any claim that the optimizer helped must beat a
Latin-hypercube design of the same evaluation count.

## Calibration and Validation

Calibration seeds fit parameters. Acceptance is evaluated on **frozen**
parameters against **disjoint** validation seeds.

- `SeedSplit::new(calibration, validation)` returns
  `Err(SearchError::OverlappingSeeds)` if the two sets intersect, so the
  overfitting guard is a property of the type rather than of agent discipline.
- The optimizer only ever sees `seeds.calibration()`. The driver scores the
  winner once on `seeds.validation()` and reports `validation_loss`.
- Quote the held-out score, never the calibration score, in any acceptance
  claim, archive entry, or run manifest.

A mechanical search without this split is an overfitting engine running roughly
900 times faster than a human. It is worse than hand-tuning, not better.

## Pre-Registration

The objective is Rust code. It must be **committed before the search runs** and
referenced by git SHA in the protocol. Never author, edit, or reweight an
objective after seeing results — that converts calibration into fitting the
acceptance criterion, which is the failure the split exists to prevent. If the
objective is wrong, say so, return to the orchestrator, and let the protocol be
amended and re-registered.

Before committing an objective, check it for completeness:

- [ ] The loss encodes **every** acceptance assertion, including comparative and
      ordering constraints between conditions and ablations — not only the
      primary performance assertion. An omitted ablation ordering is a free
      degree of freedom, and the optimizer will take it, returning parameters
      that satisfy the stated objective while invalidating the mechanism the
      experiment claims to demonstrate. Source case: the EXP-2026-011a
      calibration.

## Budget

`Budget::default()` is 10 000 evaluations capped at 60 s wall clock; the driver
halts on whichever limit binds first. If the search exhausts its budget without
meeting the acceptance threshold, **stop and report**. Do not raise the budget,
widen the bounds, or restart with a new seed — a search that needs more than
60 s on this substrate is evidence about the substrate, not a scheduling
problem.

## Artifacts

| Artifact | Contents | Lifecycle |
| :--- | :--- | :--- |
| `params.json` | `report.best_params` only | Frozen output; read once at experiment start |
| `calibration.json` | Serialised `CalibrationReport` | Search provenance; committed with the run |

`CalibrationReport` carries the space definition, optimizer name, budget, seed
split, evaluation count, stop reason, best parameters, calibration loss,
held-out validation loss, and the full trace. The run manifest records the
search budget, the evaluation count, and the held-out validation score.

An experiment must never call `calibrate` during its measured run. Calibration
is a separate, earlier step whose only output is `params.json`.
