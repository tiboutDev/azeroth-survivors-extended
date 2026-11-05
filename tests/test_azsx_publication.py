"""Static publication integrity for minimal and historical release inventories."""
import base64,hashlib,importlib.util,json,tempfile,unittest,io,zipfile
from pathlib import Path
from unittest.mock import patch
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from cryptography.exceptions import InvalidSignature
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location("publication",ROOT/"tools/build/azsx_verify_publication.py")
publication=importlib.util.module_from_spec(spec);spec.loader.exec_module(publication)
class PublicationTests(unittest.TestCase):
 def prepare(self,legacy=False,version="0.125.0-azsx.2"):
  temp=tempfile.TemporaryDirectory();self.addCleanup(temp.cleanup);root=Path(temp.name);(root/"UPLOAD").mkdir()
  key=Ed25519PrivateKey.generate();dll=b"MZ normal release DLL fixture"
  game={"version":version.split("-")[0],"sha256":"a"*64}
  manifest={"version":version,"game":game,"files":{"AZSX/App/Bin/avrt.dll":{"size":len(dll),"sha256":publication.digest(dll)}}}
  stream=io.BytesIO()
  with zipfile.ZipFile(stream,"w")as z:z.writestr("AZSX/App/Bin/avrt.dll",dll)
  archive=stream.getvalue()
  feed=json.dumps(dict(product="azeroth-survivors-extended",sequence=12,manifest=manifest,package=dict(asset="azsx-"+version+".zip",sha256=publication.digest(archive)))).encode()
  data={"AZSX.zip":archive,"azsx-"+version+".zip":archive,"azsx-latest.json":feed,"azsx-latest.json.sig":key.sign(feed)}
  if legacy:data.update({"latest.json":b"legacy feed","latest.json.sig":key.sign(b"legacy feed"),"coop-old.zip":b"legacy archive"})
  meta=dict(repository="example/azsx",tag="v"+version,version=version,sequence=12,public_key=base64.b64encode(key.public_key().public_bytes(serialization.Encoding.Raw,serialization.PublicFormat.Raw)).decode(),assets={n:dict(size=len(b),sha256=publication.digest(b))for n,b in data.items()})
  for n,b in data.items():(root/"UPLOAD"/n).write_bytes(b)
  (root/"PUBLICATION.json").write_text(json.dumps(meta));return root,meta
 def test_minimal_four_files_verify_without_network_or_legacy_assets(self):
  root,_=self.prepare()
  with patch.object(publication,"download",side_effect=AssertionError("local verification accessed network")):
   result=publication.verify(root,True)
  self.assertEqual((result["assets"],result["signed_feeds"]),(4,1));self.assertFalse(result["game_launched"])
 def test_historical_legacy_signed_feed_still_verified(self):
  root,_=self.prepare(True);result=publication.verify(root,True);self.assertEqual(result["signed_feeds"],2)
 def test_signature_must_verify_even_when_file_hash_matches_metadata(self):
  root,meta=self.prepare();signature=bytes(64);(root/"UPLOAD/azsx-latest.json.sig").write_bytes(signature);meta["assets"]["azsx-latest.json.sig"]["sha256"]=publication.digest(signature);(root/"PUBLICATION.json").write_text(json.dumps(meta))
  with self.assertRaises(InvalidSignature):publication.verify(root,True)
 def test_legacy_feed_cannot_be_silently_left_unverified(self):
  root,meta=self.prepare(True);del meta["assets"]["latest.json.sig"];(root/"PUBLICATION.json").write_text(json.dumps(meta))
  with self.assertRaisesRegex(ValueError,"Incomplete legacy"):publication.verify(root,True)
 def test_changed_player_alias_is_rejected(self):
  root,_=self.prepare();(root/"UPLOAD/AZSX.zip").write_bytes(b"changed")
  with self.assertRaisesRegex(ValueError,"checksum"):publication.verify(root,True)

