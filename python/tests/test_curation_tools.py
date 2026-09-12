"""Unit tests for curation scripts (record_run.py and format_and_lint.py)."""

import importlib.util
from pathlib import Path
import subprocess
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock


def load_module_from_path(module_name: str, file_path: Path):
    spec = importlib.util.spec_from_file_location(module_name, file_path)
    if spec is None or spec.loader is None:
        raise ImportError(f"Could not load module from {file_path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


record_run_path = Path(__file__).resolve(
).parents[1] / "scripts" / "curation" / "record_run.py"
record_run_mod = load_module_from_path("record_run", record_run_path)


def run_git(root: Path, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *args],
        cwd=root,
        capture_output=True,
        text=True,
        check=check,
    )


def initialize_git_repo(root: Path) -> tuple[Path, Path]:
    run_git(root, "init", "-q")
    run_git(root, "config", "user.name", "Test User")
    run_git(root, "config", "user.email", "test@example.invalid")
    manifest = root / "manifest.md"
    experiment = root / "experiment.txt"
    manifest.write_text(
        """# Experiment Run Manifest

| Dimension | Specification |
| :--- | :--- |
| Git Commit SHA | `pending` |
| Git Tag | `pending` |
| Git Status Dirty? | Yes |
""",
        encoding="utf-8",
    )
    experiment.write_text("baseline\n", encoding="utf-8")
    run_git(root, "add", "manifest.md", "experiment.txt")
    run_git(root, "commit", "-q", "-m", "baseline")
    return manifest, experiment


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
            self.assertIn(
                "| Git Commit SHA | `abcd1234efgh5678` |", updated_content)
            self.assertIn("| Git Tag | `exp/EXP-2026-999a-01` |",
                          updated_content)
            self.assertIn(
                "| Git Status Dirty? | No (Clean at tagging gate) |", updated_content)
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
            self.assertIn(
                "- **Git Tag**: `exp/EXP-2026-999a-02`", updated_content)
            self.assertIn(
                "- **Git Status Dirty**: No (Clean at tagging gate)", updated_content)
        finally:
            if temp_path.exists():
                temp_path.unlink()

    def test_update_manifest_rejects_partial_or_duplicate_schema(self):
        with tempfile.NamedTemporaryFile("w+", suffix=".md", delete=False) as file:
            file.write("| Git Status Dirty? | Yes |\n")
            partial_path = Path(file.name)
        try:
            with self.assertRaisesRegex(ValueError, "exactly one complete provenance schema"):
                record_run_mod.update_manifest_provenance(
                    partial_path, "abc123", "exp/test")
        finally:
            partial_path.unlink(missing_ok=True)

        duplicate = """| Git Commit SHA | pending |
| Git Commit SHA | example |
| Git Tag | pending |
| Git Status Dirty? | Yes |
"""
        with self.assertRaisesRegex(ValueError, "exactly one complete provenance schema"):
            record_run_mod.render_manifest_provenance(
                duplicate, "abc123", "exp/test")

    def test_record_run_dry_run(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            manifest, experiment = initialize_git_repo(root)
            experiment.write_text("changed\n", encoding="utf-8")
            sha, tag = record_run_mod.record_run(
                manifest_path=manifest,
                tag_name="exp/EXP-2026-999a-99",
                message="test dry run",
                artifact_paths=[experiment],
                dry_run=True,
            )
            self.assertEqual(sha, "dry-run-sha")
            self.assertEqual(tag, "exp/EXP-2026-999a-99")

    def test_record_run_nonexistent_manifest(self):
        non_existent = Path("/tmp/does_not_exist_manifest_12345.md")
        with self.assertRaises(FileNotFoundError):
            record_run_mod.record_run(
                manifest_path=non_existent,
                tag_name="exp/EXP-dummy",
                message="dummy",
                artifact_paths=[],
            )

    def test_record_run_rejects_changes_outside_scope(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            manifest, experiment = initialize_git_repo(root)
            experiment.write_text("changed\n", encoding="utf-8")
            unrelated = root / "unrelated.txt"
            unrelated.write_text("do not commit\n", encoding="utf-8")

            with self.assertRaisesRegex(ValueError, "outside the authorized paths"):
                record_run_mod.record_run(
                    manifest_path=manifest,
                    tag_name="exp/test-scope",
                    message="test scope",
                    artifact_paths=[experiment],
                )

            self.assertIn("?? unrelated.txt", run_git(
                root, "status", "--short").stdout)
            self.assertEqual(
                run_git(root, "tag", "-l", "exp/test-scope").stdout, "")

    def test_record_run_checks_tags_in_manifest_repository(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            manifest, experiment = initialize_git_repo(root)
            run_git(root, "tag", "-a", "exp/existing", "-m", "existing")

            with self.assertRaisesRegex(ValueError, "already exists"):
                record_run_mod.record_run(
                    manifest_path=manifest,
                    tag_name="exp/existing",
                    message="test existing tag",
                    artifact_paths=[experiment],
                )

    def test_record_run_rejects_invalid_tag_without_changing_history(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            manifest, experiment = initialize_git_repo(root)
            experiment.write_text("changed\n", encoding="utf-8")
            original_head = run_git(root, "rev-parse", "HEAD").stdout.strip()
            original_manifest = manifest.read_text(encoding="utf-8")

            with self.assertRaisesRegex(ValueError, "Invalid git tag name"):
                record_run_mod.record_run(
                    manifest_path=manifest,
                    tag_name="bad tag",
                    message="invalid tag",
                    artifact_paths=[experiment],
                )

            self.assertEqual(
                run_git(root, "rev-parse", "HEAD").stdout.strip(), original_head)
            self.assertEqual(manifest.read_text(
                encoding="utf-8"), original_manifest)
            self.assertEqual(run_git(root, "tag", "-l").stdout, "")

    def test_record_run_symlink_scope_does_not_authorize_target(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            manifest, _ = initialize_git_repo(root)
            target = root / "actual"
            target.mkdir()
            data = target / "data.txt"
            data.write_text("baseline\n", encoding="utf-8")
            link = root / "scope-link"
            link.symlink_to(target.name, target_is_directory=True)
            run_git(root, "add", "actual/data.txt", "scope-link")
            run_git(root, "commit", "-q", "-m", "add symlink")
            data.write_text("changed\n", encoding="utf-8")

            with self.assertRaisesRegex(ValueError, "outside the authorized paths"):
                record_run_mod.record_run(
                    manifest_path=manifest,
                    tag_name="exp/symlink-scope",
                    message="symlink scope",
                    artifact_paths=[link],
                )

    def test_record_run_rejects_repository_root_scope(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            manifest, _ = initialize_git_repo(root)
            with self.assertRaisesRegex(ValueError, "Repository root"):
                record_run_mod.record_run(
                    manifest_path=manifest,
                    tag_name="exp/root-scope",
                    message="root scope",
                    artifact_paths=[root],
                    dry_run=True,
                )

    def test_record_run_tags_provenance_commit_with_reachable_execution_sha(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            manifest, experiment = initialize_git_repo(root)
            experiment.write_text("changed\n", encoding="utf-8")

            final_sha, tag = record_run_mod.record_run(
                manifest_path=manifest,
                tag_name="exp/test-run",
                message="test run",
                artifact_paths=[experiment],
            )

            content = manifest.read_text(encoding="utf-8")
            match = record_run_mod.re.search(
                r"\| Git Commit SHA \| `([0-9a-f]+)` \|", content)
            self.assertIsNotNone(match)
            execution_sha = match.group(1)
            tag_sha = run_git(root, "rev-parse",
                              f"{tag}^{{commit}}").stdout.strip()
            parent_sha = run_git(
                root, "rev-parse", f"{tag}^{{commit}}^").stdout.strip()

            self.assertEqual(final_sha, tag_sha)
            self.assertEqual(execution_sha, parent_sha)
            self.assertNotEqual(execution_sha, final_sha)
            self.assertEqual(
                run_git(root, "merge-base", "--is-ancestor",
                        execution_sha, tag_sha, check=False).returncode,
                0,
            )
            self.assertEqual(run_git(root, "status", "--porcelain").stdout, "")

    def test_record_run_failure_before_ref_publication_restores_manifest_and_refs(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            manifest, experiment = initialize_git_repo(root)
            experiment.write_text("changed\n", encoding="utf-8")
            original_head = run_git(root, "rev-parse", "HEAD").stdout.strip()
            original_manifest = manifest.read_text(encoding="utf-8")

            with (
                mock.patch.object(
                    record_run_mod,
                    "create_annotated_tag_object",
                    side_effect=RuntimeError("injected tag-object failure"),
                ),
                self.assertRaisesRegex(
                    RuntimeError, "injected tag-object failure"),
            ):
                record_run_mod.record_run(
                    manifest_path=manifest,
                    tag_name="exp/injected-failure",
                    message="injected failure",
                    artifact_paths=[experiment],
                )

            self.assertEqual(
                run_git(root, "rev-parse", "HEAD").stdout.strip(), original_head)
            self.assertEqual(run_git(root, "tag", "-l").stdout, "")
            self.assertEqual(manifest.read_text(
                encoding="utf-8"), original_manifest)
            self.assertEqual(
                run_git(root, "status", "--short").stdout, " M experiment.txt\n")

    def test_record_run_force_tag_replaces_existing_tag_atomically(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            manifest, experiment = initialize_git_repo(root)
            experiment.write_text("first\n", encoding="utf-8")
            _, tag = record_run_mod.record_run(
                manifest_path=manifest,
                tag_name="exp/force",
                message="first run",
                artifact_paths=[experiment],
            )
            first_tag_object = run_git(
                root, "rev-parse", f"refs/tags/{tag}").stdout.strip()

            experiment.write_text("second\n", encoding="utf-8")
            final_sha, _ = record_run_mod.record_run(
                manifest_path=manifest,
                tag_name=tag,
                message="second run",
                artifact_paths=[experiment],
                force_tag=True,
            )
            second_tag_object = run_git(
                root, "rev-parse", f"refs/tags/{tag}").stdout.strip()
            second_tag_commit = run_git(
                root, "rev-parse", f"{tag}^{{commit}}").stdout.strip()

            self.assertNotEqual(first_tag_object, second_tag_object)
            self.assertEqual(final_sha, second_tag_commit)
            self.assertEqual(run_git(root, "status", "--porcelain").stdout, "")


format_and_lint_path = Path(__file__).resolve(
).parents[1] / "scripts" / "curation" / "format_and_lint.py"
format_and_lint_mod = load_module_from_path(
    "format_and_lint", format_and_lint_path)


class TestFormatAndLint(unittest.TestCase):
    def test_module_exports(self):
        self.assertTrue(hasattr(format_and_lint_mod, "format_rust"))
        self.assertTrue(hasattr(format_and_lint_mod, "format_markdown"))
        self.assertTrue(hasattr(format_and_lint_mod, "validate_figures"))

    def test_validate_figures_empty_dir(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_root = Path(tmpdir)
            passed = format_and_lint_mod.validate_figures(tmp_root)
            self.assertTrue(passed)

    def test_validate_figures_fails_when_validator_is_missing(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_root = Path(tmpdir)
            assets = tmp_root / "docs" / "research" / "assets"
            assets.mkdir(parents=True)
            (assets / "figure.svg").write_text("<svg />", encoding="utf-8")

            self.assertFalse(format_and_lint_mod.validate_figures(tmp_root))

    def test_missing_formatters_fail_closed(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            with mock.patch.object(format_and_lint_mod.shutil, "which", return_value=None):
                self.assertFalse(format_and_lint_mod.format_rust(root))
                self.assertFalse(format_and_lint_mod.format_markdown(root))

    def test_full_run_validates_figures_by_default(self):
        args = SimpleNamespace(
            markdown_only=False,
            rust_only=False,
            figures=False,
            no_figures=False,
            check=True,
        )
        with (
            mock.patch.object(format_and_lint_mod,
                              "parse_args", return_value=args),
            mock.patch.object(format_and_lint_mod,
                              "format_rust", return_value=True),
            mock.patch.object(format_and_lint_mod,
                              "format_markdown", return_value=True),
            mock.patch.object(format_and_lint_mod, "validate_figures", return_value=True) as validate,
        ):
            format_and_lint_mod.main()

        validate.assert_called_once_with(mock.ANY, strict=True)

    def test_format_selectors_are_mutually_exclusive(self):
        with mock.patch.object(
            format_and_lint_mod.sys,
            "argv",
            ["format_and_lint.py", "--rust-only", "--markdown-only"],
        ):
            with self.assertRaises(SystemExit) as raised:
                format_and_lint_mod.parse_args()

        self.assertEqual(raised.exception.code, 2)


if __name__ == "__main__":
    unittest.main()
