#!/usr/bin/env python3
"""Figure Generator: FIG-HYP-010 - Cascaded Dynamical Stack Substrate for Pushdown Automata & Dyck Languages.

Target Document: docs/research/hypotheses/HYP-2026-010.md
Output: docs/research/assets/fig-hyp-010-pushdown-memory.svg

Visualizes the modular pushdown memory architecture in the excitable MVA substrate:
Left Panel: Modular Substrate Circuit: Cascaded Stack Frames (C_k) and Pointer Ladder (P_k),
            depicting dual resonant rings for '(' and '[', bidirectional pointer steering gates,
            and cross-inhibitory quenching interneurons with bounded degree (k_in, k_out <= 4).
Right Panel: Dynamical Phase Space & LIFO Execution Trajectory,
             contrasting valid nested reduction '[' '(' ')' ']' -> A_accept with cross-bracket
             violation '[' '(' ']' ')' -> A_reject via top-of-stack mismatch detection.
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
    save_figure,
)


def build_pushdown_memory_figure(output_path: str) -> None:
    width = 1060
    height = 680

    d = draw.Drawing(width, height)
    # Canvas background
    d.append(draw.Rectangle(0, 0, width, height, fill=DARK_CANVAS))

    # Master Title & Subtitle
    d.append(
        draw.Text(
            "Cascaded Dynamical Stack Substrate for Pushdown Automata & Dyck Languages",
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
            "Substrate Dynamics: Modular Resonant Latch Frames, Bidirectional Pointer Steering, and Coincidence Gating (k_in, k_out <= 4)",
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
    panel_h = 585

    # =========================================================================
    # LEFT PANEL: Modular Cascaded Stack Circuit Architecture
    # =========================================================================
    lp_x = 30
    lp_w = 485
    d.append(draw.Rectangle(lp_x, panel_y, lp_w, panel_h, rx=8, ry=8, fill=DARK_PANEL, stroke=BORDER, stroke_width=1.0))
    d.append(draw.Text("Substrate Circuit: Modular Pushdown Ladder", 15, lp_x + 16, panel_y + 26, fill=TEXT, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("Cascaded Latch Units C_k & Bidirectional Pointer P_k", 11, lp_x + 16, panel_y + 44, fill=TEXT_MUTED, font_family="sans-serif"))

    # Stack Level Frames: Level 0 (bottom) and Level 1 (top)
    frame_w = 445
    frame_h = 100

    # Pointer Column x = lp_x + 30
    # Data Frame Column x = lp_x + 165
    ptr_cx = lp_x + 65
    data_round_cx = lp_x + 225
    data_sqr_cx = lp_x + 365

    # Level 1 Frame (Top)
    l1_y = panel_y + 65
    d.append(draw.Rectangle(lp_x + 15, l1_y, frame_w, frame_h, rx=6, ry=6, fill="#13161f", stroke=TERTIARY_BLUE, stroke_width=1.0, stroke_dasharray="3,3"))
    d.append(draw.Text("Stack Frame k = 1 (Depth 1 Storage)", 10, lp_x + 24, l1_y + 16, fill=TERTIARY_BLUE, font_family="sans-serif", font_weight="bold"))

    # Level 0 Frame (Bottom)
    l0_y = panel_y + 180
    d.append(draw.Rectangle(lp_x + 15, l0_y, frame_w, frame_h, rx=6, ry=6, fill="#13161f", stroke=SECONDARY_GREEN, stroke_width=1.0, stroke_dasharray="3,3"))
    d.append(draw.Text("Stack Frame k = 0 (Depth 0 Storage)", 10, lp_x + 24, l0_y + 16, fill=SECONDARY_GREEN, font_family="sans-serif", font_weight="bold"))

    # Pointer Latch P0 (Empty Indicator / Level 0)
    p0_y = l0_y + 55
    d.append(draw.Circle(ptr_cx, p0_y, 16, fill=SECONDARY_GREEN_FILL, stroke=SECONDARY_GREEN, stroke_width=1.5))
    d.append(draw.Text("P_0", 10, ptr_cx, p0_y + 4, text_anchor="middle", fill=TEXT, font_family="monospace", font_weight="bold"))

    # Pointer Latch P1 (Level 1)
    p1_y = l1_y + 55
    d.append(draw.Circle(ptr_cx, p1_y, 16, fill="#222738", stroke=TERTIARY_BLUE, stroke_width=1.5))
    d.append(draw.Text("P_1", 10, ptr_cx, p1_y + 4, text_anchor="middle", fill=TEXT, font_family="monospace", font_weight="bold"))

    # Data Cell C0: Round Ring '(' & Square Ring '['
    d.append(draw.Circle(data_round_cx, p0_y, 15, fill="#222738", stroke=SECONDARY_GREEN, stroke_width=1.5))
    d.append(draw.Text("R0,(", 9, data_round_cx, p0_y + 3, text_anchor="middle", fill=SECONDARY_GREEN, font_family="monospace", font_weight="bold"))

    d.append(draw.Circle(data_sqr_cx, p0_y, 15, fill="#222738", stroke=SECONDARY_GREEN, stroke_width=1.5))
    d.append(draw.Text("R0,[", 9, data_sqr_cx, p0_y + 3, text_anchor="middle", fill=SECONDARY_GREEN, font_family="monospace", font_weight="bold"))

    # Data Cell C1: Round Ring '(' & Square Ring '['
    d.append(draw.Circle(data_round_cx, p1_y, 15, fill="#222738", stroke=TERTIARY_BLUE, stroke_width=1.5))
    d.append(draw.Text("R1,(", 9, data_round_cx, p1_y + 3, text_anchor="middle", fill=TERTIARY_BLUE, font_family="monospace", font_weight="bold"))

    d.append(draw.Circle(data_sqr_cx, p1_y, 15, fill="#222738", stroke=TERTIARY_BLUE, stroke_width=1.5))
    d.append(draw.Text("R1,[", 9, data_sqr_cx, p1_y + 3, text_anchor="middle", fill=TERTIARY_BLUE, font_family="monospace", font_weight="bold"))

    # Pointer Shift Edges between P0 and P1
    # PUSH moves P0 -> P1
    d.append(draw.Line(ptr_cx - 6, p0_y - 18, ptr_cx - 6, p1_y + 18, stroke=AMBER, stroke_width=1.5, marker_end=arrow_amber))
    d.append(draw.Text("PUSH", 8, ptr_cx - 20, (p0_y + p1_y) / 2 + 3, text_anchor="middle", fill=AMBER, font_family="sans-serif", font_weight="bold"))

    # POP moves P1 -> P0
    d.append(draw.Line(ptr_cx + 6, p1_y + 18, ptr_cx + 6, p0_y - 18, stroke=PRIMARY_RED, stroke_width=1.5, marker_end=arrow_inh))
    d.append(draw.Text("POP", 8, ptr_cx + 18, (p0_y + p1_y) / 2 + 3, text_anchor="middle", fill=PRIMARY_RED, font_family="sans-serif", font_weight="bold"))

    # Streaming Token Demux Ingress
    ing_y = panel_y + 310
    d.append(draw.Rectangle(lp_x + 15, ing_y, frame_w, 65, rx=6, ry=6, fill="#1c2130", stroke=BORDER, stroke_width=1.0))
    d.append(draw.Text("Token Ingress Demultiplexing Ports (Cadence T_token = 16 ticks)", 10, lp_x + 24, ing_y + 18, fill=TEXT, font_family="sans-serif", font_weight="bold"))

    token_ports = [
        ("v_( : PUSH '('", lp_x + 65, ing_y + 44, SECONDARY_GREEN),
        ("v_[ : PUSH '['", lp_x + 175, ing_y + 44, TERTIARY_BLUE),
        ("v_) : POP ')'", lp_x + 285, ing_y + 44, PRIMARY_RED),
        ("v_] : POP ']'", lp_x + 395, ing_y + 44, AMBER),
    ]
    for label, px, py, col in token_ports:
        d.append(draw.Rectangle(px - 45, py - 12, 90, 24, rx=4, ry=4, fill="#13161f", stroke=col, stroke_width=1.2))
        d.append(draw.Text(label, 8, px, py + 3, text_anchor="middle", fill=col, font_family="monospace", font_weight="bold"))

    # Gating & Verification Subsystem
    gate_y = panel_y + 400
    d.append(draw.Rectangle(lp_x + 15, gate_y, frame_w, 165, rx=6, ry=6, fill="#13161f", stroke=BORDER, stroke_width=1.0))
    d.append(draw.Text("Coincidence Gating & Violation Detection Subsystem", 10, lp_x + 24, gate_y + 18, fill=TEXT, font_family="sans-serif", font_weight="bold"))

    # PUSH Gate Box
    g_push_x = lp_x + 115
    g_push_y = gate_y + 60
    d.append(draw.Rectangle(g_push_x - 85, g_push_y - 24, 170, 48, rx=5, ry=5, fill="#1e2230", stroke=SECONDARY_GREEN, stroke_width=1.2))
    d.append(draw.Text("PUSH Gate G_push,k", 9, g_push_x, g_push_y - 8, text_anchor="middle", fill=SECONDARY_GREEN, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("P_k ∧ v_open (W=0.6, θ=1.10)", 8, g_push_x, g_push_y + 6, text_anchor="middle", fill=TEXT_MUTED, font_family="monospace"))
    d.append(draw.Text("Sets C_k & Advances P_k -> P_k+1", 8, g_push_x, g_push_y + 18, text_anchor="middle", fill=TEXT, font_family="sans-serif"))

    # POP Gate Box
    g_pop_x = lp_x + 335
    g_pop_y = gate_y + 60
    d.append(draw.Rectangle(g_pop_x - 85, g_pop_y - 24, 170, 48, rx=5, ry=5, fill="#1e2230", stroke=TERTIARY_BLUE, stroke_width=1.2))
    d.append(draw.Text("POP Match Gate G_pop,k", 9, g_pop_x, g_pop_y - 8, text_anchor="middle", fill=TERTIARY_BLUE, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("P_k ∧ C_k-1 ∧ v_close (3-Way θ=1.1)", 8, g_pop_x, g_pop_y + 6, text_anchor="middle", fill=TEXT_MUTED, font_family="monospace"))
    d.append(draw.Text("Quenches C_k-1 & Drops P_k -> P_k-1", 8, g_pop_x, g_pop_y + 18, text_anchor="middle", fill=TEXT, font_family="sans-serif"))

    # Trap & Acceptance Readouts
    d.append(draw.Rectangle(lp_x + 30, gate_y + 115, 195, 36, rx=4, ry=4, fill="#2e1a1e", stroke=PRIMARY_RED, stroke_width=1.2))
    d.append(draw.Text("Underflow / Mismatch Trap v_trap", 8, lp_x + 127, gate_y + 130, text_anchor="middle", fill=PRIMARY_RED, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("Absorbing Attractor A_reject", 8, lp_x + 127, gate_y + 143, text_anchor="middle", fill=TEXT_MUTED, font_family="monospace"))

    d.append(draw.Rectangle(lp_x + 250, gate_y + 115, 195, 36, rx=4, ry=4, fill="#1a2e22", stroke=SECONDARY_GREEN, stroke_width=1.2))
    d.append(draw.Text("Acceptance Readout v_accept", 8, lp_x + 347, gate_y + 130, text_anchor="middle", fill=SECONDARY_GREEN, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("Empty Stack (P_0) ∧ v_eos -> A_accept", 8, lp_x + 347, gate_y + 143, text_anchor="middle", fill=TEXT_MUTED, font_family="monospace"))

    # =========================================================================
    # RIGHT PANEL: Dynamical Phase Space & LIFO Trajectory
    # =========================================================================
    rp_x = 545
    rp_w = 485
    d.append(draw.Rectangle(rp_x, panel_y, rp_w, panel_h, rx=8, ry=8, fill=DARK_PANEL, stroke=BORDER, stroke_width=1.0))
    d.append(draw.Text("Dynamical Phase Space & LIFO Trajectory", 15, rp_x + 16, panel_y + 26, fill=TEXT, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("Trace of Stack Evolution for Valid vs Mismatched Streams", 11, rp_x + 16, panel_y + 44, fill=TEXT_MUTED, font_family="sans-serif"))

    # Execution Step Box
    exec_y = panel_y + 55
    exec_h = 220
    d.append(draw.Rectangle(rp_x + 15, exec_y, 455, exec_h, rx=6, ry=6, fill="#13161f", stroke=BORDER, stroke_width=1.0))
    d.append(draw.Text("Step-by-Step Dyck-2 LIFO Trace: Sequence '[' '(' ')' ']'", 10, rp_x + 25, exec_y + 18, fill=TEXT, font_family="sans-serif", font_weight="bold"))

    trace_steps = [
        ("t = 0", "Initial Ground", "Stack: [] (Empty)", "Pointer: P_0 active", SECONDARY_GREEN),
        ("t = 16", "Token 1: '['", "PUSH '[', C_0 holds '['", "P_0 -> P_1 active", TERTIARY_BLUE),
        ("t = 32", "Token 2: '('", "PUSH '(', C_1 holds '('", "P_1 -> P_2 active", TERTIARY_BLUE),
        ("t = 48", "Token 3: ')'", "POP ')': Matches C_1 '(', quench C_1", "P_2 -> P_1 active", SECONDARY_GREEN),
        ("t = 64", "Token 4: ']'", "POP ']': Matches C_0 '[', quench C_0", "P_1 -> P_0 active", SECONDARY_GREEN),
        ("t = 80", "EOS Probe", "P_0 active ∧ Zero Traps -> FIRE", "State: A_accept (1.000)", AMBER),
    ]

    for idx, (t_lbl, op_lbl, st_lbl, ptr_lbl, col) in enumerate(trace_steps):
        sy = exec_y + 36 + idx * 30
        d.append(draw.Text(t_lbl, 9, rp_x + 30, sy + 3, fill=TEXT_MUTED, font_family="monospace"))
        d.append(draw.Text(op_lbl, 9, rp_x + 95, sy + 3, fill=col, font_family="monospace", font_weight="bold"))
        d.append(draw.Text(st_lbl, 9, rp_x + 215, sy + 3, fill=TEXT, font_family="sans-serif"))
        d.append(draw.Text(ptr_lbl, 8, rp_x + 380, sy + 3, fill=TEXT_MUTED, font_family="monospace"))
        if idx < len(trace_steps) - 1:
            d.append(draw.Line(rp_x + 25, sy + 12, rp_x + 455, sy + 12, stroke=GRID, stroke_width=0.8, stroke_dasharray="2,2"))

    # Violation Path Demonstration: Cross-Bracket Mismatch
    viol_y = panel_y + 285
    viol_h = 100
    d.append(draw.Rectangle(rp_x + 15, viol_y, 455, viol_h, rx=6, ry=6, fill="#20151a", stroke=PRIMARY_RED, stroke_width=1.0))
    d.append(draw.Text("Cross-Bracket Violation Trap: Sequence '[' '(' ']' ')' (Non-LIFO)", 10, rp_x + 25, viol_y + 18, fill=PRIMARY_RED, font_family="sans-serif", font_weight="bold"))

    d.append(draw.Text("1. Tokens '[' then '(' arrive -> Stack: [ '[', '(' ] at Depth k=2 (Pointer P_2 active).", 9, rp_x + 25, viol_y + 38, fill=TEXT, font_family="sans-serif"))
    d.append(draw.Text("2. Token ']' arrives at t=48: Top of stack C_1 holds '(', but token is ']'.", 9, rp_x + 25, viol_y + 56, fill=TEXT, font_family="sans-serif"))
    d.append(draw.Text("3. Mismatch Gate fires: (P_2 ∧ C_1,( ∧ v_]) = (0.4 + 0.4 + 0.4 = 1.20 >= 1.10) -> v_trap.", 9, rp_x + 25, viol_y + 74, fill=PRIMARY_RED, font_family="monospace", font_weight="bold"))
    d.append(draw.Text("4. v_trap enters self-exciting loop A_reject, hyperpolarizing v_accept permanently.", 9, rp_x + 25, viol_y + 92, fill=TEXT_MUTED, font_family="sans-serif"))

    # Summary Bullet Cards: Core Invariants & Depth Scaling
    card_y = panel_y + 395
    card_w = 455
    card_h = 175

    bullets = [
        ("Bounded Degree:", "max(k_in, k_out) <= 4 via modular 1D tiling."),
        ("Modular Depth Scaling:", "Calibrated on D <= 4; scales to D in [1, 8] with zero retuning."),
        ("Refractory Diode Shielding:", "N_ref = 2 prevents standing waves and retrograde crosstalk."),
        ("Homeostatic Criticality:", "Dynamic threshold adaptation bounds rho in [0.01, 0.25]."),
    ]

    draw_card_with_bullets(
        d,
        rp_x + 15,
        card_y,
        card_w,
        188,
        bullets,
        title="Mathematical & Architectural Invariants",
        max_chars=65,
        bg_fill="#13161f",
        border_color=BORDER,
    )

    save_figure(d, output_path)


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate Pushdown Memory Substrate Architecture Figure")
    parser.add_argument(
        "--output",
        "-o",
        type=str,
        default=str(Path(__file__).resolve().parents[3] / "docs" / "research" / "assets" / "fig-hyp-010-pushdown-memory.svg"),
        help="Output file path",
    )
    args = parser.parse_args()
    build_pushdown_memory_figure(args.output)


if __name__ == "__main__":
    main()
