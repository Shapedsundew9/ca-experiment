# Scientific Figure Style Guide & Visual Identity Specification

This guide defines the standards, color palette, formatting rules, and authoring guidelines for programmatic scientific figures (SVG and PNG) across documentation in this repository. Adhering to these standards ensures aesthetic harmony with the repository's Mermaid dark theme ([`docs/templates/mermaid-style-guide.md`](mermaid-style-guide.md)) across GitHub dark mode and VS Code previews.

---

## 1. Design Philosophy & Two-Tier Visualization Standard

Documentation in this repository employs a two-tiered visualization architecture:

| Tier | Purpose | Recommended Technology | Standards Guide |
| :--- | :--- | :--- | :--- |
| **Tier 1: System & Logic** | Software architectures, state transitions, pipelines, sequence interactions, decision trees | Mermaid code blocks | [`mermaid-style-guide.md`](mermaid-style-guide.md) |
| **Tier 2: Scientific & Empirical** | Continuous 3D/2D manifolds, discrete spatial lattices, periodic boundaries, phase portraits, telemetry curves | Programmatic Python scripts (`matplotlib`, `drawsvg`) generating vector SVGs | [`figure-style-guide.md`](figure-style-guide.md) (this document) |

---

## 2. Color Palette & Dark Theme Specification

All figures must render against the standard dark background. Never generate figures with white, light gray, or transparent canvas backgrounds.

### Theme Palette Reference

| Role | Color Name | Hex Code | Semantic Usage in Figures |
| :--- | :--- | :--- | :--- |
| **Canvas Background** | Dark Canvas | `#161922` | Full figure background canvas (`figure.facecolor`) |
| **Panel / Axes Fill** | Dark Panel | `#1e2230` | Plot area fill, card backgrounds (`axes.facecolor`) |
| **Borders & Spines** | Dark Slate Border | `#434c5e` | Axis lines, card borders, wireframe mesh lines |
| **Subtle Grid Lines** | Dark Grid | `#33394a` | Coordinate grid lines (dashed, $\alpha = 0.7$) |
| **Primary Text** | Soft White | `#e2e8f0` | Main titles, primary axis labels, node identifiers |
| **Muted Text** | Slate Muted | `#94a3b8` | Tick labels, subtitles, secondary coordinates |
| **Primary Accent** | Gentle Red (Rosewood) | `#e06c75` | Target nodes, active conditions, invariant markers |
| **Primary Fill** | Deep Rosewood | `#422026` | Filled nodes, active region highlights |
| **Secondary Accent** | Gentle Green (Forest Sage) | `#73c991` | Critical stability bands, neighbor stencils, success channels |
| **Secondary Fill** | Deep Forest Sage | `#1b3528` | Shaded stability regions, neighbor cell fills |
| **Tertiary Accent** | Gentle Blue (Royal Slate) | `#61afef` | 3D surfaces, baseline/control traces, infrastructure |
| **Tertiary Fill** | Deep Royal Slate | `#1d2c44` | Baseline area fills, datastore cards |
| **Callout / Amber** | Muted Amber | `#e5c07b` | Ablation conditions, threshold limits, periodic wrap loops |
| **Callout Fill** | Deep Amber | `#2e271a` | Warning / ablation fills |

---

## 3. Matplotlib Styling Setup

In Python visualization scripts, import and apply the shared theme from `tools.viz`:

```python
import matplotlib.pyplot as plt
from tools.viz import apply_dark_theme, save_figure, PRIMARY_RED, SECONDARY_GREEN

fig, ax = plt.subplots(figsize=(8, 5))
apply_dark_theme(fig, ax)

# Plot your data
ax.plot(x, y, color=PRIMARY_RED, linewidth=2.0, label="Active Condition")
ax.axhspan(0.05, 0.20, color=SECONDARY_GREEN, alpha=0.15, label="Target Band")

# Save as vector SVG (or PNG)
save_figure(fig, "docs/research/assets/fig-example.svg")
plt.close(fig)
```

---

## 4. Vector SVG vs. Raster PNG Standards

1. **Vector SVG (`.svg`) — 90% Mandatory Default**:
   - Resolution-independent: crisply rendered on mobile, 4K, and Retina screens.
   - Text-based XML: version-controllable and diffable in Git.
   - Size limit: Must remain under **1 MB** (typically 10 KB–300 KB).
   - Embedding: `![Figure Description](../../assets/figures/fig-name.svg)`.

2. **Raster PNG (`.png`) — 10% Exception**:
   - Reserved strictly for dense 3D shaded polygon meshes or massive micro-state raster heatmaps (>10,000 pixels) where SVG DOM elements would cause browser rendering lag.
   - Resolution: Must be exported at **300 DPI** minimum (`dpi=300`).
   - Canvas: Must still enforce `#161922` canvas background.

---

## 5. File Organization & Naming Conventions

All scientific figures follow a strict naming and storage convention:

### Directory Structure

- **General / Architectural Figures**: `docs/assets/figures/`
- **Scientific Research Figures**: `docs/research/assets/`
- **Generator Scripts**: `python/scripts/figures/`
- **Catalog Registry**: `python/scripts/figures/README.md`

### File Naming Pattern

```text
fig-<doc-type>-<doc-number>-<descriptor>.<ext>
```

**Examples**:

- `fig-hyp-001-torus-manifold.svg` (Hypothesis figure)
- `fig-exp-002a-attractor-separation.svg` (Protocol figure)
- `fig-diag-004a-hebbian-highway.svg` (Diagnostic report figure)

---

