"""The audit includes Git history and untracked sources, but keeps matches private."""
import importlib.util
import json
import subprocess
import tempfile
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("local_audit", Path(__file__).resolve().parents[1] / "scripts/audit_local_information.py")
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)
gate_spec = importlib.util.spec_from_file_location("hygiene_gate", Path(__file__).resolve().parents[1] / "scripts/hygiene_gate.py")
gate = importlib.util.module_from_spec(gate_spec)
gate_spec.loader.exec_module(gate)


class LocalInformationAudit(unittest.TestCase):
    def test_history_untracked_sources_and_ignored_state_have_distinct_scope(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            def git(*args):
                subprocess.run(["git", "-C", directory, *args], check=True, capture_output=True)
            git("init", "-q")
            git("config", "user.name", "Fixture")
            git("config", "user.email", "fixture@example.invalid")
            private = "/".join(("", "home", "fixture-owner", "checkout", "file"))
            (root / ".gitignore").write_text("local-only.txt\n")
            (root / "old.json").write_text(json.dumps({"source": private}))
            git("add", ".")
            git("commit", "-qm", "Historical fixture")
            (root / "old.json").write_text("{}")
            (root / "new.py").write_text(private)
            (root / "local-only.txt").write_text(private)
            report = audit.audit(root, history=True)
            self.assertEqual([hit["path"] for hit in report["current_findings"]], ["new.py"])
            self.assertTrue(any(hit["path"] == "old.json" for hit in report["historical_findings"]))
            self.assertNotIn(private, json.dumps(report))
            self.assertNotIn("fixture-owner", json.dumps(report))
            self.assertEqual((root / "old.json").read_text(), "{}")

    def test_windows_escaped_and_forward_slash_profiles_are_detected(self):
        for profile in ("fixture-owner", "Fixture Owner", "Équipe Exemple"):
            for separator in ("/", "\\", "\\\\"):
                value = separator.join(("C:", "Users", profile, "project"))
                self.assertIn("windows_profile", [hit["category"] for hit in audit.findings(value.encode())])

    def test_publication_gate_excludes_private_ignored_state_but_rejects_links_and_assets(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "-q", directory], check=True)
            (root / ".gitignore").write_text("private/\n")
            (root / "private").mkdir()
            (root / "private/secret.exe").write_bytes(b"not source")
            (root / "asset.bin").write_bytes(b"forbidden published binary")
            (root / "link").symlink_to(root / "private/secret.exe")
            original = gate.ROOT
            original_references = gate.REVIEWED_ASSET_REFERENCE_LINES
            try:
                gate.ROOT = root
                # This isolated fixture has no reviewed asset-name references.
                gate.REVIEWED_ASSET_REFERENCE_LINES = {}
                hits, _ = gate.scan()
            finally:
                gate.ROOT = original
                gate.REVIEWED_ASSET_REFERENCE_LINES = original_references
            self.assertEqual({hit[0] for hit in hits}, {"asset.bin", "link"})

    def test_feature_module_guard_resolves_the_rust_file_module_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "-q", directory], check=True)
            (root / "src/official").mkdir(parents=True)
            (root / "src/official.rs").write_text('#[cfg(feature = "test-hooks")]\nmod fixture;\n')
            (root / "src/official/fixture.rs").write_text('fn probe(){ let _ = std::env::var("AZCOOP_TEST_FIXTURE"); }\n')
            original = gate.ROOT
            try:
                gate.ROOT = root
                _, hooks = gate.scan()
            finally:
                gate.ROOT = original
            self.assertEqual(len(hooks), 1)
            self.assertTrue(hooks[0][3])
