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


if __name__ == "__main__":
    unittest.main()
