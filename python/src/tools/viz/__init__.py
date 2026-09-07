"""Scientific visualization and figure generation package.

Provides dark-theme styling, palette constants, and exporters for Matplotlib and DrawSVG,
matching the repository design specification.
"""

from __future__ import annotations

from tools.viz import palette
from tools.viz.draw_utils import draw_card_with_bullets, draw_multiline_text, wrap_text
from tools.viz.export import save_figure
from tools.viz.math_format import (
    MATH_SYMBOLS,
    format_scientific_notation,
    format_subscript,
    to_subscript,
    to_superscript,
)
from tools.viz.palette import (
    AMBER,
    AMBER_FILL,
    AMBER_TEXT,
    BORDER,
    DARK_CANVAS,
    DARK_COLOR_CYCLE,
    DARK_PANEL,
    EDGE,
    GRID,
    PRIMARY_RED,
    PRIMARY_RED_FILL,
    PRIMARY_RED_TEXT,
    SECONDARY_GREEN,
    SECONDARY_GREEN_FILL,
    SECONDARY_GREEN_TEXT,
    TERTIARY_BLUE,
    TERTIARY_BLUE_FILL,
    TERTIARY_BLUE_TEXT,
    TEXT,
    TEXT_MUTED,
)
from tools.viz.theme import apply_dark_theme, get_mpl_rc_params, setup_matplotlib

__all__ = [
    "palette",
    "apply_dark_theme",
    "get_mpl_rc_params",
    "setup_matplotlib",
    "save_figure",
    "DARK_CANVAS",
    "DARK_PANEL",
    "BORDER",
    "GRID",
    "TEXT",
    "TEXT_MUTED",
    "EDGE",
    "PRIMARY_RED",
    "PRIMARY_RED_FILL",
    "PRIMARY_RED_TEXT",
    "SECONDARY_GREEN",
    "SECONDARY_GREEN_FILL",
    "SECONDARY_GREEN_TEXT",
    "TERTIARY_BLUE",
    "TERTIARY_BLUE_FILL",
    "TERTIARY_BLUE_TEXT",
    "AMBER",
    "AMBER_FILL",
    "AMBER_TEXT",
    "DARK_COLOR_CYCLE",
    "format_subscript",
    "to_subscript",
    "to_superscript",
    "format_scientific_notation",
    "MATH_SYMBOLS",
    "draw_multiline_text",
    "draw_card_with_bullets",
    "wrap_text",
]
