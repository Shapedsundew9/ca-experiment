# Trajectory Inefficiency Rubric & Evaluation Criteria

This reference defines the evaluation rubric used by the `trajectory-audit` skill and the `trajectory-auditor` subagent to classify and score execution inefficiencies in AI agent conversations.

---

## 1. The Inefficiency Taxonomy

### Category 1: Tool Friction & Churn

Indicators:

- **Command Syntax & Quoting Failures**: Multiline Python `-c` commands with broken quotes or invalid syntax failing repeatedly with exit code 1.
- **File Edit Mismatches**: `replace_file_content` failing because `TargetContent` whitespace or indentation does not match, repeated 2+ times before recovering.
- **Invalid Paths / Nonexistent Files**: Invoking `view_file` or `list_dir` on paths that do not exist instead of using `find_by_name`.
- **Command Retries**: Running the exact same shell command or tool invocation within a small window without addressing the underlying error.

### Category 2: Exploration Drift ("Wandering")

Indicators:

- **Aimless Searches**: Rapidly executing `grep_search` or `find_by_name` queries that return 0 hits, changing queries arbitrarily without forming a coherent hypothesis.
- **Excessive Inspection**: Calling `list_dir` or `view_file` across many files unrelated to the user prompt.
- **Web Wandering**: Performing web searches for information that already exists in the repository or local documentation.

### Category 3: Stubborn Autonomy ("Refusal to Ask")

Indicators:

- **Failure Streaks Without User Escalation**: Continuing for 3+ consecutive failed tool calls without pausing to ask the user or invoking `ask_question`.
- **Assumption Overconfidence**: Making major architectural or stylistic assumptions when the prompt was ambiguous, rather than clarifying early.
- **Classic AI Trapping**: Getting stuck in an infinite fix-and-fail loop on an environment issue (e.g. missing API keys, disconnected database, broken build) instead of reporting the blocker to the user immediately.

### Category 4: Context & Token Bloat

Indicators:

- **Unbounded Tool Payloads**: Viewing 800+ lines of a file when only a 10-line function was relevant.
- **Unfiltered Shell Outputs**: Running commands that dump megabytes or tens of thousands of lines of logs (e.g., unfiltered test logs, `git log` without `-n`).
- **Repetitive Planning**: Generating verbose intermediate thinking steps that re-hash the same observations without making tool calls or forward progress.

### Category 5: Sub-optimal Trajectory

Indicators:

- **Trial-and-Error Code Modification**: Making 15 separate small file edits to solve a problem that a local script or compiler check could have identified in 1 step.
- **Premature Implementation**: Modifying files before verifying tests or reading existing patterns.
- **Over-Engineering**: Implementing extraneous abstractions, unused helpers, or unnecessary refactors not requested by the user prompt.

---

## 2. Scoring Methodology

The efficiency score starts at **100 points** and applies transparent deductions:

| Factor | Deduction Weight | Max Penalty |
| :--- | :--- | :--- |
| **Tool Failure Rate** | `failed_calls / total_calls * 70` | -35 pts |
| **High Severity Events** | -10 pts per 3+ error streak or stubborn refusal to escalate | -30 pts |
| **Medium Severity Events** | -4 pts per 2-error streak or redundant command repetition | -20 pts |
| **Context Bloat Spikes** | -3 pts per tool output > 15 KB | -15 pts |
| **Exploration Drift** | -4 pts per streak of 3+ consecutive zero-hit searches | -12 pts |

### Grade Scale

- **A (90–100)**: Clean, purposeful execution. Direct path from prompt to resolution. Minor or zero tool errors.
- **B (80–89)**: Good execution with minor friction (e.g., 1 isolated tool syntax error quickly fixed).
- **C (70–79)**: Moderate friction. Noticeable wandering, 1 retry streak, or unnecessary context bloat.
- **D (60–69)**: Significant inefficiency. Multiple failure streaks, redundant commands, or stubborn retry loops.
- **F (< 60)**: Severe execution failure. Chronic loops, refusal to escalate when stuck, or massive token burn on failed attempts.

---

## 3. Postmortem Recommendations Framework

Every postmortem should translate observed inefficiencies into three concrete preventative tiers:

1. **Prompt Recommendations (User Guidance)**:
   - What additional context or constraints in the initial user prompt would have prevented the agent from drifting?
2. **Project Rules (`GEMINI.md` / `AGENTS.md`)**:
   - What explicit workspace rule would prevent future agents from repeating this pattern (e.g., "Always write scratch scripts to disk rather than multiline `python -c` strings")?
3. **Tool & Workflow Helpers**:
   - Would a dedicated helper script, reusable skill, or subagent streamline this task from 30 steps to 3?
