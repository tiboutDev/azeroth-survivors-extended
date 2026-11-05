"""Indirect field wiring must fail before a DLL or game trial is considered."""
import copy
import unittest
from tools.dev.field_audit import audit, current_report

class FieldAuditTests(unittest.TestCase):
 def setUp(self):
  self.values={'GAME_RESOLVED_FIELDS':[(0x10,0x17,0x100)],'ACTOR_RESOLVED_FIELDS':[(0x20,0x23,0x200)],
               'FIELD117_RESOLVED_FIELDS':[],'FIELD1201_RESOLVED_FIELDS':[],'POOL':496,'KEYS':[0x10,0x11]}
 def contract(self,argument='0x10',target='game_field'):
  args=['host',argument]if target=='row_field'else[argument]
  return dict(id='owner',adapter_source_sha256='source-A',access_inventory_complete=True,
              native_accesses=[dict(kind='field',target=target,arguments=args,line=9)])
 def review(self,contract,**use):
  edge=contract['native_accesses'][0]
  return dict(owner='owner',source_sha256='source-A',uses=[dict(target=edge['target'],arguments=edge['arguments'],**use)])
 def test_literal_and_selected_constant_accesses_require_one_mapping(self):
  for arg in ['0x10','v::KEYS[0]','crate::current::KEYS[1]']:
   r=audit([self.contract(arg)],self.values,[]);self.assertTrue(r['passed'],r['errors'])
  self.assertFalse(audit([self.contract('0x18')],self.values,[])['passed'])
 def test_indirect_access_cannot_disappear_from_the_required_review(self):
  c=self.contract('offset');self.assertFalse(audit([c],self.values,[])['passed'])
  review=self.review(c,cases=[dict(namespace='GAME_RESOLVED_FIELDS',key=0x10),dict(namespace='ACTOR_RESOLVED_FIELDS',key=0x20)])
  self.assertTrue(audit([c],self.values,[review])['passed'])
  missing=copy.deepcopy(self.values);missing['ACTOR_RESOLVED_FIELDS']=[]
  self.assertFalse(audit([c],missing,[review])['passed'])
 def test_changed_closure_body_and_removed_call_invalidate_old_reviews(self):
  c=self.contract('offset');review=self.review(c,cases=[dict(namespace='GAME_RESOLVED_FIELDS',key=0x10)])
  c['adapter_source_sha256']='source-B';self.assertFalse(audit([c],self.values,[review])['passed'])
  c['native_accesses']=[];self.assertFalse(audit([c],self.values,[review])['passed'])
 def test_retired_branch_stays_excluded_only_for_its_reviewed_guard(self):
  c=self.contract('0x190');review=self.review(c,status='retired',guard=dict(constant='POOL',value=496))
  self.assertTrue(audit([c],self.values,[review])['passed'])
  self.values['POOL']=0;self.assertFalse(audit([c],self.values,[review])['passed'])
 def test_old_inventory_without_ast_data_cannot_claim_coverage(self):
  c=self.contract();c['access_inventory_complete']=False
  self.assertFalse(audit([c],self.values,[])['passed'])
 def test_explicit_private_context_never_uses_the_host_namespace(self):
  c=self.contract('0x20','row_field');c['native_accesses'][0]['arguments'][0]='false'
  self.assertTrue(audit([c],self.values,[])['passed'])
  c['native_accesses'][0]['arguments'][0]='true'
  self.assertFalse(audit([c],self.values,[])['passed'])
 def test_current_source_has_no_unreviewed_or_missing_field_uses(self):
  report=current_report();self.assertTrue(report['passed'],report['errors'][:10])

if __name__=='__main__':unittest.main()
