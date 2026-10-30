import json,unittest,struct
from tools.dev.coverage_evidence import guardian,world
class CoverageEvidenceTests(unittest.TestCase):
 def test_kill_without_offer_is_not_reward_proof(self):
  self.assertFalse(guardian(['1.0s coverage native guardian killed: Ok(Some(9))',''])['passed'])
 def test_duplicate_or_missing_guest_offer_fails(self):
  row='2.0s coverage native choices: '+json.dumps(dict(why='guardian',offer=1,choices=[{},{},{}]))
  host='1.0s coverage native guardian killed: Ok(Some(9))\n'+row
  self.assertTrue(guardian([host,row])['passed'])
  self.assertFalse(guardian([host,row+'\n'+row])['passed'])
  self.assertFalse(guardian([host,''])['passed'])
 def test_queue_success_alone_does_not_prove_native_layout(self):
  self.assertFalse(world(['native heal text: periodic=false event=HealText { amount: 4 } result=Ok(true)']*2)['passed'])
 def test_wrong_scale_or_missing_periodic_draw_fails(self):
  raw=list(struct.pack('<13f',*([10.]*13)))
  scaled=[21.]+[7.]*9+[10.,7.,7.]
  text=f'Native plate layout: index=0 original={raw} scaled={scaled}\n'+'\n'.join('native heal text: periodic='+kind+' event=HealText { amount: 4 } result=Ok(true)'for kind in('true','false'))
  self.assertTrue(world([text,text])['passed'])
  self.assertFalse(world([text,text.replace('21.0','20.0')])['passed'])
  self.assertFalse(world([text,text.replace('periodic=true','periodic=false')])['passed'])
