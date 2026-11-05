"""A release DLL matches its own report; old hashes never gate new releases."""
import copy,hashlib,importlib.util,sys,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"tools/build"))
from azsx_reproducibility import verify_artifact,TARGET
class ArtifactTests(unittest.TestCase):
    def setUp(self):
        self.raw=b"MZ normal release fixture"
        self.report={"status":"verified","builder":"xwin-msvc",
            "artifact":{"file":"avrt.dll","game_version":"0.141.0","target":TARGET,
                        "sha256":hashlib.sha256(self.raw).hexdigest(),"size_bytes":len(self.raw)},
            "verification":{"independent_fresh_builds":2,"target_directories_distinct":True,
                            "artifacts_byte_identical":True,"source_unchanged_before_between_after":True}}
    def test_new_artifact_requires_no_global_reference_or_approval(self):
        verify_artifact(self.report,self.raw,"0.141.0")
        historical=copy.deepcopy(self.report);historical["status"]="pending-review"
        historical.update(approved=False,requires_human_approval=True)
        verify_artifact(historical,self.raw,"0.141.0")
    def test_wrong_dll_game_builder_or_single_build_is_rejected(self):
        for field,value in (("builder","windows-msvc"),("status","failed"),("artifact",{}),
                            ("verification",dict(self.report["verification"],independent_fresh_builds=1))):
            with self.subTest(field=field),self.assertRaises(ValueError):
                verify_artifact(dict(self.report,**{field:value}),self.raw,"0.141.0")
        with self.assertRaises(ValueError):verify_artifact(self.report,b"MZ changed DLL","0.141.0")
        with self.assertRaises(ValueError):verify_artifact(self.report,self.raw,"0.142.0")
    def test_unstable_sources_and_diagnostic_dlls_are_rejected(self):
        for gate in("target_directories_distinct","artifacts_byte_identical","source_unchanged_before_between_after"):
            changed=copy.deepcopy(self.report);changed["verification"][gate]=False
            with self.subTest(gate=gate),self.assertRaises(ValueError):verify_artifact(changed,self.raw,"0.141.0")
        raw=b"MZ AZCOOP_TEST_fixture";report=copy.deepcopy(self.report)
        report["artifact"].update(sha256=hashlib.sha256(raw).hexdigest(),size_bytes=len(raw))
        with self.assertRaisesRegex(ValueError,"normal Windows DLL"):verify_artifact(report,raw,"0.141.0")
