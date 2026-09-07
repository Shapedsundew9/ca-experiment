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
