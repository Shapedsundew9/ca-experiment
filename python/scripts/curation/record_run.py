#!/usr/bin/env python3
"""Automated Git provenance and tagging for explicitly scoped artifacts.

The workflow creates an execution commit followed by a provenance commit. The
manifest records the execution commit, and the annotated tag points to the
provenance commit whose parent is that execution commit. A commit cannot contain
its own SHA, so amending the execution commit after writing its SHA is invalid.
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
from typing import Optional, Sequence, Tuple


def run_cmd(
    args: list[str],
    cwd: Optional[Path] = None,
    check: bool = True,
    env: Optional[dict[str, str]] = None,
    input_text: Optional[str] = None,
) -> subprocess.CompletedProcess[str]:
    """Execute a shell command via subprocess."""
    res = subprocess.run(
        args,
        cwd=cwd or Path.cwd(),
        capture_output=True,
        text=True,
        check=False,
        env=env,
        input=input_text,
    )
    if check and res.returncode != 0:
        cmd_str = " ".join(args)
        raise RuntimeError(
            f"Command failed [{res.returncode}]: {cmd_str}\nStderr: {res.stderr}\nStdout: {res.stdout}")
    return res


def get_git_root(start: Optional[Path] = None) -> Path:
    """Resolve git root directory."""
    res = run_cmd(["git", "rev-parse", "--show-toplevel"], cwd=start)
    return Path(res.stdout.strip())


def repository_relative_path(path: Path, git_root: Path) -> str:
    """Return a literal repository-relative path without following symlinks."""
    candidate = path if path.is_absolute() else git_root / path
    try:
        absolute = Path(os.path.abspath(candidate))
        relative = absolute.relative_to(git_root.resolve())
    except ValueError as exc:
        raise ValueError(
            f"Path is outside the git repository: {path}") from exc

    if relative == Path("."):
        raise ValueError(
            "Repository root is not an allowed provenance path; list artifacts explicitly.")
    return relative.as_posix()


def changed_paths(git_root: Path) -> set[str]:
    """Return tracked, staged, and untracked paths changed in the repository."""
    commands = (
        ["git", "diff", "--name-only", "--no-renames", "-z"],
        ["git", "diff", "--cached", "--name-only", "--no-renames", "-z"],
        ["git", "ls-files", "--others", "--exclude-standard", "-z"],
    )
    paths: set[str] = set()
    for command in commands:
        result = run_cmd(command, cwd=git_root)
        paths.update(path for path in result.stdout.split("\0") if path)
    return paths


def path_is_scoped(path: str, scopes: Sequence[str]) -> bool:
    """Return whether path is equal to or nested beneath an explicit scope."""
    candidate = Path(path)
    return any(candidate == Path(scope) or Path(scope) in candidate.parents for scope in scopes)


def ensure_changes_are_scoped(git_root: Path, scopes: Sequence[str]) -> None:
    """Reject dirty paths outside the explicitly authorized artifact scopes."""
    outside_scope = sorted(path for path in changed_paths(
        git_root) if not path_is_scoped(path, scopes))
    if outside_scope:
        formatted = "\n".join(f"  - {path}" for path in outside_scope)
        raise ValueError(
            "Refusing provenance operation because changes exist outside the authorized paths:\n"
            f"{formatted}\nCommit, stash, or explicitly include those artifacts before retrying."
        )


TABLE_PATTERNS = {
    "sha": re.compile(r"^(\|\s*Git Commit SHA\s*\|\s*)([^|\n]*?)(\s*\|)\s*$", re.MULTILINE),
    "tag": re.compile(r"^(\|\s*Git Tag\s*\|\s*)([^|\n]*?)(\s*\|)\s*$", re.MULTILINE),
    "dirty": re.compile(r"^(\|\s*Git Status Dirty\??\s*\|\s*)([^|\n]*?)(\s*\|)\s*$", re.MULTILINE),
}
LIST_PATTERNS = {
    "sha": re.compile(r"^(-\s*\*\*Git SHA\*\*:\s*)([^\n]*?)\s*$", re.MULTILINE),
    "tag": re.compile(r"^(-\s*\*\*Git Tag\*\*:\s*)([^\n]*?)\s*$", re.MULTILINE),
    "dirty": re.compile(r"^(-\s*\*\*Git Status Dirty\??\*\*:\s*)([^\n]*?)\s*$", re.MULTILINE),
}


def render_manifest_provenance(content: str, commit_sha: str, tag_name: str) -> str:
    """Render one complete, unambiguous provenance schema."""
    pattern_sets = (TABLE_PATTERNS, LIST_PATTERNS)
    counts = [{name: len(pattern.findall(content)) for name,
               pattern in patterns.items()} for patterns in pattern_sets]
    complete = [index for index, count in enumerate(
        counts) if all(value == 1 for value in count.values())]
    total_matches = sum(sum(count.values()) for count in counts)
    if len(complete) != 1 or total_matches != 3:
        raise ValueError(
            "Manifest must contain exactly one complete provenance schema: Git Commit SHA/Git SHA, "
            "Git Tag, and Git Status Dirty. Duplicate, mixed, or partial schemas are not supported."
        )

    patterns = pattern_sets[complete[0]]
    clean_msg = "No (Clean at tagging gate)"
    if patterns is TABLE_PATTERNS:
        rendered = patterns["sha"].sub(
            lambda match: f"{match.group(1)}`{commit_sha}`{match.group(3)}",
            content,
        )
        rendered = patterns["tag"].sub(
            lambda match: f"{match.group(1)}`{tag_name}`{match.group(3)}",
            rendered,
        )
        return patterns["dirty"].sub(lambda match: f"{match.group(1)}{clean_msg}{match.group(3)}", rendered)
    rendered = patterns["sha"].sub(
        lambda match: f"{match.group(1)}`{commit_sha}`", content)
    rendered = patterns["tag"].sub(
        lambda match: f"{match.group(1)}`{tag_name}`", rendered)
    return patterns["dirty"].sub(lambda match: f"{match.group(1)}{clean_msg}", rendered)


def update_manifest_provenance(manifest_path: Path, commit_sha: str, tag_name: str) -> bool:
    """Update a validated provenance schema in the manifest."""
    content = manifest_path.read_text(encoding="utf-8")
    rendered = render_manifest_provenance(content, commit_sha, tag_name)
    manifest_path.write_text(rendered, encoding="utf-8")
    return True


def check_tag_exists(tag_name: str, git_root: Path) -> bool:
    """Check if git tag already exists."""
    res = run_cmd(["git", "tag", "-l", tag_name], cwd=git_root)
    return tag_name in res.stdout.split()


def validate_tag_name(tag_name: str, git_root: Path) -> str:
    """Validate and return the fully qualified tag ref."""
    tag_ref = f"refs/tags/{tag_name}"
    result = run_cmd(["git", "check-ref-format", tag_ref],
                     cwd=git_root, check=False)
    if result.returncode != 0:
        raise ValueError(f"Invalid git tag name: {tag_name}")
    return tag_ref


def create_commit(tree_sha: str, parent_sha: str, message: str, git_root: Path) -> str:
    """Create an unreferenced commit object without moving HEAD."""
    return run_cmd(
        ["git", "commit-tree", tree_sha, "-p", parent_sha],
        cwd=git_root,
        input_text=f"{message}\n",
    ).stdout.strip()


def create_annotated_tag_object(tag_name: str, commit_sha: str, git_root: Path) -> str:
    """Create an annotated tag object without publishing its ref."""
    tagger = run_cmd(["git", "var", "GIT_COMMITTER_IDENT"],
                     cwd=git_root).stdout.strip()
    payload = (
        f"object {commit_sha}\n"
        "type commit\n"
        f"tag {tag_name}\n"
        f"tagger {tagger}\n\n"
        f"Experiment Run: {tag_name} (Commit {commit_sha[:8]})\n"
    )
    return run_cmd(["git", "mktag"], cwd=git_root, input_text=payload).stdout.strip()


def publish_refs(
    branch_ref: str,
    original_head: str,
    final_sha: str,
    tag_ref: str,
    tag_object_sha: str,
    previous_tag_sha: Optional[str],
    git_root: Path,
) -> None:
    """Publish branch and tag refs in one atomic ref transaction."""
    tag_command = (
        f"update {tag_ref} {tag_object_sha} {previous_tag_sha}"
        if previous_tag_sha
        else f"create {tag_ref} {tag_object_sha}"
    )
    transaction = (
        "start\n"
        f"update {branch_ref} {final_sha} {original_head}\n"
        f"{tag_command}\n"
        "prepare\n"
        "commit\n"
    )
    run_cmd(["git", "update-ref", "--stdin"],
            cwd=git_root, input_text=transaction)


def record_run(
    manifest_path: Path,
    tag_name: str,
    message: str,
    artifact_paths: Sequence[Path],
    force_tag: bool = False,
    dry_run: bool = False,
) -> Tuple[str, str]:
    """Commit scoped artifacts, record their commit, then tag provenance."""
    manifest_path = manifest_path.resolve()
    if not manifest_path.is_file():
        raise FileNotFoundError(
            f"Run manifest file not found: {manifest_path}")
    if not artifact_paths:
        raise ValueError("At least one explicit artifact path is required.")

    git_root = get_git_root(manifest_path.parent)
    original_head = run_cmd(["git", "rev-parse", "HEAD"],
                            cwd=git_root).stdout.strip()
    branch_ref_result = run_cmd(
        ["git", "symbolic-ref", "--quiet", "HEAD"], cwd=git_root, check=False)
    if branch_ref_result.returncode != 0:
        raise ValueError(
            "Provenance recording requires an attached branch; detached HEAD is not supported.")
    branch_ref = branch_ref_result.stdout.strip()
    manifest_relative = repository_relative_path(manifest_path, git_root)
    scopes = sorted(
        {
            manifest_relative,
            *(repository_relative_path(path, git_root)
              for path in artifact_paths),
        }
    )

    tag_ref = validate_tag_name(tag_name, git_root)
    tag_exists = check_tag_exists(tag_name, git_root)
    if tag_exists and not force_tag:
        raise ValueError(
            f"Git tag '{tag_name}' already exists. Use --force-tag to overwrite.")
    previous_tag_sha = (
        run_cmd(["git", "rev-parse", tag_ref],
                cwd=git_root).stdout.strip() if tag_exists else None
    )

    ensure_changes_are_scoped(git_root, scopes)
    original_manifest = manifest_path.read_text(encoding="utf-8")
    render_manifest_provenance(original_manifest, original_head, tag_name)

    if dry_run:
        print(
            f"[DRY RUN] Would stage only these repository paths: {', '.join(scopes)}")
        print(f"[DRY RUN] Would commit with message: {message}")
        print(
            f"[DRY RUN] Would record the execution commit in: {manifest_relative}")
        print(
            f"[DRY RUN] Would create a provenance commit and tag: {tag_name}")
        return "dry-run-sha", tag_name

    refs_published = False
    try:
        with tempfile.TemporaryDirectory(prefix="record-run-index-") as tmpdir:
            index_env = os.environ.copy()
            index_env["GIT_INDEX_FILE"] = str(Path(tmpdir) / "index")
            run_cmd(["git", "read-tree", original_head],
                    cwd=git_root, env=index_env)
            run_cmd(["git", "add", "-A", "--", *scopes],
                    cwd=git_root, env=index_env)

            execution_tree = run_cmd(
                ["git", "write-tree"], cwd=git_root, env=index_env).stdout.strip()
            head_tree = run_cmd(
                ["git", "rev-parse", f"{original_head}^{{tree}}"], cwd=git_root).stdout.strip()
            execution_sha = (
                original_head
                if execution_tree == head_tree
                else create_commit(execution_tree, original_head, message, git_root)
            )

            manifest_path.write_text(
                render_manifest_provenance(
                    original_manifest, execution_sha, tag_name),
                encoding="utf-8",
            )
            run_cmd(["git", "add", "--", manifest_relative],
                    cwd=git_root, env=index_env)
            provenance_tree = run_cmd(
                ["git", "write-tree"], cwd=git_root, env=index_env).stdout.strip()
            final_sha = (
                execution_sha
                if provenance_tree == execution_tree
                else create_commit(
                    provenance_tree,
                    execution_sha,
                    f"docs(runs): record git provenance for {tag_name}",
                    git_root,
                )
            )
            tag_object_sha = create_annotated_tag_object(
                tag_name, final_sha, git_root)
            publish_refs(
                branch_ref,
                original_head,
                final_sha,
                tag_ref,
                tag_object_sha,
                previous_tag_sha,
                git_root,
            )
            refs_published = True
    except Exception:
        if not refs_published:
            manifest_path.write_text(original_manifest, encoding="utf-8")
        raise

    # Synchronize the real index with the atomically published tree.
    run_cmd(["git", "read-tree", final_sha], cwd=git_root)
    final_status = run_cmd(
        ["git", "status", "--porcelain"], cwd=git_root).stdout.strip()
    if final_status:
        raise RuntimeError(
            f"Git working tree is dirty after tagging:\n{final_status}")

    print("✓ Provenance successfully recorded.")
    print(f"  Execution SHA: {execution_sha}")
    print(f"  Provenance/Tagged SHA: {final_sha}")
    print(f"  Git Tag: {tag_name}")
    print(f"  Manifest: {manifest_relative}")

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
        "--path",
        dest="artifact_paths",
        action="append",
        type=Path,
        required=True,
        help="Artifact file or directory to include; repeat for each authorized path",
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
            artifact_paths=args.artifact_paths,
            force_tag=args.force_tag,
            dry_run=args.dry_run,
        )
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
