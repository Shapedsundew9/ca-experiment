# Scientific Figure Repertoire & Generator Catalog

This directory contains version-controlled Python generator scripts for programmatic scientific figures (SVG/PNG) embedded in documentation. Every figure in `docs/` must have a corresponding generator script in this catalog.

---

## Authoring Guidelines for Agents

When creating or modifying figures, adhere to the **"Survey, Adapt, or Author"** protocol:

1. **Survey**: Check this directory and `.agents/skills/scientific-figures/scripts/archetypes/` for an existing generator.
2. **Adapt**: Add CLI arguments (`argparse`) to generalize an existing script when possible.
3. **Author**: Create a new standalone script `fig_<doc_id>_<descriptor>.py`:
   - Import dark theme utilities: `from tools.viz import apply_dark_theme, save_figure, DARK_CANVAS, PRIMARY_RED, ...`
   - Export to `docs/assets/figures/` (architecture) or `docs/research/assets/` (research experiments).
   - Ensure the figure is reproducible via `.venv/bin/python python/scripts/figures/<script>.py`.
4. **Register**: Add an entry to the catalog table below.
5. **Validate**: Run `.venv/bin/python .agents/skills/scientific-figures/scripts/validate_figure.py <output-path>`.

---

## Figure Catalog Index

| Figure ID | Generator Script | Output Asset | Target Documents | Description |
| :--- | :--- | :--- | :--- | :--- |
| `FIG-HYP-001` | [`fig_hyp_001_torus_manifold.py`](fig_hyp_001_torus_manifold.py) | `docs/research/assets/fig-hyp-001-torus-manifold.svg` | [`HYP-2026-001.md`](../../docs/research/hypotheses/HYP-2026-001.md) | Dual-panel $4 \times 4$ Torus cellular automaton lattice, von Neumann stencil, periodic wrap-around, and 3D manifold embedding. |
| `FIG-DIAG-004a` | [`fig_diag_004a_hebbian_highway.py`](fig_diag_004a_hebbian_highway.py) | `docs/research/assets/fig-diag-004a-hebbian-highway.svg` | [`DIAG-2026-004a.md`](../../docs/research/diagnostics/DIAG-2026-004a.md) | Wavefront collision and refractory annihilation in isotropic lattice vs. chiral resonant highway formed under Hebbian plasticity. |
