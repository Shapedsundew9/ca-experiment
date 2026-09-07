"""Unit tests for tools.viz package."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

import drawsvg as draw
import matplotlib.pyplot as plt

from tools.viz import (
    DARK_CANVAS,
    DARK_PANEL,
    PRIMARY_RED,
    apply_dark_theme,
    save_figure,
    setup_matplotlib,
)


class TestVizTools(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp_dir = tempfile.TemporaryDirectory()
        self.output_dir = Path(self.tmp_dir.name)

    def tearDown(self) -> None:
        self.tmp_dir.cleanup()

    def test_theme_setup(self) -> None:
        setup_matplotlib()
        self.assertEqual(plt.rcParams["figure.facecolor"], DARK_CANVAS)
        self.assertEqual(plt.rcParams["axes.facecolor"], DARK_PANEL)

    def test_matplotlib_export_svg_and_png(self) -> None:
        fig, ax = plt.subplots()
        apply_dark_theme(fig, ax)
        ax.plot([0, 1, 2], [0, 1, 4], color=PRIMARY_RED, label="Quadratic")
        ax.legend()

        svg_path = self.output_dir / "test_fig.svg"
        png_path = self.output_dir / "test_fig.png"

        save_figure(fig, svg_path)
        save_figure(fig, png_path)

        plt.close(fig)

        self.assertTrue(svg_path.exists())
        self.assertGreater(svg_path.stat().st_size, 0)
        content = svg_path.read_text()
        self.assertIn("<svg", content)

        self.assertTrue(png_path.exists())
        self.assertGreater(png_path.stat().st_size, 0)

    def test_drawsvg_export_svg(self) -> None:
        d = draw.Drawing(200, 200)
        d.append(draw.Rectangle(0, 0, 200, 200, fill=DARK_CANVAS))
        d.append(draw.Circle(100, 100, 40, fill=PRIMARY_RED))

        svg_path = self.output_dir / "test_draw.svg"
        save_figure(d, svg_path)

        self.assertTrue(svg_path.exists())
        self.assertGreater(svg_path.stat().st_size, 0)
        content = svg_path.read_text()
        self.assertIn("<svg", content)

    def test_math_format_subscripts(self) -> None:
        from tools.viz import format_scientific_notation, format_subscript, to_subscript, to_superscript

        self.assertEqual(format_subscript("v_0"), "v₀")
        self.assertEqual(format_subscript("v_15"), "v₁₅")
        self.assertEqual(format_subscript("W_ij"), "Wᵢⱼ")
        self.assertEqual(format_subscript("d_in"), "dᵢₙ")
        self.assertEqual(to_subscript("123"), "₁₂₃")
        self.assertEqual(to_superscript("2"), "²")
        self.assertEqual(format_scientific_notation("1e-12"), "10⁻¹²")
        self.assertEqual(format_scientific_notation("5.0e-33"), "5.0 × 10⁻³³")

    def test_draw_utils_wrapping_and_card(self) -> None:
        from tools.viz import draw_card_with_bullets, wrap_text

        text = "This is a very long line of scientific explanation that needs to wrap properly inside its container box."
        wrapped = wrap_text(text, width_chars=35)
        self.assertGreater(len(wrapped), 1)
        for line in wrapped:
            self.assertLessEqual(len(line), 35)

        d = draw.Drawing(400, 300)
        entries = [
            ("Periodic Closure", "Coordinates wrap modulo 4 across boundaries."),
            ("Graph Regularity", "Every node possesses in-degree and out-degree of exactly 4."),
        ]
        draw_card_with_bullets(d, 20, 20, 360, 200, entries, title="Test Card", max_chars=40)
        svg_path = self.output_dir / "test_card.svg"
        save_figure(d, svg_path)
        self.assertTrue(svg_path.exists())


if __name__ == "__main__":
    unittest.main()
