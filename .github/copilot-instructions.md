# Project Guidelines

## Layout

- This is a mixed Rust and Python experimentation workspace.
- Put Rust code in `src/` and Rust integration tests in `tests/`.
- Put reusable Python code in `python/src/tools/`.
- Put Python tests in `python/tests/`.
- Put one-off Python programs in `python/scripts/`.
- Put Python scientific experiment packages in `python/experiments/`.
- Do not mix Python files into the Rust `src/` directory.
- Do not assume Rust/Python FFI unless explicitly requested.

## Python

- Use `.venv/bin/python`; the devcontainer installs `python/` as an editable package.
- Import reusable code as `tools`.
- Do not set or modify `PYTHONPATH`.
- Declare Python package dependencies in `python/pyproject.toml`.
- Avoid multiline inline scripts via `python -c` with nested quotes. Write a temporary scratch script to disk instead to prevent escaping errors and shell friction.

## Services And Secrets

- PostgreSQL and Neo4j are available through Docker Compose but are not started by default.
- Read credentials and tokens from the existing environment variables.
- The environment variables may direct to a remote service. Do not assume a docker or local service is running.
- Never hard-code, print, or commit secret values.
- Relevant variables include `DATABASE_URL`, `NEO4J_URI`,
  `NEO4J_USERNAME`, `NEO4J_PASSWORD`, `NEO4J_DATABASE`,
  `GEMINI_API_KEY`, `ARC_AGI_API`, and `HF_READ_TOKEN`.

## Validation

- Run `cargo fmt --check`, `cargo clippy`, and `cargo test` for Rust changes.
- Run `.venv/bin/python -m unittest discover -s python/tests -v`
  for Python changes.
- After creating or editing Markdown, run `markdownlint-cli2 --fix "**/*.md"`,
  inspect the resulting diff, and then run `markdownlint-cli2 "**/*.md"`.
- Use the Markdownlint Fix All action from the Problems panel when working
  interactively. The shared policy is defined in `.markdownlint-cli2.jsonc`.
- Keep intentional Markdown exceptions narrow and document them with a
  targeted configuration or inline suppression.
- Validate only the services and language surfaces affected by an experiment.

## Diagramming & Scientific Visualization

- **System & Logic Diagrams**: Use Mermaid for software architectures, state machines, sequence diagrams, and pipeline logic. Follow `docs/templates/mermaid-style-guide.md`.
- **Scientific Figures & Mathematical Visualizations**: Use Python-generated SVGs (or 300 DPI PNGs) when Mermaid is structurally incapable of representing the concept (3D geometric manifolds, continuous surfaces, discrete spatial lattices, phase portraits, and empirical telemetry distributions). Follow `docs/templates/figure-style-guide.md`.
- **Visual Repertoire & Script Authorship**: Agents are empowered to create new generator scripts in `python/scripts/figures/` or adapt existing archetypes in `.github/skills/scientific-figures/scripts/` (symbolically linked with `.agents/skills/scientific-figures/scripts/`). Every figure must be 100% reproducible via its generator script.
- **Theme Consistency**: All figures must adopt the repo's dark theme palette (`#161922` canvas, `#1e2230` panel, gentle RGB accents `#e06c75`, `#73c991`, `#61afef`).
- **Storage**: Store general figures in `docs/assets/figures/` and experiment-specific figures in `docs/research/assets/`. Document scripts in `python/scripts/figures/README.md`.

## Mathematical Notation

- Ensure LaTeX formulas render identically in both GitHub web preview (MathJax/CommonMark) and VS Code preview (KaTeX).
- Follow the style and compatibility rules in `docs/templates/math-style-guide.md`.
- CommonMark unescapes ASCII punctuation characters after backslashes before MathJax parses them. To avoid delimiter errors:
  - Use `\lbrace` and `\rbrace` instead of `\{` and `\}` for set brackets and delimiters (e.g., `\lbrace 0, 1 \rbrace`, `\big\lbrace ... \big\rbrace`, `\left\lbrace ... \right\rbrace`). Never use `\big\{` or `\left\{`.
  - Use `\lVert` and `\rVert` (or `\Vert`) instead of `\|` for vector/matrix norms.
  - Avoid `, \,`; standard commas `,` in LaTeX already provide punctuation spacing. Use named spacing commands (`\thinspace`, `\quad`) if explicit spacing is needed.
  - Avoid underscores inside `\text{...}`; use hyphens (e.g., `\text{sum-max}` instead of `\text{sum\_max}`).
  - Avoid `\%` inside math spans; write percentages in prose as `95%` or use `\text{\%}`.
  - For complex standalone multi-line equations (such as systems of cases or matrices), use fenced code blocks with the `math` language identifier (```math ...```) or standard `$$ ... $$` with control-word delimiters.

## 3rd Party Packages

- Agents are explicitly authorized to install any pip packages or Rust crates needed to execute experiments efficiently.
- Do not spend hours or multiple tool loops implementing ad-hoc workarounds for functionality readily provided by standard libraries.
- Well-established, high-quality, maintained packages (e.g., `numpy`, `scipy`, `rand`, `serde`) are fully supported.
- Avoid pulling in obscure, single-maintainer, or redundant packages if the task is trivial in-house.
- Always declare added dependencies in the appropriate project file:
  - Python: declare in `python/pyproject.toml` and install into `.venv` (`.venv/bin/pip install <pkg>`).
  - Rust: declare in `Cargo.toml` or add via `cargo add <crate>`.

## Pre-Flight Sanity Checks

- Before starting an autonomous campaign or multi-cycle workflow, run the universal pre-flight check:
  `python3 .agents/skills/preflight/scripts/preflight.py`.
  - Confirms required Python dependencies are present in `.venv` (or install and declare them).
  - Confirms Rust toolchain is functional (`cargo --version`, `cargo check`).
  - Confirms subagents have the necessary `tools:` declared in their `.agents/agents/*.md` definitions so they possess write and execution capabilities.
  - Verifies workspace write access.
