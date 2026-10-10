"""Regression tests for fail-closed publication checkpoints."""
import os
from pathlib import Path
import tempfile
import unittest
import zipfile
from unittest.mock import patch

import release as r
import release_workflow as workflow


class PublicationTests(unittest.TestCase):
    def test_core_registry_origins_preserve_shorthand_and_table_requirements(self):
        reader = r.read_toml
        manifest = r.ROOT / "crates" / r.PROJECT / "Cargo.toml"
        for table in (False, True):
            with self.subTest(table=table):
                dependencies = {name: {"version": "0.1.0"} if table else "0.1.0"
                                for name in r.CONFIG["core_dependencies"]}
                with patch.object(r, "read_toml", side_effect=lambda path:
                                  {"dependencies": dependencies} if path == manifest else reader(path)):
                    origins = r.core_dependencies()
                self.assertEqual(set(origins), {"ciglet-rs", "liblrhsmm-rs"})
                for origin in origins.values():
                    self.assertEqual(origin["requirement"], "0.1.0")
                    self.assertEqual(origin["version"], "0.1.0")
                    self.assertEqual(origin["source"], "registry+https://github.com/rust-lang/crates.io-index")
                    self.assertRegex(origin["checksum"], r"^[0-9a-f]{64}$")

    def test_dependency_sources_with_epoch_timestamps_are_preserved(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "source"
            source.mkdir()
            dependency = source / "dependency.rs"
            dependency.write_bytes(b"pub fn example() {}\n")
            os.utime(dependency, (300000000, 300000000))
            destination = root / "sources.zip"
            workflow.archive_sources(source, destination)
            with zipfile.ZipFile(destination) as archive:
                self.assertEqual(archive.read("dependency.rs"), dependency.read_bytes())
                self.assertEqual(archive.getinfo("dependency.rs").date_time[:3], (1980, 1, 1))

    def test_no_token_is_rejected_without_disclosing_values(self):
        with patch.dict(os.environ, {}, clear=True):
            with self.assertRaisesRegex(ValueError, "must configure"):
                workflow.require_token()

    def test_static_token_cannot_publish_formal_npm(self):
        with patch.dict(os.environ, {"NODE_AUTH_TOKEN": "private-test-value"}, clear=True):
            with self.assertRaisesRegex(ValueError, "GitHub OIDC"):
                workflow.stage_npm()

    def test_moving_source_ref_is_rejected(self):
        with patch.dict(os.environ, {"RELEASE_SHA": "main"}, clear=True):
            with self.assertRaisesRegex(ValueError, "full commit SHA"):
                workflow.preflight()

    def test_failed_or_automatic_build_cannot_supply_artifacts(self):
        record = {"conclusion": "success", "event": "workflow_dispatch", "path": ".github/workflows/publish.yml", "head_sha": "a" * 40}
        workflow.validate_run(record, "a" * 40)
        for key, value in (("conclusion", "failure"), ("event", "pull_request"), ("path", "other.yml"), ("head_sha", "b" * 40)):
            with self.subTest(key=key), self.assertRaises(ValueError):
                workflow.validate_run({**record, key: value}, "a" * 40)

    def test_alpha_and_formal_phases_are_distinct(self):
        with patch.object(r, "check"), patch.object(r, "version", return_value="0.1.0-alpha.1"):
            with patch.dict(os.environ, {"RELEASE_SHA": "a" * 40, "RELEASE_PHASE": "npm-stage"}, clear=True):
                with self.assertRaisesRegex(ValueError, "bootstrap"):
                    workflow.preflight()

    def test_workflow_and_source_commits_must_match(self):
        with patch.dict(os.environ, {"RELEASE_SHA": "a" * 40, "GITHUB_SHA": "b" * 40}, clear=True):
            with self.assertRaisesRegex(ValueError, "OIDC provenance"):
                workflow.preflight()

    def test_changed_napi_loader_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "index.js"
            path.write_text("new incompatible generated loader")
            with self.assertRaisesRegex(ValueError, "format changed"):
                r.patch_loader(path)


if __name__ == "__main__":
    unittest.main()
