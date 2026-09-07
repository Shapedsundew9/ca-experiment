#!/usr/bin/env python3
"""Convenience script and CLI utility for scientific figure theming."""

from __future__ import annotations

import sys
from tools.viz import (
    AMBER,
    BORDER,
    DARK_CANVAS,
    DARK_COLOR_CYCLE,
    DARK_PANEL,
    GRID,
    PRIMARY_RED,
    SECONDARY_GREEN,
    TERTIARY_BLUE,
    TEXT,
    TEXT_MUTED,
    apply_dark_theme,
    get_mpl_rc_params,
    save_figure,
    setup_matplotlib,
)

__all__ = [
    "DARK_CANVAS",
    "DARK_PANEL",
    "BORDER",
    "GRID",
    "TEXT",
    "TEXT_MUTED",
    "PRIMARY_RED",
    "SECONDARY_GREEN",
    "TERTIARY_BLUE",
    "AMBER",
    "DARK_COLOR_CYCLE",
    "apply_dark_theme",
    "get_mpl_rc_params",
    "setup_matplotlib",
    "save_figure",
]


def print_palette() -> None:
    """Print the palette values to terminal."""
    print("Scientific Figure Dark Palette Specification:")
    print(f"  Canvas Background:   {DARK_CANVAS}")
    print(f"  Panel/Axes Fill:     {DARK_PANEL}")
    print(f"  Borders/Spines:      {BORDER}")
    print(f"  Grid Lines:          {GRID}")
    print(f"  Primary Text:        {TEXT}")
    print(f"  Muted Text:          {TEXT_MUTED}")
    print(f"  Primary Accent:      {PRIMARY_RED} (Gentle Red)")
    print(f"  Secondary Accent:    {SECONDARY_GREEN} (Gentle Green)")
    print(f"  Tertiary Accent:     {TERTIARY_BLUE} (Gentle Blue)")
    print(f"  Callout/Amber:       {AMBER} (Muted Amber)")


if __name__ == "__main__":
    print_palette()
    sys.exit(0)
