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

class FocusedShieldEvidence(unittest.TestCase):
 def test_requires_exact_native_shield_cast_recipient_and_break(self):
  host="support121 cast: index=5 spell=17 caster=0 target=1 before=50 result=Ok(true)\nsupport121 observed: index=5 spell=17 caster=0 target=1 before=50 after=60 slots=Ok([1])\nsupport121 shield break: Ok(true)"
  self.assertTrue(all(assess(host,"shield")["checks"].values()))
  for before,after in (("slots=Ok([1])","slots=Ok([0])"),("spell=17","spell=774"),("shield break: Ok(true)","shield break: Ok(false)"),("result=Ok(true)","result=Ok(false)")):
   with self.subTest(before=before):self.assertFalse(all(assess(host.replace(before,after),"shield")["checks"].values()))
