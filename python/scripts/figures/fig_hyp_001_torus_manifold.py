#!/usr/bin/env python3
"""Figure Generator: FIG-HYP-001 - 16-Node 2D Torus Manifold & von Neumann Stencil.

Target Document: docs/research/hypotheses/HYP-2026-001.md
Output: docs/research/assets/fig-hyp-001-torus-manifold.svg

Visualizes the discrete 2D torus cellular automaton lattice (N = 16),
flat indexing i = 4y + x, regular von Neumann 4-neighborhood channels,
periodic wrap-around boundary conditions, and the continuous 3D toroidal manifold (𝕋²).
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
    GRID,
    PRIMARY_RED,
    PRIMARY_RED_FILL,
    SECONDARY_GREEN,
    SECONDARY_GREEN_FILL,
    TERTIARY_BLUE,
    TERTIARY_BLUE_FILL,
    TEXT,
    TEXT_MUTED,
    draw_card_with_bullets,
    format_subscript,
    save_figure,
)


def build_torus_manifold_figure(output_path: str) -> None:
    width = 1040
    height = 580

    d = draw.Drawing(width, height)
    # Dark canvas
    d.append(draw.Rectangle(0, 0, width, height, fill=DARK_CANVAS))

    # Master Title & Mathematical Subtitle
    d.append(
        draw.Text(
            "16-Node Discrete Torus Substrate Manifold & Neighborhood Stencil",
            20,
            width / 2,
            36,
            text_anchor="middle",
            fill=TEXT,
            font_family="sans-serif",
            font_weight="bold",
        )
    )
    d.append(
        draw.Text(
            "Topology: 𝒱 = ℤ₄ × ℤ₄,  |𝒱| = 16,  dᵢₙ = dₒᵤₜ = 4,  |ℰ| = 64 Tracks,  Topological Diameter D = 4",
            13,
            width / 2,
            60,
            text_anchor="middle",
            fill=TEXT_MUTED,
            font_family="monospace",
        )
    )

    # Arrow markers
    arrow_edge = draw.Marker(-1, -0.5, 0.9, 0.5, scale=4, orient="auto")
    arrow_edge.append(draw.Lines(-1, -0.5, -1, 0.5, 0, 0, fill=EDGE, close=True))
    d.append(arrow_edge)

    arrow_green = draw.Marker(-1, -0.5, 0.9, 0.5, scale=4, orient="auto")
    arrow_green.append(draw.Lines(-1, -0.5, -1, 0.5, 0, 0, fill=SECONDARY_GREEN, close=True))
    d.append(arrow_green)

    arrow_red = draw.Marker(-1, -0.5, 0.9, 0.5, scale=4, orient="auto")
    arrow_red.append(draw.Lines(-1, -0.5, -1, 0.5, 0, 0, fill=PRIMARY_RED, close=True))
    d.append(arrow_red)

    arrow_amber = draw.Marker(-1, -0.5, 0.9, 0.5, scale=4, orient="auto")
    arrow_amber.append(draw.Lines(-1, -0.5, -1, 0.5, 0, 0, fill=AMBER, close=True))
    d.append(arrow_amber)

    # ==========================================
    # LEFT PANEL: 4x4 Torus Lattice & Stencil
    # ==========================================
    margin_x = 80
    margin_y = 105
    cell_size = 88
    cols, rows = 4, 4

    panel_w = cols * cell_size + 140
    panel_h = rows * cell_size + 130
    d.append(
        draw.Rectangle(
            margin_x - 70,
            margin_y - 25,
            panel_w,
            panel_h,
            rx=10,
            ry=10,
            fill=DARK_PANEL,
            stroke=BORDER,
            stroke_width=1.0,
        )
    )
    d.append(
        draw.Text(
            "Flat Torus Coordinate Grid (ℤ₄ × ℤ₄)",
            14,
            margin_x - 50,
            margin_y - 6,
            fill=TEXT,
            font_family="sans-serif",
            font_weight="bold",
        )
    )

    # Target node is (1, 1) -> Node 5
    tx, ty = 1, 1
    von_neumann = {
        ((tx, (ty - 1) % rows)): ("North (v₁)", (0, -1)),
        (((tx + 1) % cols, ty)): ("East (v₆)", (1, 0)),
        ((tx, (ty + 1) % rows)): ("South (v₉)", (0, 1)),
        (((tx - 1) % cols, ty)): ("West (v₄)", (-1, 0)),
    }

    # Draw all 16 nodes with proper Unicode subscripts
    for y in range(rows):
        for x in range(cols):
            cx = margin_x + x * cell_size + cell_size / 2
            cy = margin_y + y * cell_size + cell_size / 2
            node_id = y * cols + x
            subscript_label = format_subscript(f"v_{node_id}")

            if (x, y) == (tx, ty):
                fill_color = PRIMARY_RED_FILL
                stroke_color = PRIMARY_RED
                stroke_w = 2.5
                badge = "Target vᵢ"
            elif (x, y) in von_neumann:
                fill_color = SECONDARY_GREEN_FILL
                stroke_color = SECONDARY_GREEN
                stroke_w = 2.0
                badge = von_neumann[(x, y)][0].split()[0]
            else:
                fill_color = DARK_CANVAS
                stroke_color = BORDER
                stroke_w = 1.2
                badge = None

            r = cell_size * 0.40
            d.append(
                draw.Rectangle(
                    cx - r,
                    cy - r,
                    r * 2,
                    r * 2,
                    rx=6,
                    ry=6,
                    fill=fill_color,
                    stroke=stroke_color,
                    stroke_width=stroke_w,
                )
            )

            # Node label vᵢ with proper Unicode subscript
            d.append(
                draw.Text(
                    subscript_label,
                    15,
                    cx,
                    cy - 3,
                    text_anchor="middle",
                    fill=TEXT,
                    font_family="sans-serif",
                    font_weight="bold",
                )
            )
            # Coordinates (x, y)
            d.append(
                draw.Text(
                    f"({x},{y})",
                    10,
                    cx,
                    cy + 13,
                    text_anchor="middle",
                    fill=TEXT_MUTED,
                    font_family="monospace",
                )
            )

            if badge:
                d.append(
                    draw.Text(
                        badge,
                        9.5,
                        cx,
                        cy + 25,
                        text_anchor="middle",
                        fill=AMBER if (x, y) == (tx, ty) else SECONDARY_GREEN,
                        font_family="sans-serif",
                        font_weight="bold",
                    )
                )

    # Periodic Wrap-Around Boundary Arcs
    for y in range(rows):
        cy = margin_y + y * cell_size + cell_size / 2
        lx = margin_x + cell_size / 2 - cell_size * 0.40
        rx = margin_x + 3 * cell_size + cell_size / 2 + cell_size * 0.40
        p = draw.Path(stroke=EDGE, stroke_width=1.2, fill="none", stroke_dasharray="3,3", marker_end=arrow_edge)
        p.M(rx, cy - 6)
        p.C(rx + 30, cy - 25, lx - 30, cy - 25, lx, cy - 6)
        d.append(p)

    for x in range(cols):
        cx = margin_x + x * cell_size + cell_size / 2
        ty_edge = margin_y + cell_size / 2 - cell_size * 0.40
        by_edge = margin_y + 3 * cell_size + cell_size / 2 + cell_size * 0.40
        p = draw.Path(stroke=AMBER, stroke_width=1.2, fill="none", stroke_dasharray="3,3", marker_end=arrow_amber)
        p.M(cx + 6, by_edge)
        p.C(cx + 25, by_edge + 30, cx + 25, ty_edge - 30, cx + 6, ty_edge)
        d.append(p)

    # ==========================================
    # RIGHT PANEL: Continuous Torus & Properties
    # ==========================================
    right_x = margin_x + panel_w + 25
    right_w = width - right_x - 20
    d.append(
        draw.Rectangle(
            right_x,
            margin_y - 25,
            right_w,
            panel_h,
            rx=10,
            ry=10,
            fill=DARK_PANEL,
            stroke=BORDER,
            stroke_width=1.0,
        )
    )
    d.append(
        draw.Text(
            "Continuous Toroidal Embedding (𝕋²)",
            14,
            right_x + 20,
            margin_y - 6,
            fill=TEXT,
            font_family="sans-serif",
            font_weight="bold",
        )
    )

    # 3D Torus Graphic with Isometric Depth
    tc_x = right_x + right_w / 2
    tc_y = margin_y + 115

    # Shaded outer toroidal silhouette
    d.append(draw.Ellipse(tc_x, tc_y, 160, 85, fill=TERTIARY_BLUE_FILL, stroke=TERTIARY_BLUE, stroke_width=2.0))
    # Inner hole
    d.append(draw.Ellipse(tc_x, tc_y, 60, 30, fill=DARK_PANEL, stroke=BORDER, stroke_width=1.8))

    # Toroidal coordinate equator circle φ (Green dashed ring)
    loop_phi = draw.Path(stroke=SECONDARY_GREEN, stroke_width=2.0, fill="none", stroke_dasharray="4,3")
    loop_phi.M(tc_x - 160, tc_y)
    loop_phi.A(160, 85, 0, 0, 0, tc_x + 160, tc_y)
    d.append(loop_phi)

    # Poloidal coordinate meridian circle θ (Amber dashed ring)
    loop_theta = draw.Path(stroke=AMBER, stroke_width=2.0, fill="none", stroke_dasharray="4,3")
    loop_theta.M(tc_x + 60, tc_y)
    loop_theta.A(50, 60, 0, 1, 0, tc_x + 160, tc_y)
    d.append(loop_theta)

    # Coordinates labels with Greek mathematical typography
    d.append(draw.Text("Poloidal Angle θ (Row Periodicity: y + 1 mod 4)", 11, tc_x, tc_y + 105, text_anchor="middle", fill=AMBER, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("Toroidal Angle φ (Col Periodicity: x + 1 mod 4)", 11, tc_x, tc_y - 95, text_anchor="middle", fill=SECONDARY_GREEN, font_family="sans-serif", font_weight="bold"))

    # Mathematical Properties Card (Auto-wrapped, zero overflow!)
    card_y = margin_y + 240
    card_h = panel_h - 265
    bullets = [
        ("Periodic Closure", "Node coordinates (x, y) wrap modulo 4 across boundaries."),
        ("Graph Regularity", "Every node has exactly 4 incoming and 4 outgoing tracks."),
        ("Transmission Channels", "64 directed bit tracks with unit delay (latency τ = 1)."),
        ("Topological Diameter", "D = 2 + 2 = 4 (Maximum Manhattan graph distance)."),
        ("Boundary Invariant", "Zero edge reflections or artificial boundary damping."),
    ]
    draw_card_with_bullets(
        d,
        right_x + 15,
        card_y,
        right_w - 30,
        card_h,
        bullets,
        title="Toroidal Substrate Invariants",
        max_chars=44,
        bg_fill=DARK_CANVAS,
        border_color=BORDER,
    )

    # Footer note
    d.append(
        draw.Text(
            "Figure FIG-HYP-001 | ca-experiment | Generated via python/scripts/figures/fig_hyp_001_torus_manifold.py",
            10,
            width / 2,
            height - 12,
            text_anchor="middle",
            fill=TEXT_MUTED,
            font_family="monospace",
        )
    )

    save_figure(d, output_path)
    print(f"Generated FIG-HYP-001 at: {output_path}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate FIG-HYP-001 Torus Manifold SVG.")
    parser.add_argument(
        "--output",
        default="docs/research/assets/fig-hyp-001-torus-manifold.svg",
        help="Target output path.",
    )
    args = parser.parse_args()

    build_torus_manifold_figure(args.output)
    return 0


if __name__ == "__main__":
    sys.exit(main())
