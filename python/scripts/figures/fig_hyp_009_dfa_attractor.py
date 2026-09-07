#!/usr/bin/env python3
"""Figure Generator: FIG-HYP-009 - Coupled Resonant Attractor Basins for Deterministic Finite Automata.

Target Document: docs/research/hypotheses/HYP-2026-009.md
Output: docs/research/assets/fig-hyp-009-dfa-attractor.svg

Visualizes the coupled resonant attractor basin architecture for DFA sequence tracking:
Left Panel: Dual-ring mutually exclusive resonant topology (States q0 and q1),
            showing coincidence transition gates, cross-inhibitory quenching interneurons,
            and nondestructive continuous sense readout.
Right Panel: Discrete phase space partition (Basins B0 and B1), limit-cycle orbits (period P = 4),
             and deterministic causal switching trajectory driven by streaming symbol tokens.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

# Add python/src to sys.path for tools.viz imports
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

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
    measure_card_with_bullets,
    save_figure,
)


def build_dfa_attractor_figure(output_path: str) -> None:
    width = 1040
    height = 660

    d = draw.Drawing(width, height)
    # Canvas background
    d.append(draw.Rectangle(0, 0, width, height, fill=DARK_CANVAS))

    # Master Title & Subtitle
    d.append(
        draw.Text(
            "Coupled Resonant Attractor Basins for Deterministic Finite Automata",
            20,
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
            "Substrate Dynamics: Mutually Exclusive Resonant Loops (L=4), Coincidence Gating, and Quenching Inhibition (W-inh = -1.50)",
            12,
            width / 2,
            56,
            text_anchor="middle",
            fill=TEXT_MUTED,
            font_family="monospace",
        )
    )

    # Markers for directed edges
    arrow_fwd = draw.Marker(-1, -0.5, 0.9, 0.5, scale=4, orient="auto")
    arrow_fwd.append(draw.Lines(-1, -0.5, -1, 0.5, 0, 0, fill=SECONDARY_GREEN, close=True))
    d.append(arrow_fwd)

    arrow_inh = draw.Marker(-1, -0.5, 0.9, 0.5, scale=4, orient="auto")
    arrow_inh.append(draw.Lines(-1, -0.5, -1, 0.5, 0, 0, fill=PRIMARY_RED, close=True))
    d.append(arrow_inh)

    arrow_blue = draw.Marker(-1, -0.5, 0.9, 0.5, scale=4, orient="auto")
    arrow_blue.append(draw.Lines(-1, -0.5, -1, 0.5, 0, 0, fill=TERTIARY_BLUE, close=True))
    d.append(arrow_blue)

    arrow_amber = draw.Marker(-1, -0.5, 0.9, 0.5, scale=4, orient="auto")
    arrow_amber.append(draw.Lines(-1, -0.5, -1, 0.5, 0, 0, fill=AMBER, close=True))
    d.append(arrow_amber)

    panel_y = 75
    panel_h = 565

    # =========================================================================
    # LEFT PANEL: Dual-Ring Coupled Resonant Circuit Architecture
    # =========================================================================
    lp_x = 35
    lp_w = 470
    d.append(draw.Rectangle(lp_x, panel_y, lp_w, panel_h, rx=8, ry=8, fill=DARK_PANEL, stroke=BORDER, stroke_width=1.0))
    d.append(draw.Text("Substrate Circuit: 2-State Parity DFA", 15, lp_x + 16, panel_y + 26, fill=TEXT, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("Mutually Exclusive Resonant Cores & Causal Switching", 11, lp_x + 16, panel_y + 44, fill=TEXT_MUTED, font_family="sans-serif"))

    # Ring 0 (Even State q0) Center: (150, 180), Radius: 48
    r0_cx, r0_cy = lp_x + 105, panel_y + 115
    r_radius = 42

    # Ring 0 Box/Boundary
    d.append(draw.Rectangle(lp_x + 20, panel_y + 55, 170, 125, rx=6, ry=6, fill="#13161f", stroke=SECONDARY_GREEN, stroke_width=1.0, stroke_dasharray="3,3"))
    d.append(draw.Text("Ring R0 (State q-even)", 11, lp_x + 28, panel_y + 72, fill=SECONDARY_GREEN, font_family="sans-serif", font_weight="bold"))

    r0_nodes = [
        (r0_cx - 30, r0_cy),      # R0,0
        (r0_cx, r0_cy - 30),      # R0,1
        (r0_cx + 30, r0_cy),      # R0,2
        (r0_cx, r0_cy + 30),      # R0,3
    ]
    r0_labels = ["R0,0", "R0,1", "R0,2", "R0,3"]

    # Ring 0 forward edges
    for i in range(4):
        x1, y1 = r0_nodes[i]
        x2, y2 = r0_nodes[(i + 1) % 4]
        # shorten for arrow
        dx, dy = x2 - x1, y2 - y1
        dist = (dx**2 + dy**2)**0.5
        ux, uy = dx / dist, dy / dist
        d.append(draw.Line(x1 + ux * 13, y1 + uy * 13, x2 - ux * 13, y2 - uy * 13, stroke=SECONDARY_GREEN, stroke_width=1.5, marker_end=arrow_fwd))

    # Ring 1 (Odd State q1) Center: (370, 180), Radius: 48
    r1_cx, r1_cy = lp_x + 365, panel_y + 115
    d.append(draw.Rectangle(lp_x + 280, panel_y + 55, 170, 125, rx=6, ry=6, fill="#13161f", stroke=TERTIARY_BLUE, stroke_width=1.0, stroke_dasharray="3,3"))
    d.append(draw.Text("Ring R1 (State q-odd)", 11, lp_x + 288, panel_y + 72, fill=TERTIARY_BLUE, font_family="sans-serif", font_weight="bold"))

    r1_nodes = [
        (r1_cx - 30, r1_cy),      # R1,0
        (r1_cx, r1_cy - 30),      # R1,1
        (r1_cx + 30, r1_cy),      # R1,2
        (r1_cx, r1_cy + 30),      # R1,3
    ]
    r1_labels = ["R1,0", "R1,1", "R1,2", "R1,3"]

    # Ring 1 forward edges
    for i in range(4):
        x1, y1 = r1_nodes[i]
        x2, y2 = r1_nodes[(i + 1) % 4]
        dx, dy = x2 - x1, y2 - y1
        dist = (dx**2 + dy**2)**0.5
        ux, uy = dx / dist, dy / dist
        d.append(draw.Line(x1 + ux * 13, y1 + uy * 13, x2 - ux * 13, y2 - uy * 13, stroke=TERTIARY_BLUE, stroke_width=1.5, marker_end=arrow_blue))

    # Draw Nodes of R0
    for idx, (nx, ny) in enumerate(r0_nodes):
        fill_color = SECONDARY_GREEN_FILL if idx == 0 else "#222738"
        stroke_color = SECONDARY_GREEN
        d.append(draw.Circle(nx, ny, 11, fill=fill_color, stroke=stroke_color, stroke_width=1.5))
        d.append(draw.Text(r0_labels[idx], 8, nx, ny + 3, text_anchor="middle", fill=TEXT, font_family="monospace"))

    # Draw Nodes of R1
    for idx, (nx, ny) in enumerate(r1_nodes):
        fill_color = TERTIARY_BLUE_FILL if idx == 0 else "#222738"
        stroke_color = TERTIARY_BLUE
        d.append(draw.Circle(nx, ny, 11, fill=fill_color, stroke=stroke_color, stroke_width=1.5))
        d.append(draw.Text(r1_labels[idx], 8, nx, ny + 3, text_anchor="middle", fill=TEXT, font_family="monospace"))

    # Streaming Input Port & Transition Gating in the middle
    token_in_x = lp_x + 235
    token_in_y = panel_y + 225

    d.append(draw.Rectangle(token_in_x - 45, token_in_y - 14, 90, 28, rx=4, ry=4, fill="#2e271a", stroke=AMBER, stroke_width=1.2))
    d.append(draw.Text("Token σ(t) ∈ {0,1}", 9, token_in_x, token_in_y + 4, text_anchor="middle", fill=AMBER, font_family="monospace", font_weight="bold"))

    # Transition Coincidence Gates
    g01_x, g01_y = lp_x + 155, panel_y + 295
    g10_x, g10_y = lp_x + 315, panel_y + 295

    # Gate G0->1
    d.append(draw.Rectangle(g01_x - 45, g01_y - 18, 90, 36, rx=5, ry=5, fill="#1e2230", stroke=SECONDARY_GREEN, stroke_width=1.2))
    d.append(draw.Text("Gate G 0→1", 9, g01_x, g01_y - 3, text_anchor="middle", fill=SECONDARY_GREEN, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("R0 ∧ σ=1", 8, g01_x, g01_y + 9, text_anchor="middle", fill=TEXT_MUTED, font_family="monospace"))

    # Gate G1->0
    d.append(draw.Rectangle(g10_x - 45, g10_y - 18, 90, 36, rx=5, ry=5, fill="#1e2230", stroke=TERTIARY_BLUE, stroke_width=1.2))
    d.append(draw.Text("Gate G 1→0", 9, g10_x, g10_y - 3, text_anchor="middle", fill=TERTIARY_BLUE, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("R1 ∧ σ=1", 8, g10_x, g10_y + 9, text_anchor="middle", fill=TEXT_MUTED, font_family="monospace"))

    # Inhibitory Interneurons
    inh0_x, inh0_y = lp_x + 105, panel_y + 380
    inh1_x, inh1_y = lp_x + 365, panel_y + 380

    d.append(draw.Circle(inh0_x, inh0_y, 14, fill=PRIMARY_RED_FILL, stroke=PRIMARY_RED, stroke_width=1.5))
    d.append(draw.Text("v-inh0", 8, inh0_x, inh0_y + 3, text_anchor="middle", fill=PRIMARY_RED, font_family="monospace", font_weight="bold"))

    d.append(draw.Circle(inh1_x, inh1_y, 14, fill=PRIMARY_RED_FILL, stroke=PRIMARY_RED, stroke_width=1.5))
    d.append(draw.Text("v-inh1", 8, inh1_x, inh1_y + 3, text_anchor="middle", fill=PRIMARY_RED, font_family="monospace", font_weight="bold"))

    # Connect token input to gates
    d.append(draw.Line(token_in_x - 20, token_in_y + 14, g01_x + 10, g01_y - 18, stroke=AMBER, stroke_width=1.2, marker_end=arrow_amber))
    d.append(draw.Line(token_in_x + 20, token_in_y + 14, g10_x - 10, g10_y - 18, stroke=AMBER, stroke_width=1.2, marker_end=arrow_amber))

    # Connect R0 sense tap to G0->1
    d.append(draw.Line(r0_nodes[0][0], r0_nodes[0][1] + 12, g01_x - 20, g01_y - 18, stroke=SECONDARY_GREEN, stroke_width=1.0, stroke_dasharray="2,2", marker_end=arrow_fwd))

    # Connect R1 sense tap to G1->0
    d.append(draw.Line(r1_nodes[0][0], r1_nodes[0][1] + 12, g10_x + 20, g10_y - 18, stroke=TERTIARY_BLUE, stroke_width=1.0, stroke_dasharray="2,2", marker_end=arrow_blue))

    # G0->1 connects to R1,0 (Set R1) and inh0 (Reset R0)
    d.append(draw.Line(g01_x + 30, g01_y - 10, r1_nodes[0][0] - 12, r1_nodes[0][1] + 15, stroke=TERTIARY_BLUE, stroke_width=1.5, marker_end=arrow_blue))
    d.append(draw.Line(g01_x - 15, g01_y + 18, inh0_x, inh0_y - 14, stroke=PRIMARY_RED, stroke_width=1.2, marker_end=arrow_inh))

    # G1->0 connects to R0,0 (Set R0) and inh1 (Reset R1)
    d.append(draw.Line(g10_x - 30, g10_y - 10, r0_nodes[2][0] + 12, r0_nodes[2][1] + 15, stroke=SECONDARY_GREEN, stroke_width=1.5, marker_end=arrow_fwd))
    d.append(draw.Line(g10_x + 15, g10_y + 18, inh1_x, inh1_y - 14, stroke=PRIMARY_RED, stroke_width=1.2, marker_end=arrow_inh))

    # inh0 hyperpolarizes R0 nodes
    d.append(draw.Line(inh0_x, inh0_y - 14, r0_cx, r0_cy + 42, stroke=PRIMARY_RED, stroke_width=1.5, stroke_dasharray="3,3", marker_end=arrow_inh))
    d.append(draw.Text("W-inh = -1.50", 8, inh0_x - 42, inh0_y - 30, fill=PRIMARY_RED, font_family="monospace"))

    # inh1 hyperpolarizes R1 nodes
    d.append(draw.Line(inh1_x, inh1_y - 14, r1_cx, r1_cy + 42, stroke=PRIMARY_RED, stroke_width=1.5, stroke_dasharray="3,3", marker_end=arrow_inh))
    d.append(draw.Text("W-inh = -1.50", 8, inh1_x + 10, inh1_y - 30, fill=PRIMARY_RED, font_family="monospace"))

    # Sense & Accept Readout at bottom
    out_y = panel_y + 510
    d.append(draw.Rectangle(lp_x + 270, out_y - 15, 180, 30, rx=5, ry=5, fill="#1d2c44", stroke=TERTIARY_BLUE, stroke_width=1.2))
    d.append(draw.Text("Accept Port v-accept (State q1)", 9, lp_x + 360, out_y + 4, text_anchor="middle", fill="#e4f0fc", font_family="sans-serif", font_weight="bold"))
    d.append(draw.Line(r1_nodes[2][0] + 12, r1_nodes[2][1], lp_x + 440, out_y - 15, stroke=TERTIARY_BLUE, stroke_width=1.2, stroke_dasharray="2,2", marker_end=arrow_blue))

    # =========================================================================
    # RIGHT PANEL: Attractor Basin Phase Space & Transition Manifold
    # =========================================================================
    rp_x = 535
    rp_w = 470
    d.append(draw.Rectangle(rp_x, panel_y, rp_w, panel_h, rx=8, ry=8, fill=DARK_PANEL, stroke=BORDER, stroke_width=1.0))
    d.append(draw.Text("Dynamical Phase Space & Transition Manifold", 15, rp_x + 16, panel_y + 26, fill=TEXT, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("Mutually Exclusive Attractor Basins B0 and B1 (Period P = 4)", 11, rp_x + 16, panel_y + 44, fill=TEXT_MUTED, font_family="sans-serif"))

    # Attractor Basin B0
    b0_cx, b0_cy = rp_x + 120, panel_y + 140
    d.append(draw.Circle(b0_cx, b0_cy, 68, fill="#16231d", stroke=SECONDARY_GREEN, stroke_width=1.5, stroke_dasharray="4,4"))
    d.append(draw.Text("Basin B0 (State q-even)", 11, b0_cx, b0_cy - 48, text_anchor="middle", fill=SECONDARY_GREEN, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Circle(b0_cx, b0_cy, 26, fill="none", stroke=SECONDARY_GREEN, stroke_width=2.0))
    d.append(draw.Text("Orbit A0", 9, b0_cx, b0_cy + 3, text_anchor="middle", fill=TEXT, font_family="monospace"))
    d.append(draw.Text("P = 4 ticks", 8, b0_cx, b0_cy + 15, text_anchor="middle", fill=TEXT_MUTED, font_family="monospace"))

    # Attractor Basin B1
    b1_cx, b1_cy = rp_x + 350, panel_y + 140
    d.append(draw.Circle(b1_cx, b1_cy, 68, fill="#192338", stroke=TERTIARY_BLUE, stroke_width=1.5, stroke_dasharray="4,4"))
    d.append(draw.Text("Basin B1 (State q-odd)", 11, b1_cx, b1_cy - 48, text_anchor="middle", fill=TERTIARY_BLUE, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Circle(b1_cx, b1_cy, 26, fill="none", stroke=TERTIARY_BLUE, stroke_width=2.0))
    d.append(draw.Text("Orbit A1", 9, b1_cx, b1_cy + 3, text_anchor="middle", fill=TEXT, font_family="monospace"))
    d.append(draw.Text("P = 4 ticks", 8, b1_cx, b1_cy + 15, text_anchor="middle", fill=TEXT_MUTED, font_family="monospace"))

    # Transition Trajectories between Basins
    # B0 -> B1
    d.append(draw.Arc(rp_x + 235, panel_y + 110, 115, 200, 340, fill="none", stroke=AMBER, stroke_width=2.0, marker_end=arrow_amber))
    d.append(draw.Text("σ = 1: Switch B0 → B1", 9, rp_x + 235, panel_y + 82, text_anchor="middle", fill=AMBER, font_family="monospace", font_weight="bold"))

    # B1 -> B0
    d.append(draw.Arc(rp_x + 235, panel_y + 170, 115, 20, 160, fill="none", stroke=AMBER, stroke_width=2.0, marker_end=arrow_amber))
    d.append(draw.Text("σ = 1: Switch B1 → B0", 9, rp_x + 235, panel_y + 205, text_anchor="middle", fill=AMBER, font_family="monospace", font_weight="bold"))

    # Self-Loops for σ = 0
    d.append(draw.Arc(b0_cx - 45, b0_cy + 25, 22, 90, 270, fill="none", stroke=SECONDARY_GREEN, stroke_width=1.5, marker_end=arrow_fwd))
    d.append(draw.Text("σ = 0 (Hold)", 8, b0_cx - 75, b0_cy + 52, text_anchor="middle", fill=SECONDARY_GREEN, font_family="monospace"))

    d.append(draw.Arc(b1_cx + 45, b1_cy + 25, 22, 270, 90, fill="none", stroke=TERTIARY_BLUE, stroke_width=1.5, marker_end=arrow_blue))
    d.append(draw.Text("σ = 0 (Hold)", 8, b1_cx + 75, b1_cy + 52, text_anchor="middle", fill=TERTIARY_BLUE, font_family="monospace"))

    # Substrate Invariants & Guardrails Card
    bullets = [
        ("Mutual Exclusivity Invariant", "Σ 𝕀(Ring-j active) ≡ 1 at all settled ticks (zero crosstalk)."),
        ("Causal Transition Settling", "Inter-token window T-token ≥ τ-trans + P-ring = 4 + 4 = 8 ticks."),
        ("Refractory Diode Shielding", "N-ref = 2 prevents retrograde reflections & standing waves."),
        ("Instantaneous Quenching", "W-inh = -1.50 guarantees single-tick phase-invariant reset."),
        ("Thermal Noise Immunity", "Dynamic adaptation (β-θ = 0.05, θ ≥ 1.02) suppresses percolation."),
    ]
    card_y = panel_y + 245
    draw_card_with_bullets(d, rp_x + 20, card_y, rp_w - 40, 305, bullets, title="DFA Substrate Invariants & Boundary Constraints", max_chars=46)

    save_figure(d, output_path)
    print(f"Generated FIG-HYP-009 DFA Attractor Figure at: {output_path}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate FIG-HYP-009 DFA Attractor Figure")
    parser.add_argument(
        "--output",
        "-o",
        type=str,
        default="docs/research/assets/fig-hyp-009-dfa-attractor.svg",
        help="Output file path (SVG)",
    )
    args = parser.parse_args()
    build_dfa_attractor_figure(args.output)


if __name__ == "__main__":
    main()
