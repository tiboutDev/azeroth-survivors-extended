import copy
import importlib.util
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("xwin_reference", Path(__file__).resolve().parents[1] / "scripts/check_xwin_reference.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class Reference(unittest.TestCase):
    def setUp(self):
        self.reference = {"status": "technically-reviewed-local-reference", "builder": "xwin-msvc",
            "artifact": {"file": "avrt.dll", "game_version": module.selected_client()[0], "sha256": "a" * 64, "size_bytes": 10},
            "cargo_lock_sha256": "b" * 64, "source_fingerprint": {"sha256": "c" * 64, "files": []}, "build_recipe_sha256": "d" * 64}
        self.candidate = copy.deepcopy(self.reference)
        self.candidate.update(status="pending-review", verification={"independent_fresh_builds": 2,
            "target_directories_distinct": True, "artifacts_byte_identical": True, "source_unchanged_before_between_after": True})

    def test_exact_candidate_matches_but_other_builder_and_changed_inputs_do_not(self):
        module.verify(self.reference, self.candidate)
        for field, value in (("builder", "windows-msvc"), ("cargo_lock_sha256", "changed"), ("artifact", {}), ("source_fingerprint", {})):
            changed = dict(self.candidate, **{field: value})
            with self.assertRaises(ValueError):
                module.verify(self.reference, changed)

    def test_single_incremental_build_or_unreviewed_reference_is_rejected(self):
        self.candidate["verification"]["independent_fresh_builds"] = 1
        with self.assertRaises(ValueError): module.verify(self.reference, self.candidate)
        self.reference["status"] = "pending-review"
        with self.assertRaises(ValueError): module.verify(self.reference, self.candidate)
