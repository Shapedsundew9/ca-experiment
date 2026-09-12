"""Unit tests for curation scripts (record_run.py and format_and_lint.py)."""

import importlib.util
from pathlib import Path
import tempfile
import unittest


def load_module_from_path(module_name: str, file_path: Path):
    spec = importlib.util.spec_from_file_location(module_name, file_path)
    if spec is None or spec.loader is None:
        raise ImportError(f"Could not load module from {file_path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


record_run_path = Path(__file__).resolve().parents[1] / "scripts" / "curation" / "record_run.py"
record_run_mod = load_module_from_path("record_run", record_run_path)


class TestRecordRun(unittest.TestCase):
    def test_update_manifest_provenance_table(self):
        content = """# Experiment Run Manifest: EXP-2026-999a (Run 01)

| Dimension | Specification |
| :--- | :--- |
| Git Commit SHA | `pending` |
| Git Tag | `pending` |
| Git Status Dirty? | Yes |
| Runtime / Compiler | Rust 1.98 |
"""
        with tempfile.NamedTemporaryFile("w+", suffix=".md", delete=False) as f:
            f.write(content)
            temp_path = Path(f.name)

        try:
            modified = record_run_mod.update_manifest_provenance(
                manifest_path=temp_path,
                commit_sha="abcd1234efgh5678",
                tag_name="exp/EXP-2026-999a-01",
            )
            self.assertTrue(modified)
            updated_content = temp_path.read_text(encoding="utf-8")
            self.assertIn("| Git Commit SHA | `abcd1234efgh5678` |", updated_content)
            self.assertIn("| Git Tag | `exp/EXP-2026-999a-01` |", updated_content)
            self.assertIn("| Git Status Dirty? | No (Clean at tagging gate) |", updated_content)
        finally:
            if temp_path.exists():
                temp_path.unlink()

    def test_update_manifest_provenance_list(self):
        content = """# Experiment Run Manifest: EXP-2026-999a (Run 01)

- **Protocol Reference**: [docs/research/protocols/EXP-2026-999a.md](docs/research/protocols/EXP-2026-999a.md)
- **Git SHA**: `0000000000000000`
- **Git Tag**: `exp/EXP-old`
- **Git Status Dirty**: Yes
"""
        with tempfile.NamedTemporaryFile("w+", suffix=".md", delete=False) as f:
            f.write(content)
            temp_path = Path(f.name)

        try:
            modified = record_run_mod.update_manifest_provenance(
                manifest_path=temp_path,
                commit_sha="1111222233334444",
                tag_name="exp/EXP-2026-999a-02",
            )
            self.assertTrue(modified)
            updated_content = temp_path.read_text(encoding="utf-8")
            self.assertIn("- **Git SHA**: `1111222233334444`", updated_content)
            self.assertIn("- **Git Tag**: `exp/EXP-2026-999a-02`", updated_content)
            self.assertIn("- **Git Status Dirty**: No (Clean at tagging gate)", updated_content)
        finally:
            if temp_path.exists():
                temp_path.unlink()

    def test_record_run_dry_run(self):
        with tempfile.NamedTemporaryFile("w+", suffix=".md", delete=False) as f:
            f.write("# Dummy Manifest\n")
            temp_path = Path(f.name)

        try:
            sha, tag = record_run_mod.record_run(
                manifest_path=temp_path,
                tag_name="exp/EXP-2026-999a-99",
                message="test dry run",
                dry_run=True,
            )
            self.assertEqual(sha, "dry-run-sha")
            self.assertEqual(tag, "exp/EXP-2026-999a-99")
        finally:
            if temp_path.exists():
                temp_path.unlink()

    def test_record_run_nonexistent_manifest(self):
        non_existent = Path("/tmp/does_not_exist_manifest_12345.md")
        with self.assertRaises(FileNotFoundError):
            record_run_mod.record_run(
                manifest_path=non_existent,
                tag_name="exp/EXP-dummy",
                message="dummy",
            )


format_and_lint_path = Path(__file__).resolve().parents[1] / "scripts" / "curation" / "format_and_lint.py"
format_and_lint_mod = load_module_from_path("format_and_lint", format_and_lint_path)


class TestFormatAndLint(unittest.TestCase):
    def test_module_exports(self):
        self.assertTrue(hasattr(format_and_lint_mod, "format_rust"))
        self.assertTrue(hasattr(format_and_lint_mod, "format_markdown"))
        self.assertTrue(hasattr(format_and_lint_mod, "validate_figures"))

    def test_validate_figures_empty_dir(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_root = Path(tmpdir)
            # Should pass when no docs/research/assets exists
            passed = format_and_lint_mod.validate_figures(tmp_root)
            self.assertTrue(passed)


if __name__ == "__main__":
    unittest.main()
