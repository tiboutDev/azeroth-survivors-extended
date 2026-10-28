import importlib.util,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location("trial_limits",Path(__file__).resolve().parents[1]/"tools/dev/trial_limits.py");m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Limits(unittest.TestCase):
 def test_sparse_affinity_stays_within_the_current_process_allowance(self):
  for available in (1,3,0xffff,0xffffffff,0b1010101010101010):
   mask=m.affinity_mask(available);self.assertEqual(mask&available,mask);self.assertEqual(mask.bit_count(),max(1,available.bit_count()//5));self.assertGreater(mask,0)
 def test_invalid_budget_or_empty_mask_is_refused(self):
  for available,budget in ((0,20),(3,0),(3,100)):
   with self.assertRaises(ValueError):m.affinity_mask(available,budget)
 def test_visual_capture_is_not_a_per_frame_default(self):
  self.assertEqual(m.environment()["WOW_ROGUE_SHOTS_EVERY"],"15")
  self.assertNotIn("WOW_NO_FRAME_PACE",m.environment())
if __name__=="__main__":unittest.main()
