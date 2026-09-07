"""Layout and text wrapping helpers for DrawSVG vector diagrams.

Ensures that text cards and multi-line descriptions stay within container boundaries
and do not clip outside the SVG frame.
"""

from __future__ import annotations

import sys
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


def measure_card_with_bullets(
    entries: list[tuple[str, str]],
    *,
    title: str | None = None,
    max_chars: int = 42,
    title_spacing: float = 22.0,
    bullet_title_spacing: float = 15.0,
    line_spacing: float = 15.0,
    gap_between_bullets: float = 4.0,
    top_padding: float = 20.0,
    bottom_padding: float = 10.0,
) -> float:
    """Calculate the required height for a card with the given bullet entries.
    
    Returns the minimum height needed to render all content without overflow.
    Call this before draw_card_with_bullets to determine proper sizing.
    """
    total = top_padding
    if title:
        total += title_spacing
    for label, desc in entries:
        if label:
            total += bullet_title_spacing
        wrapped = wrap_text(desc, width_chars=max_chars)
        total += len(wrapped) * line_spacing
        total += gap_between_bullets
    return total + bottom_padding


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
) -> float:
    """Draw a beautifully formatted card with structured, wrapped bullet points that fit within width."""
    required_height = measure_card_with_bullets(entries, title=title, max_chars=max_chars)
    actual_height = max(height, required_height)
    if required_height > height:
        print(f"WARNING: Card content requires {required_height:.0f}px but container is {height:.0f}px. Auto-expanding.", file=sys.stderr)

    drawing.append(
        draw.Rectangle(
            x,
            y,
            width,
            actual_height,
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

    return actual_height


def auto_size_canvas(
    panels: list[dict],
    *,
    margin: float = 20.0,
    header_space: float = 80.0,
    footer_space: float = 30.0,
) -> tuple[float, float]:
    """Calculate minimum canvas dimensions to fit all planned panels.
    
    Each panel dict has keys: 'x', 'y', 'width', 'height'.
    Returns (canvas_width, canvas_height).
    """
    if not panels:
        return (0.0, 0.0)
        
    width = max(p['x'] + p['width'] for p in panels) + margin
    height = max(p['y'] + p['height'] for p in panels) + footer_space + margin
        
    return width, height