class ReleaseLayoutTests(PublicationTests):
 def layout(self):
  import sys
  sys.path.insert(0,str(ROOT/'tools/build'))
  import azsx_release_layout
  return azsx_release_layout
 def release(self,version="0.125.0-azsx.2"):
  root,meta=self.prepare(version=version)
  for name in ('README-PREPARATION.md','RELEASE-NOTES.md','PUBLISH-ON-GITHUB.md'):(root/name).write_text(name)
  (root/'VALIDATION.json').write_text('{"technical_validation_passed":true,"scope":"static fixture only"}')
  (root/'SOURCE-COMMIT.json').write_text(json.dumps({"release_source_commit":"1"*40,"payload_source_commit":"1"*40}))
  feed=json.loads((root/'UPLOAD/azsx-latest.json').read_bytes());spec=feed["manifest"]["files"]["AZSX/App/Bin/avrt.dll"]
  report={"status":"verified","builder":"xwin-msvc","source":{"commit":"1"*40,"dirty":False},
          "artifact":{"file":"avrt.dll","target":"x86_64-pc-windows-msvc","game_version":feed["manifest"]["game"]["version"],"sha256":spec["sha256"],"size_bytes":spec["size"]},
          "verification":{"independent_fresh_builds":2,"target_directories_distinct":True,"artifacts_byte_identical":True,"source_unchanged_before_between_after":True},
          "source_fingerprint":{"sha256":"b"*64},"build_recipe_sha256":"c"*64}
  (root/'REPRODUCIBILITY.json').write_text(json.dumps(report))
  (root/'test.log').write_bytes(b'complete preserved evidence')
  return root,meta
 def test_finalize_preserves_assets_evidence_and_verification_stays_outside(self):
  root,meta=self.release();evidence=root.parent/(root.name+'-evidence');self.addCleanup(__import__('shutil').rmtree,evidence,True)
  before={p.name:p.read_bytes()for p in(root/'UPLOAD').iterdir()}
  self.layout().finalize(root,evidence)
  self.assertEqual({p.name for p in root.iterdir()},self.layout().KEEP)
  self.assertEqual(before,{p.name:p.read_bytes()for p in(root/'UPLOAD').iterdir()})
  self.assertEqual((evidence/'test.log').read_bytes(),b'complete preserved evidence')
  inventory=(evidence/'layout-inventory.json').read_bytes()
  publication.verify(root,True,evidence)
  self.assertFalse((root/'LOCAL-VERIFICATION.json').exists())
  self.layout().finalize(root,evidence)
  self.assertEqual(inventory,(evidence/'layout-inventory.json').read_bytes())
 def test_revision_titles_and_assets_survive_finalization_without_renaming_history(self):
  for version,title in (
   ("0.141.0-r1","0.141.0 — AZSX Update 1"),
   ("0.141.0-r2","0.141.0 — AZSX Update 2"),
   ("0.142.0-r1","0.142.0 — AZSX Update 1"),
   ("0.139.2-azsx.1","AZSX 0.139.2-azsx.1"),
  ):
   with self.subTest(version=version):
    root,meta=self.release(version);evidence=root.parent/(root.name+'-evidence')
    self.addCleanup(__import__('shutil').rmtree,evidence,True)
    before={p.name:p.read_bytes()for p in(root/'UPLOAD').iterdir()}
    self.layout().finalize(root,evidence)
    self.assertTrue((root/'README-PREPARATION.md').read_text().startswith("# "+title+"\n"))
    guide=(root/'PUBLISH-ON-GITHUB.md').read_text()
    self.assertIn("GitHub release title: **"+title+"**.",guide)
    self.assertIn("Prepared tag **v"+version+"**, sequence **12**.",guide)
    self.assertEqual((root/'RELEASE-NOTES.md').read_text(),'RELEASE-NOTES.md')
    self.assertEqual(before,{p.name:p.read_bytes()for p in(root/'UPLOAD').iterdir()})
    self.assertEqual(json.loads((evidence/'PUBLICATION.json').read_text()),meta)
 def test_conflicting_evidence_does_not_remove_any_original(self):
  root,_=self.release();evidence=root.parent/(root.name+'-evidence');evidence.mkdir();self.addCleanup(__import__('shutil').rmtree,evidence,True)
  (evidence/'test.log').write_bytes(b'previous different proof')
  with self.assertRaisesRegex(ValueError,'conflict'):self.layout().finalize(root,evidence)
  self.assertEqual((root/'test.log').read_bytes(),b'complete preserved evidence')
  self.assertEqual((evidence/'test.log').read_bytes(),b'previous different proof')
 def test_bad_signed_asset_preserves_original_evidence(self):
  root,_=self.release();(root/'UPLOAD/AZSX.zip').write_bytes(b'corrupted')
  evidence=root.parent/(root.name+'-evidence')
  with self.assertRaisesRegex(ValueError,'checksum'):self.layout().finalize(root,evidence)
  self.assertTrue((root/'test.log').exists());self.assertFalse(evidence.exists())

 def test_new_release_identity_is_automatic_frozen_and_bound_to_archive(self):
  root,meta=self.release("0.141.0-r1");evidence=root.parent/(root.name+'-evidence')
  self.addCleanup(__import__('shutil').rmtree,evidence,True)
  self.layout().finalize(root,evidence)
  record=json.loads((evidence/'RELEASE-IDENTITY.json').read_text())
  self.assertEqual(record["version"],"0.141.0-r1")
  self.assertEqual(record["source"]["payload_source_commit"],"1"*40)
  with zipfile.ZipFile(root/'UPLOAD/azsx-0.141.0-r1.zip')as z:dll=z.read("AZSX/App/Bin/avrt.dll")
  self.assertEqual(record["artifact"]["sha256"],publication.digest(dll))
  self.assertEqual(record["validation"]["results"]["scope"],"static fixture only")
  self.assertNotIn("approval",(root/'README-PREPARATION.md').read_text().lower())
  before=(evidence/'RELEASE-IDENTITY.json').read_bytes()
  publication.verify(root,True,evidence);self.layout().finalize(root,evidence)
  self.assertEqual(before,(evidence/'RELEASE-IDENTITY.json').read_bytes())
  source=json.loads((evidence/'SOURCE-COMMIT.json').read_text());source["payload_source_commit"]="2"*40
  (evidence/'SOURCE-COMMIT.json').write_text(json.dumps(source))
  with self.assertRaisesRegex(ValueError,"identity differs"):publication.verify(root,True,evidence)
 def test_wrong_reproducibility_artifact_preserves_preparation_and_old_evidence(self):
  root,_=self.release("0.141.0-r1");evidence=root.parent/(root.name+'-evidence')
  report=json.loads((root/'REPRODUCIBILITY.json').read_text());report["artifact"]["sha256"]="0"*64
  (root/'REPRODUCIBILITY.json').write_text(json.dumps(report))
  with self.assertRaisesRegex(ValueError,"DLL differs"):self.layout().finalize(root,evidence)
  self.assertFalse(evidence.exists());self.assertTrue((root/'test.log').exists())
 def test_missing_identity_inputs_do_not_turn_into_an_approval_prompt(self):
  root,_=self.release("0.141.0-r1");(root/'REPRODUCIBILITY.json').unlink()
  evidence=root.parent/(root.name+'-evidence')
  with self.assertRaises(FileNotFoundError):self.layout().finalize(root,evidence)
  self.assertTrue((root/'test.log').exists())
