"""Figure export utilities supporting SVG vector and PNG raster formats.

Handles directory provisioning, canvas background enforcement, and format auto-detection
for both Matplotlib figures and DrawSVG vector drawings.
"""

from __future__ import annotations

import os
from pathlib import Path
from typing import Any

from tools.viz.palette import DARK_CANVAS


def save_figure(
    fig_or_drawing: Any,
    output_path: str | Path,
    *,
    dpi: int = 300,
    facecolor: str = DARK_CANVAS,
    bbox_inches: str = "tight",
) -> Path:
    """Save a Matplotlib Figure or DrawSVG Drawing to disk.

    Args:
        fig_or_drawing: A matplotlib.figure.Figure or drawsvg.Drawing instance.
        output_path: Target path (e.g., docs/assets/figures/fig-name.svg).
        dpi: Dots per inch for raster export or vector sizing (default 300).
        facecolor: Canvas background color (defaults to repo dark canvas).
        bbox_inches: Bounding box mode for Matplotlib (default "tight").

    Returns:
        The resolved Path of the written file.
    """
    path = Path(output_path)
    path.parent.mkdir(parents=True, exist_ok=True)

    # Check if DrawSVG Drawing
    if hasattr(fig_or_drawing, "save_svg") or hasattr(fig_or_drawing, "save_png"):
        fmt = path.suffix.lower().lstrip(".")
        if fmt == "png":
            if hasattr(fig_or_drawing, "save_png"):
                fig_or_drawing.save_png(str(path))
            else:
                raise ValueError("DrawSVG object does not support PNG export directly")
        else:
            fig_or_drawing.save_svg(str(path))
        return path

    # Matplotlib Figure
    if hasattr(fig_or_drawing, "savefig"):
        fig_or_drawing.savefig(
            str(path),
            dpi=dpi,
            facecolor=facecolor,
            edgecolor=facecolor,
            bbox_inches=bbox_inches,
        )
        return path

    raise TypeError(
        f"Unsupported figure object type: {type(fig_or_drawing)}. "
        "Expected matplotlib.figure.Figure or drawsvg.Drawing."
    )
