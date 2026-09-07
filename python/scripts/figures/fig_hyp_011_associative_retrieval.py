#!/usr/bin/env python3
"""Figure Generator: FIG-HYP-011 - Associative Key-Value Retrieval & Dynamic Addressing Substrate.

Target Document: docs/research/hypotheses/HYP-2026-011.md
Output: docs/research/assets/fig-hyp-011-associative-retrieval.svg

Visualizes the associative key-value memory architecture in the excitable MVA substrate:
Left Panel: Substrate Architecture: 16 Addressable Memory Slots (M_i),
            Dual Resonant Rings (R_i,0, R_i,1), Coincidence Write Gating (G_write),
            Distractor Attenuation Sink, and Dual-Rail Interrogation Readout Trees
            under strict bounded degree (k_in, k_out <= 4).
Right Panel: Associative Retrieval Dynamics: "Needle in a Haystack" sequence trajectory,
             demonstrating key-value binding, distractor immunity across long context horizons
             (L in [64, 512]), and query-triggered non-destructive readout Q(K_j) -> V_j.
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
    save_figure,
)


def build_associative_retrieval_figure(output_path: str) -> None:
    width = 1140
    height = 720

    d = draw.Drawing(width, height)
    # Canvas background
    d.append(draw.Rectangle(0, 0, width, height, fill=DARK_CANVAS))

    # Master Title & Subtitle
    d.append(
        draw.Text(
            "Associative Key-Value Variable Binding & Dynamic Addressing in Excitable CA",
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
            "Milestone 3.2: 16-Slot Resonant Attractor Array, Write Coincidence Gating, Distractor Immunity, and Dual-Rail Query Readout",
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
    panel_h = 625

    # =========================================================================
    # LEFT PANEL: Substrate Circuit Architecture
    # =========================================================================
    lp_x = 25
    lp_w = 530
    d.append(draw.Rectangle(lp_x, panel_y, lp_w, panel_h, rx=8, ry=8, fill=DARK_PANEL, stroke=BORDER, stroke_width=1.0))
    d.append(draw.Text("Substrate Circuit: 16-Slot Resonant Array", 15, lp_x + 16, panel_y + 26, fill=TEXT, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("Modular Addressable Slots Mⱼ & Coincidence Gating (degree ≤ 4)", 11, lp_x + 16, panel_y + 44, fill=TEXT_MUTED, font_family="sans-serif"))

    # Ingress Demux Header Box
    ing_y = panel_y + 60
    ing_w = 490
    d.append(draw.Rectangle(lp_x + 20, ing_y, ing_w, 65, rx=6, ry=6, fill="#13161f", stroke=BORDER, stroke_width=1.0))
    d.append(draw.Text("Streaming Ingress Demultiplexing Ports (Cadence T = 16 ticks)", 10, lp_x + 30, ing_y + 18, fill=TEXT, font_family="sans-serif", font_weight="bold"))

    ports = [
        ("Keys K₀..K₁₅", lp_x + 80, ing_y + 42, TERTIARY_BLUE),
        ("Values V₀, V₁", lp_x + 205, ing_y + 42, SECONDARY_GREEN),
        ("Distractors D", lp_x + 325, ing_y + 42, AMBER),
        ("Query Q(Kⱼ)", lp_x + 440, ing_y + 42, PRIMARY_RED),
    ]
    for lbl, px, py, col in ports:
        d.append(draw.Rectangle(px - 50, py - 12, 100, 24, rx=4, ry=4, fill="#1c2130", stroke=col, stroke_width=1.2))
        d.append(draw.Text(lbl, 8.5, px, py + 3, text_anchor="middle", fill=col, font_family="monospace", font_weight="bold"))

    # Addressable Memory Slots Representation (Slot M_j Detail)
    slot_y = panel_y + 140
    slot_w = ing_w
    slot_h = 240
    d.append(draw.Rectangle(lp_x + 20, slot_y, slot_w, slot_h, rx=6, ry=6, fill="#13161f", stroke=TERTIARY_BLUE, stroke_width=1.0, stroke_dasharray="4,3"))
    d.append(draw.Text("Addressable Memory Slot Mⱼ (Slot j ∈ {0, ..., 15})", 11, lp_x + 30, slot_y + 20, fill=TERTIARY_BLUE, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("Dual 4-Node Resonant Rings with Cross-Inhibitory Hyperpolarization", 9.5, lp_x + 30, slot_y + 36, fill=TEXT_MUTED, font_family="sans-serif"))

    # Ring 0 (Value = 0)
    r0_cx = lp_x + 140
    r0_cy = slot_y + 120
    d.append(draw.Rectangle(r0_cx - 100, r0_cy - 65, 200, 130, rx=6, ry=6, fill="#1a1e2b", stroke=SECONDARY_GREEN, stroke_width=1.0))
    d.append(draw.Text("Ring Rⱼ,₀ [Stores Bit 0]", 10, r0_cx, r0_cy - 48, text_anchor="middle", fill=SECONDARY_GREEN, font_family="sans-serif", font_weight="bold"))

    # Ring 0 4-nodes in square
    n_offset = 26
    r0_nodes = [
        (r0_cx - n_offset, r0_cy - 16, "r00"),
        (r0_cx + n_offset, r0_cy - 16, "r01"),
        (r0_cx + n_offset, r0_cy + 32, "r02"),
        (r0_cx - n_offset, r0_cy + 32, "r03"),
    ]
    for nx, ny, nid in r0_nodes:
        d.append(draw.Circle(nx, ny, 11, fill="#252b3d", stroke=SECONDARY_GREEN, stroke_width=1.2))
    # Directed circular arrows
    d.append(draw.Line(r0_cx - n_offset + 11, r0_cy - 16, r0_cx + n_offset - 11, r0_cy - 16, stroke=SECONDARY_GREEN, stroke_width=1.2, marker_end=arrow_fwd))
    d.append(draw.Line(r0_cx + n_offset, r0_cy - 5, r0_cx + n_offset, r0_cy + 21, stroke=SECONDARY_GREEN, stroke_width=1.2, marker_end=arrow_fwd))
    d.append(draw.Line(r0_cx + n_offset - 11, r0_cy + 32, r0_cx - n_offset + 11, r0_cy + 32, stroke=SECONDARY_GREEN, stroke_width=1.2, marker_end=arrow_fwd))
    d.append(draw.Line(r0_cx - n_offset, r0_cy + 21, r0_cx - n_offset, r0_cy - 5, stroke=SECONDARY_GREEN, stroke_width=1.2, marker_end=arrow_fwd))
    d.append(draw.Text("Period = 4 ticks, W_fb = 1.0", 8, r0_cx, r0_cy + 55, text_anchor="middle", fill=TEXT_MUTED, font_family="monospace"))

    # Ring 1 (Value = 1)
    r1_cx = lp_x + 375
    r1_cy = slot_y + 120
    d.append(draw.Rectangle(r1_cx - 100, r1_cy - 65, 200, 130, rx=6, ry=6, fill="#1a1e2b", stroke=TERTIARY_BLUE, stroke_width=1.0))
    d.append(draw.Text("Ring Rⱼ,₁ [Stores Bit 1]", 10, r1_cx, r1_cy - 48, text_anchor="middle", fill=TERTIARY_BLUE, font_family="sans-serif", font_weight="bold"))

    r1_nodes = [
        (r1_cx - n_offset, r1_cy - 16, "r10"),
        (r1_cx + n_offset, r1_cy - 16, "r11"),
        (r1_cx + n_offset, r1_cy + 32, "r12"),
        (r1_cx - n_offset, r1_cy + 32, "r13"),
    ]
    for nx, ny, nid in r1_nodes:
        d.append(draw.Circle(nx, ny, 11, fill="#252b3d", stroke=TERTIARY_BLUE, stroke_width=1.2))
    # Directed circular arrows
    d.append(draw.Line(r1_cx - n_offset + 11, r1_cy - 16, r1_cx + n_offset - 11, r1_cy - 16, stroke=TERTIARY_BLUE, stroke_width=1.2, marker_end=arrow_blue))
    d.append(draw.Line(r1_cx + n_offset, r1_cy - 5, r1_cx + n_offset, r1_cy + 21, stroke=TERTIARY_BLUE, stroke_width=1.2, marker_end=arrow_blue))
    d.append(draw.Line(r1_cx + n_offset - 11, r1_cy + 32, r1_cx - n_offset + 11, r1_cy + 32, stroke=TERTIARY_BLUE, stroke_width=1.2, marker_end=arrow_blue))
    d.append(draw.Line(r1_cx - n_offset, r1_cy + 21, r1_cx - n_offset, r1_cy - 5, stroke=TERTIARY_BLUE, stroke_width=1.2, marker_end=arrow_blue))
    d.append(draw.Text("Period = 4 ticks, W_fb = 1.0", 8, r1_cx, r1_cy + 55, text_anchor="middle", fill=TEXT_MUTED, font_family="monospace"))

    # Mutual cross-inhibition arrows
    d.append(draw.Line(r0_cx + 100, r0_cy - 10, r1_cx - 100, r0_cy - 10, stroke=PRIMARY_RED, stroke_width=1.5, marker_end=arrow_inh))
    d.append(draw.Line(r1_cx - 100, r0_cy + 10, r0_cx + 100, r0_cy + 10, stroke=PRIMARY_RED, stroke_width=1.5, marker_end=arrow_inh))
    d.append(draw.Text("W_inh = -1.50 (Quench)", 8, (r0_cx + r1_cx) / 2, r0_cy - 16, text_anchor="middle", fill=PRIMARY_RED, font_family="sans-serif", font_weight="bold"))

    # Coincidence Gating & Control Subsystem
    cg_y = panel_y + 395
    cg_h = 135
    d.append(draw.Rectangle(lp_x + 20, cg_y, ing_w, cg_h, rx=6, ry=6, fill="#13161f", stroke=BORDER, stroke_width=1.0))
    d.append(draw.Text("Write Coincidence & Interrogation Gating Subsystem", 10.5, lp_x + 30, cg_y + 18, fill=TEXT, font_family="sans-serif", font_weight="bold"))

    # Write Gate Box
    wg_x = lp_x + 140
    wg_y = cg_y + 68
    d.append(draw.Rectangle(wg_x - 105, wg_y - 36, 210, 72, rx=5, ry=5, fill="#1c2230", stroke=SECONDARY_GREEN, stroke_width=1.2))
    d.append(draw.Text("Write Coincidence Gate G_write,j", 9.5, wg_x, wg_y - 20, text_anchor="middle", fill=SECONDARY_GREEN, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("Inputs: Key Kⱼ (0.60) ∧ Value V (0.60)", 8.5, wg_x, wg_y - 6, text_anchor="middle", fill=TEXT, font_family="monospace"))
    d.append(draw.Text("Sum = 1.20 >= θ_write = 1.10", 8.5, wg_x, wg_y + 8, text_anchor="middle", fill=TEXT_MUTED, font_family="monospace"))
    d.append(draw.Text("Ignites target ring & quenches opposite", 8, wg_x, wg_y + 22, text_anchor="middle", fill=SECONDARY_GREEN, font_family="sans-serif"))

    # Query Gate Box
    qg_x = lp_x + 375
    qg_y = cg_y + 68
    d.append(draw.Rectangle(qg_x - 105, qg_y - 36, 210, 72, rx=5, ry=5, fill="#1c2230", stroke=TERTIARY_BLUE, stroke_width=1.2))
    d.append(draw.Text("Interrogation Gate G_query,j", 9.5, qg_x, qg_y - 20, text_anchor="middle", fill=TERTIARY_BLUE, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("Inputs: Query Q(Kⱼ) (0.60) ∧ Sense (0.60)", 8.5, qg_x, qg_y - 6, text_anchor="middle", fill=TEXT, font_family="monospace"))
    d.append(draw.Text("Sum = 1.20 >= θ_query = 1.10", 8.5, qg_x, qg_y + 8, text_anchor="middle", fill=TEXT_MUTED, font_family="monospace"))
    d.append(draw.Text("Gates circulating pulse to readout bus", 8, qg_x, qg_y + 22, text_anchor="middle", fill=TERTIARY_BLUE, font_family="sans-serif"))

    # Bottom Readout & Sink Bar
    br_y = panel_y + 542
    br_h = 70
    d.append(draw.Rectangle(lp_x + 20, br_y, ing_w, br_h, rx=6, ry=6, fill="#13161f", stroke=BORDER, stroke_width=1.0))

    # Dual-rail Readout Ports
    d.append(draw.Rectangle(lp_x + 35, br_y + 14, 205, 42, rx=4, ry=4, fill="#1a2e22", stroke=SECONDARY_GREEN, stroke_width=1.2))
    d.append(draw.Text("Dual-Rail Readout Ports", 9, lp_x + 137, br_y + 28, text_anchor="middle", fill=SECONDARY_GREEN, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("v_out,0 (Bit 0)  |  v_out,1 (Bit 1)", 8.5, lp_x + 137, br_y + 45, text_anchor="middle", fill=TEXT, font_family="monospace"))

    # Distractor Sink Box
    d.append(draw.Rectangle(lp_x + 255, br_y + 14, 240, 42, rx=4, ry=4, fill="#2e271a", stroke=AMBER, stroke_width=1.2))
    d.append(draw.Text("Distractor Dissipation Sink Line", 9, lp_x + 375, br_y + 28, text_anchor="middle", fill=AMBER, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("W_dist <= 0.30 << θ, λ = 0.05 (Subthreshold)", 8.5, lp_x + 375, br_y + 45, text_anchor="middle", fill=TEXT_MUTED, font_family="monospace"))

    # =========================================================================
    # RIGHT PANEL: Associative Retrieval Dynamics ("Needle in a Haystack")
    # =========================================================================
    rp_x = 580
    rp_w = 530
    d.append(draw.Rectangle(rp_x, panel_y, rp_w, panel_h, rx=8, ry=8, fill=DARK_PANEL, stroke=BORDER, stroke_width=1.0))
    d.append(draw.Text("Associative Retrieval Dynamics ('Needle in a Haystack')", 15, rp_x + 16, panel_y + 26, fill=TEXT, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("Temporal Stream Execution: 16 Pairs, Filler Distractors, and Query", 11, rp_x + 16, panel_y + 44, fill=TEXT_MUTED, font_family="sans-serif"))

    # Timeline Visualization
    tl_y = panel_y + 60
    tl_w = 490
    tl_h = 240
    d.append(draw.Rectangle(rp_x + 20, tl_y, tl_w, tl_h, rx=6, ry=6, fill="#13161f", stroke=BORDER, stroke_width=1.0))
    d.append(draw.Text("Temporal Stream Timeline (Sequence Horizon L in [64, 512] tokens)", 10.5, rp_x + 30, tl_y + 20, fill=TEXT, font_family="sans-serif", font_weight="bold"))

    # Draw timeline tokens as colored blocks
    tokens_meta = [
        ("K₃→1", TERTIARY_BLUE),
        ("D₁", AMBER),
        ("D₂", AMBER),
        ("K₇→0", SECONDARY_GREEN),
        ("D₃", AMBER),
        ("···", TEXT_MUTED),
        ("K₁₂→1", TERTIARY_BLUE),
        ("D₄", AMBER),
        ("···", TEXT_MUTED),
        ("Q(K₇)", PRIMARY_RED),
    ]

    bx_start = rp_x + 30
    bw = 43
    gap = 5.0
    for i, (tok, col) in enumerate(tokens_meta):
        bx = bx_start + i * (bw + gap)
        by = tl_y + 42
        d.append(draw.Rectangle(bx, by, bw, 28, rx=4, ry=4, fill="#1e2333", stroke=col, stroke_width=1.2))
        d.append(draw.Text(tok, 8, bx + bw / 2, by + 18, text_anchor="middle", fill=col, font_family="monospace", font_weight="bold"))

    # Callout for Target Needle: K_7 -> 0
    needle_bx = bx_start + 3 * (bw + gap) + bw / 2
    d.append(draw.Line(needle_bx, tl_y + 70, needle_bx, tl_y + 115, stroke=SECONDARY_GREEN, stroke_width=1.5, stroke_dasharray="2,2"))
    d.append(draw.Rectangle(needle_bx - 70, tl_y + 115, 140, 48, rx=5, ry=5, fill="#14261c", stroke=SECONDARY_GREEN, stroke_width=1.0))
    d.append(draw.Text("Target 'Needle' Bound", 9, needle_bx, tl_y + 130, text_anchor="middle", fill=SECONDARY_GREEN, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("Slot M₇ set to Bit 0", 8, needle_bx, tl_y + 144, text_anchor="middle", fill=TEXT, font_family="monospace"))
    d.append(draw.Text("Ring R₇,₀ circulating", 8, needle_bx, tl_y + 156, text_anchor="middle", fill=TEXT_MUTED, font_family="sans-serif"))

    # Retention Arrow across Haystack
    haystack_x1 = needle_bx + 75
    haystack_x2 = bx_start + 9 * (bw + gap) - 10
    d.append(draw.Line(haystack_x1, tl_y + 140, haystack_x2, tl_y + 140, stroke=AMBER, stroke_width=1.5, marker_end=arrow_amber))
    d.append(draw.Text("Distractor Horizon (L >= 256 ticks)", 8.5, (haystack_x1 + haystack_x2) / 2, tl_y + 132, text_anchor="middle", fill=AMBER, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("Indefinite Retention (No Decay, W_fb=1.0)", 8, (haystack_x1 + haystack_x2) / 2, tl_y + 152, text_anchor="middle", fill=TEXT_MUTED, font_family="monospace"))

    # Query Callout
    query_bx = bx_start + 9 * (bw + gap) + bw / 2
    d.append(draw.Line(query_bx, tl_y + 70, query_bx, tl_y + 175, stroke=PRIMARY_RED, stroke_width=1.5, stroke_dasharray="2,2"))
    d.append(draw.Rectangle(query_bx - 120, tl_y + 175, 140, 48, rx=5, ry=5, fill="#2e1a1e", stroke=PRIMARY_RED, stroke_width=1.0))
    d.append(draw.Text("Query Probe Q(K₇)", 9, query_bx - 50, tl_y + 190, text_anchor="middle", fill=PRIMARY_RED, font_family="sans-serif", font_weight="bold"))
    d.append(draw.Text("Triggers G_query,7", 8, query_bx - 50, tl_y + 204, text_anchor="middle", fill=TEXT, font_family="monospace"))
    d.append(draw.Text("v_out,0 = 1 (Match)", 8, query_bx - 50, tl_y + 216, text_anchor="middle", fill=SECONDARY_GREEN, font_family="sans-serif", font_weight="bold"))

    # Phase Space / State Evolution Trajectory Box
    traj_y = panel_y + 312
    traj_w = tl_w
    traj_h = 165
    d.append(draw.Rectangle(rp_x + 20, traj_y, traj_w, traj_h, rx=6, ry=6, fill="#13161f", stroke=BORDER, stroke_width=1.0))
    d.append(draw.Text("Multi-Slot Dynamical Phase Space & Attractor Basin Orthogonality", 10.5, rp_x + 30, traj_y + 20, fill=TEXT, font_family="sans-serif", font_weight="bold"))

    # Attractor Basins Illustration
    basins = [
        ("Basin M₀", rp_x + 85, traj_y + 80, SECONDARY_GREEN, "Bit 0 Active"),
        ("Basin M₃", rp_x + 190, traj_y + 80, TERTIARY_BLUE, "Bit 1 Active"),
        ("Basin M₇", rp_x + 295, traj_y + 80, SECONDARY_GREEN, "Bit 0 (Target)"),
        ("Basin M₁₅", rp_x + 405, traj_y + 80, TERTIARY_BLUE, "Bit 1 Active"),
    ]
    for bname, bx, by, col, bdesc in basins:
        d.append(draw.Circle(bx, by, 30, fill="#1c2130", stroke=col, stroke_width=1.5))
        d.append(draw.Circle(bx, by, 12, fill=col, stroke=TEXT, stroke_width=1.0, opacity=0.8))
        d.append(draw.Text(bname, 9, bx, by - 36, text_anchor="middle", fill=col, font_family="sans-serif", font_weight="bold"))
        d.append(draw.Text(bdesc, 7.5, bx, by + 46, text_anchor="middle", fill=TEXT_MUTED, font_family="monospace"))

    d.append(draw.Text("16 Mutually Orthogonal Limit Cycles: Zero Cross-Slot Attenuation or Interference", 9, rp_x + 250, traj_y + 150, text_anchor="middle", fill=TEXT, font_family="sans-serif"))

    # Pre-Registered 6 Falsification Gates Summary Card
    card_y = panel_y + 490
    card_w = tl_w
    card_h = 122
    d.append(draw.Rectangle(rp_x + 20, card_y, card_w, card_h, rx=6, ry=6, fill="#181c28", stroke=SECONDARY_GREEN, stroke_width=1.2))
    d.append(draw.Text("Pre-Registered Empirical Falsification Gates (HYP-2026-011 / EXP-2026-011a)", 10.5, rp_x + 30, card_y + 18, fill=SECONDARY_GREEN, font_family="sans-serif", font_weight="bold"))

    gates = [
        ("Gate 1: Retrieval Accuracy (N=16, ε=0.00)", ">= 0.990 (Predicted 1.000)", SECONDARY_GREEN),
        ("Gate 2: Long Context Horizon (L >= 256)", ">= 0.990 (Indefinite Retention)", SECONDARY_GREEN),
        ("Gate 3: Distractor Immunity (χ_distractor)", "<= 0.010 (Subthreshold Passivity)", AMBER),
        ("Gate 4: Noise Resilience (ε=0.05)", "Accuracy >= 0.900, BER <= 0.050", TERTIARY_BLUE),
        ("Gate 5: Associative Advantage vs Unindexed", "Welch's p < 10^-6, Cohen's d >= 2.0", SECONDARY_GREEN),
        ("Gate 6: Homeostatic Density Envelope", "ρ_bar in [0.01, 0.25] in >= 99% runs", TEXT),
    ]

    col1_x = rp_x + 30
    col2_x = rp_x + 270
    for idx, (g_name, g_val, col) in enumerate(gates):
        gx = col1_x if idx % 2 == 0 else col2_x
        gy = card_y + 38 + (idx // 2) * 26
        d.append(draw.Circle(gx + 5, gy + 3, 3.5, fill=col))
        d.append(draw.Text(g_name, 8, gx + 14, gy + 7, fill=TEXT, font_family="sans-serif", font_weight="bold"))
        d.append(draw.Text(g_val, 7.5, gx + 14, gy + 18, fill=col, font_family="monospace"))

    save_figure(d, output_path)


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate Figure FIG-HYP-011.")
    parser.add_argument(
        "--output",
        "-o",
        type=str,
        default="docs/research/assets/fig-hyp-011-associative-retrieval.svg",
        help="Path for SVG output file.",
    )
    args = parser.parse_args()
    build_associative_retrieval_figure(args.output)
    print(f"Generated {args.output}")


if __name__ == "__main__":
    main()
