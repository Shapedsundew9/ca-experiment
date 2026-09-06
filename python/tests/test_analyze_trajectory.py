import importlib.util
from pathlib import Path
import sys
import unittest

# Load analyze_trajectory from .agents/skills/trajectory-audit/scripts/analyze_trajectory.py
script_path = (
    Path(__file__).resolve().parents[2]
    / ".agents"
    / "skills"
    / "trajectory-audit"
    / "scripts"
    / "analyze_trajectory.py"
)
spec = importlib.util.spec_from_file_location("analyze_trajectory", script_path)
if spec is None or spec.loader is None:
    raise ImportError(f"Could not load module from {script_path}")
analyze_trajectory = importlib.util.module_from_spec(spec)
sys.modules["analyze_trajectory"] = analyze_trajectory
spec.loader.exec_module(analyze_trajectory)


class TrajectoryAnalysisTests(unittest.TestCase):
    def test_evaluate_tool_output_clean(self) -> None:
        is_err, reason = analyze_trajectory._evaluate_tool_output(
            "run_command",
            {"CommandLine": "cargo test"},
            "The command exited with code 0.\nOutput:\ntest result: ok. 5 passed",
        )
        self.assertFalse(is_err)
        self.assertEqual(reason, "")

    def test_evaluate_tool_output_exit_code_1(self) -> None:
        is_err, reason = analyze_trajectory._evaluate_tool_output(
            "run_command",
            {"CommandLine": "python3 -c 'bad code'"},
            "The command exited with code 1.\nOutput:\nSyntaxError: invalid syntax",
        )
        self.assertTrue(is_err)
        self.assertIn("SyntaxError", reason)

    def test_evaluate_tool_output_replace_file_content_mismatch(self) -> None:
        is_err, reason = analyze_trajectory._evaluate_tool_output(
            "replace_file_content",
            {"TargetFile": "/foo/bar.py"},
            "Encountered error: target content not found in file",
        )
        self.assertTrue(is_err)
        self.assertIn("mismatch", reason.lower())

    def test_parse_trajectory_clean(self) -> None:
        steps = [
            {
                "step_index": 0,
                "type": "USER_INPUT",
                "content": "<USER_REQUEST>Run cargo test</USER_REQUEST>",
                "created_at": "2026-09-06T12:00:00Z",
            },
            {
                "step_index": 1,
                "type": "PLANNER_RESPONSE",
                "tool_calls": [
                    {
                        "name": "run_command",
                        "args": {"CommandLine": "cargo test"},
                    }
                ],
                "created_at": "2026-09-06T12:00:01Z",
            },
            {
                "step_index": 2,
                "type": "GENERIC",
                "status": "DONE",
                "content": "The command exited with code 0.\ntest result: ok",
                "created_at": "2026-09-06T12:00:05Z",
            },
        ]

        analysis = analyze_trajectory.parse_trajectory(steps, conv_id="test-clean")
        self.assertEqual(analysis.total_steps, 3)
        self.assertEqual(analysis.failed_tool_calls, 0)
        self.assertEqual(analysis.score, 100)
        self.assertEqual(analysis.letter_grade, "A")
        self.assertEqual(len(analysis.anomalies), 0)
        self.assertEqual(analysis.initial_user_prompt, "Run cargo test")

    def test_parse_trajectory_failure_streak_and_loops(self) -> None:
        steps = [
            {
                "step_index": 0,
                "type": "USER_INPUT",
                "content": "Fix the bug",
                "created_at": "2026-09-06T12:00:00Z",
            },
            {
                "step_index": 1,
                "type": "PLANNER_RESPONSE",
                "tool_calls": [{"name": "run_command", "args": {"CommandLine": "python3 foo.py"}}],
            },
            {
                "step_index": 2,
                "type": "GENERIC",
                "status": "DONE",
                "content": "The command exited with code 1.\nTraceback...",
            },
            {
                "step_index": 3,
                "type": "PLANNER_RESPONSE",
                "tool_calls": [{"name": "run_command", "args": {"CommandLine": "python3 foo.py"}}],
            },
            {
                "step_index": 4,
                "type": "GENERIC",
                "status": "DONE",
                "content": "The command exited with code 1.\nTraceback...",
            },
            {
                "step_index": 5,
                "type": "PLANNER_RESPONSE",
                "tool_calls": [{"name": "run_command", "args": {"CommandLine": "python3 foo.py"}}],
            },
            {
                "step_index": 6,
                "type": "GENERIC",
                "status": "DONE",
                "content": "The command exited with code 1.\nTraceback...",
            },
        ]

        analysis = analyze_trajectory.parse_trajectory(steps, conv_id="test-failures")
        self.assertEqual(analysis.failed_tool_calls, 3)
        self.assertLess(analysis.score, 70)
        self.assertIn(analysis.letter_grade, ("D", "F"))

        categories = [a.category for a in analysis.anomalies]
        self.assertIn("Tool Friction & Retry Streak", categories)
        self.assertIn("Redundant Command Execution", categories)
        self.assertIn("Escalation Deficit (Stubborn Autonomy)", categories)

    def test_generate_markdown_report(self) -> None:
        steps = [
            {
                "step_index": 0,
                "type": "USER_INPUT",
                "content": "Check repository",
                "created_at": "2026-09-06T12:00:00Z",
            }
        ]
        analysis = analyze_trajectory.parse_trajectory(steps, conv_id="test-report")
        report = analyze_trajectory.generate_markdown_report(analysis)
        self.assertIn("# Trajectory Efficiency Postmortem: `test-rep`", report)
        self.assertIn("Executive Telemetry", report)

    def test_parse_trajectory_multi_turn(self) -> None:
        steps = [
            {
                "step_index": 0,
                "type": "USER_INPUT",
                "content": "Turn 1 request",
                "created_at": "2026-09-06T12:00:00Z",
            },
            {
                "step_index": 1,
                "type": "PLANNER_RESPONSE",
                "tool_calls": [{"name": "view_file", "args": {"AbsolutePath": "/foo"}}],
                "created_at": "2026-09-06T12:00:05Z",
            },
            {
                "step_index": 2,
                "type": "GENERIC",
                "status": "DONE",
                "content": "file content",
                "created_at": "2026-09-06T12:00:06Z",
            },
            {
                "step_index": 3,
                "type": "USER_INPUT",
                "content": "Turn 2 request",
                "created_at": "2026-09-06T12:01:00Z",
            },
            {
                "step_index": 4,
                "type": "PLANNER_RESPONSE",
                "tool_calls": [{"name": "run_command", "args": {"CommandLine": "cargo test"}}],
                "created_at": "2026-09-06T12:01:05Z",
            },
            {
                "step_index": 5,
                "type": "GENERIC",
                "status": "DONE",
                "content": "The command exited with code 0.\ntest ok",
                "created_at": "2026-09-06T12:01:10Z",
            },
        ]
        analysis = analyze_trajectory.parse_trajectory(steps, conv_id="test-multi-turn")
        self.assertEqual(analysis.user_turns_count, 2)
        self.assertEqual(len(analysis.turns), 2)
        self.assertEqual(analysis.turns[0].user_prompt, "Turn 1 request")
        self.assertEqual(analysis.turns[0].total_tool_calls, 1)
        self.assertEqual(analysis.turns[0].tool_counts.get("view_file"), 1)
        self.assertEqual(analysis.turns[1].user_prompt, "Turn 2 request")
        self.assertEqual(analysis.turns[1].total_tool_calls, 1)
        self.assertEqual(analysis.turns[1].tool_counts.get("run_command"), 1)

        report = analyze_trajectory.generate_markdown_report(analysis)
        self.assertIn("Turn-by-Turn Telemetry Breakdown", report)
        self.assertIn("Turn 1", report)
        self.assertIn("Turn 2", report)


if __name__ == "__main__":
    unittest.main()

