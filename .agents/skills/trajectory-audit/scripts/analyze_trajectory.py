#!/usr/bin/env python3
"""Trajectory Analysis Tool.

Deterministically parses Antigravity conversation transcripts to identify
execution friction, tool failure loops, exploration drift, and token bloat.
Segments telemetry by user turn and enables zero-token step inspection.
"""

from __future__ import annotations

import argparse
from collections import Counter
from dataclasses import asdict, dataclass, field
from datetime import datetime
import json
import os
from pathlib import Path
import re
import sys
from typing import Any, Dict, List, Optional, Tuple


@dataclass
class ToolExecution:
    step_index: int
    tool_name: str
    args: Dict[str, Any]
    output_step_index: Optional[int] = None
    output_preview: str = ""
    payload_size: int = 0
    is_error: bool = False
    error_reason: str = ""
    turn_index: int = 1


@dataclass
class Anomaly:
    category: str
    description: str
    step_indices: List[int]
    severity: str  # "high", "medium", "low"
    turn_index: int = 1
    details: Dict[str, Any] = field(default_factory=dict)


@dataclass
class TurnAnalysis:
    turn_index: int
    start_step_index: int
    end_step_index: int
    user_prompt: str
    duration_seconds: float = 0.0
    total_steps: int = 0
    planner_turns_count: int = 0
    total_tool_calls: int = 0
    failed_tool_calls: int = 0
    tool_counts: Dict[str, int] = field(default_factory=dict)
    tool_failures: Dict[str, int] = field(default_factory=dict)
    payload_bytes: int = 0
    estimated_payload_tokens: int = 0
    anomalies: List[Anomaly] = field(default_factory=list)


@dataclass
class TrajectoryAnalysis:
    conversation_id: str
    conversation_path: str
    start_time: Optional[str] = None
    end_time: Optional[str] = None
    duration_seconds: float = 0.0
    total_steps: int = 0
    user_turns_count: int = 0
    planner_turns_count: int = 0
    initial_user_prompt: str = ""
    subsequent_user_prompts: List[str] = field(default_factory=list)
    total_tool_calls: int = 0
    failed_tool_calls: int = 0
    tool_stats: Dict[str, Dict[str, Any]] = field(default_factory=dict)
    total_payload_bytes: int = 0
    estimated_payload_tokens: int = 0
    estimated_wasted_tokens: int = 0
    anomalies: List[Anomaly] = field(default_factory=list)
    turns: List[TurnAnalysis] = field(default_factory=list)
    score: int = 100
    letter_grade: str = "A"
    deductions: List[str] = field(default_factory=list)
    tool_executions: List[ToolExecution] = field(default_factory=list)


def default_brain_dir() -> Path:
    """Resolve the default brain directory from environment or standard paths."""
    env_brain = os.environ.get("ANTIGRAVITY_BRAIN_DIR")
    if env_brain and Path(env_brain).is_dir():
        return Path(env_brain)

    cli_brain = Path("/home/vscode/.gemini/antigravity-cli/brain")
    if cli_brain.is_dir():
        return cli_brain

    home_brain = Path.home() / ".gemini" / "antigravity-cli" / "brain"
    return home_brain


def find_target_conversation(
    brain_dir: Path,
    target: str = "latest",
    exclude_id: Optional[str] = None,
) -> Path:
    """Locate the target conversation folder by path, ID, prefix, or latest."""
    if not brain_dir.is_dir():
        raise FileNotFoundError(f"Brain directory does not exist: {brain_dir}")

    target_path = Path(target)
    if target_path.is_dir() and (target_path / ".system_generated").exists():
        return target_path

    candidates: List[Tuple[float, Path]] = []
    for entry in brain_dir.iterdir():
        if entry.is_dir() and not entry.name.startswith("."):
            t_file = entry / ".system_generated" / "logs" / "transcript.jsonl"
            if t_file.is_file():
                mtime = entry.stat().st_mtime
                candidates.append((mtime, entry))

    if not candidates:
        raise FileNotFoundError(f"No conversation transcripts found in {brain_dir}")

    candidates.sort(key=lambda x: x[0], reverse=True)

    if target.lower() in ("latest", "last"):
        for _, c_path in candidates:
            if exclude_id and (c_path.name == exclude_id or c_path.name.startswith(exclude_id)):
                continue
            return c_path
        return candidates[0][1]

    for _, c_path in candidates:
        if c_path.name == target or c_path.name.startswith(target):
            return c_path

    raise FileNotFoundError(
        f"Could not find conversation matching '{target}' in {brain_dir}. "
        f"Available ({len(candidates)}): {[c[1].name[:8] for c in candidates[:5]]}"
    )


