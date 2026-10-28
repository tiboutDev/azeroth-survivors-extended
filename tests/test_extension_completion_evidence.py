import sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"tools/dev"))
from extension_completion_evidence import assess
class CompletionEvidence(unittest.TestCase):
 def test_missing_native_observations_fail(self):
  for mode in ("support","fatigue"):
   checks=assess("",mode)["checks"]
   self.assertTrue(checks)
   self.assertFalse(any(checks.values()))
 def test_passive_hp_regeneration_does_not_establish_cast_receipts(self):
  text="support121 observed: index=0 spell=635 caster=0 target=1 before=20 after=21 slots=Ok([])"
  checks=assess(text,"support")["checks"]
  self.assertFalse(checks["twelve_native_casts"])
  self.assertFalse(checks["direct_healing_receipts_both_ways"])
