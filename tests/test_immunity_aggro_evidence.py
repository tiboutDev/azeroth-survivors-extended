import sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"tools/dev"))
from immunity_aggro_evidence import assess
class Evidence(unittest.TestCase):
 def fixture(self):
  lines=[]
  for i,spell in enumerate([642,20580,7744,1856]):
   slot=i%2;target="None"if i==1 else f"Some({1-slot})"
   lines.extend([f"immunity fixture begin: case={i} slot={slot} spell={spell} active=true old=Some({slot}) cleared=None",f"immunity fixture retarget: case={i} target={target} expected={1-slot} restored=true effective=Some(0.0) retaunt=Some({slot})",f"immunity fixture held: case={i} owner=Some({slot}) expected={slot}",f"immunity fixture expired: case={i} active=false owner=Some({slot}) expected={slot}",f"official taunt pursuit: enemy=10 caster={slot} position=[0,0,0]"])
  return "\n".join(lines)
 def test_cleared_target_during_reward_barrier_is_valid_exclusion(self):self.assertTrue(assess(self.fixture())["passed"])
 def test_targeting_immune_or_missing_native_immunity_cannot_pass(self):
  self.assertFalse(assess(self.fixture().replace("case=0 target=Some(1)","case=0 target=Some(0)"))["passed"])
  self.assertFalse(assess(self.fixture().replace("effective=Some(0.0)","effective=Some(1.0)"))["passed"])
 def test_missing_case_or_unrestored_pool_cannot_pass(self):
  self.assertFalse(assess(self.fixture().replace("restored=true","restored=false"))["passed"])
  self.assertFalse(assess(self.fixture().replace("case=3","case=4"))["passed"])
if __name__=="__main__":unittest.main()
