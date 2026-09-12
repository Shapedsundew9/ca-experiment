"""Declarative schematic diagramming primitives for excitable cellular automata.

Provides high-level building blocks (cards, connection arrows, bus labels, state nodes)
that enforce repository theme consistency, automatic mathematical subscript conversion,
and canvas boundary protection.
"""

from __future__ import annotations

import dataclasses
from typing import Any

import drawsvg as draw
from tools.viz.draw_utils import draw_card_with_bullets, measure_card_with_bullets
from tools.viz.math_format import format_math_text
from tools.viz.palette import (
    BORDER,
    DARK_CANVAS,
    DARK_PANEL,
    EDGE,
    PRIMARY_RED,
    SECONDARY_GREEN,
    TERTIARY_BLUE,
    TEXT,
    TEXT_MUTED,
)


@dataclasses.dataclass
class SchematicCard:
    """Specification for a container panel or component card."""

    x: float
    y: float
    width: float
    entries: list[tuple[str, str]]
    title: str | None = None
    height: float = 0.0
    bg_fill: str = DARK_CANVAS
    border_color: str = BORDER
    max_chars: int = 42

    def render(self, drawing: draw.Drawing) -> float:
        """Render the card onto the drawing, sanitizing all math labels.

        Returns the rendered actual height.
        """
        sanitized_title = format_math_text(self.title) if self.title else None
        sanitized_entries = [
            (format_math_text(label), format_math_text(desc))
            for label, desc in self.entries
        ]
        return draw_card_with_bullets(
            drawing,
            self.x,
            self.y,
            self.width,
            self.height,
            sanitized_entries,
            title=sanitized_title,
            max_chars=self.max_chars,
            bg_fill=self.bg_fill,
            border_color=self.border_color,
        )


def draw_directed_connection(
    drawing: draw.Drawing,
    start: tuple[float, float],
    end: tuple[float, float],
    *,
    label: str | None = None,
    color: str = EDGE,
    stroke_width: float = 1.5,
    dashed: bool = False,
) -> None:
    """Draw a directed synaptic edge or signal bus connection with an arrowhead."""
    x1, y1 = start
    x2, y2 = end

    stroke_args: dict[str, Any] = {
        "stroke": color,
        "stroke_width": stroke_width,
    }
    if dashed:
        stroke_args["stroke_dasharray"] = "4,3"

    drawing.append(draw.Line(x1, y1, x2, y2, **stroke_args))

    if label:
        sanitized_label = format_math_text(label)
        mid_x = (x1 + x2) / 2
        mid_y = (y1 + y2) / 2 - 4
        drawing.append(
            draw.Text(
                sanitized_label,
                9.5,
                mid_x,
                mid_y,
                text_anchor="middle",
                fill=TEXT_MUTED,
                font_family="sans-serif",
            )
        )
