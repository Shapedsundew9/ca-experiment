"""Automated parameter optimization framework for excitable substrate simulations.

Transfers the cognitive overhead of parameter search (thresholds, refractory timings,
leak factors, and coupling weights) from interactive LLM turns to local mechanical compute.
"""

from __future__ import annotations

import dataclasses
from typing import Any, Callable

import numpy as np
from scipy.optimize import minimize


@dataclasses.dataclass
class Parameter:
    """A bounded numerical parameter for dynamical substrate tuning."""

    name: str
    min_val: float
    max_val: float
    initial_val: float | None = None

    def __post_init__(self) -> None:
        if self.initial_val is None:
            self.initial_val = 0.5 * (self.min_val + self.max_val)


@dataclasses.dataclass
class TuningResult:
    """Outcome of an automated parameter optimization run."""

    best_params: dict[str, float]
    best_score: float
    evaluations_count: int
    converged: bool
    history: list[tuple[dict[str, float], float]]


def optimize_parameters(
    parameters: list[Parameter],
    objective_fn: Callable[[dict[str, float]], float],
    *,
    max_evals: int = 200,
    method: str = "Nelder-Mead",
) -> TuningResult:
    """Optimize a vector of continuous substrate parameters against an objective loss function.

    Args:
        parameters: List of Parameter specs with allowable bounds and initial guesses.
        objective_fn: Callable taking a dict of {param_name: value} and returning a loss float
                      (lower is better; e.g. 1.0 - accuracy + penalty * violations).
        max_evals: Maximum allowable evaluations.
        method: Scipy optimization algorithm (e.g. 'Nelder-Mead', 'Powell').

    Returns:
        TuningResult containing the optimal parameter dict and convergence details.
    """
    names = [p.name for p in parameters]
    bounds = [(p.min_val, p.max_val) for p in parameters]
    x0 = np.array([p.initial_val for p in parameters], dtype=float)

    history: list[tuple[dict[str, float], float]] = []

    def wrapped_objective(x: np.ndarray) -> float:
        # Clamp values to valid domain bounds
        clamped_vals = [
            float(np.clip(val, b[0], b[1])) for val, b in zip(x, bounds)
        ]
        param_dict = dict(zip(names, clamped_vals))

        score = float(objective_fn(param_dict))
        history.append((param_dict, score))
        return score

    res = minimize(
        wrapped_objective,
        x0,
        method=method,
        bounds=bounds if method not in ("Nelder-Mead",) else None,
        options={"maxiter": max_evals, "disp": False},
    )

    # Extract best candidate from history
    best_param_dict, best_score = min(history, key=lambda item: item[1])

    return TuningResult(
        best_params=best_param_dict,
        best_score=best_score,
        evaluations_count=len(history),
        converged=bool(res.success),
        history=history,
    )
