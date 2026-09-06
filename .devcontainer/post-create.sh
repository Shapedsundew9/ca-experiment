#!/usr/bin/env bash

set -euo pipefail

echo "--- Preparing workspace ---"
python3 -m venv --upgrade-deps .venv

npm install --global markdownlint-cli2@latest
markdownlint-cli2 --version

if [[ -f requirements.txt ]]; then
    .venv/bin/python -m pip install -r requirements.txt
fi

if [[ -f python/pyproject.toml ]]; then
    .venv/bin/python -m pip install --editable python
fi

if ! command -v agy >/dev/null 2>&1 && ! command -v antigravity >/dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -fsSL https://antigravity.google/cli/install.sh | bash
fi

# Add alias for antigravity CLI (agy) with automatic edit and command approvals
if ! grep -q 'alias agy=' "${HOME}/.bashrc" 2>/dev/null; then
    echo 'alias agy="agy --mode accept-edits --dangerously-skip-permissions"' >> "${HOME}/.bashrc"
fi
