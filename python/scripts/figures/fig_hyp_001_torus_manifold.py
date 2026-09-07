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
    measure_card_with_bullets,
    format_subscript,
    save_figure,
)



import numpy as np
import matplotlib.pyplot as plt
from tools.viz import apply_dark_theme

def render_torus_3d(output_path: str) -> None:
    n_points = 50
    r_major, r_minor = 3.0, 1.0
    theta = np.linspace(0, 2 * np.pi, n_points)
    phi = np.linspace(0, 2 * np.pi, n_points)
    theta, phi = np.meshgrid(theta, phi)

    x = (r_major + r_minor * np.cos(theta)) * np.cos(phi)
    y = (r_major + r_minor * np.cos(theta)) * np.sin(phi)
    z = r_minor * np.sin(theta)

    fig = plt.figure(figsize=(8, 6))
    ax = fig.add_subplot(111, projection="3d")
    apply_dark_theme(fig, ax)

    surf = ax.plot_surface(
        x,
        y,
        z,
        color=TERTIARY_BLUE,
        edgecolor=BORDER,
        linewidth=0.4,
        alpha=0.85,
        shade=True,
    )

    ax.view_init(elev=32.0, azim=45.0)
    ax.set_title(r"$\mathbb{T}^2$ Continuous Toroidal Embedding", pad=12)
    ax.set_xlabel(r"X (Toroidal Angle $\phi$)", labelpad=8)
    ax.set_ylabel(r"Y (Poloidal Angle $\theta$)", labelpad=8)
    ax.set_zlabel("Z", labelpad=8)

    save_figure(fig, output_path, dpi=300)
    print(f"Generated FIG-HYP-001 3D Torus at: {output_path}")


def build_torus_manifold_figure(output_path: str) -> None:

    bullets = [
        ("Periodic Closure", "Node coordinates (x, y) wrap modulo 4 across boundaries."),
        ("Graph Regularity", "Every node has exactly 4 incoming and 4 outgoing tracks."),
        ("Transmission Channels", "64 directed bit tracks with unit delay (latency τ = 1)."),
        ("Topological Diameter", "D = 2 + 2 = 4 (Maximum Manhattan graph distance)."),
        ("Boundary Invariant", "Zero edge reflections or artificial boundary damping."),
    ]
    required_card_height = measure_card_with_bullets(bullets, title="Toroidal Substrate Invariants", max_chars=44)
    
    width = 1040
    # Left panel needs 4*88+130 = 482
    # Right panel top part is roughly 160 px height? Let's check card_y = margin_y + 160
    # original card_y = margin_y + 240. So top part takes 240 px.
    # required panel_h = max(482, 240 + required_card_height + 20)
    
    margin_x = 80
    margin_y = 105
    cell_size = 88
    cols, rows = 4, 4

    panel_w = cols * cell_size + 140
    panel_h = max(rows * cell_size + 130, 240 + required_card_height + 20)
    height = margin_y + panel_h + 35

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

    # 3D Torus Graphic reference
    tc_x = right_x + right_w / 2
    tc_y = margin_y + 115

    d.append(
        draw.Text(
            "See companion 3D render:",
            16,
            tc_x,
            tc_y - 15,
            text_anchor="middle",
            fill=TEXT,
            font_family="sans-serif",
        )
    )
    d.append(
        draw.Text(
            "fig-hyp-001-torus-3d.png",
            18,
            tc_x,
            tc_y + 15,
            text_anchor="middle",
            fill=TERTIARY_BLUE,
            font_family="monospace",
            font_weight="bold",
        )
    )

    # Mathematical Properties Card (Auto-wrapped, zero overflow!)
    card_y = margin_y + 240
    card_h = required_card_height
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
    parser.add_argument(
        "--torus-3d-output",
        default="docs/research/assets/fig-hyp-001-torus-3d.png",
        help="Target output path for the companion 3D torus render.",
    )
    args = parser.parse_args()

    build_torus_manifold_figure(args.output)
    render_torus_3d(args.torus_3d_output)
    return 0

if __name__ == "__main__":
    sys.exit(main())
