"""Layout and text wrapping helpers for DrawSVG vector diagrams.

Ensures that text cards and multi-line descriptions stay within container boundaries
and do not clip outside the SVG frame.
"""

from __future__ import annotations

import textwrap
from typing import Any

import drawsvg as draw
from tools.viz.palette import BORDER, DARK_CANVAS, DARK_PANEL, TEXT, TEXT_MUTED


def wrap_text(text: str, width_chars: int = 40) -> list[str]:
    """Wrap a long string into a list of lines fitting width_chars."""
    return textwrap.wrap(text, width=width_chars, break_long_words=False)


def draw_multiline_text(
    drawing: draw.Drawing,
    lines: list[str],
    x: float,
    y: float,
    *,
    line_spacing: float = 16.0,
    font_size: float = 10.5,
    fill: str = TEXT_MUTED,
    font_family: str = "sans-serif",
    font_weight: str = "normal",
    text_anchor: str = "start",
) -> float:
    """Draw multiple lines of text with consistent line spacing.

    Returns:
        The bottom y-coordinate after rendering all lines.
    """
    current_y = y
    for line in lines:
        drawing.append(
            draw.Text(
                line,
                font_size,
                x,
                current_y,
                text_anchor=text_anchor,
                fill=fill,
                font_family=font_family,
                font_weight=font_weight,
            )
        )
        current_y += line_spacing
    return current_y


def draw_card_with_bullets(
    drawing: draw.Drawing,
    x: float,
    y: float,
    width: float,
    height: float,
    entries: list[tuple[str, str]],  # (bold_label, description)
    *,
    title: str | None = None,
    max_chars: int = 42,
    bg_fill: str = DARK_CANVAS,
    border_color: str = BORDER,
) -> None:
    """Draw a beautifully formatted card with structured, wrapped bullet points that fit within width."""
    drawing.append(
        draw.Rectangle(
            x,
            y,
            width,
            height,
            rx=6,
            ry=6,
            fill=bg_fill,
            stroke=border_color,
            stroke_width=1.0,
        )
    )

    current_y = y + 20
    if title:
        drawing.append(
            draw.Text(
                title,
                12,
                x + 14,
                current_y,
                fill=TEXT,
                font_family="sans-serif",
                font_weight="bold",
            )
        )
        current_y += 22

    for label, desc in entries:
        # Title of bullet
        if label:
            drawing.append(
                draw.Text(
                    f"• {label}",
                    11.0,
                    x + 14,
                    current_y,
                    fill=TEXT,
                    font_family="sans-serif",
                    font_weight="bold",
                )
            )
            current_y += 15

        # Description wrapped inside card width
        wrapped = wrap_text(desc, width_chars=max_chars)
        for line in wrapped:
            prefix = "   " if label else "• "
            drawing.append(
                draw.Text(
                    f"{prefix}{line}",
                    10.5,
                    x + 14,
                    current_y,
                    fill=TEXT_MUTED,
                    font_family="sans-serif",
                )
            )
            current_y += 15
        current_y += 4  # small gap between bullet items
