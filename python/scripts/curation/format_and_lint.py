#!/usr/bin/env python3
"""Unified Formatting and Linting Curation Tool.

Automates formatting and linting passes for Rust code, Markdown documentation,
and scientific figures, preventing manual agent iteration loops on style compliance.
"""

from __future__ import annotations

import argparse
from pathlib import Path
import shutil
import subprocess
import sys


def run_cmd(args: list[str], cwd: Path, check: bool = False) -> subprocess.CompletedProcess[str]:
    """Execute command in cwd and return result."""
    return subprocess.run(
        args,
        cwd=cwd,
        capture_output=True,
        text=True,
        check=check,
    )


def format_rust(root: Path, check_only: bool = False) -> bool:
    """Format Rust source files with rustfmt/cargo fmt."""
    print("→ Checking/Formatting Rust codebase...")
    cargo_bin = shutil.which("cargo")
    if not cargo_bin:
        print("  WARNING: cargo not found in PATH; skipping Rust formatting.", file=sys.stderr)
        return True

    cmd = [cargo_bin, "fmt"]
    if check_only:
        cmd.extend(["--", "--check"])

    res = run_cmd(cmd, cwd=root)
    if res.returncode != 0:
        print(f"  Rust formatting error:\n{res.stderr or res.stdout}", file=sys.stderr)
        return False
    print("  ✓ Rust formatting clean.")
    return True


def format_markdown(root: Path, check_only: bool = False) -> bool:
    """Format Markdown files with markdownlint-cli2."""
    print("→ Checking/Formatting Markdown documentation...")
    md_bin = shutil.which("markdownlint-cli2")
    if not md_bin:
        print("  WARNING: markdownlint-cli2 not found in PATH; skipping Markdown linting.", file=sys.stderr)
        return True

    cmd = [md_bin]
    if not check_only:
        cmd.append("--fix")
    cmd.append("**/*.md")

    res = run_cmd(cmd, cwd=root)
    if res.returncode != 0:
        # If running --fix, inspect if errors remain
        if not check_only:
            # Run check pass to report remaining issues
            verify_res = run_cmd([md_bin, "**/*.md"], cwd=root)
            if verify_res.returncode != 0:
                print(f"  Markdownlint issues remain after --fix:\n{verify_res.stderr or verify_res.stdout}", file=sys.stderr)
                return False
        else:
            print(f"  Markdownlint check failed:\n{res.stderr or res.stdout}", file=sys.stderr)
            return False

    print("  ✓ Markdown documentation clean.")
    return True


def validate_figures(root: Path, strict: bool = True) -> bool:
    """Validate scientific SVG figures under docs/research/assets/."""
    print("→ Validating scientific figures...")
    validator = root / ".agents" / "skills" / "scientific-figures" / "scripts" / "validate_figure.py"
    if not validator.is_file():
        # Fallback to .github
        validator = root / ".github" / "skills" / "scientific-figures" / "scripts" / "validate_figure.py"
        if not validator.is_file():
            print("  NOTE: Scientific figure validator script not found; skipping.", file=sys.stderr)
            return True

    python_bin = root / ".venv" / "bin" / "python"
    if not python_bin.is_file():
        python_bin = Path(sys.executable)

    assets_dir = root / "docs" / "research" / "assets"
    if not assets_dir.is_dir():
        print("  No research assets directory found.")
        return True

    svg_files = sorted(assets_dir.glob("*.svg"))
    if not svg_files:
        print("  No SVG assets to validate.")
        return True

    all_passed = True
    for svg in svg_files:
        cmd = [str(python_bin), str(validator)]
        if strict:
            cmd.append("--strict")
        cmd.append(str(svg))
        res = run_cmd(cmd, cwd=root)
        if res.returncode != 0:
            print(f"  Figure validation failed for {svg.name}:\n{res.stdout or res.stderr}", file=sys.stderr)
            all_passed = False
        else:
            print(f"  ✓ {svg.name} valid.")

    return all_passed


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Unified formatting and linting tool for ca-experiment workspace.",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Check formatting without applying automatic fixes",
    )
    parser.add_argument(
        "--rust-only",
        action="store_true",
        help="Run only Rust formatting",
    )
    parser.add_argument(
        "--markdown-only",
        action="store_true",
        help="Run only Markdown linting",
    )
    parser.add_argument(
        "--figures",
        action="store_true",
        help="Validate scientific SVG figures in docs/research/assets/",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    root = Path(__file__).resolve().parents[3]  # root of ca-experiment
    success = True

    if not args.markdown_only:
        if not format_rust(root, check_only=args.check):
            success = False

    if not args.rust_only:
        if not format_markdown(root, check_only=args.check):
            success = False

    if args.figures:
        if not validate_figures(root, strict=True):
            success = False

    if not success:
        print("\n❌ Formatting or linting failed.", file=sys.stderr)
        sys.exit(1)
    else:
        print("\n✨ All formatting and linting checks passed.")


if __name__ == "__main__":
    main()
