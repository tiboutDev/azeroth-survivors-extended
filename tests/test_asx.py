"""ASX installation/update tests: fake game files, real signatures, no user profile."""
import copy,importlib.util,io,json,os,sys,tempfile,unittest,zipfile
from pathlib import Path
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"asx"))
import core,updates
spec=importlib.util.spec_from_file_location("asx_launch",ROOT/"asx/launch.py");launch=importlib.util.module_from_spec(spec);spec.loader.exec_module(launch)
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from cryptography.hazmat.primitives import serialization
class ASXTests(unittest.TestCase):
 def setUp(self):
  self.temp=tempfile.TemporaryDirectory(prefix="ASX & native é ");self.addCleanup(self.temp.cleanup);self.root=Path(self.temp.name)
  self.exe=b"MZ official native game";core.atomic(self.root/"AzerothSurvivors.exe",self.exe);core.atomic(self.root/"Data/patch.MPQ",b"official assets")
  registry=patch("core.supported_native_games",return_value={"0.125.0":{"sha256":core.digest(self.exe)}});registry.start();self.addCleanup(registry.stop)
  core.atomic(self.root/"benilla-config/config.toml",b"native settings");core.atomic(self.root/"benilla-config/AddOns/SomeoneElse/user.lua",b"other addon")
  core.atomic(self.root/"progression.txt",b"native progression")
  self.key=Ed25519PrivateKey.generate();self.public=base64key(self.key)
  self.m,self.data=self.release(1)
  self.extract(self.m,self.data)
 def release(self,n,extra=True):
  files={k:(ROOT/"asx"/source).read_bytes()for k,source in [("start-asx.cmd","start-asx.cmd"),("README-ASX.txt","README-ASX.txt"),("ASX/Client/launch.py","launch.py"),("ASX/Defaults/channel.json","channel.json"),("ASX/Client/supported-games.json","supported-games.json")]}
  files["ASX/Client/supported-games.json"]=core.encoded({"schema":1,"games":{"0.125.0":{"sha256":core.digest(self.exe)}}})
  files.update({"ASX/Bin/avrt.dll":b"MZ ASX normal "+str(n).encode(),"ASX/Interface/ASX.toc":b"Presentation.lua","ASX/Interface/Presentation.lua":b"-- normal addon "+str(n).encode()})
  if extra:files["ASX/Client/old.py"]=b"owned old file"
  m={"schema":1,"product":core.PRODUCT,"version":f"0.125.0-asx.{n}","game":{"version":"0.125.0","sha256":core.digest(self.exe),"archives":["Data/patch.MPQ"]},"files":{k:{"size":len(v),"sha256":core.digest(v)}for k,v in files.items()}}
  return m,files
 def extract(self,m,data):
  for name,b in data.items():core.atomic(core.path(self.root,name),b)
  core.put(self.root,core.MANIFEST,m)
 def install(self):
  with core.locked(self.root):return core.apply(self.root,self.m,self.data,manual=True)
 def intact(self):
  self.assertEqual((self.root/"AzerothSurvivors.exe").read_bytes(),self.exe)
  self.assertEqual((self.root/"Data/patch.MPQ").read_bytes(),b"official assets")
  self.assertEqual((self.root/"benilla-config/config.toml").read_bytes(),b"native settings")
  self.assertEqual((self.root/"benilla-config/AddOns/SomeoneElse/user.lua").read_bytes(),b"other addon")
  self.assertEqual((self.root/"progression.txt").read_bytes(),b"native progression")
 def signed(self,m,data,sequence):
  stream=io.BytesIO()
  with zipfile.ZipFile(stream,"w",zipfile.ZIP_DEFLATED)as z:
   for k,b in {**data,core.MANIFEST:core.encoded(m)}.items():z.writestr(k,b)
  package=stream.getvalue();feed={"schema":1,"product":core.PRODUCT,"sequence":sequence,"manifest":m,"package":{"asset":"asx-"+m["version"]+".zip","size":len(package),"sha256":core.digest(package)}}
  raw=core.encoded(feed);return raw,self.key.sign(raw),package
 def configure(self):core.put(self.root,"ASX/Settings/channel.json",{"schema":1,"enabled":True,"base":"https://example.test/releases/","keys":[self.public]})
 def update(self,release):
  self.configure();raw,sig,package=release
  def download(url,limit):return raw if url.endswith("asx-latest.json")else sig if url.endswith(".sig")else package
  with core.locked(self.root):return updates.automatic(self.root,core.digest(self.exe),download,confirm=lambda feed:True)
 def bundled_ui(self,n):
  m,data=self.release(n)
  for name in core.ASXUI_FILES:
   data[core.ASXUI_SOURCE+name]=(ROOT/"addons/ASXUI"/name).read_bytes()
  data[core.ASXUI_SOURCE+"ASXUI.lua"]+=f"\n-- test release {n}\n".encode()
  m["files"]={k:{"size":len(b),"sha256":core.digest(b)}for k,b in data.items()}
  return m,data
 def ui_preferences(self):
  core.atomic(self.root/"benilla-config/SavedVariables/ASXUI.lua",b"ASXUIDB = {scale=1.25}")
  core.atomic(self.root/(core.ASXUI_TARGET+"user-note.txt"),b"keep addon note")
 def assert_ui_preferences(self):
  self.assertEqual((self.root/"benilla-config/SavedVariables/ASXUI.lua").read_bytes(),b"ASXUIDB = {scale=1.25}")
  self.assertEqual((self.root/(core.ASXUI_TARGET+"user-note.txt")).read_bytes(),b"keep addon note")
 def test_bundled_ui_install_update_and_uninstall_preserve_preferences(self):
  self.ui_preferences();self.m,self.data=self.bundled_ui(1);self.extract(self.m,self.data)
  self.install()
  for name in core.ASXUI_FILES:
   target=core.ASXUI_TARGET+name
   self.assertEqual((self.root/target).read_bytes(),self.data[core.ASXUI_SOURCE+name]);self.assertIn(target,core.installed(self.root)["owned"])
  m,data=self.bundled_ui(2);self.assertEqual(self.update(self.signed(m,data,2)),"installed")
  self.assertEqual((self.root/(core.ASXUI_TARGET+"ASXUI.lua")).read_bytes(),data[core.ASXUI_SOURCE+"ASXUI.lua"])
  import remove
  remove.uninstall(self.root)
  self.assertFalse((self.root/(core.ASXUI_TARGET+"ASXUI.toc")).exists());self.assert_ui_preferences();self.intact()
 def test_first_launch_projects_bundled_ui_licenses_and_preserves_existing_notes(self):
  self.m,self.data=self.bundled_ui(1);self.extract(self.m,self.data)
  core.adopt(self.root,self.m,self.data)
  core.apply(self.root,self.m,core.sources(self.root,self.m,tolerant=True),manual=True,launch=True)
  for name in core.ASXUI_FILES:
   self.assertEqual((self.root/(core.ASXUI_TARGET+name)).read_bytes(),self.data[core.ASXUI_SOURCE+name])
  target=self.root/(core.ASXUI_TARGET+"README.md");target.write_bytes(b"personal notes")
  (self.root/(core.ASXUI_TARGET+"Events.lua")).unlink()
  core.apply(self.root,self.m,core.sources(self.root,self.m,tolerant=True),manual=True,launch=True)
  self.assertEqual(target.read_bytes(),b"personal notes")
 def test_unowned_ui_migration_requires_consent_and_backups(self):
  self.install();self.ui_preferences();target=core.ASXUI_TARGET+"Buffs.lua";core.atomic(self.root/target,b"older independent addon")
  m,data=self.bundled_ui(2);before=self.snapshot()
  for approve in (None,lambda names:False):
   with self.assertRaises(ValueError):core.apply(self.root,m,data,approve=approve)
   self.assertEqual(self.snapshot(),before)
  seen=[];core.apply(self.root,m,data,approve=lambda names:seen.extend(names)or True)
  self.assertEqual(seen,[target]);self.assertTrue(any(p.read_bytes()==b"older independent addon"for p in(self.root/"ASX/Backups/Files").rglob("Buffs.lua")))
  self.assert_ui_preferences();self.intact()
 def test_fresh_bootstrap_does_not_silently_adopt_conflicting_ui(self):
  self.m,self.data=self.bundled_ui(1);self.extract(self.m,self.data);target=core.ASXUI_TARGET+"ASXUI.lua";core.atomic(self.root/target,b"custom ui")
  core.adopt(self.root,self.m,self.data);self.assertNotIn(target,core.installed(self.root)["owned"])
  with self.assertRaises(ValueError):core.apply(self.root,self.m,self.data,manual=True,launch=True,approve=lambda names:False)
  self.assertEqual((self.root/target).read_bytes(),b"custom ui");self.assertFalse((self.root/"avrt.dll").exists())
  core.apply(self.root,self.m,self.data,manual=True,launch=True,approve=lambda names:True)
  self.assertEqual((self.root/target).read_bytes(),self.data[core.ASXUI_SOURCE+"ASXUI.lua"])
 def test_ui_diagnosis_repair_and_rollback_keep_user_files(self):
  self.install();self.ui_preferences();m,data=self.bundled_ui(2);self.update(self.signed(m,data,2))
  target=core.ASXUI_TARGET+"Events.lua";(self.root/target).unlink()
  import maintenance
  self.assertTrue(any(x["file"]==target and x["severity"]=="critical"for x in maintenance.diagnose(self.root)["issues"]))
  core.apply(self.root,m,data,manual=True,launch=True);self.assertTrue((self.root/target).exists())
  core.atomic(self.root/(core.ASXUI_TARGET+"README.md"),b"my notes")
  self.assertTrue(maintenance.diagnose(self.root)["ready"])
  core.apply(self.root,m,data,repair=True,approve=lambda names:True)
  self.assertEqual((self.root/(core.ASXUI_TARGET+"README.md")).read_bytes(),b"my notes")
  core.rollback(self.root);self.assertFalse((self.root/target).exists());self.assert_ui_preferences();self.intact()
 def test_ui_projection_inventory_rejects_preferences_unknown_files_and_partial_bundle(self):
  m,data=self.bundled_ui(1);self.assertIs(core.validate(m),m)
  for suffix in ("SavedVariables.lua","../Other/code.lua","Fonts/unknown.ttf"):
   with self.assertRaises(ValueError):core.owned_name(core.ASXUI_TARGET+suffix)
  del m["files"][core.ASXUI_SOURCE+"Buffs.lua"]
  with self.assertRaises(ValueError):core.validate(m)
 def test_declining_signed_update_does_not_download_archive_or_advance_sequence(self):
  self.install();self.configure()
  m,data=self.release(2);raw,sig,package=self.signed(m,data,13)
  downloads=[]
  def download(url,limit):
   downloads.append(url)
   if url.endswith("asx-latest.json"):return raw
   if url.endswith(".sig"):return sig
   self.fail("Deferred update downloaded its archive")
  before=core.installed(self.root)
  with core.locked(self.root):
   self.assertEqual(updates.automatic(self.root,core.digest(self.exe),download,confirm=lambda feed:False),"deferred")
  self.assertEqual(len(downloads),2)
  self.assertIsNone(core.get(self.root,"ASX/State/updates.json"))
  self.assertEqual(core.installed(self.root),before)
  self.intact()
 def test_invalid_update_is_rejected_before_player_is_asked(self):
  self.install();self.configure()
  m,data=self.release(2);raw,sig,package=self.signed(m,data,13)
  def download(url,limit):return raw if url.endswith("asx-latest.json")else b"x"*64
  with self.assertRaises(ValueError):
   updates.automatic(self.root,core.digest(self.exe),download,confirm=lambda feed:self.fail("Untrusted update offered"))
 def test_cancelled_update_never_launches_game(self):
  self.install()
  with patch.object(core,"game_running",return_value=False),patch.object(core,"native_paths",return_value=(self.root/"progression.txt",self.root/"benilla-config")),patch.object(updates,"automatic",side_effect=launch.UpdateCancelled()),patch.object(launch.subprocess,"Popen",side_effect=AssertionError("Cancelled launch started game")):
   self.assertEqual(launch.run(self.root,confirm_update=launch.confirm_update),0)
  self.intact()
 def snapshot(self):
  return {p.relative_to(self.root).as_posix():p.read_bytes() for p in self.root.rglob("*") if p.is_file()}
 def repair_inputs(self,m=None,data=None):
  m=m or self.m;data=data or self.data;self.configure()
  raw,sig,archive=self.signed(m,data,1)
  folder=self.root/"repair-input";folder.mkdir(exist_ok=True)
  for name,value in (("feed.json",raw),("feed.sig",sig),("package.zip",archive)):(folder/name).write_bytes(value)
  return dict(archive=folder/"package.zip",feed_path=folder/"feed.json",signature_path=folder/"feed.sig")
 def test_check_collects_all_issues_without_writing_or_network(self):
  self.install();(self.root/"README-ASX.txt").write_bytes(b"personal text")
  (self.root/"ASX/Bin/avrt.dll").unlink();(self.root/"ASX/Client/launch.py").unlink()
  before=self.snapshot()
  with patch.object(updates,"fetch",side_effect=AssertionError("network during check")):
   report=launch.maintenance.diagnose(self.root)
   self.assertFalse(report["ready"])
   self.assertGreaterEqual(len(report["issues"]),3)
   self.assertEqual(launch.run(self.root,check=True),1)
  self.assertEqual(before,self.snapshot())
 def test_documentation_and_bootstrap_do_not_block_preparation(self):
  self.install();(self.root/"README-ASX.txt").write_bytes(b"personal text")
  (self.root/"start-asx.cmd").unlink()
  data=core.sources(self.root,self.m,tolerant=True)
  core.apply(self.root,self.m,data,manual=True,launch=True)
  self.assertTrue(launch.maintenance.diagnose(self.root)["ready"])
  self.assertEqual((self.root/"README-ASX.txt").read_bytes(),b"personal text")
 def test_same_version_repair_restores_critical_file_preserves_documentation_and_rollback(self):
  self.install();m,data=self.release(2);self.update(self.signed(m,data,2))
  tx=core.installed(self.root)["rollback"]
  inputs=self.repair_inputs(m,data)
  (self.root/"ASX/Bin/avrt.dll").unlink();(self.root/"README-ASX.txt").write_bytes(b"my notes")
  self.assertEqual(launch.maintenance.repair(self.root,lambda plan:True,**inputs),"installed")
  self.assertEqual((self.root/"ASX/Bin/avrt.dll").read_bytes(),data["ASX/Bin/avrt.dll"])
  self.assertEqual((self.root/"README-ASX.txt").read_bytes(),b"my notes")
  self.assertEqual(core.installed(self.root)["rollback"],tx);self.intact()
 def test_repair_cancellation_preserves_installation(self):
  self.install();inputs=self.repair_inputs();(self.root/"ASX/Bin/avrt.dll").unlink();before=self.snapshot()
  self.assertEqual(launch.maintenance.repair(self.root,lambda plan:False,**inputs),"cancelled")
  self.assertEqual(before,self.snapshot())
 def test_repair_rejects_invalid_signature_before_writes(self):
  self.install();inputs=self.repair_inputs();inputs["signature_path"].write_bytes(b"x"*64);before=self.snapshot()
  with self.assertRaises(ValueError):launch.maintenance.repair(self.root,lambda plan:self.fail("untrusted repair offered"),**inputs)
  self.assertEqual(before,self.snapshot())
 def test_repair_refuses_to_change_release(self):
  self.install();m,data=self.release(2);inputs=self.repair_inputs(m,data)
  with self.assertRaisesRegex(ValueError,"installed version"):launch.maintenance.repair(self.root,lambda plan:True,**inputs)
  self.assertEqual(core.installed(self.root)["version"],self.m["version"])
 def test_modified_critical_repair_requires_consent_and_backups(self):
  self.install();inputs=self.repair_inputs();(self.root/"ASX/Bin/avrt.dll").write_bytes(b"changed owned DLL")
  launch.maintenance.repair(self.root,lambda plan:True,**inputs)
  backups=list((self.root/"ASX/Backups/Files").rglob("avrt.dll"))
  self.assertTrue(any(p.read_bytes()==b"changed owned DLL" for p in backups));self.intact()
 def test_lost_receipt_never_adopts_foreign_loader(self):
  self.install();inputs=self.repair_inputs();(self.root/core.STATE).unlink();(self.root/"avrt.dll").write_bytes(b"foreign loader")
  with self.assertRaisesRegex(ValueError,"Foreign loader"):launch.maintenance.repair(self.root,lambda plan:True,**inputs)
  self.assertEqual((self.root/"avrt.dll").read_bytes(),b"foreign loader")
 def test_invalid_receipt_can_be_recovered_with_verified_package(self):
  self.install();inputs=self.repair_inputs();(self.root/core.STATE).write_bytes(b"bad json")
  (self.root/"ASX/Bin/avrt.dll").unlink()
  launch.maintenance.repair(self.root,lambda plan:True,**inputs)
  self.assertEqual(core.installed(self.root)["version"],self.m["version"])
  self.assertTrue(launch.maintenance.diagnose(self.root)["ready"])
 def test_manifest_damage_does_not_prevent_diagnostic_export(self):
  self.install();(self.root/core.MANIFEST).write_bytes(b"invalid json")
  archive=launch.export(self.root)
  with zipfile.ZipFile(archive) as z:
   self.assertIn("metadata_unavailable",json.loads(z.read("ASX-version.json")))
   self.assertFalse(json.loads(z.read("installation-check.json"))["ready"])
 def test_update_needs_confirmation_and_preserves_modified_readme(self):
  self.install();self.configure();m,data=self.release(2);raw,sig,archive=self.signed(m,data,2)
  def download(url,limit):return raw if url.endswith("asx-latest.json") else sig if url.endswith(".sig") else archive
  self.assertIn("confirmation required",updates.automatic(self.root,core.digest(self.exe),download))
  (self.root/"README-ASX.txt").write_bytes(b"personal README")
  self.assertEqual(updates.automatic(self.root,core.digest(self.exe),download,confirm=lambda feed:True),"installed")
  self.assertTrue(any(p.read_bytes()==b"personal README" for p in (self.root/"ASX/Backups/Files").rglob("README-ASX.txt")))
 def test_failed_update_requires_explicit_retry(self):
  self.install();self.configure();m,data=self.release(2);raw,sig,archive=self.signed(m,data,2)
  def download(url,limit):return raw if url.endswith("asx-latest.json") else sig if url.endswith(".sig") else archive
  with patch.object(core,"apply",side_effect=OSError("disk full")),self.assertRaises(OSError):
   updates.automatic(self.root,core.digest(self.exe),download,confirm=lambda feed:True)
  self.assertIn("held",updates.automatic(self.root,core.digest(self.exe),download,confirm=lambda feed:self.fail("automatic retry")))
  self.assertEqual(updates.automatic(self.root,core.digest(self.exe),download,confirm=lambda feed:True,retry=True),"installed")
 def test_projection_repair_keeps_previous_release_rollback(self):
  self.install();m,data=self.release(2);self.update(self.signed(m,data,2));tx=core.installed(self.root)["rollback"]
  (self.root/"avrt.dll").unlink();core.apply(self.root,m,core.sources(self.root,m,tolerant=True),manual=True,launch=True)
  self.assertEqual(core.installed(self.root)["rollback"],tx)
 def test_signed_future_registry_allows_update_but_not_old_dll_activation(self):
  self.install();self.configure();m,data=self.release(2)
  new_exe=b"MZ reviewed future official game";m["game"].update(version="0.131.0",sha256=core.digest(new_exe))
  registry=core.encoded({"schema":1,"games":{"0.131.0":{"sha256":core.digest(new_exe)}}})
  data["ASX/Client/supported-games.json"]=registry
  m["files"]["ASX/Client/supported-games.json"]={"size":len(registry),"sha256":core.digest(registry)}
  raw,sig,archive=self.signed(m,data,3);(self.root/"AzerothSurvivors.exe").write_bytes(new_exe)
  with self.assertRaises(ValueError):core.inspect(self.root,self.m)
  def download(url,limit):return raw if url.endswith("asx-latest.json") else sig if url.endswith(".sig") else archive
  self.assertEqual(updates.automatic(self.root,core.digest(new_exe),download,confirm=lambda feed:True),"installed")
  self.assertEqual(core.installed(self.root)["game"]["version"],"0.131.0")
 def test_readme_warning_allows_actual_launch_path(self):
  self.install();(self.root/"README-ASX.txt").unlink()
  class Child:
   pid=123
   def poll(self):return None
   def wait(self,**kwargs):return 0
   def terminate(self):pass
  child=Child()
  def spawn(*args,**kwargs):
   session=Path(kwargs["env"]["ASX_READY_FILE"])
   session.write_bytes(core.encoded({"product":"asx","game_sha256":core.digest(self.exe)}))
   return child
  with patch.object(core,"game_running",return_value=False),patch.object(core,"native_paths",return_value=(self.root/"progression.txt",self.root/"benilla-config")),patch.object(launch.subprocess,"Popen",side_effect=spawn):
   self.assertEqual(launch.run(self.root,offline=True),0)
  self.intact()
 def test_read_only_check_does_not_require_profile_or_recover(self):
  self.install()
  with patch.object(core,"native_paths",side_effect=AssertionError("profile consulted")),patch.object(core,"recover",side_effect=AssertionError("recovery during check")),patch.object(core,"locked",side_effect=AssertionError("write lock during check")):
   self.assertEqual(launch.run(self.root,check=True),0)
 def test_offline_cache_repairs_without_download(self):
  self.install();self.configure();raw,sig,data=self.signed(self.m,self.data,1)
  feed=updates.verify(raw,sig,[self.public],core.digest(self.exe))
  updates.cache(self.root,raw,sig,data,feed);(self.root/"ASX/Bin/avrt.dll").unlink()
  def download(*args):self.fail("offline repair used network")
  self.assertEqual(launch.maintenance.repair(self.root,lambda plan:True,offline=True,download=download),"installed")
 def test_windows_runtime_replacement_requires_manual_archive_before_mutation(self):
  import types
  self.install();m,data=self.release(2)
  name="ASX/Runtime/python.exe";data[name]=b"MZ new runtime";m["files"][name]={"size":len(data[name]),"sha256":core.digest(data[name])}
  before=self.snapshot()
  with patch.object(core,"os",types.SimpleNamespace(name="nt")),self.assertRaisesRegex(ValueError,"runtime"):
   core.apply(self.root,m,data)
  self.assertEqual(before,self.snapshot())
 def test_update_failure_exit_is_not_ready_success(self):
  self.install()
  with patch.object(core,"game_running",return_value=False),patch.object(updates,"automatic",side_effect=OSError("offline")),patch.object(core,"native_paths",side_effect=AssertionError("profile required for update")),self.assertRaises(OSError):
   launch.run(self.root,update_only=True)
 def test_embedded_official_channel_matches_packaged_defaults(self):
  self.assertEqual(updates.DEFAULT_CHANNEL,json.loads((ROOT/"asx/channel.json").read_text()))
 def test_repaired_personal_readme_is_backed_up_before_rollback(self):
  self.install();m,data=self.release(2)
  data["README-ASX.txt"]+=b"Updated documentation"
  m["files"]["README-ASX.txt"]={"size":len(data["README-ASX.txt"]),"sha256":core.digest(data["README-ASX.txt"])}
  self.update(self.signed(m,data,2))
  inputs=self.repair_inputs(m,data);(self.root/"README-ASX.txt").write_bytes(b"personal rollback notes")
  (self.root/"ASX/Bin/avrt.dll").unlink();launch.maintenance.repair(self.root,lambda plan:True,**inputs)
  core.rollback(self.root)
  self.assertTrue(any(p.read_bytes()==b"personal rollback notes" for p in (self.root/"ASX/Backups/Files").rglob("README-ASX.txt")))
 def test_update_metadata_and_license_are_not_runtime_requirements(self):
  self.install();(self.root/"ASX/Defaults/channel.json").unlink()
  data=core.sources(self.root,self.m,tolerant=True)
  core.apply(self.root,self.m,data,launch=True)
  self.assertTrue(launch.maintenance.diagnose(self.root)["ready"])
  self.assertEqual(core.file_role("ASX/Client/LICENSE"),"documentation")
 def test_cli_check_works_in_isolated_deployed_layout_without_profile(self):
  import subprocess
  for module in ("core.py","updates.py","maintenance.py"):
   name="ASX/Client/"+module;raw=(ROOT/"asx"/module).read_bytes();self.data[name]=raw
   self.m["files"][name]={"size":len(raw),"sha256":core.digest(raw)}
  name="ASX/Client/supported-games.json"
  raw=core.encoded({"schema":1,"games":{"0.125.0":{"sha256":core.digest(self.exe),"archives":["Data/patch.MPQ"]}}})
  self.data[name]=raw;self.m["files"][name]={"size":len(raw),"sha256":core.digest(raw)}
  self.extract(self.m,self.data);self.install()
  (self.root/"README-ASX.txt").write_bytes(b"personal README")
  before=self.snapshot();env={k:v for k,v in os.environ.items() if k not in ("APPDATA","BENILLA_HOME","WOW_ROGUE_SAVE")}
  result=subprocess.run([sys.executable,"-I","-B","-X","utf8",str(self.root/"ASX/Client/launch.py"),"--check"],cwd=self.root,env=env,capture_output=True,text=True,encoding="utf-8",timeout=30)
  self.assertEqual(result.returncode,0,result.stdout+result.stderr)
  self.assertIn("README-ASX.txt: modified",result.stdout);self.assertIn("Ready to play",result.stdout)
  self.assertEqual(before,self.snapshot())
  (self.root/"ASX/Bin/avrt.dll").unlink();(self.root/"ASX/Interface/Presentation.lua").unlink();before=self.snapshot()
  result=subprocess.run([sys.executable,"-I","-B","-X","utf8",str(self.root/"ASX/Client/launch.py"),"--check"],cwd=self.root,env=env,capture_output=True,text=True,encoding="utf-8",timeout=30)
  self.assertEqual(result.returncode,1,result.stdout+result.stderr)
  self.assertIn("ASX/Bin/avrt.dll: missing",result.stdout);self.assertIn("ASX/Interface/Presentation.lua: missing",result.stdout)
  self.assertEqual(before,self.snapshot())
 def test_runtime_update_is_reported_before_archive_download_or_confirmation(self):
  import types
  self.install();self.configure();m,data=self.release(2)
  name="ASX/Runtime/python.exe";data[name]=b"MZ runtime";m["files"][name]={"size":len(data[name]),"sha256":core.digest(data[name])}
  raw,sig,archive=self.signed(m,data,2)
  def download(url,limit):
   if url.endswith("asx-latest.json"):return raw
   if url.endswith(".sig"):return sig
   self.fail("runtime archive downloaded before manual-install notice")
  with patch.object(updates,"os",types.SimpleNamespace(name="nt")):
   status=updates.automatic(self.root,core.digest(self.exe),download,confirm=lambda feed:self.fail("runtime update offered as automatic"))
  self.assertIn("manual installation required",status)
 def test_repaired_installation_does_not_prompt_again_after_restart(self):
  self.install();inputs=self.repair_inputs();(self.root/"ASX/Bin/avrt.dll").unlink()
  launch.maintenance.repair(self.root,lambda plan:True,**inputs)
  self.assertEqual(launch.maintenance.repair(self.root,lambda plan:self.fail("second confirmation"),**inputs),"current")
 def test_missing_receipt_is_reconstructed_after_consent(self):
  self.install();inputs=self.repair_inputs();(self.root/core.STATE).unlink()
  asked=[]
  self.assertEqual(launch.maintenance.repair(self.root,lambda plan:asked.append(plan) or True,**inputs),"installed")
  self.assertEqual(len(asked),1);self.assertEqual(core.installed(self.root)["version"],self.m["version"])
 def test_signed_repair_does_not_require_a_working_local_registry(self):
  self.install()
  data=dict(self.data);m=copy.deepcopy(self.m);name="ASX/Client/supported-games.json"
  data[name]=core.encoded({"schema":1,"games":{"0.125.0":{"sha256":core.digest(self.exe)}}})
  m["files"][name]={"size":len(data[name]),"sha256":core.digest(data[name])}
  inputs=self.repair_inputs(m,data);(self.root/name).unlink()
  with patch.object(core,"supported_native_games",side_effect=FileNotFoundError("registry missing")):
   self.assertEqual(launch.maintenance.repair(self.root,lambda plan:True,**inputs),"installed")
  self.assertTrue((self.root/name).is_file())
 def test_signed_package_registry_mismatch_is_rejected(self):
  self.install();self.configure();m,data=self.release(2)
  name="ASX/Client/supported-games.json";data[name]=core.encoded({"schema":1,"games":{}})
  m["files"][name]={"size":len(data[name]),"sha256":core.digest(data[name])}
  with self.assertRaisesRegex(ValueError,"registry"):self.update(self.signed(m,data,2))
  self.assertEqual(core.installed(self.root)["version"],self.m["version"])
 def test_current_patch_b_inventory(self):
  manifest=copy.deepcopy(self.m)
  manifest["game"]["archives"]=["Data/patch.MPQ",*[f"Data/patch-{n}.MPQ"for n in range(2,10)],"Data/patch-A.MPQ","Data/patch-B.MPQ"]
  self.assertEqual(core.validate(manifest),manifest)
  for name in ("Data/../patch-B.MPQ","Data/patch-B.MPQ/extra"):
   invalid=copy.deepcopy(manifest);invalid["game"]["archives"][-1]=name
   with self.assertRaises(ValueError):core.validate(invalid)
 def test_future_native_mpq_names_do_not_require_a_parser_release(self):
  manifest=copy.deepcopy(self.m)
  manifest["game"]["archives"] += ["Data/patch-C.MPQ","Data/patch-10.MPQ","Data/patch-AB.MPQ","Data/locale-enUS.mpq","Data/expansion_2.MPQ"]
  self.assertEqual(core.validate(manifest),manifest)
 def test_native_inventory_rejects_unsafe_windows_paths_devices_and_aliases(self):
  invalid=[None,[],"Data/patch.MPQ",[7],["Data/patch.MPQ"]*1025]
  names=["Data/../escape.MPQ","Data/sub/patch.MPQ","../Data/patch.MPQ","/Data/patch.MPQ","C:/Data/patch.MPQ","Data\\patch.MPQ","//server/share/patch.MPQ","Data/patch.MPQ:stream","Data/patch.MPQ/extra","Data/patch.MPQ.","Data/patch.MPQ ","Data/.hidden.MPQ","Data/patch%2fescape.MPQ","Data/CON.MPQ","Data/nul.mpq","Data/AuX.MPQ","Data/COM1.MPQ","Data/lpt9.MPQ","Data/patch-É.MPQ","Data/patch\n.MPQ","Data/patch.EXE","Data/"+"x"*121+".MPQ"]
  invalid += [[name]for name in names]
  invalid += [["Data/patch.MPQ","Data/patch.MPQ"],["Data/patch.MPQ","Data/PATCH.mpq"]]
  for archives in invalid:
   with self.subTest(archives=archives):
    manifest=copy.deepcopy(self.m);manifest["game"]["archives"]=archives
    with self.assertRaisesRegex(ValueError,"native archive inventory"):core.validate(manifest)
 def test_signed_update_accepts_new_native_dependencies_without_owning_them(self):
  self.install();m,data=self.release(2)
  added={"Data/patch-C.MPQ":b"native C","Data/patch-10.MPQ":b"native 10","Data/locale-enUS.MPQ":b"native locale"}
  for name,value in added.items():core.atomic(core.path(self.root,name),value)
  m["game"]["archives"]+=list(added)
  self.assertEqual(self.update(self.signed(m,data,2)),"installed")
  for name,value in added.items():
   self.assertEqual(core.path(self.root,name).read_bytes(),value)
   self.assertNotIn(name,core.installed(self.root)["owned"])
  self.intact()
 def test_new_native_dependency_must_exist_before_update_is_applied(self):
  self.install();before=core.installed(self.root);m,data=self.release(2);m["game"]["archives"].append("Data/patch-C.MPQ")
  with self.assertRaisesRegex(ValueError,"Required native game file missing"):self.update(self.signed(m,data,2))
  self.assertEqual(core.installed(self.root),before);self.assertEqual((self.root/"avrt.dll").read_bytes(),self.data["ASX/Bin/avrt.dll"]);self.intact()
 def test_future_inventory_cannot_bypass_signature_or_exact_game_checks(self):
  m,data=self.release(2);m["game"]["archives"].append("Data/patch-C.MPQ");raw,sig,_=self.signed(m,data,2)
  for signature,game in ((bytes(64),core.digest(self.exe)),(sig,"0"*64)):
   with self.subTest(game=game,signature_valid=signature==sig):
    with self.assertRaises(ValueError):updates.verify(raw,signature,[self.public],game)
 def test_native_dependencies_cannot_enter_the_signed_mod_payload(self):
  m,data=self.release(2);native="Data/patch-C.MPQ";m["files"][native]={"size":0,"sha256":core.digest(b"")}
  with self.assertRaises(ValueError):core.validate(m)
 def test_native_archive_symlinks_are_rejected_before_installation(self):
  outside=self.root/"outside.MPQ";outside.write_bytes(b"not an owned native dependency")
  link=self.root/"Data/patch-C.MPQ"
  try:link.symlink_to(outside)
  except OSError:self.skipTest("Symlinks unavailable")
  m,data=self.release(2);m["game"]["archives"].append("Data/patch-C.MPQ")
  with self.assertRaisesRegex(ValueError,"will not follow links"):core.apply(self.root,m,data,manual=True)
  self.assertFalse((self.root/"avrt.dll").exists());self.intact()
 def test_package_signing_refuses_a_wrong_key_before_creating_output(self):
  spec=importlib.util.spec_from_file_location("asx_package_test",ROOT/"tools/build/asx_package.py")
  package=importlib.util.module_from_spec(spec);spec.loader.exec_module(package)
  keyfile=self.root/"disposable-key.pem"
  keyfile.write_bytes(self.key.private_bytes(serialization.Encoding.PEM,serialization.PrivateFormat.PKCS8,serialization.NoEncryption()))
  dll=self.root/"normal.dll";dll.write_bytes(b"MZ normal test artifact")
  output=self.root/"release-output"
  with self.assertRaisesRegex(ValueError,"trust keys"):
   package.build(output,dll,self.root/"unused-runtime",key=keyfile,game_version="0.130.1")
  self.assertFalse(output.exists())
 def test_package_refuses_retired_game_before_reading_artifacts_or_creating_output(self):
  spec=importlib.util.spec_from_file_location("asx_package_retired",ROOT/"tools/build/asx_package.py")
  package=importlib.util.module_from_spec(spec);spec.loader.exec_module(package)
  output=self.root/"retired-release-output"
  with self.assertRaisesRegex(ValueError,"reviewed ASX distribution"):
   package.build(output,self.root/"absent.dll",self.root/"unused-runtime",game_version="0.121.0")
  self.assertFalse(output.exists())
 def test_native_current_inventory_accepts_patch_B_and_keeps_path_validation(self):
  manifest=copy.deepcopy(self.m);manifest["game"]["archives"].append("Data/patch-B.MPQ")
  core.validate(manifest)
  for name in ("Data/../patch-B.MPQ","Data/patch-B.MPQ/extra"):
   invalid=copy.deepcopy(manifest);invalid["game"]["archives"][-1]=name
   with self.assertRaises(ValueError):core.validate(invalid)
 def test_install_preserves_native_profile_and_detects_existing_loader(self):
  core.atomic(self.root/"avrt.dll",b"some other loader")
  with self.assertRaises(ValueError):self.install()
  self.assertEqual((self.root/"avrt.dll").read_bytes(),b"some other loader");self.intact()
  (self.root/"avrt.dll").unlink();self.assertEqual(self.install(),"installed");self.intact();self.assertEqual(self.install(),"current")
 def test_native_settings_and_progression_paths_are_shared_without_redirect(self):
  env={"APPDATA":str(self.root/"appdata"),"LOCALAPPDATA":"native-local","BENILLA_HOME":str(self.root/"benilla-config"),"WOW_ROGUE_SAVE":str(self.root/"progression.txt"),"AZCOOP_TEST_CRASH":"1","COOP_OLD":"1","ASX_ACTIVE":"old","native":"yes"}
  save,home=core.native_paths(self.root,env);self.assertEqual(save,self.root/"progression.txt")
  child=launch.child_environment(self.root,env,self.root/"ASX/Logs/session")
  for name in("APPDATA","LOCALAPPDATA","BENILLA_HOME","WOW_ROGUE_SAVE"):self.assertEqual(child[name],env[name])
  self.assertNotIn("AZCOOP_TEST_CRASH",child);self.assertNotIn("COOP_OLD",child);self.assertEqual(child["ASX_ACTIVE"],"1")
  self.assertEqual(core.native_paths(self.root,{"APPDATA":str(self.root/"appdata")})[0],self.root/"appdata/wenilla-rogue/characters.txt")
  backup=launch.backup(self.root,save);self.assertEqual(backup.read_bytes(),save.read_bytes());self.intact()
 def test_goblins_can_be_disabled_without_disabling_asx_or_native_profile(self):
  core.atomic(self.root/"ASX/options.json",b'{"treasure_goblins":false}')
  env={"APPDATA":"native-profile","ASX_TREASURE_GOBLINS":"1"}
  child=launch.child_environment(self.root,env,self.root/"ASX/Logs/session")
  self.assertEqual(child["ASX_TREASURE_GOBLINS"],"0")
  self.assertEqual(child["ASX_ACTIVE"],"1");self.assertEqual(child["AZCOOP_ENGINE"],"official")
  self.assertEqual(child["APPDATA"],"native-profile")
 def test_optional_events_are_independent_of_each_other_and_coop(self):
  core.atomic(self.root/"ASX/options.json",b'{"treasure_goblins":false,"darkmoon_trials":false,"lost_supplies":true}')
  env=launch.child_environment(self.root,{},self.root/"session")
  self.assertEqual(env["ASX_ACTIVE"],"1")
  self.assertEqual(env["AZCOOP_ENGINE"],"official")
  self.assertEqual(env["ASX_TREASURE_GOBLINS"],"0")
  self.assertEqual(env["ASX_DARKMOON_TRIALS"],"0")
  self.assertEqual(env["ASX_LOST_SUPPLIES"],"1")
 def test_invalid_optional_goblin_settings_do_not_stop_core_asx(self):
  for value in (b'bad json',b'{"treasure_goblins":"false"}',b'[]'):
   core.atomic(self.root/"ASX/options.json",value)
   child=launch.child_environment(self.root,{},self.root/"ASX/Logs/session")
   self.assertEqual(child["ASX_TREASURE_GOBLINS"],"0");self.assertEqual(child["ASX_ACTIVE"],"1")
 def test_unsupported_client_never_receives_proxy_or_addon(self):
  core.atomic(self.root/"AzerothSurvivors.exe",b"new official build")
  with self.assertRaises(ValueError):self.install()
  self.assertFalse((self.root/"avrt.dll").exists());self.assertFalse((self.root/"benilla-config/AddOns/ASX").exists())
 def test_addon_collision_and_user_edits_are_preserved(self):
  core.atomic(self.root/"benilla-config/AddOns/ASX/Presentation.lua",b"unknown addon")
  with self.assertRaises(ValueError):self.install()
  self.assertFalse((self.root/"avrt.dll").exists())
  (self.root/"benilla-config/AddOns/ASX/Presentation.lua").unlink();self.install()
  core.atomic(self.root/"avrt.dll",b"user changed loader")
  m,data=self.release(2)
  with self.assertRaises(ValueError):core.apply(self.root,m,data)
  self.assertEqual((self.root/"avrt.dll").read_bytes(),b"user changed loader");self.intact()
 def test_signed_updates_remove_obsolete_files_and_rollback_exactly(self):
  self.install();m,data=self.release(2,extra=False);self.assertEqual(self.update(self.signed(m,data,2)),"installed")
  self.assertFalse((self.root/"ASX/Client/old.py").exists());self.intact()
  with core.locked(self.root):self.assertEqual(core.rollback(self.root),"restored")
  self.assertEqual((self.root/"avrt.dll").read_bytes(),self.data["ASX/Bin/avrt.dll"]);self.assertTrue((self.root/"ASX/Client/old.py").exists())
  self.assertIn("held",self.update(self.signed(m,data,2)));self.intact()
 def test_signature_archive_and_game_mismatch_reject_before_mutation(self):
  self.install();m,data=self.release(2);raw,sig,package=self.signed(m,data,2)
  for signature,game in ((bytes(64),core.digest(self.exe)),(sig,"0"*64)):
   with self.assertRaises(ValueError):updates.verify(raw,signature,[self.public],game)
  f=updates.verify(raw,sig,[self.public],core.digest(self.exe))
  with self.assertRaises(ValueError):updates.unpack(package+b"tampered",f)
  self.assertEqual(core.installed(self.root)["version"],self.m["version"]);self.intact()
 def test_transaction_failure_restores_proxy_payload_and_receipt(self):
  self.install();old=core.installed(self.root);m,data=self.release(2,extra=False)
  def fail(name):
   if name=="avrt.dll":raise OSError("simulated write failure")
  with core.locked(self.root):
   with self.assertRaises(OSError):core.apply(self.root,m,data,fault=fail)
  self.assertEqual(core.installed(self.root),old);self.assertEqual((self.root/"avrt.dll").read_bytes(),self.data["ASX/Bin/avrt.dll"]);self.intact()
 def test_recovery_restores_a_process_interrupted_after_a_write(self):
  self.install();old=core.installed(self.root);tx="a"*32;base="ASX/Updates/Transactions/"+tx
  before=(self.root/"avrt.dll").read_bytes();core.atomic(core.path(self.root,base+"/files/avrt.dll"),before)
  core.put(self.root,base+"/journal.json",{"phase":"applying","state":old,"before":{"avrt.dll":core.digest(before),"ASX/Client/half.py":None}})
  core.atomic(self.root/"avrt.dll",b"half update");core.atomic(self.root/"ASX/Client/half.py",b"half update")
  with core.locked(self.root):core.recover(self.root)
  self.assertEqual((self.root/"avrt.dll").read_bytes(),before);self.assertFalse((self.root/"ASX/Client/half.py").exists());self.intact()
 def test_interrupted_client_upgrade_restarts_before_validating_a_restored_legacy_inventory(self):
  self.install()
  registry="ASX/Client/supported-games.json";client="ASX/Client/core.py"
  old_manifest=copy.deepcopy(self.m);old_manifest["files"].pop(registry)
  old_client=b"# previous exact-0.125 client without a registry dependency\n"
  old_manifest["files"][client]={"size":len(old_client),"sha256":core.digest(old_client)}
  core.atomic(core.path(self.root,client),old_client)
  core.path(self.root,registry).unlink()
  core.put(self.root,core.MANIFEST,old_manifest)
  old_state=core.installed(self.root)
  old_state["owned"].pop(registry)
  old_state["owned"][client]=core.digest(old_client)
  old_state["owned"][core.MANIFEST]=core.sha(core.path(self.root,core.MANIFEST))
  core.put(self.root,core.STATE,old_state)
  tx="b"*32;base="ASX/Updates/Transactions/"+tx
  core.atomic(core.path(self.root,base+"/files/"+client),old_client)
  core.put(self.root,base+"/journal.json",{"phase":"applying","state":old_state,"before":{client:core.digest(old_client),registry:None}})
  core.atomic(core.path(self.root,client),b"# partially upgraded client\n")
  core.atomic(core.path(self.root,registry),self.data[registry])
  with patch.object(launch,"confirm_repair",return_value=True),patch.object(core,"game_running",return_value=False),patch.object(core,"native_paths",return_value=(self.root/"progression.txt",self.root/"benilla-config")),patch.object(core,"manifest",side_effect=AssertionError("Stale client validated restored files")),patch.object(launch.subprocess,"Popen",side_effect=AssertionError("Game launched during recovery")):
   self.assertEqual(launch.run(self.root,offline=True),75)
  self.assertEqual(core.path(self.root,client).read_bytes(),old_client)
  self.assertFalse(core.path(self.root,registry).exists())
  self.assertEqual(core.installed(self.root),old_state)
  self.assertFalse(core.recover(self.root),"Restored journal must not cause a restart loop")
  self.intact()
 def test_path_traversal_native_paths_and_case_collisions_are_not_payloads(self):
  for name in("../oops","ASX/Client/../../Data/patch.MPQ","Data/patch.MPQ","benilla-config/config.toml","ASX/Backups/user.txt","ASX/Client/con.txt","ASX/Client/a:stream","ASX/Client/game.pem"):
   with self.assertRaises(ValueError,msg=name):core.payload_name(name)
  m=copy.deepcopy(self.m);m["files"]["ASX/Client/OLD.py"]=m["files"]["ASX/Client/old.py"]
  with self.assertRaises(ValueError):core.validate(m)
 def test_manual_archive_upgrade_does_not_offer_incomplete_rollback(self):
  self.install();m,data=self.release(2);self.extract(m,data)
  with core.locked(self.root):core.apply(self.root,m,data,manual=True)
  with self.assertRaisesRegex(ValueError,"no previous"):
   with core.locked(self.root):core.rollback(self.root)
  self.assertEqual(core.installed(self.root)["version"],m["version"]);self.intact()
 def test_signed_manifest_rejects_non_string_hash_and_version(self):
  for field in("version","game","files"):
   value=copy.deepcopy(self.m)
   if field=="version":value[field]=7
   elif field=="game":value[field]["sha256"]=None
   else:value[field]["ASX/Bin/avrt.dll"]["sha256"]=42
   with self.assertRaises(ValueError):core.validate(value)
 def test_symlink_target_is_rejected(self):
  other=self.root/"outside";other.mkdir();(self.root/"ASX/State").mkdir(parents=True)
  try:(self.root/"ASX/Bin").rename(self.root/"ASX/Bin-original");(self.root/"ASX/Bin").symlink_to(other,target_is_directory=True)
  except OSError:self.skipTest("Symlink creation unavailable")
  with self.assertRaises(ValueError):core.path(self.root,"ASX/Bin/avrt.dll")
 def test_first_install_adoption_does_not_deploy_an_unsupported_proxy(self):
  core.adopt(self.root,self.m,self.data);self.assertFalse((self.root/"avrt.dll").exists());self.assertEqual(core.installed(self.root)["version"],self.m["version"])
 def test_uninstall_preserves_native_data_unknown_files_and_changed_owned_files(self):
  self.install();core.atomic(self.root/"ASX/Client/personal.txt",b"untracked");core.atomic(self.root/"ASX/Logs/session/a.log",b"diagnostics")
  core.atomic(self.root/"ASX/Client/old.py",b"user changed this")
  if os.name=="nt":self.skipTest("Windows deferred cleanup is covered by the actual packaged CLI test")
  sys.path.insert(0,str(ROOT/"asx"));import remove
  with core.locked(self.root):preserved=remove.uninstall(self.root)
  self.assertIn("ASX/Client/old.py",preserved);self.assertFalse((self.root/"avrt.dll").exists());self.assertTrue((self.root/"ASX/Client/personal.txt").exists());self.assertTrue((self.root/"ASX/Logs/session/a.log").exists());self.intact()
 def test_export_omits_progression_settings_and_backups(self):
  self.install();core.atomic(self.root/"ASX/Logs/session/native.log",b"run diagnostics");core.atomic(self.root/"ASX/Backups/secret.txt",b"private progression")
  report=launch.export(self.root)
  with zipfile.ZipFile(report)as z:
   self.assertIn("Logs/session/native.log",z.namelist());self.assertFalse(any("secret"in n or"progression"in n.lower()or"config.toml"in n for n in z.namelist()))
  self.intact()
