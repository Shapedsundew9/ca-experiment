"""Mathematical formatting utilities for vector diagrams and SVG text elements.

Provides Unicode conversion for subscripts, superscripts, and common mathematical
symbols to enable publication-grade mathematical typography in DrawSVG and plain text.
"""

from __future__ import annotations

import re

# Unicode subscript and superscript mappings
SUBSCRIPTS: dict[str, str] = {
    "0": "₀",
    "1": "₁",
    "2": "₂",
    "3": "₃",
    "4": "₄",
    "5": "₅",
    "6": "₆",
    "7": "₇",
    "8": "₈",
    "9": "₉",
    "+": "₊",
    "-": "₋",
    "=": "₌",
    "(": "₍",
    ")": "₎",
    "a": "ₐ",
    "e": "ₑ",
    "h": "ₕ",
    "i": "ᵢ",
    "j": "ⱼ",
    "k": "ₖ",
    "l": "ₗ",
    "m": "ₘ",
    "n": "ₙ",
    "o": "ₒ",
    "p": "ₚ",
    "r": "ᵣ",
    "s": "ₛ",
    "t": "ₜ",
    "u": "ᵤ",
    "v": "ᵥ",
    "x": "ₓ",
}

SUPERSCRIPTS: dict[str, str] = {
    "0": "⁰",
    "1": "¹",
    "2": "²",
    "3": "³",
    "4": "⁴",
    "5": "⁵",
    "6": "⁶",
    "7": "⁷",
    "8": "⁸",
    "9": "⁹",
    "+": "⁺",
    "-": "⁻",
    "=": "⁼",
    "(": "⁽",
    ")": "⁾",
    "n": "ⁿ",
    "i": "ⁱ",
    "t": "ᵗ",
}

# Mathematical manifold and set symbols
MATH_SYMBOLS: dict[str, str] = {
    "Z_4": "ℤ₄",
    "Z_N": "ℤ_N",
    "T^2": "𝕋²",
    "R^2": "ℝ²",
    "R^3": "ℝ³",
    "x": "×",
    "->": "→",
    "<->": "↔",
    "=>": "⇒",
    "==": "≡",
    "<=": "≤",
    ">=": "≥",
    "!=": "≠",
    "approx": "≈",
    "infinity": "∞",
    "tau": "τ",
    "lambda": "λ",
    "rho": "ρ",
    "eta": "η",
    "kappa": "κ",
    "Omega": "Ω",
    "theta": "θ",
    "phi": "φ",
}


def to_subscript(text: str) -> str:
    """Convert an ASCII string to Unicode subscripts where available."""
    return "".join(SUBSCRIPTS.get(ch, ch) for ch in str(text))


def to_superscript(text: str) -> str:
    """Convert an ASCII string to Unicode superscripts where available."""
    return "".join(SUPERSCRIPTS.get(ch, ch) for ch in str(text))


def format_subscript(identifier: str) -> str:
    """Convert code identifiers like 'v_0', 'W_ij', 'd_in' to 'v₀', 'Wᵢⱼ', 'dᵢₙ'."""
    match = re.match(r"^([A-Za-z]+)_([A-Za-z0-9]+)$", identifier)
    if match:
        base, sub = match.groups()
        return f"{base}{to_subscript(sub)}"
    return identifier


def format_scientific_notation(val_str: str) -> str:
    """Convert exponential strings like '1e-12' or '5.0e-33' to '10⁻¹²' or '5.0 × 10⁻³³'."""
    match = re.match(r"^([0-9.]+)e-([0-9]+)$", val_str)
    if match:
        coeff, exp = match.groups()
        if coeff == "1":
            return f"10{to_superscript('-' + exp)}"
        return f"{coeff} × 10{to_superscript('-' + exp)}"
    return val_str
