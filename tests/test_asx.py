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
  with core.locked(self.root):return updates.automatic(self.root,core.digest(self.exe),download)
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
   package.build(output,dll,self.root/"unused-runtime",key=keyfile,game_version="0.125.0")
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
  with patch.object(core,"game_running",return_value=False),patch.object(core,"native_paths",return_value=(self.root/"progression.txt",self.root/"benilla-config")),patch.object(core,"manifest",side_effect=AssertionError("Stale client validated restored files")),patch.object(launch.subprocess,"Popen",side_effect=AssertionError("Game launched during recovery")):
   self.assertEqual(launch.run(self.root,offline=True,check=True),75)
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
  self.assertEqual(list(games),["0.125.0"])
  m={"schema":1,"product":core.PRODUCT,"version":"0.125.0-asx.1","game":{"version":"0.125.0","sha256":games["0.125.0"]["sha256"],"archives":["Data/patch.MPQ"]},"files":{}}
  required=["start-asx.cmd","README-ASX.txt","ASX/Bin/avrt.dll","ASX/Client/launch.py","ASX/Client/supported-games.json","ASX/Interface/ASX.toc","ASX/Interface/Presentation.lua","ASX/Defaults/channel.json"]
  m["files"]={name:{"size":0,"sha256":"0"*64}for name in required}
  self.assertIs(core.validate(m),m)
  for version,sha in (("0.121.0","22f7938eb4f99f2857dc091b1c57c147bd848eaabfeaa81e444c0a3393ca8150"),("0.125.0","0"*64)):
   rejected=copy.deepcopy(m);rejected["game"].update(version=version,sha256=sha)
   with self.subTest(version=version):
    with self.assertRaisesRegex(ValueError,"exact reviewed native game build"):core.validate(rejected)
