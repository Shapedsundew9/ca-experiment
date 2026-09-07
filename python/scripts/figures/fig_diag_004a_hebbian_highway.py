#!/usr/bin/env python3
"""Figure Generator: FIG-DIAG-004a - Wavefront Collision vs. Hebbian Resonant Highway.

Target Document: docs/research/diagnostics/DIAG-2026-004a.md
Output: docs/research/assets/fig-diag-004a-hebbian-highway.svg

Visualizes the core dynamical mechanism of EXP-2026-004a:
Left Panel: Static isotropic lattice with counter-propagating wavefronts colliding
            and annihilating at refractory boundaries at antipodal distance D = 4.
Right Panel: Hebbian synaptic track plasticity breaking isotropic symmetry into a
             directed chiral resonant highway (+463.6% perturbation resilience).
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

import drawsvg as draw
from tools.viz import (
    AMBER,
    AMBER_FILL,
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


def build_hebbian_highway_figure(output_path: str) -> None:
    width = 1040
    height = 620

    d = draw.Drawing(width, height)
    # Dark canvas
    d.append(draw.Rectangle(0, 0, width, height, fill=DARK_CANVAS))

    # Master Header
    d.append(
        draw.Text(
            "Mechanism of Perturbation Resilience: Wave Collision vs. Hebbian Resonant Ring",
            19,
            width / 2,
            34,
            text_anchor="middle",
            fill=TEXT,
            font_family="sans-serif",
            font_weight="bold",
        )
    )
    d.append(
        draw.Text(
            "EXP-2026-004a Diagnostic Analysis | Active Hebbian (+463.6% Resilience) vs Static Isotropic (Annihilation)",
            12,
            width / 2,
            56,
            text_anchor="middle",
            fill=TEXT_MUTED,
            font_family="monospace",
        )
    )

    # Markers
    arrow_red = draw.Marker(-1, -0.5, 0.9, 0.5, scale=4, orient="auto")
    arrow_red.append(draw.Lines(-1, -0.5, -1, 0.5, 0, 0, fill=PRIMARY_RED, close=True))
    d.append(arrow_red)

    arrow_green = draw.Marker(-1, -0.5, 0.9, 0.5, scale=4, orient="auto")
    arrow_green.append(draw.Lines(-1, -0.5, -1, 0.5, 0, 0, fill=SECONDARY_GREEN, close=True))
    d.append(arrow_green)

    arrow_amber = draw.Marker(-1, -0.5, 0.9, 0.5, scale=4, orient="auto")
    arrow_amber.append(draw.Lines(-1, -0.5, -1, 0.5, 0, 0, fill=AMBER, close=True))
    d.append(arrow_amber)

    # Side-by-side panel geometries
    panel_w = 475
    panel_h = 505
    panel1_x = 30
    panel2_x = 535
    panel_y = 80

    # Panel 1: Static Isotropic Baseline
    d.append(draw.Rectangle(panel1_x, panel_y, panel_w, panel_h, rx=8, ry=8, fill=DARK_PANEL, stroke=BORDER, stroke_width=1.0))
    d.append(draw.Text("Static Isotropic Baseline (Wᵢⱼ ≡ 1.0)", 14, panel1_x + 20, panel_y + 26, fill=AMBER, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("Outcome: Symmetric Radiation & Wave Annihilation", 11, panel1_x + 20, panel_y + 44, fill=TEXT_MUTED, font_family="sans-serif"))

    # Panel 2: Active Hebbian Adaptation
    d.append(draw.Rectangle(panel2_x, panel_y, panel_w, panel_h, rx=8, ry=8, fill=DARK_PANEL, stroke=BORDER, stroke_width=1.0))
    d.append(draw.Text("Active Hebbian Adaptation (Chiral Loop)", 14, panel2_x + 20, panel_y + 26, fill=SECONDARY_GREEN, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("Outcome: Broken Symmetry & Persistent Ring Circulation", 11, panel2_x + 20, panel_y + 44, fill=TEXT_MUTED, font_family="sans-serif"))

    # Draw mini 4x4 grids in each panel
    def draw_grid_nodes(origin_x: int, origin_y: int, cell_gap: int):
        node_coords = {}
        for y in range(4):
            for x in range(4):
                nx = origin_x + x * cell_gap
                ny = origin_y + y * cell_gap
                node_id = y * 4 + x
                node_coords[node_id] = (nx, ny)
        return node_coords

    # Grid 1
    g1 = draw_grid_nodes(panel1_x + 105, panel_y + 90, 75)
    for nid, (nx, ny) in g1.items():
        subscript_nid = format_subscript(f"v_{nid}")
        if nid == 0:
            fill_c = PRIMARY_RED_FILL
            stroke_c = PRIMARY_RED
        elif nid == 10:  # Antipodal node (2, 2)
            fill_c = AMBER_FILL
            stroke_c = AMBER
        else:
            fill_c = DARK_CANVAS
            stroke_c = BORDER

        d.append(draw.Circle(nx, ny, 17, fill=fill_c, stroke=stroke_c, stroke_width=1.5))
        d.append(draw.Text(subscript_nid, 10, nx, ny + 3.5, text_anchor="middle", fill=TEXT, font_family="sans-serif", font_weight="bold"))

    # Ingress and collision labels
    d.append(draw.Text("Sensory Ingress (v₀)", 10, g1[0][0], g1[0][1] - 22, text_anchor="middle", fill=PRIMARY_RED, font_family="sans-serif", font_weight="bold"))

    # In Panel 1: Symmetrical Wavefront Collision Lines
    path1 = draw.Path(stroke=AMBER, stroke_width=2.2, fill="none", marker_end=arrow_amber)
    path1.M(g1[0][0] + 17, g1[0][1])
    path1.L(g1[1][0] - 17, g1[1][1])
    d.append(path1)

    path2 = draw.Path(stroke=AMBER, stroke_width=2.2, fill="none", marker_end=arrow_amber)
    path2.M(g1[1][0] + 17, g1[1][1])
    path2.L(g1[2][0] - 17, g1[2][1])
    d.append(path2)

    path3 = draw.Path(stroke=AMBER, stroke_width=2.2, fill="none", marker_end=arrow_amber)
    path3.M(g1[2][0], g1[2][1] + 17)
    path3.L(g1[6][0], g1[6][1] - 17)
    d.append(path3)

    path4 = draw.Path(stroke=AMBER, stroke_width=2.2, fill="none", marker_end=arrow_amber)
    path4.M(g1[6][0], g1[6][1] + 17)
    path4.L(g1[10][0], g1[10][1] - 17)
    d.append(path4)

    # Opposing path
    path5 = draw.Path(stroke=PRIMARY_RED, stroke_width=2.2, fill="none", marker_end=arrow_red)
    path5.M(g1[0][0], g1[0][1] + 17)
    path5.L(g1[4][0], g1[4][1] - 17)
    d.append(path5)

    path6 = draw.Path(stroke=PRIMARY_RED, stroke_width=2.2, fill="none", marker_end=arrow_red)
    path6.M(g1[4][0], g1[4][1] + 17)
    path6.L(g1[8][0], g1[8][1] - 17)
    d.append(path6)

    path7 = draw.Path(stroke=PRIMARY_RED, stroke_width=2.2, fill="none", marker_end=arrow_red)
    path7.M(g1[8][0] + 17, g1[8][1])
    path7.L(g1[9][0] - 17, g1[9][1])
    d.append(path7)

    path8 = draw.Path(stroke=PRIMARY_RED, stroke_width=2.2, fill="none", marker_end=arrow_red)
    path8.M(g1[9][0] + 17, g1[9][1])
    path8.L(g1[10][0] - 17, g1[10][1])
    d.append(path8)

    # Annihilation blast at node v_10
    d.append(draw.Text("✖", 22, g1[10][0], g1[10][1] - 24, text_anchor="middle", fill=PRIMARY_RED))
    d.append(draw.Text("Mutual Annihilation (Rᵢ = Nᵣₑf)", 10.5, g1[10][0], g1[10][1] + 32, text_anchor="middle", fill=AMBER, font_family="sans-serif", font_weight="bold"))

    # Panel 1 Summary Card (Wrapped, zero overflow)
    bullets_p1 = [
        ("Symmetric Annihilation", "Uniform weights Wᵢⱼ ≡ 1.0 radiate waves that collide head-on at antipodal node v₁₀ (D = 4)."),
        ("Noise Breakdown", "5% bit-flip noise jitters wavefront collision, destroying attractor orbits (Ω = 0.0088)."),
    ]
    draw_card_with_bullets(
        d,
        panel1_x + 15,
        panel_y + panel_h - 140,
        panel_w - 30,
        130,
        bullets_p1,
        max_chars=54,
    )

    # Grid 2: Active Hebbian Highway
    g2 = draw_grid_nodes(panel2_x + 105, panel_y + 90, 75)
    for nid, (nx, ny) in g2.items():
        subscript_nid = format_subscript(f"v_{nid}")
        if nid in [0, 1, 2, 6, 7, 11, 15, 12, 8, 4]:
            fill_c = SECONDARY_GREEN_FILL
            stroke_c = SECONDARY_GREEN
        else:
            fill_c = DARK_CANVAS
            stroke_c = BORDER

        d.append(draw.Circle(nx, ny, 17, fill=fill_c, stroke=stroke_c, stroke_width=1.5))
        d.append(draw.Text(subscript_nid, 10, nx, ny + 3.5, text_anchor="middle", fill=TEXT, font_family="sans-serif", font_weight="bold"))

    # Consolidated Loop Highway: 0 -> 1 -> 2 -> 6 -> 7 -> 11 -> 15 -> (wrap to 12) -> 8 -> 4 -> 0
    highway_edges = [(0, 1), (1, 2), (2, 6), (6, 7), (7, 11), (11, 15), (15, 12), (12, 8), (8, 4), (4, 0)]
    for u, v in highway_edges:
        ux, uy = g2[u]
        vx, vy = g2[v]
        if u == 15 and v == 12:
            p = draw.Path(stroke=SECONDARY_GREEN, stroke_width=3.5, fill="none", marker_end=arrow_green)
            p.M(ux, uy + 17)
            p.C(ux + 30, uy + 45, vx - 30, vy + 45, vx, vy + 17)
            d.append(p)
        else:
            dx, dy = vx - ux, vy - uy
            dist = (dx**2 + dy**2)**0.5
            sx, sy = ux + (dx / dist) * 17, uy + (dy / dist) * 17
            ex, ey = vx - (dx / dist) * 17, vy - (dy / dist) * 17
            p = draw.Path(stroke=SECONDARY_GREEN, stroke_width=3.5, fill="none", marker_end=arrow_green)
            p.M(sx, sy)
            p.L(ex, ey)
            d.append(p)

    # Label on loop
    d.append(draw.Text("Chiral Resonant Highway (Wᵢⱼ → 2.5)", 11, panel2_x + panel_w / 2, panel_y + 190, text_anchor="middle", fill=SECONDARY_GREEN, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("Persistent Unidirectional Circulation", 10, panel2_x + panel_w / 2, panel_y + 208, text_anchor="middle", fill=TEXT, font_family="sans-serif"))

    # Panel 2 Summary Card (Wrapped, zero overflow)
    bullets_p2 = [
        ("Symmetry Breaking", "Hebbian correlation strengthens coincident tracks; depression suppresses reverse channels."),
        ("Perturbation Resilience", "Noise resilience reaches Ω = 0.0494 (+463.6% gain, p < 10⁻³² vs static baseline)."),
    ]
    draw_card_with_bullets(
        d,
        panel2_x + 15,
        panel_y + panel_h - 140,
        panel_w - 30,
        130,
        bullets_p2,
        max_chars=54,
    )

    # Footer note
    d.append(
        draw.Text(
            "Figure FIG-DIAG-004a | ca-experiment | Generated via python/scripts/figures/fig_diag_004a_hebbian_highway.py",
            10,
            width / 2,
            height - 12,
            text_anchor="middle",
            fill=TEXT_MUTED,
            font_family="monospace",
        )
    )

    save_figure(d, output_path)
    print(f"Generated FIG-DIAG-004a at: {output_path}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate FIG-DIAG-004a Hebbian Highway SVG.")
    parser.add_argument(
        "--output",
        default="docs/research/assets/fig-diag-004a-hebbian-highway.svg",
        help="Target output path.",
    )
    args = parser.parse_args()

    build_hebbian_highway_figure(args.output)
    return 0


if __name__ == "__main__":
    sys.exit(main())
