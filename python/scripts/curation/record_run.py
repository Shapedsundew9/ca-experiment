#!/usr/bin/env python3
"""Automated Git Provenance and Tagging Script.

Eliminates circular commit-tag churn and manual SHA editing by atomically:
1. Staging all experiment artifacts and deliverables.
2. Generating the commit and recording provenance metadata in the run manifest.
3. Creating the official git tag and ensuring a clean working tree.
"""

from __future__ import annotations

import argparse
from pathlib import Path
import re
import subprocess
import sys
from typing import Optional, Tuple


def run_cmd(args: list[str], cwd: Optional[Path] = None, check: bool = True) -> subprocess.CompletedProcess[str]:
    """Execute a shell command via subprocess."""
    res = subprocess.run(
        args,
        cwd=cwd or Path.cwd(),
        capture_output=True,
        text=True,
        check=False,
    )
    if check and res.returncode != 0:
        cmd_str = " ".join(args)
        raise RuntimeError(f"Command failed [{res.returncode}]: {cmd_str}\nStderr: {res.stderr}\nStdout: {res.stdout}")
    return res


def get_git_root() -> Path:
    """Resolve git root directory."""
    res = run_cmd(["git", "rev-parse", "--show-toplevel"])
    return Path(res.stdout.strip())


def update_manifest_provenance(
    manifest_path: Path,
    commit_sha: str,
    tag_name: str,
) -> bool:
    """Update Git Commit SHA, Git Tag, and Git Status Dirty fields in the manifest."""
    content = manifest_path.read_text(encoding="utf-8")
    modified = False

    # 1. Update Git Commit SHA in table: | Git Commit SHA | `...` |
    sha_table_pattern = re.compile(r"(\|\s*Git Commit SHA\s*\|\s*)(`?[^`|\n]*`?)(\s*\|)")
    if sha_table_pattern.search(content):
        content = sha_table_pattern.sub(rf"\1`{commit_sha}`\3", content)
        modified = True
    else:
        # Fallback to list format: - **Git SHA**: ...
        sha_list_pattern = re.compile(r"(\*\*\s*Git SHA\s*\*\*:\s*)(`?[^\n]*`?)")
        if sha_list_pattern.search(content):
            content = sha_list_pattern.sub(rf"\1`{commit_sha}`", content)
            modified = True

    # 2. Update Git Tag in table: | Git Tag | `...` |
    tag_table_pattern = re.compile(r"(\|\s*Git Tag\s*\|\s*)(`?[^`|\n]*`?)(\s*\|)")
    if tag_table_pattern.search(content):
        content = tag_table_pattern.sub(rf"\1`{tag_name}`\3", content)
        modified = True
    else:
        tag_list_pattern = re.compile(r"(\*\*\s*Git Tag\s*\*\*:\s*)(`?[^\n]*`?)")
        if tag_list_pattern.search(content):
            content = tag_list_pattern.sub(rf"\1`{tag_name}`", content)
            modified = True

    # 3. Update Git Status Dirty in table: | Git Status Dirty? | ... |
    dirty_table_pattern = re.compile(r"(\|\s*Git Status Dirty\??\s*\|\s*)([^|\n]*?)(\s*\|)")
    clean_msg = "No (Clean at tagging gate)"
    if dirty_table_pattern.search(content):
        content = dirty_table_pattern.sub(rf"\1{clean_msg} |", content)
        modified = True
    else:
        dirty_list_pattern = re.compile(r"(\*\*\s*Git Status Dirty\??\s*\*\*:\s*)([^\n]*)")
        if dirty_list_pattern.search(content):
            content = dirty_list_pattern.sub(rf"\1{clean_msg}", content)
            modified = True

    if modified:
        manifest_path.write_text(content, encoding="utf-8")
    return modified


def check_tag_exists(tag_name: str) -> bool:
    """Check if git tag already exists."""
    res = run_cmd(["git", "tag", "-l", tag_name])
    return tag_name in res.stdout.split()


