"""The lifecycle observer reads the native counter without rewriting saves."""
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools/dev"))
import native_exit_evidence as evidence
import native_profile_codec as azs_rse1

class NativeExitEvidence(unittest.TestCase):
    def test_native_encrypted_counter_and_ambiguous_rows(self):
        key = bytes(range(32))
        with tempfile.TemporaryDirectory() as folder:
            exe = Path(folder) / "client.exe"
            save = Path(folder) / "characters.txt"
            exe.write_bytes(b"reviewed executable")
            with patch.object(azs_rse1, "key_candidates_from_bytes", return_value=[key]):
                save.write_bytes(azs_rse1.seal("talent|1|999\ntalent|2|50125\n", key))
                before = save.read_bytes()
                self.assertEqual(evidence.class_xp(exe, save, 2), 50125)
                self.assertEqual(save.read_bytes(), before)
                with self.assertRaises(ValueError):
                    evidence.class_xp(exe, save, 11)
                save.write_bytes(azs_rse1.seal("talent|2|50000\ntalent|2|50125\n", key))
                with self.assertRaises(ValueError):
                    evidence.class_xp(exe, save, 2)

if __name__ == "__main__":
    unittest.main()
