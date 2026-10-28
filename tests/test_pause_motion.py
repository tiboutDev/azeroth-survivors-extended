import sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"tools/dev"))
from pause_evidence import resumed_motion
class MotionAcceptance(unittest.TestCase):
 def line(self,clock,host,guest,phase="Playing"):
  return f"pause fixture snapshot: clock={clock} actors=[(0, Playing, [{host}, 0., 0.]), (1, {phase}, [{guest}, 0., 0.])]\n"
 def test_pre_pause_motion_cannot_hide_a_guest_stuck_after_release(self):
  log=self.line(1,0,0)+self.line(2,10,10)+self.line(12,20,10)+self.line(14,25,10)
  self.assertEqual(resumed_motion(log),{"actor_0_moves_after_group_resume":True,"actor_1_moves_after_group_resume":False})
 def test_both_actors_must_resume_native_motion(self):
  log=self.line(12,20,10)+self.line(14,25,15)
  self.assertTrue(all(resumed_motion(log).values()))
 def test_choices_and_single_sample_cannot_prove_resume(self):
  log=self.line(12,20,10,"Choosing")+self.line(14,25,15)
  self.assertFalse(any(resumed_motion(log).values()))

class MenuAcceptance(unittest.TestCase):
 def evidence(self,drift=False,early_resume=False,stuck=False):
  from pause_evidence import menu_evidence
  rows=["[ INFO ] 10.000s official menu request: open=true role=Host", "[ INFO ] 12.000s official peer menu: slot=1 open=true"]
  for at in (13,15,17,19,21,23,25,27):
   if at==21:rows.append("[ INFO ] 20.000s official menu request: open=false role=Host")
   clock=16 if early_resume and at>20 else 15
   x=1 if drift and at>20 else 0
   rows.extend([f"[ INFO ] {at}.000s pause fixture gate: paused=true",f"[ INFO ] {at}.000s pause fixture snapshot: clock={clock} actors=[(0, Playing, [{x}, 0, 0]), (1, Playing, [0, 0, 0])]"])
  rows.append("[ INFO ] 28.000s official peer menu: slot=1 open=false")
  for at,x in ((30,1),(32,2)):
   rows.extend([f"[ INFO ] {at}.000s pause fixture gate: paused=false",f"[ INFO ] {at}.000s pause fixture snapshot: clock={at-13} actors=[(0, Playing, [{x}, 0, 0]), (1, Playing, [{0 if stuck else x}, 0, 0])]"])
  return menu_evidence(chr(10).join(rows))["checks"]
 def test_first_close_waits_for_the_other_menu_and_both_actors_resume(self):
  self.assertTrue(all(self.evidence().values()))
 def test_positions_or_clock_cannot_advance_under_either_menu(self):
  for key in ("drift","early_resume"):
   self.assertFalse(self.evidence(**{key:True})["menu_clock_and_both_positions_frozen"])
 def test_last_close_does_not_hide_a_guest_that_never_resumes(self):
  self.assertFalse(self.evidence(stuck=True)["menu_actor_1_moves_after_last_close"])