def base64key(key):
 import base64
 return base64.b64encode(key.public_key().public_bytes(serialization.Encoding.Raw,serialization.PublicFormat.Raw)).decode()
if __name__=="__main__":unittest.main()


class NativeCompatibilityTests(unittest.TestCase):
 def test_only_current_registry_entry_and_manifest_exact_hash(self):
  games=json.loads((ROOT/"asx/supported-games.json").read_text())["games"]
  self.assertEqual(list(games),["0.130.1"])
  m={"schema":1,"product":core.PRODUCT,"version":"0.130.1-asx.1","game":{"version":"0.130.1","sha256":games["0.130.1"]["sha256"],"archives":["Data/patch.MPQ"]},"files":{}}
  required=["start-asx.cmd","README-ASX.txt","ASX/Bin/avrt.dll","ASX/Client/launch.py","ASX/Client/supported-games.json","ASX/Interface/ASX.toc","ASX/Interface/Presentation.lua","ASX/Defaults/channel.json"]
  m["files"]={name:{"size":0,"sha256":"0"*64}for name in required}
  self.assertIs(core.validate(m),m)
  for version,sha in (("0.121.0","22f7938eb4f99f2857dc091b1c57c147bd848eaabfeaa81e444c0a3393ca8150"),("0.130.1","0"*64)):
   rejected=copy.deepcopy(m);rejected["game"].update(version=version,sha256=sha)
   with self.subTest(version=version):
    with self.assertRaisesRegex(ValueError,"exact reviewed native game build"):core.validate(rejected)
