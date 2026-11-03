import sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"tools/dev"))
from spell_block_evidence import assess_talents as assess
class TalentEvidence(unittest.TestCase):
 def test_missing_damage_healing_or_negative_control_fails(self):
  self.assertFalse(all(assess("").values()))
  host="\n".join(f"talent block result: case={c} result=Ok(({100 if c in (2,5)else 0}, Some(20)))"for c in range(1,7))+"\ntalent block native: case=1 source=100 target=101 hp_before=20 hp_after=60 shields=Ok([])\ntalent block native: case=6 source=100 target=101 hp_before=20 hp_after=60 shields=Ok([1])"
  self.assertTrue(all(assess(host).values()))
  self.assertFalse(all(assess(host.replace("case=4 result=Ok((0,","case=4 result=Ok((10,")).values()))
  self.assertFalse(all(assess(host.replace("hp_after=60","hp_after=20")).values()))
