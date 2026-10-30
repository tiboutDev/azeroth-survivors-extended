"""Execution-only staging retains source-side provenance and fails closed on changes."""
import json,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
from tools.dev import asx_dev,artifact_record

class BundleTests(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
  self.root=Path(self.tmp.name)/'receiver';self.root.mkdir();self.target=asx_dev.runtime(self.root);self.target.mkdir()
  exe=self.target/'AzerothSurvivors.exe';exe.write_bytes(b'unit-test native executable')
  sha=artifact_record.digest(exe)
  for module in(asx_dev,artifact_record):
   changer=patch.object(module,'SHA256',sha);changer.start();self.addCleanup(changer.stop)
  asx_dev.atomic(self.target/('IMPORT-OFFICIAL-'+asx_dev.VERSION.replace('.','')+'.json'),dict(version=asx_dev.VERSION,files={'AzerothSurvivors.exe':sha}))
  (self.target/'avrt.dll').write_bytes(b'old owned DLL')
  asx_dev.atomic(self.target/'ASX-DEV.json',dict(dll_sha256=artifact_record.digest(self.target/'avrt.dll')))
  self.dll=Path(self.tmp.name)/'new.dll';self.dll.write_bytes(b'unit-test AZCOOP_TEST_EXTENSIONS')
  inputs=artifact_record.inputs();fp=artifact_record.fingerprint(inputs)
  self.record=dict(schema=2,version=asx_dev.VERSION,native_sha256=sha,kind='diagnostic',dll_sha256=artifact_record.digest(self.dll),source_fingerprint=fp,compilation_fingerprint=fp,inputs=inputs)
  asx_dev.atomic(self.dll.with_suffix('.artifact.json'),self.record)
 def stage(self):return asx_dev.stage(self.root,self.dll)
 def test_pristine_receiver_needs_no_sources_tests_or_build_tools(self):
  (self.root/'unknown.txt').write_text('preserve')
  self.stage();asx_dev.verify_staged(self.root,kind='diagnostic')
  for name in('src','crates','tests','Cargo.toml','tools/dev/native-inventory','tools/dev/build-windows.sh'):
   self.assertFalse((self.root/name).exists(),name)
  self.assertEqual((self.root/'unknown.txt').read_text(),'preserve')
  self.assertLess(len(asx_dev.execution_inputs()),60)
  self.stage() # Idempotent byte identities, even on a pristine minimal receiver.
 def test_changed_execution_bytes_and_missing_inventory_are_rejected(self):
  self.stage();file=self.root/'native-ui/official-party.lua';saved=file.read_bytes();file.write_bytes(saved+b'changed')
  with self.assertRaises(ValueError):asx_dev.verify_staged(self.root)
  file.write_bytes(saved)
  path=self.root/'.asx-stage.json';manifest=json.loads(path.read_text());manifest['files'].pop('tools/dev/extensions_trial.py');asx_dev.atomic(path,manifest)
  with self.assertRaises(ValueError):asx_dev.verify_staged(self.root)
 def test_native_dll_receipt_and_kind_changes_are_rejected(self):
  self.stage()
  with self.assertRaises(ValueError):asx_dev.verify_staged(self.root,kind='normal')
  for name in('avrt.dll','AzerothSurvivors.exe','avrt.artifact.json'):
   p=self.target/name;saved=p.read_bytes()
   if name.endswith('.json'):
    changed=json.loads(saved);changed['source_fingerprint']='tampered';asx_dev.atomic(p,changed)
   else:p.write_bytes(saved+b' ')
   with self.assertRaises(ValueError):asx_dev.verify_staged(self.root)
   p.write_bytes(saved)
 def test_bad_source_or_unowned_receiver_prevents_staging(self):
  old=(self.target/'avrt.dll').read_bytes()
  record=dict(self.record,source_fingerprint='bad');asx_dev.atomic(self.dll.with_suffix('.artifact.json'),record)
  with self.assertRaises(ValueError):self.stage()
  self.assertEqual((self.target/'avrt.dll').read_bytes(),old)
  asx_dev.atomic(self.dll.with_suffix('.artifact.json'),self.record)
  (self.target/'avrt.dll').write_bytes(b'foreign')
  with self.assertRaises(ValueError):self.stage()
  self.assertFalse((self.root/'.asx-stage.json').exists())
 def test_redirected_destination_is_rejected_before_copy(self):
  outside=Path(self.tmp.name)/'outside';outside.mkdir();(self.root/'tools').symlink_to(outside,target_is_directory=True)
  with self.assertRaises(ValueError):self.stage()
  self.assertEqual(list(outside.iterdir()),[])

 def test_redirected_receipt_temporary_path_is_rejected_before_copy(self):
  outside=Path(self.tmp.name)/'outside.json';outside.write_text('preserve')
  (self.root/'.asx-stage.json.tmp').symlink_to(outside)
  before=(self.target/'avrt.dll').read_bytes()
  with self.assertRaises(ValueError):self.stage()
  self.assertEqual(outside.read_text(),'preserve')
  self.assertEqual((self.target/'avrt.dll').read_bytes(),before)
