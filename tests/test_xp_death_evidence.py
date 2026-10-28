import importlib.util,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location("xp_death_evidence",Path(__file__).resolve().parents[1]/"tools/dev/xp_death_evidence.py");m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Recovery(unittest.TestCase):
 def good(self):
  t=""
  for s in(0,1):t+=f"XP death confirmed: slot={s}\nXP death revived: slot={s}\nofficial XP deferred: slot={s} guid={s+10} amount=7 pending=7\nofficial XP replayed: slot={s} guid={s+10} amount=7\n"
  return t+"XP death state: stage=6 clock=50 actors=[(0,10,Playing,5,4),(1,11,Playing,5,4)]\n"*3
 def test_requires_two_actual_deaths_and_both_replays(self):
  self.assertTrue(all(m.assess(self.good())["checks"].values()));self.assertFalse(m.assess(self.good().replace("XP death confirmed: slot=1","absent"))["checks"]["both_native_deaths"])
 def test_missing_duplicate_or_wrong_identity_replay_fails(self):
  for replacement in ["", "official XP replayed: slot=1 guid=11 amount=8", "official XP replayed: slot=1 guid=99 amount=7"]:
   self.assertFalse(m.assess(self.good().replace("official XP replayed: slot=1 guid=11 amount=7",replacement))["checks"]["all_deferred_xp_replayed_exactly_once"])
 def test_missing_or_unequal_recovery_samples_fail(self):
  self.assertFalse(m.assess("")["checks"]["post_revival_progression_equal"]);self.assertFalse(m.assess(self.good().replace("(1,11,Playing,5,4)","(1,11,Playing,5,3)"))["checks"]["post_revival_progression_equal"])
