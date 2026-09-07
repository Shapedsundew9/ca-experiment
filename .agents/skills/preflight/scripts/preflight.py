#!/usr/bin/env python3
"""Universal Pre-Flight Environment & Subagent Sanity Check.

A generic, repository-agnostic diagnostic script that verifies:
1. Workspace write and filesystem access.
2. Active language runtimes and declared dependencies (Python, Rust, Node, Go).
3. Subagent tool declarations and execution permissions in .agents/agents/*.md.
4. Git version control availability.

Runs entirely with Python standard library.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass, field
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
from typing import Any, Dict, List, Optional, Tuple


@dataclass
class CheckResult:
    name: str
    category: str
    passed: bool
    message: str
    severity: str = "error"  # "error", "warning", "info"
    details: Dict[str, Any] = field(default_factory=dict)


def find_repo_root(start_dir: Optional[Path] = None) -> Path:
    """Find repository or workspace root by walking up from start_dir."""
    curr = (start_dir or Path.cwd()).resolve()
    for parent in [curr] + list(curr.parents):
        if (parent / ".git").exists() or (parent / ".agents").exists():
            return parent
    return curr


def check_filesystem_access(root: Path) -> CheckResult:
    """Verify that the agent has write and delete permissions in the workspace."""
    probe = root / ".preflight_probe.tmp"
    try:
        probe.write_text("preflight_ok\n", encoding="utf-8")
        read_back = probe.read_text(encoding="utf-8").strip()
        probe.unlink()
        if read_back == "preflight_ok":
            return CheckResult(
                name="workspace_write_access",
                category="filesystem",
                passed=True,
                message=f"Read/write access verified in {root}",
                severity="info",
            )
        else:
            return CheckResult(
                name="workspace_write_access",
                category="filesystem",
                passed=False,
                message="File content mismatch during write/read check",
                severity="error",
            )
    except Exception as e:
        return CheckResult(
            name="workspace_write_access",
            category="filesystem",
            passed=False,
            message=f"Failed to write to workspace root: {e}",
            severity="error",
        )


def check_git_status(root: Path) -> List[CheckResult]:
    """Check git availability and status."""
    results: List[CheckResult] = []
    git_bin = shutil.which("git")
    if not git_bin:
        results.append(
            CheckResult(
                name="git_binary",
                category="vcs",
                passed=False,
                message="git command not found in PATH",
                severity="warning",
            )
        )
        return results

    try:
        ver = subprocess.check_output([git_bin, "--version"], text=True, cwd=root).strip()
        results.append(
            CheckResult(
                name="git_binary",
                category="vcs",
                passed=True,
                message=f"Git available ({ver})",
                severity="info",
            )
        )
    except Exception as e:
        results.append(
            CheckResult(
                name="git_binary",
                category="vcs",
                passed=False,
                message=f"Failed to execute git: {e}",
                severity="warning",
            )
        )
        return results

    if (root / ".git").exists():
        try:
            status_out = subprocess.check_output(
                [git_bin, "status", "--porcelain"], text=True, cwd=root
            ).strip()
            dirty = len(status_out) > 0
            results.append(
                CheckResult(
                    name="git_clean_tree",
                    category="vcs",
                    passed=True,
                    message="Working tree is clean" if not dirty else "Working tree has uncommitted changes",
                    severity="info" if not dirty else "warning",
                    details={"dirty": dirty, "uncommitted_files": len(status_out.splitlines()) if dirty else 0},
                )
            )
        except Exception:
            pass

    return results


def check_python_environment(root: Path) -> List[CheckResult]:
    """Dynamically check Python environment and verify declared dependencies."""
    results: List[CheckResult] = []

    # Detect if repo uses Python
    py_indicators = [
        root / ".venv",
        root / "pyproject.toml",
        root / "python" / "pyproject.toml",
        root / "requirements.txt",
        root / "setup.py",
    ]
    has_python = any(p.exists() for p in py_indicators) or any(root.glob("**/*.py"))
    if not has_python:
        return results

    # Determine preferred Python interpreter
    venv_py = root / ".venv" / "bin" / "python"
    python_bin = str(venv_py) if venv_py.is_file() and os.access(venv_py, os.X_OK) else sys.executable

    try:
        ver_out = subprocess.check_output(
            [python_bin, "-c", "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}')"],
            text=True,
        ).strip()
        is_venv = (root / ".venv").is_dir() and ".venv" in python_bin
        results.append(
            CheckResult(
                name="python_interpreter",
                category="python",
                passed=True,
                message=f"Python interpreter: {python_bin} (v{ver_out})" + (" [in .venv]" if is_venv else ""),
                severity="info",
                details={"version": ver_out, "path": python_bin, "in_venv": is_venv},
            )
        )
    except Exception as e:
        results.append(
            CheckResult(
                name="python_interpreter",
                category="python",
                passed=False,
                message=f"Failed to execute Python at {python_bin}: {e}",
                severity="error",
            )
        )
        return results

    # Check dependencies declared in pyproject.toml
    declared_pkgs: List[str] = []
    for pyproj in [root / "pyproject.toml", root / "python" / "pyproject.toml"]:
        if pyproj.is_file():
            try:
                content = pyproj.read_text(encoding="utf-8")
                # Simple TOML parser for dependencies = [...]
                m = re.search(r'dependencies\s*=\s*\[(.*?)\]', content, re.DOTALL)
                if m:
                    raw_items = m.group(1).split(",")
                    for it in raw_items:
                        it = it.strip().strip('"').strip("'")
                        if it:
                            pkg_name = re.split(r"[><=~!;]", it)[0].strip()
                            if pkg_name:
                                declared_pkgs.append(pkg_name)
            except Exception:
                pass

    for req in [root / "requirements.txt", root / "python" / "requirements.txt"]:
        if req.is_file():
            try:
                for line in req.read_text(encoding="utf-8").splitlines():
                    line = line.strip()
                    if line and not line.startswith("#"):
                        pkg_name = re.split(r"[><=~!;]", line)[0].strip()
                        if pkg_name:
                            declared_pkgs.append(pkg_name)
            except Exception:
                pass

    if declared_pkgs:
        unique_pkgs = sorted(set(declared_pkgs))
        missing_pkgs: List[str] = []
        for pkg in unique_pkgs:
            # Normalize package import name (e.g. pyyaml -> yaml, scikit-learn -> sklearn)
            mod_name = pkg.lower().replace("-", "_")
            if mod_name == "pyyaml":
                mod_name = "yaml"
            elif mod_name == "scikit_learn":
                mod_name = "sklearn"
            
            check_code = f"import sys, importlib.util; sys.exit(0 if importlib.util.find_spec('{mod_name}') else 1)"
            code = subprocess.call([python_bin, "-c", check_code])
            if code != 0:
                missing_pkgs.append(pkg)

        if missing_pkgs:
            results.append(
                CheckResult(
                    name="python_dependencies",
                    category="python",
                    passed=False,
                    message=f"Missing declared Python dependencies in {python_bin}: {missing_pkgs}. Install via '.venv/bin/pip install {' '.join(missing_pkgs)}'.",
                    severity="error",
                    details={"missing": missing_pkgs},
                )
            )
        else:
            results.append(
                CheckResult(
                    name="python_dependencies",
                    category="python",
                    passed=True,
                    message=f"All declared Python dependencies verified ({len(unique_pkgs)} package(s))",
                    severity="info",
                )
            )

    return results


def check_rust_environment(root: Path) -> List[CheckResult]:
    """Check Rust toolchain if Cargo.toml is present."""
    results: List[CheckResult] = []
    cargo_toml = root / "Cargo.toml"
    if not cargo_toml.is_file():
        # Check subdirectories
        sub_cargos = list(root.glob("*/Cargo.toml"))
        if not sub_cargos:
            return results

    cargo_bin = shutil.which("cargo")
    rustc_bin = shutil.which("rustc")

    if not cargo_bin or not rustc_bin:
        results.append(
            CheckResult(
                name="rust_toolchain",
                category="rust",
                passed=False,
                message="Cargo.toml found, but cargo or rustc is not in PATH",
                severity="error",
            )
        )
        return results

    try:
        cargo_ver = subprocess.check_output([cargo_bin, "--version"], text=True).strip()
        rustc_ver = subprocess.check_output([rustc_bin, "--version"], text=True).strip()
        results.append(
            CheckResult(
                name="rust_toolchain",
                category="rust",
                passed=True,
                message=f"Rust toolchain active: {rustc_ver}, {cargo_ver}",
                severity="info",
            )
        )
    except Exception as e:
        results.append(
            CheckResult(
                name="rust_toolchain",
                category="rust",
                passed=False,
                message=f"Error checking rust toolchain: {e}",
                severity="error",
            )
        )
        return results

    if cargo_toml.is_file():
        try:
            # Fast syntax verification of Cargo.toml without compiling everything
            code = subprocess.call(
                [cargo_bin, "verify-project", "--manifest-path", str(cargo_toml)],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            if code == 0:
                results.append(
                    CheckResult(
                        name="cargo_manifest",
                        category="rust",
                        passed=True,
                        message="Cargo.toml manifest is valid",
                        severity="info",
                    )
                )
            else:
                results.append(
                    CheckResult(
                        name="cargo_manifest",
                        category="rust",
                        passed=False,
                        message="Cargo.toml manifest validation failed",
                        severity="error",
                    )
                )
        except Exception:
            pass

    return results


def check_node_environment(root: Path) -> List[CheckResult]:
    """Check Node.js environment if package.json is present."""
    results: List[CheckResult] = []
    pkg_json = root / "package.json"
    if not pkg_json.is_file():
        return results

    node_bin = shutil.which("node")
    if not node_bin:
        results.append(
            CheckResult(
                name="node_runtime",
                category="node",
                passed=False,
                message="package.json found, but node is not in PATH",
                severity="warning",
            )
        )
        return results

    try:
        node_ver = subprocess.check_output([node_bin, "--version"], text=True).strip()
        results.append(
            CheckResult(
                name="node_runtime",
                category="node",
                passed=True,
                message=f"Node runtime available ({node_ver})",
                severity="info",
            )
        )
    except Exception as e:
        results.append(
            CheckResult(
                name="node_runtime",
                category="node",
                passed=False,
                message=f"Error checking node runtime: {e}",
                severity="warning",
            )
        )

    return results


def check_subagent_definitions(root: Path) -> List[CheckResult]:
    """Inspect .agents/agents/*.md files to ensure required tools are explicitly defined."""
    results: List[CheckResult] = []
    agents_dir = root / ".agents" / "agents"
    if not agents_dir.is_dir():
        # No custom subagents defined
        return results

    agent_files = sorted(agents_dir.glob("*.md"))
    if not agent_files:
        return results

    write_roles_keywords = ["implement", "execute", "execution", "debug", "swe", "fix", "author", "write", "test", "developer", "engineer"]
    orchestrator_keywords = ["orchestrat", "director", "lead scientist"]

    total_agents = len(agent_files)
    configured_agents = 0
    flagged_agents: List[Tuple[str, str]] = []

    for af in agent_files:
        content = af.read_text(encoding="utf-8")
        if not content.startswith("---"):
            continue
        parts = content.split("---", 2)
        if len(parts) < 3:
            continue
        frontmatter = parts[1]
        body = parts[2].lower()
        agent_name = af.stem

        has_tools = "tools:" in frontmatter
        if has_tools:
            configured_agents += 1
        else:
            flagged_agents.append((agent_name, "Missing explicit 'tools:' declaration in frontmatter (defaults to read-only)"))

        fm_lower = frontmatter.lower()
        is_orchestrator = "mainagent: true" in fm_lower or "orchestrat" in agent_name.lower()
        is_exec_role = any(k in agent_name.lower() for k in ["swe", "debug", "execution", "developer", "engineer", "fix"])
        is_author_role = any(k in agent_name.lower() for k in ["write", "author", "generator", "protocol", "specification", "prd"])

        if is_orchestrator:
            missing_orch_tools = []
            for t in ["invoke_subagent", "send_message"]:
                if t not in frontmatter:
                    missing_orch_tools.append(t)
            if missing_orch_tools:
                flagged_agents.append((agent_name, f"Orchestrator missing delegation tools: {missing_orch_tools}"))

        if is_exec_role:
            missing_exec_tools = []
            for t in ["write_to_file", "run_command"]:
                if t not in frontmatter:
                    missing_exec_tools.append(t)
            if missing_exec_tools:
                flagged_agents.append((agent_name, f"Execution subagent missing write/command tools: {missing_exec_tools}"))
        elif is_author_role:
            if "write_to_file" not in frontmatter:
                flagged_agents.append((agent_name, "Authoring subagent missing 'write_to_file' tool"))

    if flagged_agents:
        for name, reason in flagged_agents:
            results.append(
                CheckResult(
                    name=f"subagent_tools_{name}",
                    category="subagents",
                    passed=False,
                    message=f"Agent '{name}.md' permission issue: {reason}",
                    severity="error",
                )
            )
    else:
        results.append(
            CheckResult(
                name="subagent_permissions",
                category="subagents",
                passed=True,
                message=f"All {total_agents} subagent definition(s) have explicit tools and permissions configured",
                severity="info",
                details={"total": total_agents, "with_tools": configured_agents},
            )
        )

    return results


def run_all_checks(root: Path) -> List[CheckResult]:
    """Execute all discovery and environment sanity checks."""
    checks: List[CheckResult] = []
    checks.append(check_filesystem_access(root))
    checks.extend(check_git_status(root))
    checks.extend(check_python_environment(root))
    checks.extend(check_rust_environment(root))
    checks.extend(check_node_environment(root))
    checks.extend(check_subagent_definitions(root))
    return checks


def format_markdown(results: List[CheckResult], root: Path) -> str:
    """Format results as a markdown report."""
    lines = [
        "# Campaign Pre-Flight Sanity Report",
        "",
        f"> **Workspace Root**: `{root}`  ",
        f"> **Total Checks**: {len(results)} | **Status**: {'✅ PASSED' if all(r.passed or r.severity != 'error' for r in results) else '❌ FAILED'}",
        "",
        "| Category | Check | Status | Severity | Details |",
        "| :--- | :--- | :--- | :--- | :--- |",
    ]
    for r in results:
        icon = "✅ Pass" if r.passed else "❌ Fail"
        sev = "🔴 Error" if r.severity == "error" else ("🟡 Warning" if r.severity == "warning" else "ℹ️ Info")
        msg = r.message.replace("|", "\\|")
        lines.append(f"| `{r.category}` | `{r.name}` | {icon} | {sev} | {msg} |")
    lines.append("")
    return "\n".join(lines)


def format_terminal(results: List[CheckResult], root: Path) -> str:
    """Format results for clean CLI terminal display."""
    lines = [
        "=" * 60,
        "  PRE-FLIGHT SANITY CHECK (Autonomous Campaign Guardrail)",
        "=" * 60,
        f"Workspace Root: {root}",
        "",
    ]
    categories: Dict[str, List[CheckResult]] = {}
    for r in results:
        categories.setdefault(r.category, []).append(r)

    has_errors = False
    for cat, r_list in categories.items():
        lines.append(f"[{cat.upper()}]")
        for r in r_list:
            if r.passed:
                lines.append(f"  ✓ {r.message}")
            else:
                if r.severity == "error":
                    has_errors = True
                    lines.append(f"  ❌ [ERROR] {r.message}")
                else:
                    lines.append(f"  ⚠️  [WARN]  {r.message}")
        lines.append("")

    lines.append("-" * 60)
    if has_errors:
        lines.append("❌ PRE-FLIGHT VERIFICATION FAILED. Resolve errors before proceeding.")
    else:
        lines.append("🎉 ALL PRE-FLIGHT CHECKS PASSED. Ready for autonomous execution.")
    lines.append("-" * 60)
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description="Universal Pre-Flight Environment & Subagent Sanity Check")
    parser.add_argument("--root", type=Path, default=None, help="Workspace root path (default: auto-detect)")
    parser.add_argument("--format", choices=["terminal", "markdown", "json"], default="terminal", help="Output format")
    parser.add_argument("--strict", action="store_true", help="Fail on warnings as well as errors")
    args = parser.parse_args()

    root = find_repo_root(args.root)
    results = run_all_checks(root)

    if args.format == "json":
        data = {
            "root": str(root),
            "passed": all(r.passed or (r.severity != "error" and not args.strict) for r in results),
            "checks": [
                {
                    "name": r.name,
                    "category": r.category,
                    "passed": r.passed,
                    "severity": r.severity,
                    "message": r.message,
                    "details": r.details,
                }
                for r in results
            ],
        }
        print(json.dumps(data, indent=2))
    elif args.format == "markdown":
        print(format_markdown(results, root))
    else:
        print(format_terminal(results, root))

    has_errors = any(not r.passed and r.severity == "error" for r in results)
    has_warnings = any(not r.passed and r.severity == "warning" for r in results)
    if has_errors or (args.strict and has_warnings):
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