def record_run(
    manifest_path: Path,
    tag_name: str,
    message: str,
    two_phase: bool = False,
    force_tag: bool = False,
    dry_run: bool = False,
) -> Tuple[str, str]:
    """Execute the atomic git staging, commit, manifest update, and tagging flow."""
    manifest_path = manifest_path.resolve()
    if not manifest_path.is_file():
        raise FileNotFoundError(f"Run manifest file not found: {manifest_path}")

    git_root = get_git_root()

    if check_tag_exists(tag_name) and not force_tag:
        raise ValueError(f"Git tag '{tag_name}' already exists. Use --force-tag to overwrite.")

    if dry_run:
        print(f"[DRY RUN] Would stage changes in git repository: {git_root}")
        print(f"[DRY RUN] Would commit with message: {message}")
        print(f"[DRY RUN] Would update manifest: {manifest_path}")
        print(f"[DRY RUN] Would create git tag: {tag_name}")
        return "dry-run-sha", tag_name

    # Step 1: Stage all current changes
    run_cmd(["git", "add", "-A"], cwd=git_root)

    # Check if there are staged changes
    status_res = run_cmd(["git", "status", "--porcelain"], cwd=git_root)
    if not status_res.stdout.strip():
        # Check if HEAD already has the commit
        curr_sha = run_cmd(["git", "rev-parse", "HEAD"], cwd=git_root).stdout.strip()
        print(f"Working tree clean. Using current HEAD SHA: {curr_sha}")
        execution_sha = curr_sha
    else:
        # Step 2: Make the primary commit
        run_cmd(["git", "commit", "-m", message], cwd=git_root)
        execution_sha = run_cmd(["git", "rev-parse", "HEAD"], cwd=git_root).stdout.strip()

    # Step 3: Update the manifest with execution_sha and tag_name
    update_manifest_provenance(manifest_path, execution_sha, tag_name)

    # Step 4: Finalize commit
    run_cmd(["git", "add", str(manifest_path)], cwd=git_root)
    manifest_dirty = run_cmd(["git", "status", "--porcelain"], cwd=git_root).stdout.strip()

    if manifest_dirty:
        if two_phase:
            manifest_commit_msg = f"docs(runs): record git provenance for {tag_name}"
            run_cmd(["git", "commit", "-m", manifest_commit_msg], cwd=git_root)
        else:
            run_cmd(["git", "commit", "--amend", "--no-edit"], cwd=git_root)

    final_sha = run_cmd(["git", "rev-parse", "HEAD"], cwd=git_root).stdout.strip()

    # Step 5: Create git tag
    tag_cmd = ["git", "tag", "-a", tag_name, "-m", f"Experiment Run: {tag_name} (Commit {final_sha[:8]})"]
    if force_tag:
        tag_cmd.append("-f")
    run_cmd(tag_cmd, cwd=git_root)

    # Step 6: Verify clean working tree
    final_status = run_cmd(["git", "status", "--porcelain"], cwd=git_root).stdout.strip()
    if final_status:
        raise RuntimeError(f"Git working tree is dirty after tagging:\n{final_status}")

    print(f"✓ Provenance successfully recorded.")
    print(f"  Execution SHA: {execution_sha}")
    print(f"  Final Tagged SHA: {final_sha}")
    print(f"  Git Tag: {tag_name}")
    print(f"  Manifest: {manifest_path.relative_to(git_root)}")

    return final_sha, tag_name


def parse_args() -> argparse.Namespace:
    """Parse command-line arguments."""
    parser = argparse.ArgumentParser(
        description="Record git provenance and tag experiment runs cleanly.",
    )
    parser.add_argument(
        "--manifest",
        "-m",
        type=Path,
        required=True,
        help="Path to the experiment run manifest (e.g., docs/research/runs/RUN-EXP-*.md)",
    )
    parser.add_argument(
        "--tag",
        "-t",
        type=str,
        required=True,
        help="Git tag name to apply (e.g., exp/EXP-2026-012a-01)",
    )
    parser.add_argument(
        "--message",
        "-msg",
        type=str,
        required=True,
        help="Commit message for the experiment deliverables",
    )
    parser.add_argument(
        "--two-phase",
        action="store_true",
        help="Create a separate provenance commit for the manifest instead of amending HEAD",
    )
    parser.add_argument(
        "--force-tag",
        action="store_true",
        help="Overwrite existing git tag if present",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Simulate actions without staging, committing, or tagging",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    try:
        record_run(
            manifest_path=args.manifest,
            tag_name=args.tag,
            message=args.message,
            two_phase=args.two_phase,
            force_tag=args.force_tag,
            dry_run=args.dry_run,
        )
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
