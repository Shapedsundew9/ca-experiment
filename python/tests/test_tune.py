"""Tests for automated parameter tuning framework."""

import unittest
from tools.tune import Parameter, optimize_parameters


class TestParameterTuner(unittest.TestCase):
    def test_convex_optimization_convergence(self):
        # Minimize f(x, y) = (x - 1.5)^2 + (y + 2.0)^2
        params = [
            Parameter("theta_gate", min_val=0.0, max_val=3.0, initial_val=0.5),
            Parameter("beta_theta", min_val=-4.0, max_val=0.0, initial_val=-1.0),
        ]

        def objective(p: dict[str, float]) -> float:
            return (p["theta_gate"] - 1.5) ** 2 + (p["beta_theta"] + 2.0) ** 2

        result = optimize_parameters(params, objective, max_evals=100)

        self.assertAlmostEqual(result.best_params["theta_gate"], 1.5, places=2)
        self.assertAlmostEqual(result.best_params["beta_theta"], -2.0, places=2)
        self.assertLess(result.best_score, 0.01)
        self.assertGreater(result.evaluations_count, 5)


if __name__ == "__main__":
    unittest.main()
