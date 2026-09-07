#!/usr/bin/env python3
"""Archetype 3: Dynamical Systems Phase Portraits & Attractor Landscapes.

Generates 2D phase plane portraits, vector fields, streamplots, nullclines, and
trajectory curves on the repository dark theme using Matplotlib.

Usage:
    .venv/bin/python phase_space.py --system oscillator --output docs/assets/figures/phase_portrait.svg
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


def render_phase_portrait(system_name: str, output_path: str) -> None:
    fig, ax = plt.subplots(figsize=(8, 6))
    apply_dark_theme(fig, ax)

    # Grid for vector field
    x = np.linspace(-2.5, 2.5, 25)
    y = np.linspace(-2.5, 2.5, 25)
    X, Y = np.meshgrid(x, y)

    if system_name == "oscillator":
        # Van der Pol oscillator: dx/dt = y, dy/dt = mu*(1 - x^2)*y - x
        mu = 1.0
        U = Y
        V = mu * (1 - X**2) * Y - X
        title = r"Van der Pol Limit Cycle Phase Portrait ($\mu = 1.0$)"
        xlabel = r"State Variable $x(t)$ (Potential $V$)"
        ylabel = r"Velocity $\dot{x}(t)$ (Current $I$)"

        # Nullclines
        x_null = np.linspace(-2.5, 2.5, 200)
        y_null_x = np.zeros_like(x_null)  # dx/dt = 0 => y = 0
        y_null_y = x_null / (mu * (1 - x_null**2) + 1e-6)  # dy/dt = 0

        ax.axhline(0, color=BORDER, linestyle=":", alpha=0.8, label=r"$\dot{x} = 0$ Nullcline")

    elif system_name == "saddle":
        # Saddle point: dx/dt = x, dy/dt = -y
        U = X
        V = -Y
        title = r"Hyperbolic Saddle Point ($\dot{x} = x, \dot{y} = -y$)"
        xlabel = r"$x(t)$"
        ylabel = r"$y(t)$"
        ax.axhline(0, color=BORDER, linestyle=":")
        ax.axvline(0, color=BORDER, linestyle=":")

    else:
        # Attractor well: dx/dt = -x - y, dy/dt = x - y
        U = -X - Y
        V = X - Y
        title = r"Stable Spiral Attractor Basin"
        xlabel = r"$x(t)$"
        ylabel = r"$y(t)$"

    # Streamplot with velocity coloring
    speed = np.sqrt(U**2 + V**2)
    strm = ax.streamplot(
        X,
        Y,
        U,
        V,
        color=speed,
        cmap="coolwarm",
        density=1.2,
        linewidth=1.0,
        arrowsize=1.2,
    )

    # Sample trajectory
    t = np.linspace(0, 10, 500)
    if system_name == "oscillator":
        # Simulated closed limit cycle
        theta = np.linspace(0, 2 * np.pi, 200)
        traj_x = 2.0 * np.cos(theta)
        traj_y = -2.0 * np.sin(theta) * (1 - 0.3 * np.cos(theta) ** 2)
        ax.plot(traj_x, traj_y, color=PRIMARY_RED, linewidth=2.5, label="Stable Limit Cycle (Attractor)")
        ax.scatter([0], [0], color=AMBER, s=80, zorder=5, label="Unstable Fixed Point")
    elif system_name == "saddle":
        ax.scatter([0], [0], color=AMBER, s=80, zorder=5, label="Saddle Fixed Point")
    else:
        # Spiral in
        traj_r = np.exp(-0.3 * t) * 2.2
        traj_x = traj_r * np.cos(t * 2)
        traj_y = traj_r * np.sin(t * 2)
        ax.plot(traj_x, traj_y, color=PRIMARY_RED, linewidth=2.2, label="Attractor Trajectory")
        ax.scatter([0], [0], color=SECONDARY_GREEN, s=80, zorder=5, label="Stable Fixed Point")

    ax.set_title(title, pad=12)
    ax.set_xlabel(xlabel, labelpad=8)
    ax.set_ylabel(ylabel, labelpad=8)
    ax.legend(loc="upper right")
    ax.set_xlim(-2.5, 2.5)
    ax.set_ylim(-2.5, 2.5)

    save_figure(fig, output_path)
    plt.close(fig)
    print(f"Phase portrait generated at: {output_path}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate phase portrait visualizations.")
    parser.add_argument(
        "--system",
        choices=["oscillator", "saddle", "spiral"],
        default="oscillator",
        help="Dynamical system model.",
    )
    parser.add_argument("--output", required=True, help="Target output file (.svg or .png).")
    args = parser.parse_args()

    render_phase_portrait(args.system, args.output)
    return 0


if __name__ == "__main__":
    sys.exit(main())
