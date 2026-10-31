"""Reject missing Crystal Resonance gameplay witnesses in core trials."""
import sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"tools/dev"))
from crystal_core_evidence import assess_core
class CrystalCoreEvidence(unittest.TestCase):
 def test_requires_claims_all_colors_native_values_and_retired_amulet(self):
  text="amulet gems retired\ncrystal collected: event=2 slot=0 owner=11 color=0 counts=[1, 1, 1, 1]\ncrystal native stats: slot=0 owner=11 counts=[1, 1, 1, 1] base=[1, 1, 100, 0, 0, 0, 0, 1] actual=Ok([1.1, 0.95238095, 115, 0, 0, 0, 0, 1.1]) stable=true restored=true"
  self.assertTrue(assess_core(text,1)["passed"])
  for broken in (text.replace("crystal collected:","unrelated:"),text.replace("[1, 1, 1, 1]","[1, 1, 1, 0]"),text.replace("restored=true","restored=false"),text+"\ngem locked: slot=0",text.replace("1.1, 0.95238095","1.5, 0.95238095")):
   self.assertFalse(assess_core(broken,1)["passed"])
  self.assertFalse(assess_core(text,2)["passed"])
if __name__=="__main__":unittest.main()
