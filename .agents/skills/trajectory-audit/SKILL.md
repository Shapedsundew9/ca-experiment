---
name: trajectory-audit
description: >-
  Audits previous AI conversations for execution friction, tool failure loops,
  exploration drift, token waste, and refusal to escalate to the user.
  Use when asked to review or evaluate a past conversation, analyze prompt execution
  inefficiencies, or postmortem token consumption.
---

# Trajectory Audit Skill

This skill provides a systematic procedure for evaluating previous agent conversations, identifying execution friction, and proposing actionable prompt, rule, and workflow improvements.

To avoid burning tokens re-reading massive transcripts, this skill pairs a deterministic Python telemetry parser with a qualitative LLM postmortem.

---

## Workflow Steps

### Step 1: Run the Deterministic Pre-Processor

Execute the trajectory analysis script using `run_command`:

```bash
# To audit the most recent completed conversation:
.venv/bin/python .agents/skills/trajectory-audit/scripts/analyze_trajectory.py --exclude "$CONVERSATION_ID"

# Or to audit a specific conversation ID:
.venv/bin/python .agents/skills/trajectory-audit/scripts/analyze_trajectory.py --conversation "<CONVERSATION_ID>"
```

*(Note: pass the active conversation ID to `--exclude` so the script inspects the prior completed session).*

The script will output:

- Executive telemetry (steps, turns, tool counts, error rates, payload volume).
- Inefficiency Scorecard (0–100 score and letter grade).
- Flagged friction anomalies with exact step indices.
- Snippet previews of failing steps.

---

### Step 2: Inspect Flagged Steps (Targeted Context Retrieval)

Do **NOT** view the entire `transcript.jsonl` file.

Only if necessary to understand the agent's internal reasoning on a specific failure, inspect the specific step in `transcript.jsonl` using `python3` or `grep_search`:

```bash
# Retrieve thinking and tool call for a specific step index:
python3 -c "
import json
with open('/home/vscode/.gemini/antigravity-cli/brain/<CONVERSATION_ID>/.system_generated/logs/transcript.jsonl') as f:
    for line in f:
        d = json.loads(line)
        if d.get('step_index') == <STEP_INDEX>:
            print('Thinking:', d.get('thinking', '')[:500])
            print('Tool Calls:', json.dumps(d.get('tool_calls', []), indent=2))
"
```

---

### Step 3: Apply the Inefficiency Rubric

Evaluate the detected friction points against the 5 core dimensions:

1. **Tool Friction & Churn**: Syntax errors, broken quotes, string mismatches, retries.
2. **Exploration Drift**: Aimless grep/file searches without a clear hypothesis.
3. **Stubborn Autonomy**: Multiple consecutive failures without asking the user or using `ask_question`.
4. **Context & Token Bloat**: Dumping huge files or command logs into context.
5. **Sub-optimal Trajectory**: Over-complicated solutions where a 2-step alternative existed.

See [Inefficiency Rubric](./references/inefficiency-rubric.md) for detailed descriptions and penalty weights.

---

### Step 4: Author the Postmortem Artifact

Create an artifact in the active conversation directory summarizing the findings:

- **Executive Scorecard**: Score, letter grade, duration, wasted tokens.
- **Root Cause Analysis**: Why did the agent struggle?
- **Actual vs. Optimal Path**: Number of steps taken vs. minimum steps required.
- **Actionable Recommendations**:
  - Prompt changes the user could make in future requests.
  - Workspace rules to add to `GEMINI.md`.
  - Reusable scripts or tool improvements.
