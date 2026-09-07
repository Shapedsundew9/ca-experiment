#!/usr/bin/env python3
"""Archetype 4: Multi-Panel Empirical Telemetry Reduction Charts.

Generates dual-panel scientific telemetry figures showing temporal trajectories
with critical bounds and statistical distributions across experimental conditions.

Usage:
    .venv/bin/python telemetry_timeseries.py --output docs/assets/figures/telemetry_sample.svg
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
from tools.viz import (
    AMBER,
    BORDER,
    GRID,
    PRIMARY_RED,
    SECONDARY_GREEN,
    TERTIARY_BLUE,
    TEXT,
    TEXT_MUTED,
    apply_dark_theme,
    save_figure,
)


def render_telemetry_chart(output_path: str, title: str = "Empirical Telemetry Reduction") -> None:
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5), gridspec_kw={"width_ratios": [2, 1]})
    apply_dark_theme(fig, [ax1, ax2])

    np.random.seed(42)
    ticks = np.linspace(0, 1000, 200)

    # Shaded Critical Band on Panel 1
    ax1.axhspan(0.05, 0.20, color=SECONDARY_GREEN, alpha=0.15, label="Critical Target Band [0.05, 0.20]")
    ax1.axhline(0.10, color=SECONDARY_GREEN, linestyle="--", alpha=0.6, label=r"Target Setpoint $r_{\mathrm{target}} = 0.10$")

    # Active Condition: Stabilizes in critical band
    active_trace = 0.10 + 0.03 * np.exp(-ticks / 200) * np.sin(ticks / 30) + np.random.normal(0, 0.008, len(ticks))
    ax1.plot(ticks, active_trace, color=PRIMARY_RED, linewidth=2.0, label="Active Regulated Substrate")

    # Ablation 1: Static threshold runaway
    runaway_trace = np.clip(0.10 + 0.0008 * ticks + np.random.normal(0, 0.01, len(ticks)), 0, 0.8)
    ax1.plot(ticks, runaway_trace, color=AMBER, linewidth=1.5, linestyle="-.", label="Static Threshold Ablation")

    # Ablation 2: Memoryless decay extinction
    extinct_trace = 0.10 * np.exp(-ticks / 120) + np.random.normal(0, 0.003, len(ticks))
    ax1.plot(ticks, extinct_trace, color=TERTIARY_BLUE, linewidth=1.5, linestyle=":", label="Uncoupled Memoryless Ablation")

    ax1.set_title("Temporal Firing Density Stabilization", pad=10)
    ax1.set_xlabel("Time (Discrete Ticks $t$)", labelpad=8)
    ax1.set_ylabel(r"Firing Density $\bar{\rho}(t)$", labelpad=8)
    ax1.set_ylim(-0.02, 0.50)
    ax1.legend(loc="upper right", framealpha=0.85)

    # Panel 2: Steady-State Distribution Box/Bar Plot
    conditions = ["Active", "Static", "Memoryless"]
    means = [np.mean(active_trace[100:]), np.mean(runaway_trace[100:]), np.mean(extinct_trace[100:])]
    stds = [np.std(active_trace[100:]), np.std(runaway_trace[100:]), np.std(extinct_trace[100:])]
    colors = [PRIMARY_RED, AMBER, TERTIARY_BLUE]

    bars = ax2.bar(
        conditions,
        means,
        yerr=stds,
        capsize=5,
        error_kw=dict(ecolor=TEXT_MUTED, lw=1.2, capthick=1.2),
        color=colors,
        edgecolor=BORDER,
        width=0.5,
        alpha=0.9,
    )
    ax2.axhspan(0.05, 0.20, color=SECONDARY_GREEN, alpha=0.15)
    ax2.set_title("Steady-State Mean Activity", pad=10)
    ax2.set_ylabel(r"Mean Firing Density $\langle \bar{\rho} \rangle$", labelpad=8)
    ax2.set_ylim(-0.02, 0.50)

    # Add numeric labels above bars
    for bar, m in zip(bars, means):
        ax2.text(
            bar.get_x() + bar.get_width() / 2,
            m + 0.03,
            f"{m:.3f}",
            ha="center",
            va="bottom",
            color=TEXT,
            fontsize=10,
            fontweight="bold",
        )

    fig.suptitle(title, fontsize=14, fontweight="bold", y=1.02)
    save_figure(fig, output_path)
    plt.close(fig)
    print(f"Telemetry chart generated at: {output_path}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate empirical telemetry chart.")
    parser.add_argument("--output", required=True, help="Target output file (.svg or .png).")
    parser.add_argument("--title", default="Empirical Telemetry Reduction", help="Chart super-title.")
    parser.add_argument('--validate', action='store_true', help='Run validation after generation.')
    args = parser.parse_args()

    render_telemetry_chart(args.output, title=args.title)

    if args.validate:
        import subprocess
        result = subprocess.run(
            [sys.executable, str(Path(__file__).resolve().parents[1] / 'validate_figure.py'), '--strict', args.output],
            capture_output=True, text=True
        )
        print(result.stdout)
        if result.returncode != 0:
            print(result.stderr, file=sys.stderr)
            sys.exit(result.returncode)

    return 0


if __name__ == "__main__":
    sys.exit(main())
