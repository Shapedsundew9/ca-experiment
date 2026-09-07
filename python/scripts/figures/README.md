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
| `FIG-DIAG-001a` | [`fig_diag_001a_homeostasis.py`](fig_diag_001a_homeostasis.py) | `docs/research/assets/fig-diag-001a-homeostasis.svg` | [`DIAG-2026-001a.md`](../../docs/research/diagnostics/DIAG-2026-001a.md) | Dual-panel homeostatic firing density stabilization within critical band $[0.05, 0.20]$ vs. extinction/saturation across ablation conditions. |
| `FIG-DIAG-002a` | [`fig_diag_002a_attractor_separation.py`](fig_diag_002a_attractor_separation.py) | `docs/research/assets/fig-diag-002a-attractor-separation.svg` | [`DIAG-2026-002a.md`](../../docs/research/diagnostics/DIAG-2026-002a.md) | Multistable phase space orbit clustering and empirical distance separation metrics ($D_{\text{inter}} = 0.0417$, $\text{SNR} = 3.63$, Cohen's $d = 0.78$). |
| `FIG-DIAG-003a` | [`fig_diag_003a_temporal_horizon.py`](fig_diag_003a_temporal_horizon.py) | `docs/research/assets/fig-diag-003a-temporal-horizon.svg` | [`DIAG-2026-003a.md`](../../docs/research/diagnostics/DIAG-2026-003a.md) | Topological transit horizon ($D_{\text{diam}} = 4$) and sharp memory decay beyond graph diameter due to refractory wavefront collision. |
| `FIG-DIAG-004a` | [`fig_diag_004a_hebbian_highway.py`](fig_diag_004a_hebbian_highway.py) | `docs/research/assets/fig-diag-004a-hebbian-highway.svg` | [`DIAG-2026-004a.md`](../../docs/research/diagnostics/DIAG-2026-004a.md) | Wavefront collision and refractory annihilation in isotropic lattice vs. chiral resonant highway formed under Hebbian plasticity. |
| `FIG-DIAG-005a` | [`fig_diag_005a_continual_learning.py`](fig_diag_005a_continual_learning.py) | `docs/research/assets/fig-diag-005a-continual-learning.svg` | [`DIAG-2026-005a.md`](../../docs/research/diagnostics/DIAG-2026-005a.md) | Sensory ingress allocation ($A \to B \to C$) and continual retention metrics ($\text{AR} > 0.97$, $\text{BWT} \ge -0.028$) under local metaplasticity. |
| `FIG-DIAG-006a` | [`fig_diag_006a_signal_transport.py`](fig_diag_006a_signal_transport.py) | `docs/research/assets/fig-diag-006a-signal-transport.svg` | [`DIAG-2026-006a.md`](../../docs/research/diagnostics/DIAG-2026-006a.md) | Dual-panel spatial distance scaling (zero attenuation $\mathcal{F} = 1.000$ for $D \geq 30$) and background noise resilience ($\mathrm{BER} \leq 0.001$ under $\epsilon = 0.05$). |
| `FIG-DIAG-007a` | [`fig_diag_007a_multi_gate_composition.py`](fig_diag_007a_multi_gate_composition.py) | `docs/research/assets/fig-diag-007a-multi-gate-composition.svg` | [`DIAG-2026-007a.md`](../../docs/research/diagnostics/DIAG-2026-007a.md) | Dual-panel 1-bit Full Adder logic synthesis accuracy across conditions ($1.000$ vs $0.375$) and cascaded noise resilience under $\epsilon \in [0.00, 0.05]$. |

