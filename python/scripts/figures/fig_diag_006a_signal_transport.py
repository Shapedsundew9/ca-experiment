#!/usr/bin/env python3
"""Figure Generator: FIG-DIAG-006a - Directed Regenerative Transmission Tracks & Branching Fan-Out.

Target Document: docs/research/diagnostics/DIAG-2026-006a.md
Output: docs/research/assets/fig-diag-006a-signal-transport.svg

Visualizes the empirical findings of EXP-2026-006a:
Left Panel: Transmission Fidelity F across spatial distance D in {10, 20, 30, 50} cells.
            Contrasts active regenerative thresholding (F = 1.000) against passive attenuation
            extinction and unshielded bidirectional reverberation.
Right Panel: Noise resilience sweep at D = 30 across background noise rates eps in {0.0, 0.01, 0.05, 0.10}.
             Demonstrates homeostatic threshold adaptation filtering percolation noise vs fixed-theta breakdown.
"""

from __future__ import annotations

import argparse
import json
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


def render_fig_diag_006a(output_path: str, telemetry_path: str | None = None) -> None:
    # Default empirical values matching summary.json
    dist_x = [10, 20, 30, 50]
    fid_active = [1.000, 1.000, 1.000, 1.000]
    fid_passive = [0.735, 0.735, 0.735, 0.735]
    fid_unshielded = [0.643, 0.633, 0.623, 0.599]

    noise_x = [0.00, 0.01, 0.05, 0.10]
    ber_active = [0.000, 0.000, 0.0005, 0.0009]
    ber_fixed = [0.000, 0.0046, 0.0224, 0.0403]
    ber_unshielded = [0.214, 0.197, 0.187, 0.194]

    # Load actual empirical data if telemetry summary exists
    if telemetry_path and Path(telemetry_path).exists():
        with open(telemetry_path, "r", encoding="utf-8") as f:
            summary = json.load(f)
            d_sweep = summary.get("distance_sweep", {})
            for idx, d in enumerate(dist_x):
                d_key = f"d{d}"
                if d_key in d_sweep:
                    fid_active[idx] = d_sweep[d_key].get("active_regenerative", {}).get("fidelity_mean", fid_active[idx])
                    fid_passive[idx] = d_sweep[d_key].get("baseline_passive", {}).get("fidelity_mean", fid_passive[idx])
                    fid_unshielded[idx] = d_sweep[d_key].get("ablation_unshielded", {}).get("fidelity_mean", fid_unshielded[idx])

            n_sweep = summary.get("noise_sweep", {})
            for idx, eps in enumerate(noise_x):
                eps_key = f"eps_{eps:.2f}".replace(".", "_")
                if eps_key in n_sweep:
                    ber_active[idx] = n_sweep[eps_key].get("active_regenerative", {}).get("ber_mean", ber_active[idx])
                    ber_fixed[idx] = n_sweep[eps_key].get("ablation_fixed_theta", {}).get("ber_mean", ber_fixed[idx])
                    ber_unshielded[idx] = n_sweep[eps_key].get("ablation_unshielded", {}).get("ber_mean", ber_unshielded[idx])

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5))
    apply_dark_theme(fig, [ax1, ax2])

    # ----------------------------------------------------
    # Panel 1: Spatial Distance Scaling & Fan-Out Fidelity
    # ----------------------------------------------------
    ax1.plot(
        dist_x,
        fid_active,
        color=PRIMARY_RED,
        marker="o",
        linewidth=2.5,
        markersize=7,
        label=r"Active Regenerative ($W_{\mathrm{fwd}} \geq \theta$)",
    )
    ax1.plot(
        dist_x,
        fid_passive,
        color=TERTIARY_BLUE,
        marker="s",
        linewidth=1.8,
        linestyle="--",
        markersize=6,
        label=r"Passive Leaky Cable ($\lambda = 0.10$)",
    )
    ax1.plot(
        dist_x,
        fid_unshielded,
        color=AMBER,
        marker="^",
        linewidth=1.8,
        linestyle=":",
        markersize=6,
        label=r"Ablation Unshielded ($N_{\mathrm{ref}} = 0$)",
    )

    # Gate 2 threshold benchmark
    ax1.axhline(1.0, color=SECONDARY_GREEN, linestyle="--", alpha=0.6, linewidth=1.5, label=r"Gate 2 Threshold ($\mathcal{F} = 1.000$)")
    ax1.axvline(30, color=BORDER, linestyle=":", alpha=0.5, linewidth=1.2)

    ax1.set_title(r"Transmission Fidelity vs Distance $D$", pad=10)
    ax1.set_xlabel(r"Spatial Distance $D$ (Cells)", labelpad=8)
    ax1.set_ylabel(r"Combined Branch Fidelity $\mathcal{F}$", labelpad=8)
    ax1.set_ylim(0.40, 1.05)
    ax1.set_xlim(5, 55)
    ax1.legend(loc="lower left", framealpha=0.85, fontsize=8.5)

    ax1.text(
        30,
        0.45,
        r"Zero Attenuation Invariant: $\mathcal{F} = 1.000$" + "\n" + r"$\mathrm{BER}_1 = \mathrm{BER}_2 = 0.000$ for $D \geq 30$",
        ha="center",
        color=TEXT,
        fontsize=8.5,
        bbox=dict(boxstyle="round,pad=0.3", facecolor=DARK_CANVAS, edgecolor=BORDER),
    )

    # ----------------------------------------------------
    # Panel 2: Background Noise Sweep & Adaptive Shielding
    # ----------------------------------------------------
    ax2.plot(
        noise_x,
        ber_active,
        color=PRIMARY_RED,
        marker="o",
        linewidth=2.5,
        markersize=7,
        label=r"Active Adaptive ($\beta_\theta = 0.05$)",
    )
    ax2.plot(
        noise_x,
        ber_fixed,
        color=AMBER,
        marker="D",
        linewidth=1.8,
        linestyle="--",
        markersize=6,
        label=r"Ablation Fixed Somatic ($\theta \equiv 1.00$)",
    )

    # Gate 4 threshold benchmark
    ax2.axhline(0.010, color=SECONDARY_GREEN, linestyle="--", linewidth=1.5, alpha=0.8, label=r"Gate 4 Ceiling ($\mathrm{BER} \leq 0.010$)")
    ax2.axvline(0.05, color=BORDER, linestyle=":", alpha=0.5, linewidth=1.2)

    ax2.set_title(r"Noise Robustness at Distance $D = 30$", pad=10)
    ax2.set_xlabel(r"Background Noise Rate $\epsilon$", labelpad=8)
    ax2.set_ylabel(r"Mean Bit Error Rate ($\mathrm{BER}$)", labelpad=8)
    ax2.set_ylim(-0.002, 0.050)
    ax2.set_xlim(-0.005, 0.105)
    ax2.legend(loc="upper left", framealpha=0.85, fontsize=8.5)

    ax2.text(
        0.055,
        0.032,
        r"Homeostatic Somatic Shield:" + "\n" + r"$\theta_i \geq 1.02$ suppresses percolation" + "\n" + r"Welch $p = 1.22 \times 10^{-86}$",
        ha="left",
        color=TEXT,
        fontsize=8.5,
        bbox=dict(boxstyle="round,pad=0.3", facecolor=DARK_CANVAS, edgecolor=BORDER),
    )

    save_figure(fig, output_path)
    plt.close(fig)
    print(f"[OK] Figure generated successfully: {output_path}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate FIG-DIAG-006a signal transport diagnostic figure")
    parser.add_argument(
        "--output",
        "-o",
        type=str,
        default="docs/research/assets/fig-diag-006a-signal-transport.svg",
        help="Target output asset path (.svg or .png)",
    )
    parser.add_argument(
        "--telemetry",
        "-t",
        type=str,
        default="data/telemetry/EXP-2026-006a/summary.json",
        help="Path to evaluation summary JSON file",
    )
    args = parser.parse_args()

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    render_fig_diag_006a(str(out), args.telemetry)


if __name__ == "__main__":
    main()