## 6. The Cumulative Repertoire Protocol

Never commit an unscripted image asset. Every figure must have a corresponding Python generator script in `python/scripts/figures/` following the "Survey, Adapt, or Author" pattern:

1. **Survey**: Check `python/scripts/figures/` and `.agents/skills/scientific-figures/scripts/archetypes/` for an existing generator.
2. **Adapt**: Add CLI arguments to an existing script if it can be generalized.
3. **Author**: Create a new modular generator script if a novel visual concept is required.
4. **Register**: Add an entry to `python/scripts/figures/README.md`.
5. **Validate**: Run `.venv/bin/python .agents/skills/scientific-figures/scripts/validate_figure.py <figure-path>`.

---

## 7.5 Compositing Mixed 2D/3D Figures

When a figure combines flat schematics with 3D surface renders:

1. Render 3D components with matplotlib `mplot3d` as **separate companion PNGs** (300 DPI).
2. Keep 2D schematics as **pure vector SVGs** via DrawSVG.
3. Never embed base64 raster data inside SVG files — this destroys git diffability.
4. Name companion files with a `-3d` suffix: `fig-hyp-001-torus-3d.png`.
5. The markdown document embeds both files to compose the final visual.

---

## 7. Mathematical Typography & Layout Overflow Invariants

To guarantee publication-grade visual and mathematical quality, all figure generator scripts must strictly adhere to the following invariants:

### Rule 1: No Raw Programming Syntax in Labels

Never place raw code identifiers or programming variable names with underscores into user-facing diagram labels:

| ❌ Avoid in Diagrams | ✅ Use in DrawSVG (Unicode) | ✅ Use in Matplotlib (LaTeX) | Rendered Meaning |
| :--- | :--- | :--- | :--- |
| `v_0`, `v_15` | `v₀`, `v₁₅` | `$v_0$`, `$v_{15}$` | Node indexing |
| `W_ij ≡ 1.0` | `Wᵢⱼ ≡ 1.0` | `$W_{ij} \equiv 1.0$` | Synaptic weight |
| `d_in = d_out = 4` | `dᵢₙ = dₒᵤₜ = 4` | `$d_{\mathrm{in}} = d_{\mathrm{out}} = 4$` | Node degrees |
| `Z_4 x Z_4` | `ℤ₄ × ℤ₄` | `$\mathbb{Z}_4 \times \mathbb{Z}_4$` | Toroidal discrete manifold |
| `(T^2)` | `𝕋²` | `$\mathbb{T}^2$` | Continuous torus |
| `D_diam = 4` | `D_diam = 4` or `D_diam = 4` | `$D_{\mathrm{diam}} = 4$` | Topological diameter |
| `p < 1e-12`, `1e-32` | `p < 10⁻¹²`, `10⁻³²` | `$p < 10^{-12}$`, `$10^{-32}$` | Statistical significance |
| `R_i = N_ref` | `Rᵢ = Nᵣₑf` | `$R_i = N_{\mathrm{ref}}$` | Refractory counter |

### Rule 2: Container Width Budgeting & Text Wrapping

SVG `<text>` does NOT auto-wrap. Unbounded strings will overflow container cards and spill off the canvas frame:

- **Card Budgeting**: Calculate `max_chars = int(box_width_px / 7.5)` for proportional fonts and `/ 8.5` for monospace.
- **Auto-Wrapping Helper**: Use `from tools.viz import draw_card_with_bullets, wrap_text` to automatically wrap descriptions into clean multi-line bullet entries.
- **Canvas Margins**: Ensure all text elements have at least 15px clearance from outer canvas borders.

### Rule 3: Layout Pre-Measurement

All card-based layouts with bullet content must pre-calculate required heights:

- **Pre-measure**: Call `measure_card_with_bullets()` from `tools.viz` before creating the SVG Drawing to determine minimum card heights.
- **Dynamic sizing**: Use `auto_size_canvas()` to compute canvas dimensions from panel specifications instead of hardcoding pixel values.
- **Return values**: `draw_card_with_bullets()` now returns the actual height used. Use this return value to position subsequent elements.

### Rule 4: Light Typography & Contrast on Dark Canvas

Never emit unstyled black text, ticks, spines, or error bars on the repository dark theme:

- **Primary Text**: Use `TEXT` (`#e2e8f0`) for master titles, panel headers, and primary axis labels.
- **Secondary & Ticks**: Use `TEXT_MUTED` (`#94a3b8`) for tick marks, tick labels, subtitles, and error bars.
- **Matplotlib Theming**: Always call `apply_dark_theme(fig, ax)`, which applies `ax.tick_params(colors=TEXT_MUTED, labelcolor=TEXT_MUTED)` across all subplots.
- **Error Bars & Caps**: When using `ax.bar()` or `ax.errorbar()`, always pass `error_kw=dict(ecolor=TEXT_MUTED, lw=1.2, capthick=1.2)` to prevent Matplotlib defaulting error bars to black.

### Rule 5: Automated Linter Enforcement

Before committing any figure, run `validate_figure.py --strict`. It inspects SVG XML for:

- Raw code underscores in text elements (`\b[A-Za-z]+_[A-Za-z0-9]+\b`).
- Raw scientific notation (`\b\d+e-\d+\b`).
- Horizontal boundary overflow (`x + estimated_width > canvas_width`).
- Vertical canvas overflow (`y > canvas_height - 5`).
- Container containment violations (text exceeding card boundaries).
- Text-text collisions (overlapping vertical text elements).
- Unstyled black strokes or fills (`#000000`, `black`) on the dark canvas.
