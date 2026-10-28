import importlib.util,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location("pause_evidence",Path(__file__).resolve().parents[1]/"tools/dev/pause_evidence.py");m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class PauseEvidence(unittest.TestCase):
 def sample(self):
  host="pause fixture native cache: true\npause fixture held: kind=cache\n";guest="pause fixture held: kind=cache\n"
  for kind,clock in [("level",1),("cache",4)]:
   for elapsed in (4,6):
    window=f'[("{kind}", {elapsed}, 3)]'
    host+=f"pause fixture gate: paused=true blocked=1 nonmovement=0 windows={window}\n"
    host+=f"pause fixture snapshot: clock={clock} actors=[(0, Playing, [1,2,3]), (1, Choosing, [2,3,4])]\n"
    guest+=f"pause fixture gate: paused=true blocked=1 nonmovement=0 windows={window}\n"
  host+='pause fixture snapshot: clock=8 actors=[(0, Playing, [1,2,3]), (1, Playing, [2,3,4])]\n'
  return [host,guest]
 def test_complete_asymmetric_windows(self):self.assertTrue(all(m.assess_logs(self.sample())["checks"].values()))
 def test_missing_or_premature_resume_cannot_pass(self):
  self.assertFalse(any(m.assess_logs(["",""])["checks"].values()))
  logs=self.sample();logs[0]=logs[0].replace("paused=true","paused=false");self.assertFalse(m.assess_logs(logs)["checks"]["level_waiting_player_blocks_movement"])
 def test_movement_clock_advancement_and_blocking_choices_are_failures(self):
  logs=self.sample();logs[0]=logs[0].replace("nonmovement=0","nonmovement=1");self.assertFalse(m.assess_logs(logs)["checks"]["level_waiting_player_blocks_movement"])
  logs=self.sample();logs[0]=logs[0].replace("clock=1 actors","clock=2 actors",1);self.assertFalse(m.assess_logs(logs)["checks"]["level_mixed_phase_clock_and_positions_frozen"])
  logs=self.sample();logs[0]=logs[0].replace("[1,2,3]","[9,2,3]",1);self.assertFalse(m.assess_logs(logs)["checks"]["level_mixed_phase_clock_and_positions_frozen"])
