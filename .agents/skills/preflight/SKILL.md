---
name: preflight
description: >-
  Universal pre-flight environment, toolchain, and subagent permissions check.
  Verifies filesystem access, active language runtimes, declared dependencies, and
  ensures subagents in .agents/agents/*.md have explicit write and delegation tools configured.
---

# Pre-Flight Sanity Check Skill

The `preflight` skill provides an automated, repository-agnostic diagnostic routine to ensure all execution prerequisites, language runtimes, filesystem permissions, and subagent tool declarations are functional before launching an autonomous campaign, multi-agent workflow, or intensive execution cycle.

---

## Why Pre-Flight Matters

In Antigravity CLI and multi-agent workflows:

1. **Subagent Read-Only Default Trap**: Subagents defined in `.agents/agents/*.md` default to **read-only** exploration tools unless explicit `tools:` lists are declared in their frontmatter. CLI flags like `--dangerously-skip-permissions` only auto-approve prompts; they do **not** grant missing tools.
2. **Missing Dependencies**: Undeclared or uninstalled dependencies cause execution tasks to fail midway through deep agent reasoning loops.
3. **Write Permission Blockers**: Probing filesystem write and unlink capabilities upfront prevents downstream pipeline failures.

---

## Quick Start

Run the pre-flight check using `run_command`:

```bash
# Standard terminal output
python3 .agents/skills/preflight/scripts/preflight.py

# Markdown formatted summary (useful for logging into runbooks or artifacts)
python3 .agents/skills/preflight/scripts/preflight.py --format markdown

# JSON output for automated ingestion
python3 .agents/skills/preflight/scripts/preflight.py --format json

# Strict mode (fails on warnings as well as errors)
python3 .agents/skills/preflight/scripts/preflight.py --strict
```

---

## Discovery & Verification Capabilities

The pre-flight script requires only the **Python Standard Library** (Python 3.8+) and dynamically auto-detects the repository layout:

| Category | Diagnostic Check | Description |
| :--- | :--- | :--- |
| **Filesystem** | `workspace_write_access` | Creates, verifies, and removes an ephemeral probe file in workspace root. |
| **VCS** | `git_binary`, `git_clean_tree` | Checks git availability and detects uncommitted changes. |
| **Python** | `python_interpreter`, `python_dependencies` | Detects `.venv` / system Python and verifies packages declared in `pyproject.toml` or `requirements.txt`. |
| **Rust** | `rust_toolchain`, `cargo_manifest` | Verifies `cargo` and `rustc` presence, and validates `Cargo.toml` manifests. |
| **Node.js** | `node_runtime` | Verifies `node` availability if `package.json` is detected. |
| **Subagents** | `subagent_permissions` | Audits all `.agents/agents/*.md` definitions: ensures explicit `tools:` lists, confirms orchestrators have `invoke_subagent` and `send_message`, and confirms execution agents have `write_to_file` and `run_command`. |

---

## Remediation Playbook

When a pre-flight check fails, apply the appropriate fix:

- **Missing Python Dependency**:
  Install package and declare in manifest:

  ```bash
  .venv/bin/pip install <package-name>
  ```

- **Missing Rust Toolchain / Manifest Error**:
  Verify toolchain with `cargo --version` and check project syntax with `cargo check`.
- **Subagent Missing Tools**:
  Inspect `.agents/agents/<agent-name>.md` and add required tools to the frontmatter:

  ```yaml
  ---
  name: <agent-name>
  tools:
    - run_command
    - write_to_file
    - replace_file_content
    - view_file
    - list_dir
    - grep_search
    - find_by_name
  ---
  ```
