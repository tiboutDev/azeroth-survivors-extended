import importlib.util
import json
import os
import subprocess
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest import mock


MODULE_PATH = Path(__file__).resolve().parents[1] / "tools/dev/local_artifacts.py"
SPEC = importlib.util.spec_from_file_location("local_artifacts", MODULE_PATH)
local_artifacts = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(local_artifacts)


class LocalArtifactsTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        subprocess.run(["git", "init", "-q"], cwd=self.root, check=True)

    def tearDown(self):
        self.temporary.cleanup()

    def write(self, relative, content):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        return path

    def archive_paths(self, result):
        archive = self.root / result["archive"]
        index = self.root / result["index"]
        return archive, index

    def test_dry_run_only_summarizes_and_apply_archives_exact_evidence_and_binaries(self):
        report = self.write("test-results/run/report.json", b'{"status":"kept"}\n')
        backup = self.write("test-results/repository-cleanup/originals/source.rs", b"original source\x00\xff")
        binary_bytes = bytes(range(256)) * 100
        binary = self.write("target/asx/release/asx.dll", binary_bytes)
        receipt = self.write("target/asx/release/build-receipt.json", b'{"sha256":"abc"}\n')
        cache = self.write("target/debug/deps/unused.rmeta", b"disposable cache")

        dry = local_artifacts.summary(self.root)
        self.assertEqual(dry["mode"], "dry-run")
        self.assertTrue(report.exists())
        self.assertTrue(cache.exists())
        result = local_artifacts.apply(self.root)

        self.assertEqual(result["mode"], "applied")
        self.assertEqual(result["removed_trees"], ["test-results", "target"])
        archive, index_path = self.archive_paths(result)
        self.assertTrue(archive.is_file())
        index = json.loads(index_path.read_text())
        self.assertEqual(index["archive"], result["archive"])
        self.assertEqual({item["path"] for item in index["entries"]}, {
            "test-results/run",
            "test-results/repository-cleanup",
            "test-results/repository-cleanup/originals",
            "target/asx",
            "target/asx/release",
            "test-results/run/report.json",
            "test-results/repository-cleanup/originals/source.rs",
            "target/asx/release/asx.dll",
            "target/asx/release/build-receipt.json",
        })
        self.assertFalse(any("unused.rmeta" in item["path"] for item in index["entries"]))
        with tarfile.open(archive, "r:gz") as tar:
            restored = {member.name: tar.extractfile(member).read() for member in tar if member.isfile()}
        self.assertEqual(restored["test-results/run/report.json"], b'{"status":"kept"}\n')
        self.assertEqual(restored["test-results/repository-cleanup/originals/source.rs"], b"original source\x00\xff")
        self.assertEqual(restored["target/asx/release/asx.dll"], binary_bytes)
        self.assertEqual(restored["target/asx/release/build-receipt.json"], b'{"sha256":"abc"}\n')
        self.assertFalse(cache.exists())

    def test_preserves_original_evidence_time_and_records_archive_checksum(self):
        report = self.write("test-results/old-proof.txt", b"old proof")
        stamp = 1234567890123456789
        os.utime(report, ns=(stamp, stamp))
        result = local_artifacts.apply(self.root)
        archive, index = self.archive_paths(result)
        metadata = json.loads(index.read_text())
        self.assertEqual(metadata["archive_sha256"], local_artifacts.sha256_file(archive))
        entry = next(item for item in metadata["entries"] if item["path"] == "test-results/old-proof.txt")
        self.assertEqual(entry["mtime_ns"], stamp)
        with tarfile.open(archive, "r:gz") as tar:
            member = tar.getmember("test-results/old-proof.txt")
            self.assertEqual(member.pax_headers["mtime"], "1234567890.123456789")
            self.assertEqual(tar.extractfile(member).read(), b"old proof")

    def test_archive_verification_failure_prevents_deletion(self):
        self.write("test-results/evidence.txt", b"proof")
        self.write("target/release/game.exe", b"binary")
        verify = local_artifacts._verify_archive

        def corrupt_then_verify(archive, entries):
            archive.write_bytes(b"corrupt archive")
            verify(archive, entries)

        with mock.patch.object(local_artifacts, "_verify_archive", side_effect=corrupt_then_verify):
            with self.assertRaises(local_artifacts.CleanupError):
                local_artifacts.apply(self.root)
        self.assertTrue((self.root / "test-results/evidence.txt").exists())
        self.assertTrue((self.root / "target/release/game.exe").exists())

    def test_source_change_during_archive_prevents_deletion(self):
        report = self.write("test-results/evidence.txt", b"before")
        self.write("target/release/game.exe", b"binary")
        verify = local_artifacts._verify_archive

        def change_source(archive, entries):
            verify(archive, entries)
            report.write_bytes(b"changed after archive")

        with mock.patch.object(local_artifacts, "_verify_archive", side_effect=change_source):
            with self.assertRaisesRegex(local_artifacts.CleanupError, "inventory changed"):
                local_artifacts.apply(self.root)
        self.assertTrue(report.exists())
        self.assertTrue((self.root / "target/release/game.exe").exists())

    def test_symlinks_are_archived_as_links_and_external_targets_are_untouched(self):
        external = self.root.parent / (self.root.name + "-external")
        external.write_bytes(b"outside")
        results = self.root / "test-results"
        results.mkdir()
        (results / "external-link").symlink_to(external)
        target = self.root / "target"
        target.mkdir()
        (target / "cache-link").symlink_to(external)
        self.write("target/release/game.exe", b"binary")
        try:
            result = local_artifacts.apply(self.root)
            archive, _ = self.archive_paths(result)
            with tarfile.open(archive, "r:gz") as tar:
                links = {member.name: member.linkname for member in tar if member.issym()}
            self.assertEqual(links["test-results/external-link"], str(external))
            self.assertEqual(external.read_bytes(), b"outside")
        finally:
            external.unlink(missing_ok=True)

    def test_tracked_files_are_refused(self):
        tracked = self.write("test-results/tracked.txt", b"tracked")
        subprocess.run(["git", "add", "test-results/tracked.txt"], cwd=self.root, check=True)
        with self.assertRaisesRegex(local_artifacts.CleanupError, "tracked files"):
            local_artifacts.apply(self.root)
        self.assertTrue(tracked.exists())

    def test_existing_archive_is_never_overwritten(self):
        self.write("test-results/evidence.txt", b"proof")
        self.write("target/release/game.exe", b"binary")
        evidence = self.root / ".local/evidence"
        evidence.mkdir(parents=True)
        existing_archive = evidence / "already.tar.gz"
        existing_index = evidence / "already.index.json"
        existing_archive.write_bytes(b"preexisting archive")
        existing_index.write_bytes(b"preexisting index")
        with mock.patch.object(local_artifacts, "_new_paths", return_value=(existing_archive, existing_index)):
            with self.assertRaisesRegex(local_artifacts.CleanupError, "overwrite"):
                local_artifacts.apply(self.root)
        self.assertEqual(existing_archive.read_bytes(), b"preexisting archive")
        self.assertEqual(existing_index.read_bytes(), b"preexisting index")
        self.assertTrue((self.root / "test-results/evidence.txt").exists())

    def test_redirected_evidence_path_is_refused(self):
        external = self.root.parent / (self.root.name + "-outside")
        external.mkdir()
        (self.root / ".local").symlink_to(external, target_is_directory=True)
        try:
            with self.assertRaisesRegex(local_artifacts.CleanupError, "redirected evidence"):
                local_artifacts.summary(self.root)
        finally:
            (self.root / ".local").unlink()
            external.rmdir()


if __name__ == "__main__":
    unittest.main()
