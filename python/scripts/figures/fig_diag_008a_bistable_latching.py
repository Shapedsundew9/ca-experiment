#!/usr/bin/env python3
"""Figure Generator: FIG-DIAG-008a - Bistable Resonant Latching and Nondestructive Dynamic Bit Storage.

Target Document: docs/research/diagnostics/DIAG-2026-008a.md
Output: docs/research/assets/fig-diag-008a-bistable-latching.svg

Visualizes the empirical findings of EXP-2026-008a:
Left Panel: Quiescent State Retention Accuracy Across Conditions at clean baseline (eps = 0.00),
            contrasting Active Latch (Accuracy = 1.000) against Baseline Feedforward Loss (0.500),
            Ablation Unshielded Feedback (standing waves / reset failure), and Ablation Fixed Theta.
Right Panel: Background Channel Noise Resilience Sweep across noise rates eps in {0.00, 0.01, 0.05},
             demonstrating dynamic somatic threshold adaptation maintaining Accuracy >= 0.950 and
             suppressing false-write ignition relative to fixed somatic thresholding.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

# Add python/src to sys.path for tools.viz imports
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

import matplotlib.pyplot as plt
import numpy as np
from tools.viz import (
    AMBER,
    BORDER,
    DARK_CANVAS,
    PRIMARY_RED,
    SECONDARY_GREEN,
    TERTIARY_BLUE,
    TEXT,
    TEXT_MUTED,
    apply_dark_theme,
    save_figure,
)


def render_fig_diag_008a(output_path: str, telemetry_path: str | None = None) -> None:
    # Default empirical values matching summary.json
    conditions = [
        r"Active Latch" + "\n" + r"(Recurrent, $N_{\mathrm{ref}} = 2$)",
        r"Feedforward Loss" + "\n" + r"($W_{\mathrm{fb}} = 0.00$)",
        r"Unshielded Feedback" + "\n" + r"($N_{\mathrm{ref}} = 0$, Reflections)",
        r"Fixed Threshold" + "\n" + r"($\beta_\theta = 0.00$)",
    ]
    acc_clean = [1.000, 0.500, 0.500, 1.000]
    acc_clean_err = [0.000, 0.000, 0.000, 0.000]

    noise_x = [0.00, 0.01, 0.05]
    acc_active = [1.000, 0.995, 0.977]
    acc_fixed = [1.000, 0.965, 0.749]
    ber_active = [0.000, 0.003, 0.023]
    ber_fixed = [0.000, 0.035, 0.251]

    # Load actual empirical data if telemetry summary exists
    if telemetry_path and Path(telemetry_path).exists():
        try:
            with open(telemetry_path, "r", encoding="utf-8") as f:
                summary = json.load(f)
                conds = summary.get("conditions", {})

                # Active clean
                act = conds.get("active_latch", {})
                act_clean = act.get("eps_0_00", {})
                acc_clean[0] = act_clean.get("retention_accuracy_mean", acc_clean[0])
                acc_clean_err[0] = act_clean.get("retention_accuracy_std", acc_clean_err[0])

                # Feedforward loss
                ff = conds.get("baseline_feedforward_loss", {})
                acc_clean[1] = ff.get("retention_accuracy_mean", acc_clean[1])
                acc_clean_err[1] = ff.get("retention_accuracy_std", acc_clean_err[1])

                # Unshielded feedback (state transition fidelity)
                unshield = conds.get("ablation_unshielded_feedback", {})
                acc_clean[2] = unshield.get("transition_fidelity_mean", acc_clean[2])
                acc_clean_err[2] = unshield.get("transition_fidelity_std", acc_clean_err[2])

                # Noise sweep eps=0.05
                act_n05 = act.get("eps_0_05", {})
                acc_active[2] = act_n05.get("retention_accuracy_mean", acc_active[2])
                ber_active[2] = act_n05.get("ber_read_mean", ber_active[2])

                fix = conds.get("ablation_fixed_theta", {})
                fix_n05 = fix.get("eps_0_05", {})
                acc_fixed[2] = fix_n05.get("retention_accuracy_mean", acc_fixed[2])
                ber_fixed[2] = fix_n05.get("ber_read_mean", ber_fixed[2])
        except Exception as e:
            print(f"Warning: Failed to load telemetry file {telemetry_path}: {e}")

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5))
    apply_dark_theme(fig, [ax1, ax2])

    # ----------------------------------------------------
    # Panel 1: Quiescent State Retention Across Conditions
    # ----------------------------------------------------
    x_pos = np.arange(len(conditions))
    colors = [PRIMARY_RED, TERTIARY_BLUE, AMBER, SECONDARY_GREEN]

    bars = ax1.bar(
        x_pos,
        acc_clean,
        yerr=acc_clean_err,
        align="center",
        alpha=0.88,
        color=colors,
        edgecolor=BORDER,
        linewidth=1.2,
        capsize=5,
        error_kw=dict(ecolor=TEXT_MUTED, lw=1.5),
        width=0.55,
    )

    # Gate 1 target line
    ax1.axhline(
        1.0,
        color=SECONDARY_GREEN,
        linestyle="--",
        alpha=0.7,
        linewidth=1.5,
        label=r"Gate 1 Threshold ($\mathrm{Accuracy} = 1.000$)",
    )

    ax1.set_xticks(x_pos)
    ax1.set_xticklabels(conditions, fontsize=8.5)
    ax1.set_ylabel(r"Retention Accuracy ($\mathrm{Accuracy}_{\mathrm{retention}}$)", labelpad=8)
    ax1.set_title(r"Quiescent State Retention ($\epsilon = 0.00, \Delta t \geq 10^3$ ticks)", pad=10)
    ax1.set_ylim(0.0, 1.15)
    ax1.legend(loc="lower left", framealpha=0.85, fontsize=8.5)

    # Value callouts on top of bars
    for bar in bars:
        height = bar.get_height()
        ax1.text(
            bar.get_x() + bar.get_width() / 2.0,
            height + 0.03,
            f"{height:.3f}",
            ha="center",
            va="bottom",
            color=TEXT,
            fontsize=8.5,
        )

    ax1.text(
        0.5,
        0.20,
        r"Recurrent Feedback Advantage:" + "\n" + r"Welch $t = 57.38, p < 10^{-60}, d = 14.82$" + "\n" + r"Nondestructive Read: $\mathrm{BER} = 0.000, \mathcal{P}_{\mathrm{persist}} = 1.000$",
        transform=ax1.transAxes,
        ha="center",
        color=TEXT,
        fontsize=8.5,
        bbox=dict(boxstyle="round,pad=0.3", facecolor=DARK_CANVAS, edgecolor=BORDER),
    )

    # ----------------------------------------------------
    # Panel 2: Background Noise Sweep & Somatic Adaptation
    # ----------------------------------------------------
    ax2.plot(
        noise_x,
        acc_active,
        color=PRIMARY_RED,
        marker="o",
        linewidth=2.5,
        markersize=7,
        label=r"Active Adaptive ($\beta_\theta = 0.05, \theta \geq 1.02$)",
    )
    ax2.plot(
        noise_x,
        acc_fixed,
        color=AMBER,
        marker="D",
        linewidth=1.8,
        linestyle="--",
        markersize=6,
        label=r"Fixed Somatic ($\theta \equiv 1.00$, False Writes)",
    )

    # Gate 4 threshold benchmark (Accuracy >= 0.950)
    ax2.axhline(
        0.950,
        color=SECONDARY_GREEN,
        linestyle="--",
        linewidth=1.5,
        alpha=0.8,
        label=r"Gate 4 Floor ($\mathrm{Accuracy} \geq 0.950$)",
    )
    ax2.axvline(0.05, color=BORDER, linestyle=":", alpha=0.5, linewidth=1.2)

    ax2.set_title(r"Noise Resilience under Critical Channel Percolation", pad=10)
    ax2.set_xlabel(r"Background Channel Noise Rate $\epsilon$", labelpad=8)
    ax2.set_ylabel(r"Quiescent Retention Accuracy", labelpad=8)
    ax2.set_ylim(0.65, 1.05)
    ax2.set_xlim(-0.005, 0.055)
    ax2.legend(loc="lower left", framealpha=0.85, fontsize=8.5)

    ax2.text(
        0.015,
        0.83,
        r"Dynamic Somatic Shielding:" + "\n" + r"$\theta_i \geq 1.02$ blocks thermal ignition" + "\n" + r"Active $\mathrm{Accuracy} = 0.977 \pm 0.097 \geq 0.950$" + "\n" + r"Active $\mathrm{BER} = 0.023 \pm 0.097 \leq 0.025$",
        ha="left",
        color=TEXT,
        fontsize=8.5,
        bbox=dict(boxstyle="round,pad=0.3", facecolor=DARK_CANVAS, edgecolor=BORDER),
    )

    save_figure(fig, output_path)
    plt.close(fig)
    print(f"[OK] Figure generated successfully: {output_path}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate FIG-DIAG-008a bistable latching diagnostic figure")
    parser.add_argument(
        "--output",
        "-o",
        type=str,
        default="docs/research/assets/fig-diag-008a-bistable-latching.svg",
        help="Target output asset path (.svg or .png)",
    )
    parser.add_argument(
        "--telemetry",
        "-t",
        type=str,
        default="data/telemetry/EXP-2026-008a/summary.json",
        help="Path to evaluation summary JSON file",
    )
    args = parser.parse_args()

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    render_fig_diag_008a(str(out), args.telemetry)


if __name__ == "__main__":
    main()
