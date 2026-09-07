#!/usr/bin/env python3
"""Figure validation utility for scientific documentation figures.

Validates that generated SVG or PNG figures exist, are non-empty, adhere to size bounds,
conform to the repository dark theme palette (#161922 canvas), and checks for
layout overflow and mathematical typography standards (e.g. forbidding raw programming
underscores like 'v_0' or raw scientific notation like '1e-12').

Usage:
    python3 validate_figure.py path/to/figure.svg
    python3 validate_figure.py docs/research/assets/*.svg
"""

from __future__ import annotations

import argparse
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

DARK_CANVAS_HEX = "#161922"
MAX_SVG_SIZE_BYTES = 1024 * 1024  # 1 MB


def check_svg_typography_and_layout(root: ET.Element) -> list[str]:
    """Inspect SVG text elements for layout overflow and raw programming syntax."""
    warnings: list[str] = []

    # Try to extract canvas width
    width_str = root.attrib.get("width", "")
    width_val = None
    width_match = re.match(r"^([0-9.]+)", width_str)
    if width_match:
        width_val = float(width_match.group(1))

    # Collect all text content
    text_elements = root.findall(".//{http://www.w3.org/2000/svg}text")
    if not text_elements:
        # Without namespace
        text_elements = root.findall(".//text")

    for elem in text_elements:
        text_content = "".join(elem.itertext()).strip()
        if not text_content:
            continue

        # Ignore generator script paths in footers (e.g., python/scripts/figures/fig_hyp_001.py)
        if "fig_" in text_content and ".py" in text_content:
            continue
        if "Figure FIG-" in text_content:
            continue

        # Check for raw programming underscores where math subscripts belong
        # e.g., v_0, W_ij, d_in, R_i, N_ref
        raw_underscores = re.findall(r"\b[A-Za-z]+_[A-Za-z0-9]+\b", text_content)
        # Filter out common false positives like file extensions or standard names
        raw_underscores = [u for u in raw_underscores if not u.endswith(".py") and not u.startswith("fig_")]
        if raw_underscores:
            warnings.append(
                f"Raw programming underscore found in text '{text_content[:40]}...': {raw_underscores}. "
                "Use mathematical subscripts (e.g. v₀ or LaTeX $v_0$)."
            )

        # Check for raw scientific notation (e.g., 1e-12, 5e-33)
        raw_exp = re.findall(r"\b\d+e-\d+\b", text_content)
        if raw_exp:
            warnings.append(
                f"Raw scientific notation found in text '{text_content[:40]}...': {raw_exp}. "
                "Use mathematical notation (e.g. 10⁻¹² or LaTeX $10^{-12}$)."
            )

        # Check for potential horizontal boundary overflow
        x_str = elem.attrib.get("x")
        font_size_str = elem.attrib.get("font-size", elem.attrib.get("style", ""))
        font_size = 11.0
        fs_match = re.search(r"font-size:\s*([0-9.]+)px", font_size_str) or re.search(r"\b([0-9.]+)px\b", font_size_str)
        if fs_match:
            font_size = float(fs_match.group(1))

        if width_val and x_str:
            try:
                x_val = float(x_str)
                text_anchor = elem.attrib.get("text-anchor", "start")
                # Approximate width: ~0.6 * font_size * len(text)
                estimated_width = len(text_content) * font_size * 0.62
                if text_anchor == "middle":
                    right_edge = x_val + estimated_width / 2
                elif text_anchor == "end":
                    right_edge = x_val
                else:
                    right_edge = x_val + estimated_width

                if right_edge > width_val - 5:  # within 5px of canvas edge
                    warnings.append(
                        f"Potential text overflow beyond canvas: '{text_content[:35]}...' "
                        f"(estimated right edge {right_edge:.1f}px > canvas width {width_val:.1f}px)."
                    )
            except ValueError:
                pass

    return warnings


def validate_file(path: Path) -> tuple[bool, list[str]]:
    """Validate a single figure file."""
    if not path.exists():
        return False, [f"File does not exist: {path}"]

    size = path.stat().st_size
    if size == 0:
        return False, [f"File is empty: {path}"]

    ext = path.suffix.lower()
    if ext not in [".svg", ".png", ".webp"]:
        return False, [f"Unsupported figure extension '{ext}'. Must be .svg or .png."]

    messages: list[str] = [f"Size: {size / 1024:.1f} KB"]

    if ext == ".svg":
        if size > MAX_SVG_SIZE_BYTES:
            return False, [f"SVG file exceeds 1 MB limit ({size / 1024:.1f} KB). Simplify paths or use PNG."]
        try:
            content = path.read_text(encoding="utf-8")
            root = ET.fromstring(content)
        except Exception as e:
            return False, [f"Invalid SVG XML syntax: {e}"]

        # Canvas dark background check
        if DARK_CANVAS_HEX.lower() not in content.lower():
            messages.append(f"WARNING: Dark canvas ({DARK_CANVAS_HEX}) not explicitly found in SVG.")

        # Typography and layout checks
        typography_warnings = check_svg_typography_and_layout(root)
        if typography_warnings:
            messages.extend([f"LINTER WARNING: {w}" for w in typography_warnings])

    return True, messages


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate scientific figure assets.")
    parser.add_argument("figures", nargs="+", help="Figure file paths to validate.")
    parser.add_argument("--strict", action="store_true", help="Fail on linter warnings.")
    args = parser.parse_args()

    all_passed = True
    for fig_str in args.figures:
        path = Path(fig_str)
        passed, msgs = validate_file(path)
        has_warnings = any("WARNING" in m for m in msgs)
        if args.strict and has_warnings:
            passed = False

        status = "PASS" if passed else "FAIL"
        print(f"[{status}] {path}")
        for m in msgs:
            print(f"       {m}")
        if not passed:
            all_passed = False

    return 0 if all_passed else 1


if __name__ == "__main__":
    sys.exit(main())
