"""Checkpoint safety and bounded offline analysis, without private executables."""
import contextlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools/dev"))
try:
    import compare_official as comparison
except ImportError:
    comparison = None


@unittest.skipIf(comparison is None, "Offline PE dependencies unavailable")
class OfflineComparison(unittest.TestCase):
    def test_inventory_includes_native_menu_paths_without_member_offsets(self):
        root = Path(__file__).resolve().parents[1]
        rows = comparison.descriptor_inventory(root / "crates/asx-game/src/bindings/official125.rs")
        for address in (0x5b74c30, 0x5b6c660, 0x8eafa, 0x8eb12, 0x2829765, 0x250192, 0x53ac7c2, 0x53ae60f, 0xa7c22f):
            self.assertIn(address, rows)
        for member_or_opcode in (0x10, 0x160, 0x4e8, 0xa18, 0x11bd, 0x1a78, 0x7a60):
            self.assertNotIn(member_or_opcode, rows)

    def images(self):
        class Image:
            def __init__(self, sha, starts):
                self.sha = sha
                self.ends = {start: start + 2 for start in starts}

            def owner(self, address):
                return address if address in self.ends else None

            def ins(self, start):
                return list(comparison.MD.disasm(b"\x90\xc3", start))

        return Image("old", [0x1000, 0x2000]), Image("new", [0x1100, 0x2100])

    def test_batches_resume_without_repeating_completed_or_failed_owners(self):
        old, new = self.images()
        needed = {0x1000: ["first"], 0x2000: ["second"]}
        with patch.object(comparison, "candidates", side_effect=lambda a, b, start: (start + 0x100, 1.0, "relocations-only")) as candidate:
            with contextlib.redirect_stdout(io.StringIO()):
                first = list(comparison.compare(old, new, needed, 1))[-1]
                # Exercise the real JSON round-trip (integer dictionary keys become strings).
                restored = json.loads(json.dumps(first))
                final = list(comparison.compare(old, new, needed, 1, restored))[-1]
                self.assertEqual(list(comparison.compare(old, new, needed, 1, final)), [])
            self.assertEqual(candidate.call_count, 2)
        self.assertEqual(final["pending_owners"], [])
        self.assertEqual(final["unresolved"], [])
        self.assertEqual(final["mapping"][0x1000], 0x1100)
        self.assertEqual(final["mapping"][0x2000], 0x2100)
        failed = dict(restored, failures={str(0x2000): "ambiguous"})
        self.assertEqual(list(comparison.compare(old, new, needed, 1, failed)), [])
        self.assertFalse(final["approved"])

    def test_owner_filter_preserves_other_pending_work(self):
        old, new = self.images()
        needed = {0x1000: ["first"], 0x2000: ["second"]}
        with patch.object(comparison, "candidates", return_value=(0x2100, 1.0, "relocations-only")) as candidate:
            with contextlib.redirect_stdout(io.StringIO()):
                result = list(comparison.compare(old, new, needed, 4, only_owners=[0x2000]))[-1]
        self.assertEqual(candidate.call_args.args[2], 0x2000)
        self.assertEqual(result["pending_owners"], [0x1000])
        self.assertEqual(result["unresolved"], [0x1000])

    def test_call_reference_does_not_approve_an_unreviewed_callee(self):
        old, new = self.images()
        old.ends = {0x1000: 0x1006, 0x3000: 0x3002}
        new.ends = {0x1100: 0x1106, 0x5000: 0x5002}
        old.ins = lambda start: list(comparison.MD.disasm(b"\xe8" + (0x3000 - start - 5).to_bytes(4, "little", signed=True) + b"\xc3", start))
        new.ins = lambda start: list(comparison.MD.disasm(b"\xe8" + (0x5000 - start - 5).to_bytes(4, "little", signed=True) + b"\xc3", start))
        needed = {0x1000: ["caller"], 0x3000: ["unreviewed callee"]}
        with patch.object(comparison, "candidates", return_value=(0x1100, 1.0, "relocations-only")):
            with contextlib.redirect_stdout(io.StringIO()):
                result = list(comparison.compare(old, new, needed, 1))[-1]
        self.assertEqual(result["references"]["12288"], [0x5000])
        self.assertNotIn(0x3000, result["mapping"])
        self.assertIn(0x3000, result["unresolved"])
        self.assertIn(0x3000, result["pending_owners"])
        self.assertFalse(result["approved"])

    @unittest.skipUnless(sys.platform == "linux", "Linux process address-space ceiling")
    def test_memory_budget_stops_oversized_allocation_in_child_process(self):
        import subprocess
        code = (
            "import sys; sys.path.insert(0, sys.argv[1]); "
            "from compare_official import memory_budget; memory_budget(256); "
            "\ntry:\n bytearray(512*1024*1024)\nexcept MemoryError:\n raise SystemExit(0)\nraise SystemExit(1)"
        )
        result = subprocess.run([sys.executable, "-c", code, str(Path(comparison.__file__).parent)], capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_checkpoint_rejects_changed_images_descriptor_or_approval(self):
        old, new = self.images()
        needed = {0x1000: ["first"]}
        checkpoint = dict(mapping_policy=comparison.MAPPING_POLICY, old_sha256="old", new_sha256="new", needed={"4096": ["first"]}, approved=False, offline_only=True)
        for key, value in [("mapping_policy", "legacy"), ("old_sha256", "other"), ("new_sha256", "other"), ("needed", {"4096": ["changed"]}), ("approved", True), ("offline_only", False)]:
            with self.subTest(key=key), self.assertRaises(ValueError):
                comparison.restore_checkpoint(old, new, needed, dict(checkpoint, **{key: value}))

    def test_failed_serialization_preserves_previous_checkpoint(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "evidence.json"
            comparison.write_checkpoint(path, {"complete": 1})
            original = path.read_bytes()
            with patch.object(comparison.json, "dumps", side_effect=MemoryError):
                with self.assertRaises(MemoryError):
                    comparison.write_checkpoint(path, {"complete": 2})
            self.assertEqual(path.read_bytes(), original)
