import json,tempfile,unittest,subprocess,sys,re
from unittest.mock import patch
from pathlib import Path
from tools.dev import asx_dev
from tools.dev import binding_registry,feature_catalog,compatibility,scenario_suites,diagnostic_records,artifact_record
class WorkflowTests(unittest.TestCase):
 def test_isolated_python_entrypoint_can_check_without_game(self):
  root=Path(__file__).resolve().parents[1]
  completed=subprocess.run([sys.executable,"-I","-B",str(root/"tools/dev/asx_dev.py"),"check"],capture_output=True,text=True)
  self.assertEqual(completed.returncode,0,completed.stderr)
 def test_native_flags_and_model_only_are_distinct(self):
  records=[dict(kind="ui.supply_toast",data=dict(exists=True,visible=1,shown=1,width=200,height=40,title="Found",reward="Item",icon="icon"))]
  evidence=diagnostic_records.assess_events(dict(records=records,checks=dict(valid_stream=True)))
  self.assertTrue(evidence["checks"]["toast_rendered"])
  self.assertFalse(evidence["checks"]["progress_rendered"])
 def test_proof_identity_is_scoped_to_its_own_scenario(self):
  config=scenario_suites.load_suites()
  with patch.object(scenario_suites,'load_suites',return_value=config):
   before=asx_dev.validation_fingerprint('solo-events')
   config['suites']['coop-normal']['seconds']+=1
   self.assertEqual(before,asx_dev.validation_fingerprint('solo-events'))
   config['suites']['solo-events']['seconds']+=1
   self.assertNotEqual(before,asx_dev.validation_fingerprint('solo-events'))
 def test_proof_inputs_exclude_unrelated_tools_but_include_executed_fixtures(self):
  files=asx_dev.validation_inputs('solo-events')
  self.assertIn('tools/dev/extensions_trial.py',files)
  self.assertIn('tools/dev/native_profile_codec.py',files)
  self.assertIn('tools/dev/fixtures/party-talents115.json',files)
  self.assertIn('tools/dev/diagnostics/observer.lua',files)
  self.assertNotIn('tools/dev/import_official.py',files)
  self.assertNotIn('tools/dev/build-windows.sh',files)
 def test_inventory_contains_native_gold_after_test_module_and_precise_links(self):
  contracts=compatibility.inventory()['contracts']
  gold=next(c for c in contracts if c['method']=='event_credit_gold')
  self.assertEqual(gold['features'],['ASX-03','ASX-27'])
  self.assertEqual(gold['link_basis'],'explicit')
  self.assertIn('solo-events',gold['validation_suites'])
  self.assertIn('coop-events',gold['validation_suites'])
 def test_current_contract_inventory_is_classified(self):
  self.assertFalse([c['id']for c in compatibility.inventory()['contracts']if 'unclassified'in c['domains']])
 def test_generated_binding_and_features_match(self):
  _,client,_=binding_registry.load_binding()
  self.assertEqual(binding_registry.owned_path(client["descriptor"]).read_text(),binding_registry.render())
  self.assertEqual(len(feature_catalog.load_features()),28)
 def test_production_native_access_has_no_historical_descriptor_dependency(self):
  root=Path(__file__).resolve().parents[1]
  for path in(root/"crates/asx-game/src/official_native").rglob('*.rs'):
   source=path.read_text().split('#[cfg(test)]',1)[0]
   self.assertIsNone(re.search(r'crate::official(?:117|120|1201|121|125)\b',source),str(path))
  current=(root/'crates/asx-game/src/bindings/official129.rs').read_text()
  self.assertNotIn('super::official',current)
 def test_all_plans_have_solo_before_coop(self):
  config=scenario_suites.load_suites()
  for plan in config["plans"].values():
   seen=set()
   for name in plan:
    suite=config["suites"][name]
    if suite["solo_prerequisite"]:self.assertIn(suite["solo_prerequisite"],seen)
    seen.add(name)
 def test_readiness_waits_and_resets_without_claiming_acceptance(self):
  with tempfile.TemporaryDirectory()as folder:
   path=Path(folder)/"log";suite=dict(readiness=[dict(source="log",contains="done",minimum=1)],settle_seconds=2)
   ready=scenario_suites.Readiness(suite)
   self.assertFalse(ready.complete(folder,0));path.write_text("done")
   self.assertFalse(ready.complete(folder,1));self.assertTrue(ready.complete(folder,3))
   path.unlink();self.assertFalse(ready.complete(folder,4))
 def test_event_readiness_waits_for_material_sales_and_guest_cleanup(self):
  suite=scenario_suites.load_suites()["suites"]["coop-events"]
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder);(root/'player-1').mkdir();(root/'player-2').mkdir()
   host=root/'player-1/azeroth-coop-native.log';guest=root/'player-2/azeroth-coop-native.log'
   host.write_text("treasure goblin rewarded:\ntreasure goblin escaped:\ntreasure goblin fine motion:\ntreasure goblin fine motion:\ncoop ritual completed: stage=1 recipients=2\n")
   guest.write_text("treasure goblin received: opcode=aa guid=10\n"*2)
   ready=scenario_suites.Readiness(suite);self.assertFalse(ready.complete(root,0))
   with host.open('a')as f:
    for slot in (0,1):f.write(f"coop supplies discovered: slot={slot} owner=123 event=2 reward=Material gold=24\n")
   self.assertFalse(ready.complete(root,1))
   with guest.open('a')as f:f.write("treasure goblin received: opcode=aa guid=11\n")
   self.assertFalse(ready.complete(root,2));self.assertTrue(ready.complete(root,4))
 def test_normal_party_readiness_requires_observed_shared_gold(self):
  suite=scenario_suites.load_suites()["suites"]["coop-normal"]
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder)
   for index in (1,2):
    slot=root/f"player-{index}";slot.mkdir();(slot/'native-output.log').write_text('rogue: {"type":"choices","offer":1,"why":"levelup","level":2,}')
   host=root/'player-1/azeroth-coop-native.log';host.write_text('official group pause: Some\nofficial group pause: None\n')
   ready=scenario_suites.Readiness(suite);self.assertFalse(ready.complete(root,0))
   with host.open('a')as f:f.write('official gold: observed credit\n')
   self.assertFalse(ready.complete(root,1));self.assertTrue(ready.complete(root,7))
 def test_diagnostics_reject_foreign_duplicate_missing_and_model_only(self):
  def row(n=1,**kw):return dict(schema=1,session="test",slot=0,sequence=n,clock=n,kind="observer.started",data={},**kw)
  encode=lambda r:"[INFO] ASX_DIAG "+json.dumps(r)
  good=diagnostic_records.collect(encode(row()),"test",0);self.assertTrue(good["passed"])
  self.assertFalse(diagnostic_records.collect(encode(row()),"other",0)["passed"])
  self.assertFalse(diagnostic_records.collect(encode(row())+"\n"+encode(row()),"test",0)["passed"])
  self.assertFalse(diagnostic_records.collect("","test",0)["passed"])
  self.assertFalse(diagnostic_records.assess_events(good)["passed"])
 def test_impact_never_approves_and_refuses_wrong_baseline(self):
  base=dict(client_sha256="a"*64,contracts=[dict(id="unknown",native_addresses=[5],domains=["unclassified"])],features=[dict(id="ASX-01",disposition="active",native_domains=["ui"])])
  candidate=dict(old_sha256="a"*64,new_sha256="b"*64,approved=False,offline_only=True,mapping_policy="instruction-alignment-v2",mapping={})
  report=compatibility.impact(candidate,base)
  self.assertFalse(report["approved"]);self.assertEqual(report["features"][0]["contracts"],["unknown"])
  candidate["old_sha256"]="c"*64
  with self.assertRaises(ValueError):compatibility.impact(candidate,base)
 def test_coop_refuses_stale_or_failed_solo_proof(self):
  suite=dict(solo_prerequisite="solo-events");record=dict(dll_sha256="a",source_fingerprint="b",validation_fingerprint="c",ui_fingerprint="ui")
  proof=dict(suite="solo-events",passed=True,dll_sha256="a",source_fingerprint="b",validation_fingerprint="c",ui_fingerprint="ui",version=asx_dev.VERSION)
  asx_dev.prerequisite(suite,record,[proof])
  for field,value in (("passed",False),("dll_sha256","old"),("source_fingerprint","old"),("version","old"),("validation_fingerprint","old")):
   with self.assertRaises(ValueError):asx_dev.prerequisite(suite,record,[dict(proof,**{field:value})])
 def test_artifact_changed_bytes_or_source_rejected(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder);dll=root/"avrt.dll";dll.write_bytes(b"normal")
   files=artifact_record.inputs(root)
   receipt=dict(schema=1,version=artifact_record.VERSION,native_sha256=artifact_record.SHA256,kind="normal",dll_sha256=artifact_record.digest(dll),inputs=files,source_fingerprint=artifact_record.fingerprint(files))
   artifact_record.verify(receipt,dll,root,"normal")
   dll.write_bytes(b"changed")
   with self.assertRaises(ValueError):artifact_record.verify(receipt,dll,root,"normal")
if __name__=="__main__":unittest.main()
