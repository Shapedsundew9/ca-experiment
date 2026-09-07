#!/usr/bin/env python3
"""Archetype 1: Continuous 3D Parametric Surfaces and Manifolds.

Generates publication-quality 3D mathematical surfaces (e.g. torus, sphere, saddle, paraboloid)
with coordinate grids, wireframes, or continuous colormaps on the repository dark theme.

Usage:
    .venv/bin/python surface_3d.py --surface torus --output docs/assets/figures/torus_3d.svg
    .venv/bin/python surface_3d.py --surface saddle --output docs/assets/figures/saddle_3d.svg
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
from tools.viz import (
    BORDER,
    DARK_PANEL,
    PRIMARY_RED,
    SECONDARY_GREEN,
    TERTIARY_BLUE,
    apply_dark_theme,
    save_figure,
)


def generate_torus(r_major: float = 3.0, r_minor: float = 1.0, n_points: int = 50):
    theta = np.linspace(0, 2 * np.pi, n_points)
    phi = np.linspace(0, 2 * np.pi, n_points)
    theta, phi = np.meshgrid(theta, phi)

    x = (r_major + r_minor * np.cos(theta)) * np.cos(phi)
    y = (r_major + r_minor * np.cos(theta)) * np.sin(phi)
    z = r_minor * np.sin(theta)
    return x, y, z, r"$\mathbb{T}^2$ 3D Torus Manifold: $(R + r\cos\theta)\cos\phi$"


def generate_saddle(n_points: int = 50):
    x = np.linspace(-2, 2, n_points)
    y = np.linspace(-2, 2, n_points)
    x, y = np.meshgrid(x, y)
    z = x**2 - y**2
    return x, y, z, r"Hyperbolic Saddle Landscape: $z = x^2 - y^2$"


def generate_paraboloid(n_points: int = 50):
    x = np.linspace(-2, 2, n_points)
    y = np.linspace(-2, 2, n_points)
    x, y = np.meshgrid(x, y)
    z = x**2 + y**2
    return x, y, z, r"Potential Well: $z = x^2 + y^2$"


def render_surface(
    surface_type: str,
    output_path: str,
    elevation: float = 32.0,
    azimuth: float = 45.0,
) -> None:
    if surface_type == "torus":
        x, y, z, title = generate_torus()
    elif surface_type == "saddle":
        x, y, z, title = generate_saddle()
    elif surface_type == "paraboloid":
        x, y, z, title = generate_paraboloid()
    else:
        raise ValueError(f"Unknown surface type: {surface_type}")

    fig = plt.figure(figsize=(8, 6))
    ax = fig.add_subplot(111, projection="3d")
    apply_dark_theme(fig, ax)

    # Wireframe or surface plot with dark palette
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

    ax.view_init(elev=elevation, azim=azimuth)
    ax.set_title(title, pad=12)
    ax.set_xlabel("X", labelpad=8)
    ax.set_ylabel("Y", labelpad=8)
    ax.set_zlabel("Z", labelpad=8)

    save_figure(fig, output_path)
    plt.close(fig)
    print(f"Surface figure generated at: {output_path}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate 3D surface visualizations.")
    parser.add_argument(
        "--surface",
        choices=["torus", "saddle", "paraboloid"],
        default="torus",
        help="Surface type to generate.",
    )
    parser.add_argument(
        "--output",
        required=True,
        help="Target output file (.svg or .png).",
    )
    parser.add_argument("--elev", type=float, default=32.0, help="Elevation angle.")
    parser.add_argument("--azim", type=float, default=45.0, help="Azimuth angle.")

    args = parser.parse_args()
    render_surface(args.surface, args.output, elevation=args.elev, azimuth=args.azim)
    return 0


if __name__ == "__main__":
    sys.exit(main())
