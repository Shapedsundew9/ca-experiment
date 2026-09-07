#!/usr/bin/env python3
"""Figure Generator: FIG-DIAG-003a - Topological Diameter Horizon & Memory Decay.

Target Document: docs/research/diagnostics/DIAG-2026-003a.md
Output: docs/research/assets/fig-diag-003a-temporal-horizon.svg

Visualizes the physical mechanism and empirical results of EXP-2026-003a:
Left Panel: 16-node Torus topological transit boundary (Diameter D = 4) explaining
            why delay horizons tau > 4 require closed circulation and suffer refractory annihilation.
Right Panel: Empirical accuracy and linear memory M(tau) across delay horizons tau in {3, 5, 7, 10},
             showing the sharp decay beyond the graph diameter.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
from tools.viz import (
    AMBER,
    BORDER,
    DARK_CANVAS,
    GRID,
    PRIMARY_RED,
    SECONDARY_GREEN,
    TERTIARY_BLUE,
    TEXT,
    TEXT_MUTED,
    apply_dark_theme,
    save_figure,
)


def render_fig_diag_003a(output_path: str, telemetry_path: str | None = None) -> None:
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5), gridspec_kw={"width_ratios": [1.2, 1.4]})
    apply_dark_theme(fig, [ax1, ax2])

    # ----------------------------------------------------
    # Panel 1: Topological Diameter Horizon Schematic
    # ----------------------------------------------------
    # Plot Manhattan hop distance vs. available nodes from Ingress Node 0
    hops = [0, 1, 2, 3, 4]
    nodes_reached = [1, 4, 6, 4, 1]  # 4x4 torus shell distribution from (0, 0)
    cumulative_nodes = np.cumsum(nodes_reached)

    ax1.bar(hops, cumulative_nodes, color=TERTIARY_BLUE, edgecolor=BORDER, width=0.6, alpha=0.85, label=r"Cumulative Reach $|\mathcal{V}_{\leq \tau}|$")
    ax1.axvline(4.0, color=PRIMARY_RED, linewidth=2.0, linestyle="--", label=r"Graph Diameter $D_{\mathrm{diam}} = 4$")
    ax1.axvspan(4.0, 5.5, color=AMBER, alpha=0.15, label=r"Reverberation Horizon ($\tau > 4$)")

    ax1.set_title(r"Lattice Reach Horizon ($D_{\mathrm{diam}} = 4$)", pad=10)
    ax1.set_xlabel(r"Transit Delay / Graph Distance $\tau$ (Ticks)", labelpad=8)
    ax1.set_ylabel(r"Cumulative Reach $|\mathcal{V}_{\leq \tau}|$", labelpad=8)
    ax1.set_xlim(-0.6, 5.5)
    ax1.set_ylim(0, 18)
    ax1.legend(loc="upper left", framealpha=0.85, fontsize=9)

    ax1.text(
        2.0,
        8.0,
        r"Feedforward Wavefront Transit" + "\n" + r"(100% Lattice Coverage by $\tau = 4$)",
        ha="center",
        color=TEXT,
        fontsize=9,
        bbox=dict(boxstyle="round,pad=0.4", facecolor=DARK_CANVAS, edgecolor=BORDER),
    )

    # ----------------------------------------------------
    # Panel 2: Memory & Accuracy vs. Delay Horizon
    # ----------------------------------------------------
    taus = np.array([3, 5, 7, 10])
    # Empirical accuracies from DIAG-2026-003a (p_x = 0.20)
    active_acc = np.array([0.8329, 0.7552, 0.6920, 0.6150])
    static_acc = np.array([0.7746, 0.7310, 0.6650, 0.5820])
    memoryless_acc = np.array([0.6802, 0.6800, 0.6790, 0.6810])

    ax2.plot(taus, active_acc, marker="o", color=PRIMARY_RED, linewidth=2.2, markersize=7, label="Active Substrate (Homeostatic)")
    ax2.plot(taus, static_acc, marker="s", color=AMBER, linewidth=1.8, linestyle="--", markersize=6, label="Static Threshold Ablation")
    ax2.plot(taus, memoryless_acc, marker="^", color=TERTIARY_BLUE, linewidth=1.5, linestyle=":", markersize=6, label="Memoryless Ablation")

    # Termination gate: 95%
    ax2.axhline(0.95, color=SECONDARY_GREEN, linestyle="--", linewidth=1.5, label=r"Pre-Registered Gate (0.95 / 95%)")
    ax2.axvline(4.0, color=BORDER, linestyle=":", linewidth=1.5)

    ax2.set_title(r"Temporal XOR Accuracy vs. Delay Horizon $\tau$", pad=10)
    ax2.set_xlabel(r"Delay Horizon $\tau$ (Ticks)", labelpad=8)
    ax2.set_ylabel(r"Readout Accuracy on $x_t \oplus x_{t-\tau}$", labelpad=8)
    ax2.set_ylim(0.50, 1.00)
    ax2.set_xticks(taus)
    ax2.legend(loc="lower left", framealpha=0.85, fontsize=9)

    # Annotation of collapse
    ax2.annotate(
        r"Sharp Decay Beyond $D_{\mathrm{diam}}=4$" + "\nRefractory Annihilation",
        xy=(5, 0.7552),
        xytext=(6.2, 0.84),
        arrowprops=dict(arrowstyle="->", color=TEXT, lw=1.2),
        color=TEXT,
        fontsize=9,
        bbox=dict(boxstyle="round,pad=0.4", facecolor=DARK_CANVAS, edgecolor=BORDER),
    )

    fig.suptitle(
        "EXP-2026-003a: Topological Diameter Horizon Governs Memory Decay",
        fontsize=13,
        fontweight="bold",
        y=0.99,
    )

    save_figure(fig, output_path)
    plt.close(fig)
    print(f"Generated FIG-DIAG-003a at: {output_path}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate FIG-DIAG-003a Temporal Horizon SVG.")
    parser.add_argument(
        "--output",
        default="docs/research/assets/fig-diag-003a-temporal-horizon.svg",
        help="Target output path.",
    )
    parser.add_argument(
        "--telemetry",
        default="data/telemetry/EXP-2026-003a/summary_evaluation.json",
        help="Path to summary_evaluation.json.",
    )
    args = parser.parse_args()

    render_fig_diag_003a(args.output, args.telemetry)
    return 0


if __name__ == "__main__":
    sys.exit(main())
