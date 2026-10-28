import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import tools.dev.native_runtime_import as runtime_import


class NativeRuntimeImportTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.source = self.root / "game"
        self.receiver = self.root / "receiver"
        (self.source / "Data").mkdir(parents=True)
        self.receiver.mkdir()
        (self.source / "AzerothSurvivors.exe").write_bytes(b"reviewed executable")
        (self.source / "Data/patch.MPQ").write_bytes(b"archive")
        self.sha = runtime_import.digest(self.source / "AzerothSurvivors.exe")
        self.names = ("AzerothSurvivors.exe", "Data/patch.MPQ")
        self.old_sha122, self.old_names = runtime_import.SHA122, runtime_import.NAMES
        runtime_import.SHA122, runtime_import.NAMES = self.sha, self.names

    def tearDown(self):
        runtime_import.SHA122, runtime_import.NAMES = self.old_sha122, self.old_names
        self.temp.cleanup()

    def test_inspect_requires_exact_executable_and_archive_inventory(self):
        hashes = runtime_import.inspect(self.source, self.sha, self.names)
        self.assertEqual(set(hashes), set(self.names))
        with self.assertRaisesRegex(ValueError, "reviewed 0.122.0 asset set"):
            runtime_import.inspect(self.source, "0" * 64, self.names)
        (self.source / "Data/extra.MPQ").write_bytes(b"extra")
        with self.assertRaisesRegex(ValueError, "inventory"):
            runtime_import.inspect(self.source, self.sha, self.names)

    def test_inspect_rejects_archive_symlinks(self):
        (self.source / "Data/patch.MPQ").unlink()
        (self.source / "Data/patch.MPQ").symlink_to(self.source / "AzerothSurvivors.exe")
        with self.assertRaisesRegex(ValueError, "inventory|regular"):
            runtime_import.inspect(self.source, self.sha, self.names)

    def test_source_executable_symlink_is_rejected_before_hashing(self):
        executable = self.source / "AzerothSurvivors.exe"
        outside = self.root / "outside.exe"
        outside.write_bytes(executable.read_bytes())
        executable.unlink()
        executable.symlink_to(outside)
        with patch.object(runtime_import, "digest", side_effect=AssertionError("Followed source link")):
            with self.assertRaisesRegex(ValueError, "regular official"):
                runtime_import.inspect(self.source, self.sha, self.names)

    def test_matching_target_links_cannot_substitute_for_owned_copies(self):
        hashes = runtime_import.inspect(self.source, self.sha, self.names)
        target = runtime_import.import_runtime(self.source, self.receiver, hashes)
        receipt = "IMPORT-OFFICIAL-01220.json"
        for index, name in enumerate((*self.names, receipt)):
            with self.subTest(name=name):
                destination = target / name
                original = destination.read_bytes()
                outside = self.root / ("matching-external-" + str(index))
                outside.write_bytes(original)
                destination.unlink()
                destination.symlink_to(outside)
                with self.assertRaisesRegex(ValueError, "regular owned"):
                    runtime_import.import_runtime(self.source, self.receiver, hashes)
                self.assertTrue(destination.is_symlink())
                self.assertEqual(outside.read_bytes(), original)
                destination.unlink()
                destination.write_bytes(original)
        self.assertEqual(runtime_import.import_runtime(self.source, self.receiver, hashes), target)

    def test_copy_writes_verified_receipt_and_preserves_matching_target(self):
        hashes = runtime_import.inspect(self.source, self.sha, self.names)
        target = runtime_import.import_runtime(self.source, self.receiver, hashes)
        self.assertEqual((target / "Data/patch.MPQ").read_bytes(), b"archive")
        receipt = json.loads((target / "IMPORT-OFFICIAL-01220.json").read_text())
        self.assertEqual(receipt["files"], hashes)
        self.assertEqual(runtime_import.import_runtime(self.source, self.receiver, hashes), target)

    def test_mismatched_existing_target_is_never_overwritten(self):
        hashes = runtime_import.inspect(self.source, self.sha, self.names)
        target = self.receiver / "runtime-official-01220"
        target.mkdir()
        sentinel = target / "preserve.txt"
        sentinel.write_text("keep")
        with self.assertRaisesRegex(ValueError, "preserving it"):
            runtime_import.import_runtime(self.source, self.receiver, hashes)
        self.assertEqual(sentinel.read_text(), "keep")

    def test_target_symlink_cannot_redirect_the_private_copy(self):
        hashes = runtime_import.inspect(self.source, self.sha, self.names)
        outside = self.root / "outside"
        outside.mkdir()
        sentinel = outside / "preserve.txt"
        sentinel.write_text("keep")
        (self.receiver / "runtime-official-01220").symlink_to(outside, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "escaped paired receiver"):
            runtime_import.import_runtime(self.source, self.receiver, hashes)
        self.assertEqual(sentinel.read_text(), "keep")

    def test_only_122_and_bounded_names_are_accepted(self):
        with self.assertRaisesRegex(ValueError, "Only the reviewed"):
            runtime_import.import_runtime(self.source, self.receiver, {}, "0.121.0")
        with self.assertRaisesRegex(ValueError, "manifest"):
            runtime_import.import_runtime(self.source, self.receiver, {"../outside": "0" * 64})


if __name__ == "__main__":
    unittest.main()
