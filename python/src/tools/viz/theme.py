"""Theme engine for scientific visualization matching the repository dark theme specification.

Configures Matplotlib to render figures on a #161922 dark canvas with consistent
typography, borders, axes, grids, and semantic accent colors.
"""

from __future__ import annotations

from typing import Any

import matplotlib.pyplot as plt
from tools.viz.palette import (
    BORDER,
    DARK_CANVAS,
    DARK_COLOR_CYCLE,
    DARK_PANEL,
    GRID,
    TEXT,
    TEXT_MUTED,
)


def get_mpl_rc_params() -> dict[str, Any]:
    """Return Matplotlib rcParams dictionary for the repository dark theme."""
    return {
        # Canvas and panels
        "figure.facecolor": DARK_CANVAS,
        "figure.edgecolor": DARK_CANVAS,
        "axes.facecolor": DARK_PANEL,
        "axes.edgecolor": BORDER,
        "axes.linewidth": 1.2,
        # Grid
        "axes.grid": True,
        "grid.color": GRID,
        "grid.linestyle": "--",
        "grid.linewidth": 0.8,
        "grid.alpha": 0.7,
        # Text and typography
        "text.color": TEXT,
        "axes.labelcolor": TEXT,
        "axes.titlecolor": TEXT,
        "xtick.color": TEXT_MUTED,
        "xtick.labelcolor": TEXT_MUTED,
        "ytick.color": TEXT_MUTED,
        "ytick.labelcolor": TEXT_MUTED,
        "font.family": "sans-serif",
        "font.sans-serif": [
            "ui-sans-serif",
            "system-ui",
            "-apple-system",
            "BlinkMacSystemFont",
            "Segoe UI",
            "Roboto",
            "Helvetica Neue",
            "DejaVu Sans",
            "sans-serif",
        ],
        "font.size": 11,
        "axes.titlesize": 13,
        "axes.titleweight": "bold",
        "axes.labelsize": 11,
        # Legend
        "legend.facecolor": DARK_PANEL,
        "legend.edgecolor": BORDER,
        "legend.labelcolor": TEXT,
        "legend.framealpha": 0.9,
        # Color cycle
        "axes.prop_cycle": plt.cycler(color=DARK_COLOR_CYCLE),
        # Savefig defaults
        "savefig.facecolor": DARK_CANVAS,
        "savefig.edgecolor": DARK_CANVAS,
        "savefig.transparent": False,
    }


def setup_matplotlib() -> None:
    """Apply the repository dark theme to Matplotlib global rcParams."""
    plt.rcParams.update(get_mpl_rc_params())


def style_axis(ax: Any) -> None:
    """Apply detailed dark styling to a specific 2D or 3D Matplotlib axis."""
    ax.set_facecolor(DARK_PANEL)

    # Tick marks and labels — must apply to ALL axes (2D and 3D)
    ax.tick_params(
        axis="both",
        colors=TEXT_MUTED,
        which="both",
        labelcolor=TEXT_MUTED,
    )

    # Axis labels and title — guarantee high contrast light text
    if hasattr(ax, "xaxis") and hasattr(ax.xaxis, "label"):
        ax.xaxis.label.set_color(TEXT)
    if hasattr(ax, "yaxis") and hasattr(ax.yaxis, "label"):
        ax.yaxis.label.set_color(TEXT)
    if hasattr(ax, "zaxis") and hasattr(ax.zaxis, "label"):
        ax.zaxis.label.set_color(TEXT)
    if hasattr(ax, "title"):
        ax.title.set_color(TEXT)

    # If it's a 3D axis (Axes3D), style panes and lines
    if hasattr(ax, "xaxis") and hasattr(ax.xaxis, "pane"):
        ax.xaxis.pane.set_facecolor(DARK_PANEL)
        ax.yaxis.pane.set_facecolor(DARK_PANEL)
        ax.zaxis.pane.set_facecolor(DARK_PANEL)
        ax.xaxis.pane.set_edgecolor(BORDER)
        ax.yaxis.pane.set_edgecolor(BORDER)
        ax.zaxis.pane.set_edgecolor(BORDER)
        ax.xaxis.pane.set_alpha(1.0)
        ax.yaxis.pane.set_alpha(1.0)
        ax.zaxis.pane.set_alpha(1.0)
        # 3D grid lines
        ax.xaxis._axinfo["grid"]["color"] = GRID
        ax.yaxis._axinfo["grid"]["color"] = GRID
        ax.zaxis._axinfo["grid"]["color"] = GRID
        ax.zaxis.line.set_color(BORDER)

    # 2D spines styling
    if hasattr(ax, "spines"):
        for spine in ax.spines.values():
            spine.set_edgecolor(BORDER)
            spine.set_linewidth(1.2)


def apply_dark_theme(fig: Any | None = None, ax: Any | None = None) -> None:
    """Apply dark theme to an existing figure and/or axis."""
    setup_matplotlib()
    if fig is not None:
        fig.patch.set_facecolor(DARK_CANVAS)
        fig.patch.set_edgecolor(DARK_CANVAS)
        if hasattr(fig, "_suptitle") and fig._suptitle is not None:
            fig._suptitle.set_color(TEXT)
    if ax is not None:
        if isinstance(ax, (list, tuple)):
            for a in ax:
                style_axis(a)
        elif hasattr(ax, "flat"):
            for a in ax.flat:
                style_axis(a)
        else:
            style_axis(ax)


# Automatically configure matplotlib global rcParams on module import
setup_matplotlib()