def load_transcript(conv_dir: Path, prefer_full: bool = False) -> List[Dict[str, Any]]:
    """Load steps from transcript.jsonl or transcript_full.jsonl sorted by step_index."""
    logs_dir = conv_dir / ".system_generated" / "logs"
    t_file = logs_dir / ("transcript_full.jsonl" if prefer_full and (logs_dir / "transcript_full.jsonl").exists() else "transcript.jsonl")
    if not t_file.is_file():
        raise FileNotFoundError(f"Transcript file not found: {t_file}")

    steps: List[Dict[str, Any]] = []
    with open(t_file, "r", encoding="utf-8", errors="replace") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                data = json.loads(line)
                steps.append(data)
            except json.JSONDecodeError:
                continue

    steps.sort(key=lambda s: s.get("step_index", 0))
    return steps


def _evaluate_tool_output(tool_name: str, args: Dict[str, Any], content: str) -> Tuple[bool, str]:
    """Detect failure signatures across command execution, file modifications, and searches."""
    content_lower = content.lower()

    if tool_name == "run_command":
        m = re.search(r"exited with code\s+([0-9]+)", content, re.IGNORECASE)
        if m and m.group(1) != "0":
            code = m.group(1)
            if "traceback" in content_lower:
                return True, f"Python Traceback (exit code {code})"
            if "syntaxerror" in content_lower:
                return True, f"Python SyntaxError (exit code {code})"
            if "command not found" in content_lower:
                return True, f"Command not found (exit code {code})"
            return True, f"Command exited with non-zero status ({code})"
        if "traceback (most recent call last)" in content_lower and "error" in content_lower:
            return True, "Unhandled Exception / Traceback"

    elif tool_name == "replace_file_content":
        if "error" in content_lower or "does not match" in content_lower or "not found" in content_lower:
            if "targetcontent" in content_lower or "target content" in content_lower or "not found" in content_lower:
                return True, "Target content mismatch / string not found in file"
            return True, "replace_file_content edit failure"

    elif tool_name == "write_to_file":
        if "already exists" in content_lower and not args.get("Overwrite", False):
            return True, "File already exists (overwrite not set)"
        if "permission denied" in content_lower:
            return True, "Permission denied writing file"

    elif tool_name in ("view_file", "list_dir"):
        if "no such file or directory" in content_lower or "does not exist" in content_lower:
            return True, "Path not found"
        if "permission denied" in content_lower:
            return True, "Permission denied accessing path"

    elif tool_name == "manage_task":
        if "task not found" in content_lower or "invalid task" in content_lower:
            return True, "Invalid background task reference"

    return False, ""


def _parse_time_delta(t_start: Optional[str], t_end: Optional[str]) -> float:
    """Calculate duration in seconds between two ISO timestamp strings."""
    if not t_start or not t_end:
        return 0.0
    try:
        s_dt = datetime.fromisoformat(t_start.replace("Z", "+00:00"))
        e_dt = datetime.fromisoformat(t_end.replace("Z", "+00:00"))
        return max(0.0, (e_dt - s_dt).total_seconds())
    except Exception:
        return 0.0


