#!/usr/bin/env python3
"""Archetype 2: 2D Discrete Lattices, Periodic Boundaries, and Stencils.

Generates crisp, resolution-independent vector SVG schematics of 2D cellular automata
grids, toroidal wrap-around boundary channels, and neighborhood stencils (von Neumann / Moore)
using DrawSVG.

Usage:
    .venv/bin/python lattice_grid_2d.py --rows 4 --cols 4 --stencil von_neumann --wrap --output docs/assets/figures/lattice_4x4.svg
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

import drawsvg as draw
from tools.viz import (
    AMBER,
    BORDER,
    DARK_CANVAS,
    DARK_PANEL,
    EDGE,
    PRIMARY_RED,
    PRIMARY_RED_FILL,
    SECONDARY_GREEN,
    SECONDARY_GREEN_FILL,
    TERTIARY_BLUE,
    TEXT,
    TEXT_MUTED,
    save_figure,
)


def render_lattice(
    rows: int,
    cols: int,
    output_path: str,
    stencil: str = "von_neumann",
    target_node: tuple[int, int] = (1, 1),
    show_wrap: bool = True,
    cell_size: int = 80,
    margin: int = 120,
) -> None:
    width = cols * cell_size + 2 * margin
    height = rows * cell_size + 2 * margin

    d = draw.Drawing(width, height)
    # Dark Canvas Background
    d.append(draw.Rectangle(0, 0, width, height, fill=DARK_CANVAS))

    # Title
    stencil_label = "von Neumann (4-Neighbor)" if stencil == "von_neumann" else "Moore (8-Neighbor)"
    title_text = f"{cols}x{rows} Discrete Lattice ({stencil_label})"
    if show_wrap:
        title_text += " with Toroidal Wrap-Around"
    d.append(
        draw.Text(
            title_text,
            20,
            width / 2,
            40,
            text_anchor="middle",
            fill=TEXT,
            font_family="sans-serif",
            font_weight="bold",
        )
    )

    # Subtitle / Equation
    d.append(
        draw.Text(
            f"V = {{ (x, y) | x in Z_{cols}, y in Z_{rows} }}, Flat Index i = {cols}y + x",
            13,
            width / 2,
            65,
            text_anchor="middle",
            fill=TEXT_MUTED,
            font_family="monospace",
        )
    )

    # Markers for arrowheads
    arrow = draw.Marker(-1, -0.5, 0.9, 0.5, scale=4, orient="auto")
    arrow.append(draw.Lines(-1, -0.5, -1, 0.5, 0, 0, fill=EDGE, close=True))
    d.append(arrow)

    arrow_green = draw.Marker(-1, -0.5, 0.9, 0.5, scale=4, orient="auto")
    arrow_green.append(draw.Lines(-1, -0.5, -1, 0.5, 0, 0, fill=SECONDARY_GREEN, close=True))
    d.append(arrow_green)

    arrow_amber = draw.Marker(-1, -0.5, 0.9, 0.5, scale=4, orient="auto")
    arrow_amber.append(draw.Lines(-1, -0.5, -1, 0.5, 0, 0, fill=AMBER, close=True))
    d.append(arrow_amber)

    tx, ty = target_node
    # Calculate neighborhood coordinates
    if stencil == "von_neumann":
        neighbors = {
            ((tx + 1) % cols, ty): "East",
            ((tx - 1) % cols, ty): "West",
            (tx, (ty + 1) % rows): "South",
            (tx, (ty - 1) % rows): "North",
        }
    else:  # moore
        neighbors = {}
        for dx in [-1, 0, 1]:
            for dy in [-1, 0, 1]:
                if dx == 0 and dy == 0:
                    continue
                neighbors[((tx + dx) % cols, (ty + dy) % rows)] = "Neighbor"

    # Draw grid cells & links
    for y in range(rows):
        for x in range(cols):
            cx = margin + x * cell_size + cell_size / 2
            cy = margin + y * cell_size + cell_size / 2
            flat_id = y * cols + x

            # Determine cell styling
            if (x, y) == (tx, ty):
                fill_color = PRIMARY_RED_FILL
                stroke_color = PRIMARY_RED
                stroke_w = 2.5
                badge_text = "Target (i)"
            elif (x, y) in neighbors:
                fill_color = SECONDARY_GREEN_FILL
                stroke_color = SECONDARY_GREEN
                stroke_w = 2.0
                badge_text = neighbors[(x, y)]
            else:
                fill_color = DARK_PANEL
                stroke_color = BORDER
                stroke_w = 1.2
                badge_text = None

            # Node card
            node_r = cell_size * 0.42
            d.append(
                draw.Rectangle(
                    cx - node_r,
                    cy - node_r,
                    node_r * 2,
                    node_r * 2,
                    rx=8,
                    ry=8,
                    fill=fill_color,
                    stroke=stroke_color,
                    stroke_width=stroke_w,
                )
            )

            # Node ID and (x, y)
            d.append(
                draw.Text(
                    f"v_{flat_id}",
                    14,
                    cx,
                    cy - 4,
                    text_anchor="middle",
                    fill=TEXT,
                    font_family="monospace",
                    font_weight="bold",
                )
            )
            d.append(
                draw.Text(
                    f"({x},{y})",
                    11,
                    cx,
                    cy + 12,
                    text_anchor="middle",
                    fill=TEXT_MUTED,
                    font_family="monospace",
                )
            )

            if badge_text:
                d.append(
                    draw.Text(
                        badge_text,
                        9,
                        cx,
                        cy + 24,
                        text_anchor="middle",
                        fill=AMBER if (x, y) == (tx, ty) else SECONDARY_GREEN,
                        font_family="sans-serif",
                    )
                )

    # Draw periodic wrap-around boundary arrows if requested
    if show_wrap:
        # Horizontal wrap-around (Row 0 right to left, etc.)
        for y in range(rows):
            cy = margin + y * cell_size + cell_size / 2
            left_x = margin + cell_size / 2 - cell_size * 0.42
            right_x = margin + (cols - 1) * cell_size + cell_size / 2 + cell_size * 0.42

            # Right to left wrap arc (top curve)
            p_top = draw.Path(stroke=EDGE, stroke_width=1.2, fill="none", stroke_dasharray="4,3", marker_end=arrow)
            p_top.M(right_x, cy - 8)
            p_top.C(right_x + 40, cy - 35, left_x - 40, cy - 35, left_x, cy - 8)
            d.append(p_top)

        # Vertical wrap-around (Col 0 bottom to top, etc.)
        for x in range(cols):
            cx = margin + x * cell_size + cell_size / 2
            top_y = margin + cell_size / 2 - cell_size * 0.42
            bottom_y = margin + (rows - 1) * cell_size + cell_size / 2 + cell_size * 0.42

            # Bottom to top wrap arc (side curve)
            p_vert = draw.Path(stroke=AMBER, stroke_width=1.2, fill="none", stroke_dasharray="4,3", marker_end=arrow_amber)
            p_vert.M(cx + 8, bottom_y)
            p_vert.C(cx + 35, bottom_y + 40, cx + 35, top_y - 40, cx + 8, top_y)
            d.append(p_vert)

        # Legend for wrap loops
        d.append(
            draw.Text(
                "--- Periodic Horizontal Wrap-Around (x + 1 mod cols)",
                11,
                width / 2,
                height - 35,
                text_anchor="middle",
                fill=EDGE,
                font_family="sans-serif",
            )
        )
        d.append(
            draw.Text(
                "--- Periodic Vertical Wrap-Around (y + 1 mod rows)",
                11,
                width / 2,
                height - 18,
                text_anchor="middle",
                fill=AMBER,
                font_family="sans-serif",
            )
        )

    save_figure(d, output_path)
    print(f"Lattice figure generated at: {output_path}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate 2D lattice & stencil figures.")
    parser.add_argument("--rows", type=int, default=4, help="Number of rows.")
    parser.add_argument("--cols", type=int, default=4, help="Number of columns.")
    parser.add_argument("--output", required=True, help="Output file path (.svg).")
    parser.add_argument("--stencil", choices=["von_neumann", "moore"], default="von_neumann")
    parser.add_argument("--target-x", type=int, default=1, help="Target node X.")
    parser.add_argument("--target-y", type=int, default=1, help="Target node Y.")
    parser.add_argument("--no-wrap", action="store_true", help="Disable periodic wrap arrows.")

    args = parser.parse_args()
    render_lattice(
        rows=args.rows,
        cols=args.cols,
        output_path=args.output,
        stencil=args.stencil,
        target_node=(args.target_x, args.target_y),
        show_wrap=not args.no_wrap,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
