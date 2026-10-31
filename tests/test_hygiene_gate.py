"""The publication gate scopes text exceptions without allowing game payloads."""
import importlib.util
import hashlib
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("hygiene_gate", ROOT / "scripts/hygiene_gate.py")
GATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATE)


class HygieneGateExceptions(unittest.TestCase):
    def test_scan_rejects_unknown_references_and_duplicate_reviewed_lines(self):
        lines = (ROOT / "asx/supported-games.json").read_text().splitlines()
        line = next(line for line in lines if GATE.ASSET_NAME_RULE.search(line))
        digest = hashlib.sha256(line.encode()).hexdigest()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "-q", directory], check=True)
            path = root / "manifest.json"
            with patch.object(GATE, "ROOT", root):
                path.write_text(line + "\n")
                with patch.object(GATE, "REVIEWED_ASSET_REFERENCE_LINES", {}):
                    hits, _ = GATE.scan()
                self.assertIn("game asset name", [hit[2] for hit in hits])
                path.write_text(line + "\n" + line + "\n")
                with patch.object(GATE, "REVIEWED_ASSET_REFERENCE_LINES", {path.name: {digest: 1}}):
                    hits, _ = GATE.scan()
                self.assertIn("asset-name reference count mismatch", [hit[2] for hit in hits])

    def test_current_manifest_line_is_approved_by_exact_path_and_hash(self):
        relative = "asx/supported-games.json"
        lines = (ROOT / relative).read_text().splitlines()
        line = next(line for line in lines if GATE.ASSET_NAME_RULE.search(line))
        self.assertTrue(GATE.reviewed_asset_reference(relative, line))
        self.assertIn(relative, GATE.ASSET_REFERENCE_REASONS)

    def test_changed_line_or_copied_line_in_another_file_is_not_approved(self):
        relative = "asx/supported-games.json"
        lines = (ROOT / relative).read_text().splitlines()
        line = next(line for line in lines if GATE.ASSET_NAME_RULE.search(line))
        self.assertFalse(GATE.reviewed_asset_reference("tests/new_manifest.json", line))
        self.assertFalse(GATE.reviewed_asset_reference(relative, line + " "))

    def test_new_and_stale_exception_counts_fail_closed(self):
        self.assertFalse(GATE.reviewed_reference_counts_match({}))
        self.assertFalse(GATE.reviewed_reference_counts_match({("unreviewed", "0" * 64): 1}))

    def test_only_the_exact_committed_windows_resource_bytes_are_allowed(self):
        relative = GATE.RESOURCE_PATH
        raw = (ROOT / relative).read_bytes()
        self.assertTrue(GATE.approved_embedded_resource(relative, raw))
        self.assertFalse(GATE.approved_embedded_resource(relative, raw + b"changed"))
        self.assertFalse(GATE.approved_embedded_resource("res/version.res", raw))
        self.assertFalse(GATE.forbidden_binary(relative, ".res", raw))
        self.assertTrue(GATE.forbidden_binary(relative, ".res", raw + b"changed"))

    def test_only_reviewed_ofl_font_bytes_and_license_are_allowed(self):
        raw = (ROOT / GATE.ASXUI_FONT_PATH).read_bytes()
        self.assertFalse(GATE.forbidden_binary(GATE.ASXUI_FONT_PATH, ".ttf", raw))
        self.assertTrue(GATE.forbidden_binary(GATE.ASXUI_FONT_PATH, ".ttf", raw + b"changed"))
        self.assertTrue(GATE.forbidden_binary("other/font.ttf", ".ttf", raw))
        with tempfile.TemporaryDirectory() as directory, patch.object(GATE, "ROOT", Path(directory)):
            self.assertTrue(GATE.forbidden_binary(GATE.ASXUI_FONT_PATH, ".ttf", raw))
            license_path = Path(directory) / GATE.ASXUI_FONT_LICENSE
            license_path.parent.mkdir(parents=True)
            license_path.write_bytes(b"unreviewed license")
            self.assertTrue(GATE.forbidden_binary(GATE.ASXUI_FONT_PATH, ".ttf", raw))

    def test_game_asset_payloads_remain_forbidden(self):
        extension = "mp" + "q"
        self.assertTrue(GATE.forbidden_binary("Data/patch." + extension, "." + extension))
        extension = "b" + "lp"
        self.assertTrue(GATE.forbidden_binary("icon." + extension, "." + extension))


if __name__ == "__main__":
    unittest.main()
