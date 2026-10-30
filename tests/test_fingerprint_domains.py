import json,tempfile,unittest
from pathlib import Path
from tools.dev import artifact_record,asx_dev
class FingerprintDomains(unittest.TestCase):
 def test_ui_and_validation_changes_do_not_invalidate_compiled_dll(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder);(root/'src').mkdir();(root/'native-ui').mkdir();(root/'tools').mkdir()
   source=root/'src/lib.rs';source.write_text('pub fn example(){}')
   ui=root/'native-ui/party.lua';ui.write_text('old')
   fixture=root/'tools/fixture.lua';fixture.write_text('old')
   dll=root/'avrt.dll';dll.write_bytes(b'normal')
   files=artifact_record.inputs(root);ui_before=artifact_record.ui_fingerprint(root)
   record=dict(schema=2,version=artifact_record.VERSION,native_sha256=artifact_record.SHA256,kind='normal',dll_sha256=artifact_record.digest(dll),inputs=files,source_fingerprint=artifact_record.fingerprint(files),compilation_fingerprint=artifact_record.fingerprint(files))
   ui.write_text('new');fixture.write_text('new')
   artifact_record.verify(record,dll,root,'normal')
   self.assertNotEqual(ui_before,artifact_record.ui_fingerprint(root))
   source.write_text('pub fn changed(){}')
   with self.assertRaises(ValueError):artifact_record.verify(record,dll,root,'normal')
 def test_independent_addon_changes_invalidate_ui_but_not_compilation(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder);addon=root/'addons/ASXUI';addon.mkdir(parents=True)
   lua=addon/'Minimap.lua';lua.write_text('old')
   compiled=artifact_record.inputs(root);before=artifact_record.ui_fingerprint(root)
   self.assertIn('addons/ASXUI/Minimap.lua',artifact_record.ui_inputs(root))
   lua.write_text('new')
   self.assertNotEqual(before,artifact_record.ui_fingerprint(root))
   self.assertEqual(compiled,artifact_record.inputs(root))
 def test_embedded_lua_stays_a_compilation_dependency(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder);(root/'src').mkdir();(root/'native-ui').mkdir()
   (root/'src/lib.rs').write_text('const UI: &str = include_str!("../native-ui/party.lua");')
   ui=root/'native-ui/party.lua';ui.write_text('old')
   before=artifact_record.inputs(root);self.assertIn('native-ui/party.lua',before)
   ui.write_text('new');self.assertNotEqual(before,artifact_record.inputs(root))
 def test_dynamic_embedded_input_is_not_silently_ignored(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder);(root/'src').mkdir()
   (root/'src/lib.rs').write_text('const UI: &str = include_str!(env!("UI_FILE"));')
   with self.assertRaises(ValueError):artifact_record.inputs(root)
 def test_legacy_receipt_is_not_silently_reinterpreted(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder);(root/'native-ui').mkdir();ui=root/'native-ui/party.lua';ui.write_text('old');dll=root/'avrt.dll';dll.write_bytes(b'normal')
   files=artifact_record.inputs(root,legacy=True)
   record=dict(schema=1,version=artifact_record.VERSION,native_sha256=artifact_record.SHA256,kind='normal',dll_sha256=artifact_record.digest(dll),inputs=files,source_fingerprint=artifact_record.fingerprint(files))
   artifact_record.verify(record,dll,root,'normal');ui.write_text('new')
   with self.assertRaises(ValueError):artifact_record.verify(record,dll,root,'normal')
 def test_ui_change_refuses_reuse_of_solo_proof(self):
  suite=dict(solo_prerequisite='solo');record=dict(dll_sha256='dll',source_fingerprint='compile',ui_fingerprint='new',validation_fingerprint='test',prerequisite_fingerprint='solo-test')
  receipt=dict(suite='solo',passed=True,dll_sha256='dll',source_fingerprint='compile',ui_fingerprint='old',validation_fingerprint='solo-test',version=asx_dev.VERSION)
  with self.assertRaises(ValueError):asx_dev.prerequisite(suite,record,[receipt])
  receipt['ui_fingerprint']='new';asx_dev.prerequisite(suite,record,[receipt])
if __name__=='__main__':unittest.main()
