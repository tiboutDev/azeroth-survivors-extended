import unittest,json,struct
from tools.dev.remaining_ui_evidence import assess,hot_text_observed
class RemainingUiTests(unittest.TestCase):
 def fixture(self):
  host='remaining UI native NPCs: Ok([("elite", 1), ("guardian", 2), ("boss", 3)])\n'+'\n'.join(f'remaining UI self HoT: caster={s} spell={139 if s==0 else 774} result=Ok(true)'for s in(0,1))
  view='\n'.join(f'ASX plate witness: guid={g} rank=Some(1) name=Some("Native NPC") retained=true'for g in(1,2,3))
  heal='native heal text: periodic=true event=HealText { caster: 9, target: 9, amount: 12, critical: false } result=Ok(true)'
  return host,[view,view],[view+'\n'+heal,view+'\n'+heal]
 def test_actual_matched_roles_and_periodic_writes(self):self.assertTrue(assess(*self.fixture())['passed'])
 def test_direct_heal_or_missing_view_is_not_periodic_proof(self):
  h,n,l=self.fixture();l[1]=l[1].replace('periodic=true','periodic=false');self.assertFalse(assess(h,n,l)['passed'])
  self.assertFalse(assess(h,n[:1],l)['passed'])
 def test_unrelated_special_names_cannot_substitute_for_witness(self):
  h,n,l=self.fixture();l[1]=l[1].replace('guid=3','guid=4');self.assertFalse(assess(h,n,l)['passed'])
 def test_missing_name_or_rank_zero_cannot_pass(self):
  h,n,l=self.fixture();l[1]=l[1].replace('Some(1)','Some(0)');self.assertFalse(assess(h,n,l)['passed'])

 def test_unknown_ui_rank_requires_actual_role_guid_and_retained_name(self):
  h,n,l=self.fixture();l=[v.replace('rank=Some(1)','rank=None')for v in l];self.assertTrue(assess(h,n,l)['passed'])
  l[1]=l[1].replace('guid=2','guid=5');self.assertFalse(assess(h,n,l)['passed'])

 def test_native_spell_heal_path_must_match_pure_hot_packet_and_queue(self):
  def sample(spell,amount=12,time='1.001'):
   raw=list(struct.pack('<QQII',9,9,spell,12)+bytes(8))
   return '1.000s remaining UI heal packet: periodic=false bytes=Ok('+json.dumps(raw)+')\n'+f'{time}s native heal text: periodic=false event=HealText {{ caster: 9, target: 9, amount: {amount}, critical: false }} result=Ok(true)'
  self.assertTrue(hot_text_observed(sample(139)));self.assertTrue(hot_text_observed(sample(774)))
  self.assertFalse(hot_text_observed(sample(635)));self.assertFalse(hot_text_observed(sample(774,amount=13)));self.assertFalse(hot_text_observed(sample(774,time='2.000')))
