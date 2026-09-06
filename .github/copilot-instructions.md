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

## Diagramming

- Use Mermaid for diagrams in Markdown files.
- Convert any textual diagrams to Mermaid for consistency and maintainability.
- Follow the style in `docs/templates/mermaid-style-guide.md` for Mermaid diagrams.

## 3rd Party Packages

- Minimize the number of 3rd party packages used in the project.
- OSS brings supply chain and security risk. Only use it if there is a significant benefit over implementing the functionality in-house.
- Only use well-known, widely adopted, consistently maintained packages that have a permissive license to reduce maintenance burden.
