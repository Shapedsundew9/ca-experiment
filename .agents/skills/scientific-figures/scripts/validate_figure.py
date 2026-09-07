#!/usr/bin/env python3
"""Figure validation utility for scientific documentation figures.

Validates that generated SVG or PNG figures exist, are non-empty, adhere to size bounds,
and conform to the repository dark theme palette (#161922 canvas).

Usage:
    python3 validate_figure.py path/to/figure.svg
    python3 validate_figure.py docs/research/assets/*.svg
"""

from __future__ import annotations

import argparse
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

DARK_CANVAS_HEX = "#161922"
MAX_SVG_SIZE_BYTES = 1024 * 1024  # 1 MB


def validate_file(path: Path) -> tuple[bool, str]:
    """Validate a single figure file."""
    if not path.exists():
        return False, f"File does not exist: {path}"

    size = path.stat().st_size
    if size == 0:
        return False, f"File is empty: {path}"

    ext = path.suffix.lower()
    if ext not in [".svg", ".png", ".webp"]:
        return False, f"Unsupported figure extension '{ext}'. Must be .svg or .png."

    if ext == ".svg":
        if size > MAX_SVG_SIZE_BYTES:
            return False, f"SVG file exceeds 1 MB limit ({size / 1024:.1f} KB). Simplify paths or use PNG."
        try:
            content = path.read_text(encoding="utf-8")
            ET.fromstring(content)
        except Exception as e:
            return False, f"Invalid SVG XML syntax: {e}"

        # Soft warning if canvas background is not dark
        if DARK_CANVAS_HEX.lower() not in content.lower():
            return True, f"VALID with WARNING: Dark canvas ({DARK_CANVAS_HEX}) not explicitly found in SVG."

    return True, f"VALID ({size / 1024:.1f} KB)"


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate scientific figure assets.")
    parser.add_argument("figures", nargs="+", help="Figure file paths to validate.")
    args = parser.parse_args()

    all_passed = True
    for fig_str in args.figures:
        path = Path(fig_str)
        passed, msg = validate_file(path)
        status = "PASS" if passed else "FAIL"
        print(f"[{status}] {path}: {msg}")
        if not passed:
            all_passed = False

    return 0 if all_passed else 1


if __name__ == "__main__":
    sys.exit(main())
