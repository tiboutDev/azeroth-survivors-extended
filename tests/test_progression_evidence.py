"""Negative evidence must not accept delayed catch-up as progression while dead."""
import json,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"tools/dev"))
from progression_evidence import assess
class EvidenceTests(unittest.TestCase):
 def test_empty_and_bank_only_evidence_are_rejected(self):
  self.assertFalse(assess("","")["passed"])
  result=assess("official XP deferred: amount=20\nofficial XP replayed: amount=20","")
  self.assertFalse(result["checks"]["no_deferred_bank_or_replay"])
 def test_revival_level_gain_does_not_count_as_levels_while_dead(self):
  text="\n".join("XP progression state: "+json.dumps({"stage":stage,"actors":[{"phase":phase,"level":level},{"phase":"Playing","level":3}]})for stage,phase,level in[(2,"Downed",1),(2,"Downed",1),(4,"Playing",3)])
  self.assertFalse(assess(text,"")["checks"]["slot_0_levels_advance_while_dead"])
 def test_native_guard_presence_alone_does_not_establish_progression(self):
  result=assess("official downed XP: path=0 raw=1\nofficial downed XP: path=1 raw=1","")
  self.assertTrue(result["checks"]["both_native_shared_guards_observed"])
  self.assertFalse(result["passed"])
 def test_complete_two_cycle_evidence_requires_both_dead_actors_and_live_samples(self):
  lines=["official downed XP: path=0 raw=1","official downed XP: path=1 raw=1","official downed XP: path=3 raw=1","official downed XP: path=6 admitted=true","XP progression complete:"]
  for stage,slot,level,phase in [(2,0,1,"Downed"),(2,0,2,"Downed"),(4,0,2,"Playing"),(6,1,2,"Downed"),(6,1,3,"Downed"),(8,1,3,"Playing")]:
   actors=[{"phase":"Playing","level":level,"xp":0}for _ in range(2)];actors[slot]["phase"]=phase
   lines.append("XP progression state: "+json.dumps({"stage":stage,"actors":actors}))
  audit="AZSX_DIAG "+json.dumps({"kind":"ui.progression","data":{"api":"ok","live":30,"nativeMapFrames":30,"betweenSnapshots":20}})
  host="\n".join(lines+[audit]);guest='XP received HUD: {"level":3}\n'+audit
  self.assertTrue(assess(host,guest)["passed"])
  self.assertFalse(assess(host.replace('"xp": 0}, {"phase": "Playing", "level": 3, "xp": 0','"xp": 10}, {"phase": "Playing", "level": 3, "xp": 0'),guest)["passed"])
  self.assertFalse(assess(host,guest.replace('"betweenSnapshots": 20','"betweenSnapshots": 0'))["passed"])
  self.assertFalse(assess(host.replace('"stage": 6','"stage": 0'),guest)["passed"])
if __name__=="__main__":unittest.main()
