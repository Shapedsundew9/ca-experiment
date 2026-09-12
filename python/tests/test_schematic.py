"""Tests for declarative schematic and math formatting utilities."""

import unittest
import drawsvg as draw
from tools.viz import SchematicCard, format_math_text, format_subscript


class TestSchematic(unittest.TestCase):
    def test_format_subscript_conversion(self):
        self.assertEqual(format_subscript("v_0"), "v₀")
        self.assertEqual(format_subscript("P_1"), "P₁")
        self.assertEqual(format_subscript("d_in"), "dᵢₙ")
        self.assertEqual(format_subscript("W_fb"), "$W_{\\text{fb}}$")

    def test_format_math_text_replaces_raw_underscores(self):
        text = "Gate G_write triggers quench on Ring R_0 with weight W_fb."
        formatted = format_math_text(text)
        # Verify no raw alphanumeric underscores remain
        import re
        self.assertEqual(re.findall(r"\b[A-Za-z]+_[A-Za-z0-9]+\b", formatted), [])

    def test_schematic_card_render(self):
        d = draw.Drawing(500, 300)
        card = SchematicCard(
            x=20,
            y=20,
            width=200,
            title="Coincidence Gate G_write",
            entries=[
                ("Subthreshold v_0", "Integrates W_key and W_val without leak"),
                ("Quench Diode inh_0", "Suppresses retro reflection"),
            ],
        )
        rendered_height = card.render(d)
        self.assertGreater(rendered_height, 50.0)


if __name__ == "__main__":
    unittest.main()
