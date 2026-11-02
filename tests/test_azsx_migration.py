"""Real old-brand paths, receipts and user settings; no native game needed."""
import sys,tempfile,unittest,json
from pathlib import Path
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'azsx'))
import core,migration,layout
class BrandingTests(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.root=Path(self.tmp.name)
  self.old='benilla-config/AddOns/ASX/Presentation.lua';self.toc='benilla-config/AddOns/ASX/ASX.toc'
  for n,b in {self.old:b'old API',self.toc:b'Presentation.lua','ASX/Config/options.json':b'{"treasure_goblins":false}','ASX/App/manifest.json':b'old payload','benilla-config/SavedVariables/ASXUI.lua':b'ASXUIDB={scale=0.9}','AZSX/App/manifest.json':b'new payload'}.items():core.atomic(self.root/n,b)
  core.atomic(self.root/'ASX/Data/State/install.json',core.encoded({'schema':1,'owned':{self.old:core.digest(b'old API'),self.toc:core.digest(b'Presentation.lua')}}))
  p=patch('core.game_running',return_value=False);p.start();self.addCleanup(p.stop)
 def test_migrate_preserves_settings_and_new_payload_and_retires_owned_bridge(self):
  self.assertTrue(migration.migrate(self.root));self.assertFalse(migration.migrate(self.root))
  self.assertEqual((self.root/'AZSX/App/manifest.json').read_bytes(),b'new payload')
  self.assertEqual((self.root/'AZSX/Config/options.json').read_bytes(),b'{"treasure_goblins":false}')
  self.assertFalse((self.root/self.old).exists());self.assertFalse((self.root/self.toc).exists())
  self.assertEqual((self.root/'AZSX/Data/Backups/Branding'/self.old).read_bytes(),b'old API')
  self.assertEqual((self.root/'benilla-config/SavedVariables/AZSXUI.lua').read_bytes(),b'AZSXUIDB={scale=0.9}')
  self.assertTrue((self.root/'benilla-config/SavedVariables/ASXUI.lua').exists())
 def test_conflicting_preferences_stop_before_copy_and_preserve_both(self):
  core.atomic(self.root/'AZSX/Config/options.json',b'new choice')
  with self.assertRaisesRegex(ValueError,'Conflicting'):migration.migrate(self.root)
  self.assertTrue((self.root/self.toc).exists());self.assertEqual((self.root/'AZSX/Config/options.json').read_bytes(),b'new choice')
 def test_modified_old_code_and_unknown_files_are_preserved(self):
  core.atomic(self.root/self.old,b'user code');core.atomic(self.root/'benilla-config/AddOns/ASX/note.md',b'notes')
  migration.migrate(self.root)
  self.assertEqual((self.root/self.old).read_bytes(),b'user code');self.assertFalse((self.root/self.toc).exists())
  self.assertEqual((self.root/'benilla-config/AddOns/ASX/note.md').read_bytes(),b'notes')
 def test_interrupted_update_is_not_reinterpreted(self):
  core.atomic(self.root/'ASX/Data/Updates/Transactions/old/journal.json',b'{"phase":"applying"}')
  with self.assertRaisesRegex(ValueError,'Recover'):migration.migrate(self.root)
  self.assertFalse((self.root/'AZSX/Config/options.json').exists())
 def test_receipt_names_translate_without_renaming_old_addon_identity(self):
  self.assertEqual(layout.name('ASX/Client/core.py'),'AZSX/App/Launcher/core.py')
  self.assertEqual(layout.name(self.old),self.old)

 def test_resume_after_partial_retirement_retains_complete_receipt(self):
  original=Path.unlink;removed=[]
  def interrupt(path,*a,**kw):
   if str(path).startswith(str(self.root/'ASX'))and path.name!='installation.lock':
    if removed:raise OSError('simulated interruption')
    removed.append(path.relative_to(self.root).as_posix())
   return original(path,*a,**kw)
  with patch.object(Path,'unlink',interrupt):
   with self.assertRaisesRegex(OSError,'simulated'):migration.migrate(self.root)
  self.assertEqual(len(removed),1)
  self.assertTrue(migration.migrate(self.root))
  receipt=core.get(self.root,'AZSX/Data/State/branding-migration.json',{})
  self.assertEqual(receipt['phase'],'complete')
  self.assertIn(removed[0],[f['source']for f in receipt['files']])
  self.assertFalse((self.root/'ASX').exists())
  for f in receipt['files']:self.assertEqual(core.sha(self.root/f['backup']),f['sha256'])
