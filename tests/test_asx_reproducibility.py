"""Reproducibility candidate safety, using mocked builds and no DLL compilation."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location(
    "asx_reproducibility", ROOT / "tools/build/asx_reproducibility.py"
)
repro = importlib.util.module_from_spec(spec)
spec.loader.exec_module(repro)
release_spec = importlib.util.spec_from_file_location(
    "asx_release_binary", ROOT / "tools/build/asx_release_binary.py"
)
release = importlib.util.module_from_spec(release_spec)
release_spec.loader.exec_module(release)


class CandidateWorkflowTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        for relative, content in {
            "Cargo.toml": "[workspace]\n",
            "Cargo.lock": "version = 3\n",
            "build.rs": "fn main() {}\n",
            "resources/windows/avrt-exports.txt": "1 Export\n",
            "resources/windows/version.res": "resource bytes\n",
            "crates/app/Cargo.toml": "[package]\nname='app'\n",
            "crates/app/src/lib.rs": "pub fn answer() -> u32 { 42 }\n",
            "crates/app/build.rs": "fn main() {}\n",
            "crates/app/resources/ids.json": '{"game":"0.125.0"}\n',
            "src/runtime.rs": "pub fn runtime() {}\n",
            "native-ui/interface.lua": "return true\n",
            "asx/supported-games.json": '{"versions":["0.125.0"]}\n',
            "tools/build/asx_release_binary.py": "# reviewed builder source\n",
            "tools/build/asx_reproducibility.py": "# reviewed candidate workflow\n",
            "docs/BUILD.md": "Documentation is outside the build fingerprint.\n",
            "test-results/old.json": "Results are outside the build fingerprint.\n",
            "target/old.dll": "Existing outputs are outside the build fingerprint.\n",
            "validation/old.json": "Validation evidence is outside the build fingerprint.\n",
        }.items():
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
        self.candidate = self.root / "test-results/current-native-candidate.json"

    @staticmethod
    def versions():
        return {
            "rustc": "rustc 1.90.0 (fixture)",
            "cargo": "cargo 1.90.0 (fixture)",
            "cargo_xwin": "cargo-xwin 0.18.0 (fixture)",
            "python": "3.12.0",
            "pefile": "2024.8.26",
        }

    def fake_builder(self, payloads, after_build=None):
        calls = []
        initially_empty = []

        def build(target_dir, receipt_path):
            index = len(calls)
            calls.append(target_dir)
            initially_empty.append(not any(target_dir.iterdir()))
            raw = payloads[index]
            artifact = target_dir / repro.TARGET / "release" / repro.ARTIFACT
            artifact.parent.mkdir(parents=True, exist_ok=True)
            artifact.write_bytes(raw)
            receipt = {
                "schema_version": 1,
                "artifact": {
                    "file": repro.ARTIFACT,
                    "sha256": hashlib.sha256(raw).hexdigest(),
                    "size_bytes": len(raw),
                },
                "game_version": "0.125.0",
                "target": repro.TARGET,
                "package": repro.PACKAGE,
                "profile": "release",
                "locked_dependencies": True,
                "test_hooks": "absent",
                "pe": {"timestamp": 1, "debug_directory_size": 0},
            }
            receipt_path.write_text(json.dumps(receipt), encoding="utf-8")
            if after_build is not None:
                after_build(index)

        return calls, initially_empty, build

    def create(self, builder):
        return repro.create_candidate(
            self.root,
            self.candidate,
            builder=builder,
            versions_provider=self.versions,
        )

    def test_matching_builds_use_distinct_fresh_dirs_and_only_write_pending_candidate(self):
        calls, initially_empty, builder = self.fake_builder([b"normal verified fixture dll", b"normal verified fixture dll"])
        report = self.create(builder)

        self.assertEqual(len(calls), 2)
        self.assertNotEqual(calls[0], calls[1])
        self.assertTrue(all(path.is_dir() for path in calls))
        self.assertEqual(initially_empty, [True, True])
        self.assertEqual(report["status"], "pending-review")
        self.assertFalse(report["approved"])
        self.assertTrue(report["requires_human_approval"])
        self.assertTrue(report["verification"]["artifacts_byte_identical"])
        self.assertEqual(report["builder"], "xwin-msvc")
        self.assertEqual(report["builder_script"], "tools/build/asx_release_binary.py")
        self.assertEqual(report["artifact"]["file"], "avrt.dll")
        encoded = self.candidate.read_text(encoding="utf-8")
        self.assertNotIn(str(self.root.resolve()), encoded)
        self.assertNotIn(str(Path.home().resolve()), encoded)
        self.assertNotIn(str(calls[0]), encoded)
        self.assertNotIn("asx-reproducible-build.json", encoded)
        self.assertEqual(
            report["cargo_lock_sha256"],
            hashlib.sha256((self.root / "Cargo.lock").read_bytes()).hexdigest(),
        )

    def test_source_inventory_hashes_build_inputs_but_not_docs_results_or_outputs(self):
        snapshot = repro.source_snapshot(self.root)
        paths = {item["path"] for item in snapshot["files"]}
        self.assertIn("crates/app/src/lib.rs", paths)
        self.assertIn("crates/app/resources/ids.json", paths)
        self.assertIn("asx/supported-games.json", paths)
        self.assertIn("Cargo.lock", paths)
        self.assertNotIn("docs/BUILD.md", paths)
        self.assertNotIn("test-results/old.json", paths)
        self.assertNotIn("target/old.dll", paths)
        self.assertNotIn("validation/old.json", paths)

    def test_different_artifacts_do_not_write_candidate(self):
        _, _, builder = self.fake_builder([b"artifact A", b"artifact B"])
        with self.assertRaisesRegex(RuntimeError, "builds differ"):
            self.create(builder)
        self.assertFalse(self.candidate.exists())

    def test_source_change_between_builds_does_not_write_candidate(self):
        source = self.root / "src/runtime.rs"

        def mutate_after_first(index):
            if index == 0:
                source.write_text("changed during candidate build\n", encoding="utf-8")

        _, _, builder = self.fake_builder([b"same", b"same"], mutate_after_first)
        with self.assertRaisesRegex(RuntimeError, "between builds"):
            self.create(builder)
        self.assertFalse(self.candidate.exists())

    def test_source_change_after_second_build_does_not_write_candidate(self):
        source = self.root / "src/runtime.rs"

        def mutate_after_second(index):
            if index == 1:
                source.write_text("changed during candidate build\n", encoding="utf-8")

        _, _, builder = self.fake_builder([b"same", b"same"], mutate_after_second)
        with self.assertRaisesRegex(RuntimeError, "after builds"):
            self.create(builder)
        self.assertFalse(self.candidate.exists())

    def test_reserved_validation_paths_and_existing_candidate_are_refused(self):
        calls, _, builder = self.fake_builder([b"same", b"same"])
        for path in (
            self.root / "validation/candidate.json",
            self.root / "validation/asx-reproducible-build.json",
            self.root / "test-results/asx-reproducible-build.json",
        ):
            with self.subTest(path=path), self.assertRaises(ValueError):
                repro.create_candidate(self.root, path, builder=builder, versions_provider=self.versions)
        self.assertEqual(calls, [])

        self.candidate.parent.mkdir(parents=True, exist_ok=True)
        self.candidate.write_text("keep-existing", encoding="utf-8")
        with self.assertRaises(FileExistsError):
            repro.create_candidate(self.root, self.candidate, builder=builder, versions_provider=self.versions)
        self.assertEqual(self.candidate.read_text(encoding="utf-8"), "keep-existing")
        self.assertEqual(calls, [])

    def test_candidate_must_be_under_ignored_test_results(self):
        _, _, builder = self.fake_builder([b"same", b"same"])
        with self.assertRaisesRegex(ValueError, "test-results"):
            repro.create_candidate(
                self.root,
                self.root / "candidate.json",
                builder=builder,
                versions_provider=self.versions,
            )

    def test_committed_export_and_resource_inputs_change_the_fingerprint(self):
        before = repro.source_snapshot(self.root)
        self.assertIn("resources/windows/avrt-exports.txt", {entry["path"] for entry in before["files"]})
        (self.root / "resources/windows/avrt-exports.txt").write_text("2 ChangedExport\n")
        self.assertNotEqual(before, repro.source_snapshot(self.root))
        before = repro.source_snapshot(self.root)
        (self.root / "resources/windows/version.res").write_bytes(b"changed resource")
        self.assertNotEqual(before, repro.source_snapshot(self.root))

    def test_linked_build_inputs_fail_before_any_build(self):
        calls, _, builder = self.fake_builder([b"same", b"same"])
        (self.root / "src/external.rs").symlink_to(self.root / "src/runtime.rs")
        with self.assertRaisesRegex(ValueError, "symbolic"):
            self.create(builder)
        self.assertEqual(calls, [])
        self.assertFalse(self.candidate.exists())

    def test_release_builder_accepts_isolated_target_and_receipt_without_building_here(self):
        target = self.root / "target/fresh-build"
        receipt = self.root / "target/fresh-build/receipt.json"
        with patch.object(release, "build") as build:
            release.main(["--target-dir", str(target), "--receipt", str(receipt)])
            build.assert_called_once_with(target, receipt)

        with patch.object(release, "build") as build:
            release.main([])
            build.assert_called_once_with(release.ROOT / "target/asx-distribution", None)

    def test_redirected_candidate_directory_and_config_ancestor_are_refused(self):
        _, _, builder = self.fake_builder([b"same", b"same"])
        with tempfile.TemporaryDirectory() as external:
            (self.root / "test-results/old.json").unlink()
            (self.root / "test-results").rmdir()
            (self.root / "test-results").symlink_to(external, target_is_directory=True)
            with self.assertRaises(ValueError):
                self.create(builder)
            (self.root / "test-results").unlink()
            (self.root / ".cargo").symlink_to(external, target_is_directory=True)
            (Path(external) / "config.toml").write_text("[build]\n")
            with self.assertRaisesRegex(ValueError, "symbolic"):
                repro.source_snapshot(self.root)

    def test_encoded_rustflags_cannot_replace_the_production_recipe(self):
        with patch.dict(release.os.environ, {"CARGO_ENCODED_RUSTFLAGS": "unreviewed override"}), \
             patch.object(release.subprocess, "check_output", return_value="/fixture/sysroot\n"), \
             patch.object(release.subprocess, "run", side_effect=RuntimeError("stop before build")) as command:
            with self.assertRaisesRegex(RuntimeError, "stop before build"):
                release.build(self.root / "target/probe")
            self.assertNotIn("CARGO_ENCODED_RUSTFLAGS", command.call_args.kwargs["env"])


if __name__ == "__main__":
    unittest.main()