def parse_trajectory(
    steps: List[Dict[str, Any]],
    conv_id: str = "",
    conv_path: str = "",
) -> TrajectoryAnalysis:
    """Analyze a conversation trajectory segmented by user turns and calculate telemetry."""
    analysis = TrajectoryAnalysis(
        conversation_id=conv_id,
        conversation_path=conv_path,
        total_steps=len(steps),
    )

    if not steps:
        return analysis

    t_start = steps[0].get("created_at")
    t_end = steps[-1].get("created_at")
    analysis.start_time = t_start
    analysis.end_time = t_end
    analysis.duration_seconds = _parse_time_delta(t_start, t_end)

    step_by_index: Dict[int, Dict[str, Any]] = {
        s.get("step_index", i): s for i, s in enumerate(steps)
    }

    # 1. Identify User Turns
    user_steps = [s for s in steps if s.get("type") == "USER_INPUT"]
    analysis.user_turns_count = len(user_steps)

    turn_boundaries: List[Tuple[int, int, str, int]] = []
    # (turn_index, start_step, end_step, prompt)
    if user_steps:
        for idx, u in enumerate(user_steps):
            u_step_idx = u.get("step_index", 0)
            clean_prompt = re.sub(r"<\/?USER_REQUEST>", "", u.get("content", "")).strip()
            if idx == 0:
                analysis.initial_user_prompt = clean_prompt
            else:
                analysis.subsequent_user_prompts.append(clean_prompt)

            if idx + 1 < len(user_steps):
                next_u_idx = user_steps[idx + 1].get("step_index", u_step_idx + 1)
                end_idx = next_u_idx - 1
            else:
                end_idx = max(s.get("step_index", 0) for s in steps)

            turn_boundaries.append((idx + 1, u_step_idx, end_idx, clean_prompt))
    else:
        # Synthetic / prompt-less fallback
        turn_boundaries.append((1, 0, max(s.get("step_index", 0) for s in steps), ""))

    def get_turn_index_for_step(step_idx: int) -> int:
        for t_idx, s_start, s_end, _ in turn_boundaries:
            if s_start <= step_idx <= s_end:
                return t_idx
        return turn_boundaries[-1][0] if turn_boundaries else 1

    # 2. Extract Planner Responses & Tool Executions
    planner_steps = [s for s in steps if s.get("type") == "PLANNER_RESPONSE"]
    analysis.planner_turns_count = len(planner_steps)

    tool_executions: List[ToolExecution] = []
    tool_counts: Counter[str] = Counter()
    tool_fails: Counter[str] = Counter()

    for s in planner_steps:
        step_idx = s.get("step_index", 0)
        tool_calls = s.get("tool_calls", [])
        if not tool_calls:
            continue

        turn_idx = get_turn_index_for_step(step_idx)

        for call_offset, tc in enumerate(tool_calls):
            t_name = tc.get("name") or tc.get("function", {}).get("name") or "unknown_tool"
            raw_args = tc.get("args") or tc.get("function", {}).get("arguments") or {}

            if isinstance(raw_args, str):
                try:
                    args = json.loads(raw_args)
                except Exception:
                    args = {"raw": raw_args}
            else:
                args = raw_args

            expected_out_idx = step_idx + 1 + call_offset
            out_step = step_by_index.get(expected_out_idx, {})
            out_content = out_step.get("content", "") if out_step else ""
            out_status = out_step.get("status", "")

            payload_len = len(out_content)
            analysis.total_payload_bytes += payload_len

            is_err, err_reason = _evaluate_tool_output(t_name, args, out_content)
            if out_status == "ERROR":
                is_err = True
                if not err_reason:
                    err_reason = f"Tool step status returned ERROR: {out_content[:80]}"

            tool_counts[t_name] += 1
            if is_err:
                tool_fails[t_name] += 1

            preview = out_content.strip()
            if len(preview) > 200:
                preview = preview[:200] + "..."

            exec_entry = ToolExecution(
                step_index=step_idx,
                tool_name=t_name,
                args=args,
                output_step_index=expected_out_idx if out_step else None,
                output_preview=preview,
                payload_size=payload_len,
                is_error=is_err,
                error_reason=err_reason,
                turn_index=turn_idx,
            )
            tool_executions.append(exec_entry)

    analysis.tool_executions = tool_executions
    analysis.total_tool_calls = len(tool_executions)
    analysis.failed_tool_calls = sum(1 for e in tool_executions if e.is_error)
    analysis.estimated_payload_tokens = analysis.total_payload_bytes // 4

    for name, count in tool_counts.items():
        f_count = tool_fails[name]
        analysis.tool_stats[name] = {
            "calls": count,
            "failures": f_count,
            "failure_rate": round(f_count / count * 100, 1) if count > 0 else 0.0,
        }

    # 3. Anomaly Detection
    anomalies: List[Anomaly] = []
    wasted_tokens = 0

    # 3.1 Consecutive Failure Streaks
    current_streak: List[ToolExecution] = []
    for e in tool_executions:
        if e.is_error:
            current_streak.append(e)
            wasted_tokens += (e.payload_size // 4) + 250
        else:
            if len(current_streak) >= 2:
                anomalies.append(
                    Anomaly(
                        category="Tool Friction & Retry Streak",
                        description=(
                            f"Streak of {len(current_streak)} consecutive failed tool calls "
                            f"({', '.join(t.tool_name for t in current_streak)}). "
                            f"Last error: {current_streak[-1].error_reason}"
                        ),
                        step_indices=[t.step_index for t in current_streak],
                        severity="high" if len(current_streak) >= 3 else "medium",
                        turn_index=current_streak[0].turn_index,
                        details={"streak_length": len(current_streak)},
                    )
                )
            current_streak = []
    if len(current_streak) >= 2:
        anomalies.append(
            Anomaly(
                category="Tool Friction & Retry Streak",
                description=(
                    f"Ended on a streak of {len(current_streak)} consecutive failed tool calls. "
                    f"Last error: {current_streak[-1].error_reason}"
                ),
                step_indices=[t.step_index for t in current_streak],
                severity="high",
                turn_index=current_streak[0].turn_index,
                details={"streak_length": len(current_streak)},
            )
        )

    # 3.2 Redundant Command Execution
    recent_cmds: List[Tuple[int, str, int]] = []  # (step_idx, norm_cmd, turn_idx)
    for e in tool_executions:
        if e.tool_name == "run_command":
            cmd = e.args.get("CommandLine", "").strip()
            norm_cmd = " ".join(cmd.split())
            if norm_cmd:
                prev_matches = [idx for idx, c, _ in recent_cmds[-3:] if c == norm_cmd]
                if prev_matches:
                    anomalies.append(
                        Anomaly(
                            category="Redundant Command Execution",
                            description=f"Command repeated identically within short interval: `{norm_cmd[:80]}`",
                            step_indices=[prev_matches[-1], e.step_index],
                            severity="medium",
                            turn_index=e.turn_index,
                            details={"command": norm_cmd[:120]},
                        )
                    )
                    wasted_tokens += (e.payload_size // 4) + 200
                recent_cmds.append((e.step_index, norm_cmd, e.turn_index))

    # 3.3 Context Bloat Hotspots
    for e in tool_executions:
        if e.payload_size > 15000:
            tokens = e.payload_size // 4
            anomalies.append(
                Anomaly(
                    category="Context Bloat Hotspot",
                    description=(
                        f"Large tool output from `{e.tool_name}`: "
                        f"{e.payload_size:,} bytes (~{tokens:,} tokens). "
                        f"Inflates subsequent context turns."
                    ),
                    step_indices=[e.step_index, e.output_step_index or e.step_index],
                    severity="high" if e.payload_size > 30000 else "medium",
                    turn_index=e.turn_index,
                    details={"bytes": e.payload_size, "tokens": tokens},
                )
            )

    # 3.4 Exploration Drift
    search_streak: List[ToolExecution] = []
    for e in tool_executions:
        if e.tool_name in ("find_by_name", "grep_search"):
            out_preview = e.output_preview.lower()
            if "total results: 0" in out_preview or "no matches found" in out_preview or "[]" in out_preview:
                search_streak.append(e)
            else:
                if len(search_streak) >= 3:
                    anomalies.append(
                        Anomaly(
                            category="Exploration Drift (Zero-Hit Searches)",
                            description=f"Sequence of {len(search_streak)} consecutive searches returned zero results without refinement.",
                            step_indices=[s.step_index for s in search_streak],
                            severity="medium",
                            turn_index=search_streak[0].turn_index,
                        )
                    )
                    wasted_tokens += sum(s.payload_size // 4 + 150 for s in search_streak)
                search_streak = []
        else:
            if len(search_streak) >= 3:
                anomalies.append(
                    Anomaly(
                        category="Exploration Drift (Zero-Hit Searches)",
                        description=f"Sequence of {len(search_streak)} consecutive searches returned zero results.",
                        step_indices=[s.step_index for s in search_streak],
                        severity="medium",
                        turn_index=search_streak[0].turn_index,
                    )
                )
                wasted_tokens += sum(s.payload_size // 4 + 150 for s in search_streak)
            search_streak = []

    # 3.5 Escalation Deficit
    questions_asked = tool_counts.get("ask_question", 0)
    high_sev_streaks = [a for a in anomalies if a.category == "Tool Friction & Retry Streak" and a.severity == "high"]
    if high_sev_streaks and questions_asked == 0:
        anomalies.append(
            Anomaly(
                category="Escalation Deficit (Stubborn Autonomy)",
                description=(
                    f"Agent experienced severe failure streaks ({len(high_sev_streaks)} streak(s) of 3+ consecutive errors) "
                    f"but never paused to clarify or ask the user via `ask_question`."
                ),
                step_indices=[idx for a in high_sev_streaks for idx in a.step_indices],
                severity="high",
                turn_index=high_sev_streaks[0].turn_index,
            )
        )

    analysis.anomalies = anomalies
    analysis.estimated_wasted_tokens = wasted_tokens

    # 4. Turn Telemetry Segmentation
    turns: List[TurnAnalysis] = []
    for t_idx, s_start, s_end, prompt in turn_boundaries:
        turn_steps = [s for s in steps if s_start <= s.get("step_index", 0) <= s_end]
        turn_start_time = turn_steps[0].get("created_at") if turn_steps else None
        turn_end_time = turn_steps[-1].get("created_at") if turn_steps else None
        turn_dur = _parse_time_delta(turn_start_time, turn_end_time)

        turn_execs = [e for e in tool_executions if e.turn_index == t_idx]
        t_counts: Counter[str] = Counter(e.tool_name for e in turn_execs)
        t_fails: Counter[str] = Counter(e.tool_name for e in turn_execs if e.is_error)
        turn_payload = sum(e.payload_size for e in turn_execs)
        turn_anomalies = [a for a in anomalies if a.turn_index == t_idx]

        t_analysis = TurnAnalysis(
            turn_index=t_idx,
            start_step_index=s_start,
            end_step_index=s_end,
            user_prompt=prompt,
            duration_seconds=turn_dur,
            total_steps=len(turn_steps),
            planner_turns_count=sum(1 for s in turn_steps if s.get("type") == "PLANNER_RESPONSE"),
            total_tool_calls=len(turn_execs),
            failed_tool_calls=sum(1 for e in turn_execs if e.is_error),
            tool_counts=dict(t_counts),
            tool_failures=dict(t_fails),
            payload_bytes=turn_payload,
            estimated_payload_tokens=turn_payload // 4,
            anomalies=turn_anomalies,
        )
        turns.append(t_analysis)

    analysis.turns = turns

    # 5. Scoring Engine
    score = 100
    deductions: List[str] = []

    if analysis.total_tool_calls > 0:
        fail_pct = (analysis.failed_tool_calls / analysis.total_tool_calls) * 100
        if fail_pct > 0:
            pts = min(35, int(fail_pct * 0.7))
            score -= pts
            deductions.append(f"-{pts} pts: Tool failure rate of {fail_pct:.1f}% ({analysis.failed_tool_calls}/{analysis.total_tool_calls} calls)")

    high_count = sum(1 for a in anomalies if a.severity == "high")
    med_count = sum(1 for a in anomalies if a.severity == "medium")
    if high_count > 0:
        pts = min(30, high_count * 10)
        score -= pts
        deductions.append(f"-{pts} pts: {high_count} high-severity anomaly event(s)")
    if med_count > 0:
        pts = min(20, med_count * 4)
        score -= pts
        deductions.append(f"-{pts} pts: {med_count} medium-severity anomaly event(s)")

    bloat_events = [a for a in anomalies if a.category == "Context Bloat Hotspot"]
    if len(bloat_events) >= 2:
        pts = min(15, len(bloat_events) * 3)
        score -= pts
        deductions.append(f"-{pts} pts: {len(bloat_events)} large payload spikes (>15KB) polluting context")

    score = max(10, min(100, score))
    analysis.score = score
    analysis.deductions = deductions

    if score >= 90:
        analysis.letter_grade = "A"
    elif score >= 80:
        analysis.letter_grade = "B"
    elif score >= 70:
        analysis.letter_grade = "C"
    elif score >= 60:
        analysis.letter_grade = "D"
    else:
        analysis.letter_grade = "F"

    return analysis


def generate_markdown_report(analysis: TrajectoryAnalysis, detailed: bool = True) -> str:
    """Format analysis into a clean GitHub Markdown postmortem summary with turn segmentation."""
    dur_min = round(analysis.duration_seconds / 60, 1)

    lines = [
        f"# Trajectory Efficiency Postmortem: `{analysis.conversation_id[:8]}`",
        "",
        f"> **Full Conversation ID**: `{analysis.conversation_id}`  ",
        f"> **Duration**: {dur_min} min ({int(analysis.duration_seconds)}s) | **Total Steps**: {analysis.total_steps} | **Efficiency Grade**: **{analysis.letter_grade} ({analysis.score}/100)**",
        "",
        "## 1. Executive Telemetry",
        "",
        "| Metric | Value | Assessment |",
        "| :--- | :--- | :--- |",
        f"| **User Turns** | {analysis.user_turns_count} | {'Single turn execution' if analysis.user_turns_count == 1 else f'{analysis.user_turns_count} distinct user prompts'} |",
        f"| **Planner Turns** | {analysis.planner_turns_count} | {round(analysis.total_tool_calls / max(1, analysis.planner_turns_count), 1)} tools/turn average |",
        f"| **Total Tool Calls** | {analysis.total_tool_calls} | {'Concise' if analysis.total_tool_calls < 15 else 'Extensive execution'} |",
        f"| **Failed Tool Calls** | {analysis.failed_tool_calls} ({round(analysis.failed_tool_calls / max(1, analysis.total_tool_calls) * 100, 1)}%) | {'Clean' if analysis.failed_tool_calls == 0 else 'Tool friction detected'} |",
        f"| **Payload Volume** | {analysis.total_payload_bytes:,} bytes (~{analysis.estimated_payload_tokens:,} tokens) | {'Lean' if analysis.total_payload_bytes < 50000 else 'Context heavy'} |",
        f"| **Estimated Wasted Tokens** | ~{analysis.estimated_wasted_tokens:,} tokens | Direct tool retry / failure waste |",
        "",
    ]

    # Turn-by-Turn Breakdown Table
    if analysis.turns:
        lines.extend([
            "## 2. Turn-by-Turn Telemetry Breakdown",
            "",
            "| Turn | Step Range | Duration | Tool Calls | Failures | Top Tools | Prompt Summary |",
            "| :--- | :--- | :--- | :--- | :--- | :--- | :--- |",
        ])
        for t in analysis.turns:
            t_dur_min = round(t.duration_seconds / 60, 1)
            # Top 2 tools
            top_t = sorted(t.tool_counts.items(), key=lambda x: x[1], reverse=True)[:2]
            top_str = ", ".join(f"`{k}` ({v})" for k, v in top_t) if top_t else "None"
            prompt_clean = t.user_prompt.replace("\n", " ").replace("|", "\\|")
            if len(prompt_clean) > 80:
                prompt_clean = prompt_clean[:77] + "..."
            lines.append(
                f"| **Turn {t.turn_index}** | `#{t.start_step_index}`–`#{t.end_step_index}` | {t_dur_min}m | "
                f"{t.total_tool_calls} | {t.failed_tool_calls} | {top_str} | *\"{prompt_clean}\"* |"
            )
        lines.append("")

    # Tool Execution Breakdown Table
    if analysis.tool_stats:
        lines.extend([
            "## 3. Tool Execution Breakdown",
            "",
            "| Tool Name | Calls | Failures | Error Rate | Status |",
            "| :--- | :--- | :--- | :--- | :--- |",
        ])
        for t_name, stat in sorted(analysis.tool_stats.items(), key=lambda x: x[1]["calls"], reverse=True):
            status = "🟢 Clean" if stat["failures"] == 0 else ("🔴 High Friction" if stat["failure_rate"] > 25 else "🟡 Minor Errors")
            lines.append(f"| `{t_name}` | {stat['calls']} | {stat['failures']} | {stat['failure_rate']}% | {status} |")
        lines.append("")

    # Scorecard Deductions
    lines.extend([
        "## 4. Scorecard & Friction Deductions",
        "",
        f"**Overall Score: {analysis.score}/100 (Grade: {analysis.letter_grade})**",
        "",
    ])
    if analysis.deductions:
        for d in analysis.deductions:
            lines.append(f"- {d}")
    else:
        lines.append("- No penalty deductions applied. Highly efficient execution!")
    lines.append("")

    # Anomalies
    lines.extend([
        "## 5. Detected Inefficiencies & Anomalies",
        "",
    ])
    if not analysis.anomalies:
        lines.append("No critical friction patterns or loops detected.")
    else:
        lines.extend([
            "| Severity | Turn | Category | Description | Involved Steps |",
            "| :--- | :--- | :--- | :--- | :--- |",
        ])
        for a in analysis.anomalies:
            sev_icon = "🔴 High" if a.severity == "high" else ("🟡 Medium" if a.severity == "medium" else "⚪ Low")
            steps_str = ", ".join(f"`#{idx}`" for idx in a.step_indices[:6])
            if len(a.step_indices) > 6:
                steps_str += f" (+{len(a.step_indices) - 6} more)"
            desc = a.description.replace("|", "\\|")
            lines.append(f"| {sev_icon} | Turn {a.turn_index} | **{a.category}** | {desc} | {steps_str} |")
        lines.append("")

    # Detailed Step Trace if requested
    if detailed and analysis.anomalies:
        lines.extend([
            "## 6. Friction Step Trace",
            "",
            "Examine these specific step outputs to inspect the failure root causes without re-reading the entire transcript:",
            "",
        ])
        err_steps = [e for e in analysis.tool_executions if e.is_error]
        for e in err_steps[:10]:
            lines.extend([
                f"### Step #{e.step_index} (`{e.tool_name}` - Turn {e.turn_index})",
                f"- **Failure**: {e.error_reason or 'Error response'}",
                f"- **Output Preview**:",
                "```text",
                e.output_preview[:250],
                "```",
                "",
            ])
        if len(err_steps) > 10:
            lines.append(f"*...and {len(err_steps) - 10} more failed steps.*")
            lines.append("")

    lines.extend([
        "## 7. Synthesis & Recommended Next Steps",
        "",
        "To formulate the final recommendations:",
        "1. Check if the initial prompt had ambiguities that caused early wandering.",
        "2. Check if the agent hit repeated syntax/quoting errors with bash or python one-liners.",
        "3. Check if the agent escalated to the user when blocked, or stubbornly looped.",
        "4. Separate user-directed follow-up tasks from autonomous drift.",
    ])

    return "\n".join(lines)


def inspect_single_step(conv_dir: Path, step_idx: int) -> int:
    """Safely inspect a single step from transcript without blowing up context or crashing."""
    try:
        steps = load_transcript(conv_dir, prefer_full=True)
    except Exception as e:
        sys.stderr.write(f"Error loading transcript: {e}\n")
        return 1

    target = None
    next_step = None
    for i, s in enumerate(steps):
        if s.get("step_index") == step_idx:
            target = s
            if i + 1 < len(steps):
                next_step = steps[i + 1]
            break

    if not target:
        sys.stderr.write(f"Step #{step_idx} not found in {conv_dir.name}\n")
        return 1

    print(f"=== STEP #{step_idx} ({target.get('type')}) ===")
    print(f"Timestamp: {target.get('created_at')} | Status: {target.get('status')} | Source: {target.get('source')}")

    if thinking := target.get("thinking"):
        print("\n--- Thinking ---")
        print(thinking.strip())

    if tool_calls := target.get("tool_calls"):
        print("\n--- Tool Calls ---")
        for tc in tool_calls:
            t_name = tc.get("name") or tc.get("function", {}).get("name")
            args = tc.get("args") or tc.get("function", {}).get("arguments")
            if isinstance(args, str):
                try:
                    args = json.loads(args)
                except Exception:
                    pass
            print(f"Tool: {t_name}")
            print(json.dumps(args, indent=2))

    if content := target.get("content"):
        print("\n--- Content ---")
        print(content.strip())

    if next_step and next_step.get("type") == "GENERIC":
        print(f"\n--- Output (Step #{next_step.get('step_index')}) ---")
        out_c = next_step.get("content", "").strip()
        if len(out_c) > 1000:
            print(out_c[:1000] + f"\n... [truncated {len(out_c)-1000} chars]")
        else:
            print(out_c)

    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Analyze Antigravity conversation trajectories for execution inefficiencies and token bloat."
    )
    parser.add_argument(
        "-c",
        "--conversation",
        default="latest",
        help="Conversation ID, prefix, directory path, or 'latest' (default: latest)",
    )
    parser.add_argument(
        "--brain-dir",
        type=Path,
        default=None,
        help="Path to brain directory (default: ~/.gemini/antigravity-cli/brain)",
    )
    parser.add_argument(
        "--exclude",
        default=None,
        help="Conversation ID to exclude when resolving 'latest' (e.g. current conversation ID)",
    )
    parser.add_argument(
        "--step",
        type=int,
        default=None,
        help="Inspect a specific step index in detail (zero token waste)",
    )
    parser.add_argument(
        "--format",
        choices=["markdown", "json"],
        default="markdown",
        help="Output format (markdown or json, default: markdown)",
    )
    parser.add_argument(
        "-o",
        "--output",
        type=Path,
        default=None,
        help="Save report to specified file path instead of stdout",
    )
    parser.add_argument(
        "--no-detailed",
        action="store_true",
        help="Omit detailed step preview traces from markdown output",
    )

    args = parser.parse_args()

    brain_dir = args.brain_dir or default_brain_dir()

    try:
        conv_dir = find_target_conversation(
            brain_dir=brain_dir,
            target=args.conversation,
            exclude_id=args.exclude,
        )
    except FileNotFoundError as e:
        sys.stderr.write(f"Error: {e}\n")
        return 1

    if args.step is not None:
        return inspect_single_step(conv_dir, args.step)

    try:
        steps = load_transcript(conv_dir)
        analysis = parse_trajectory(
            steps=steps,
            conv_id=conv_dir.name,
            conv_path=str(conv_dir),
        )
    except Exception as e:
        sys.stderr.write(f"Error analyzing conversation {conv_dir.name}: {e}\n")
        return 2

    if args.format == "json":
        data = asdict(analysis)
        report = json.dumps(data, indent=2)
    else:
        report = generate_markdown_report(analysis, detailed=not args.no_detailed)

    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(report, encoding="utf-8")
        sys.stderr.write(f"Report written to {args.output}\n")
    else:
        print(report)

    return 0


if __name__ == "__main__":
    sys.exit(main())
