"""Palette constants for scientific visualization matching the repository dark theme specification.

These hex values mirror the color identity defined in docs/templates/mermaid-style-guide.md.
"""

from __future__ import annotations

# Canvas and structural backgrounds
DARK_CANVAS: str = "#161922"
DARK_PANEL: str = "#1e2230"
BORDER: str = "#434c5e"
GRID: str = "#33394a"

# Typography and line accents
TEXT: str = "#e2e8f0"
TEXT_MUTED: str = "#94a3b8"
EDGE: str = "#8892b0"

# Gentle RGB Tri-color Semantic System
PRIMARY_RED: str = "#e06c75"
PRIMARY_RED_FILL: str = "#422026"
PRIMARY_RED_TEXT: str = "#fde8ec"

SECONDARY_GREEN: str = "#73c991"
SECONDARY_GREEN_FILL: str = "#1b3528"
SECONDARY_GREEN_TEXT: str = "#e6f7ee"

TERTIARY_BLUE: str = "#61afef"
TERTIARY_BLUE_FILL: str = "#1d2c44"
TERTIARY_BLUE_TEXT: str = "#e4f0fc"

# Notes, constraints, and threshold callouts
AMBER: str = "#e5c07b"
AMBER_FILL: str = "#2e271a"
AMBER_TEXT: str = "#fdf4db"

# Color cycle for multi-series line plots
DARK_COLOR_CYCLE: list[str] = [
    PRIMARY_RED,
    SECONDARY_GREEN,
    TERTIARY_BLUE,
    AMBER,
    "#c678dd",  # Muted Purple
    "#56b6c2",  # Muted Cyan
]
