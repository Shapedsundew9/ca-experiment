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

    # Try to extract canvas height
    height_str = root.attrib.get("height", "")
    height_val = None
    height_match = re.match(r"^([0-9.]+)", height_str)
    if height_match:
        height_val = float(height_match.group(1))

    # Collect container rectangles
    rect_elements = root.findall(".//{http://www.w3.org/2000/svg}rect")
    if not rect_elements:
        rect_elements = root.findall(".//rect")
    
    rect_containers = []
    for rect in rect_elements:
        try:
            rx = float(rect.attrib.get("x", 0))
            ry = float(rect.attrib.get("y", 0))
            rw = float(rect.attrib.get("width", 0))
            rh = float(rect.attrib.get("height", 0))
            # Ignore the canvas background rect if it matches canvas width/height
            if width_val and height_val and rw >= width_val - 2 and rh >= height_val - 2:
                continue
            rect_containers.append((rx, ry, rw, rh))
        except ValueError:
            pass

    # Collect all text content
    text_elements = root.findall(".//{http://www.w3.org/2000/svg}text")
    if not text_elements:
        # Without namespace
        text_elements = root.findall(".//text")

    text_info_list = []

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
        y_str = elem.attrib.get("y")
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

        if y_str:
            try:
                y_val = float(y_str)
                
                # 1a. Vertical canvas overflow
                if height_val and y_val > height_val - 5:
                    warnings.append(
                        f"Potential text overflow below canvas: '{text_content[:35]}...' "
                        f"(y={y_val:.1f}px > canvas height {height_val:.1f}px)."
                    )
                
                # 1b. Container-text containment
                # Find the smallest rect that fully contains the text position.
                # If no rect fully contains it, check if it's just below
                # a rect (i.e., text has overflowed its container).
                if x_str:
                    x_val = float(x_str)
                    # Rects that fully contain the text point
                    fully_enclosing = []
                    # Rects whose x-range contains the text but text is below bottom
                    overflowed_from = []
                    for rx, ry, rw, rh in rect_containers:
                        if rx <= x_val <= rx + rw:
                            if ry <= y_val <= ry + rh:
                                fully_enclosing.append((rx, ry, rw, rh))
                            elif y_val > ry + rh:
                                overflowed_from.append((rx, ry, rw, rh))

                    if not fully_enclosing and overflowed_from:
                        # Text is below the nearest container — find the
                        # closest one (smallest gap between container bottom
                        # and text y) to report the most relevant overflow.
                        nearest = min(overflowed_from,
                                      key=lambda r: y_val - (r[1] + r[3]))
                        rx, ry, rw, rh = nearest
                        gap = y_val - (ry + rh)
                        # Only report if the gap is modest (< 50px),
                        # avoiding false positives from unrelated rects.
                        if gap < 50:
                            warnings.append(
                                f"Text exceeds container bounds: '{text_content[:35]}...' "
                                f"(y={y_val:.1f}px > container bottom {ry+rh:.1f}px)."
                            )

                    # For 1c
                    text_info_list.append((x_val, y_val, font_size, text_content))
            except ValueError:
                pass

    # 1c. Text-text collision detection
    text_info_list.sort(key=lambda t: t[0])
    groups = []
    current_group = []
    group_base_x = None
    for t in text_info_list:
        if not current_group:
            current_group.append(t)
            group_base_x = t[0]
        else:
            if abs(t[0] - group_base_x) <= 50:
                current_group.append(t)
            else:
                groups.append(current_group)
                current_group = [t]
                group_base_x = t[0]
    if current_group:
        groups.append(current_group)

    for group in groups:
        group.sort(key=lambda t: t[1])
        for k in range(len(group) - 1):
            x1, y1, fs1, txt1 = group[k]
            x2, y2, fs2, txt2 = group[k+1]
            if y2 - y1 < min(fs1, fs2) * 0.8:
                warnings.append(
                    f"Potential text collision: '{txt1[:35]}...' at y={y1:.1f} overlaps '{txt2[:35]}...' at y={y2:.1f}."
                )

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

        # Dark canvas contrast check: forbid unstyled black elements (text, ticks, lines) on dark background
        black_matches = re.findall(
            r'(?:stroke|fill)\s*[:=]\s*["\']?(?:#000000|#000\b|black\b|rgb\(\s*0\s*,\s*0\s*,\s*0\s*\))',
            content,
            re.IGNORECASE,
        )
        if black_matches:
            messages.append(
                f"LINTER WARNING: Found {len(black_matches)} unstyled black (#000000/black) stroke or fill elements "
                f"on dark canvas ({DARK_CANVAS_HEX}). Use light typography and strokes "
                f"(e.g. TEXT '#e2e8f0', TEXT_MUTED '#94a3b8', or BORDER '#434c5e')."
            )

        # Check for unstyled text elements (where fill is omitted and defaults to black in SVG)
        unstyled_text_warnings = []
        for g in root.iter():
            gid = g.attrib.get("id", "")
            if gid.startswith("text_"):
                for child in g:
                    # Ignore bbox background patches (e.g. patch_17 inside text_41)
                    if child.attrib.get("id", "").startswith("patch_"):
                        continue
                    if child.tag.endswith("g") or child.tag == "g":
                        style = child.attrib.get("style", "")
                        fill = child.attrib.get("fill", "")
                        if not fill and "fill:" not in style:
                            c_match = re.search(rf'<g id=["\']{re.escape(gid)}["\']>\s*<!--\s*(.*?)\s*-->', content)
                            desc = f"'{c_match.group(1)}'" if c_match else f"id '{gid}'"
                            unstyled_text_warnings.append(
                                f"Unstyled text element (defaults to black) on dark canvas: {desc}. "
                                f"Ensure axis labels and text have light color (e.g. TEXT '#e2e8f0')."
                            )
            elif g.tag.endswith("text") or g.tag == "text":
                style = g.attrib.get("style", "")
                fill = g.attrib.get("fill", "")
                if not fill and "fill:" not in style:
                    txt = "".join(g.itertext()).strip()[:40]
                    if txt and not txt.startswith("Figure FIG-") and ".py" not in txt:
                        unstyled_text_warnings.append(
                            f"Unstyled SVG <text> element (defaults to black): '{txt}'. "
                            f"Specify fill=TEXT ('#e2e8f0') or fill=TEXT_MUTED ('#94a3b8')."
                        )
        if unstyled_text_warnings:
            messages.extend([f"LINTER WARNING: {w}" for w in unstyled_text_warnings])

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
