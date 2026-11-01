"""Static publication integrity for minimal and historical release inventories."""
import base64,hashlib,importlib.util,json,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from cryptography.exceptions import InvalidSignature
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location("publication",ROOT/"tools/build/asx_verify_publication.py")
publication=importlib.util.module_from_spec(spec);spec.loader.exec_module(publication)
class PublicationTests(unittest.TestCase):
 def prepare(self,legacy=False):
  temp=tempfile.TemporaryDirectory();self.addCleanup(temp.cleanup);root=Path(temp.name);(root/"UPLOAD").mkdir()
  key=Ed25519PrivateKey.generate();archive=b"static hash fixture, not an executable archive"
  feed=json.dumps(dict(product="azeroth-survivors-extended",sequence=12,manifest=dict(version="0.125.0-asx.2"),package=dict(asset="asx-0.125.0-asx.2.zip",sha256=publication.digest(archive)))).encode()
  data={"ASX.zip":archive,"asx-0.125.0-asx.2.zip":archive,"asx-latest.json":feed,"asx-latest.json.sig":key.sign(feed)}
  if legacy:data.update({"latest.json":b"legacy feed","latest.json.sig":key.sign(b"legacy feed"),"coop-old.zip":b"legacy archive"})
  meta=dict(repository="example/asx",tag="v0.125.0-asx.2",version="0.125.0-asx.2",sequence=12,public_key=base64.b64encode(key.public_key().public_bytes(serialization.Encoding.Raw,serialization.PublicFormat.Raw)).decode(),assets={n:dict(size=len(b),sha256=publication.digest(b))for n,b in data.items()})
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
  root,meta=self.prepare();signature=bytes(64);(root/"UPLOAD/asx-latest.json.sig").write_bytes(signature);meta["assets"]["asx-latest.json.sig"]["sha256"]=publication.digest(signature);(root/"PUBLICATION.json").write_text(json.dumps(meta))
  with self.assertRaises(InvalidSignature):publication.verify(root,True)
 def test_legacy_feed_cannot_be_silently_left_unverified(self):
  root,meta=self.prepare(True);del meta["assets"]["latest.json.sig"];(root/"PUBLICATION.json").write_text(json.dumps(meta))
  with self.assertRaisesRegex(ValueError,"Incomplete legacy"):publication.verify(root,True)
 def test_changed_player_alias_is_rejected(self):
  root,_=self.prepare();(root/"UPLOAD/ASX.zip").write_bytes(b"changed")
  with self.assertRaisesRegex(ValueError,"checksum"):publication.verify(root,True)

class ReleaseLayoutTests(PublicationTests):
 def layout(self):
  import sys
  sys.path.insert(0,str(ROOT/'tools/build'))
  import asx_release_layout
  return asx_release_layout
 def release(self):
  root,meta=self.prepare()
  for name in ('README-PREPARATION.md','RELEASE-NOTES.md','PUBLISH-ON-GITHUB.md'):(root/name).write_text(name)
  (root/'VALIDATION.json').write_text('{"technical_validation_passed":true,"reference_approved":true}')
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
 def test_conflicting_evidence_does_not_remove_any_original(self):
  root,_=self.release();evidence=root.parent/(root.name+'-evidence');evidence.mkdir();self.addCleanup(__import__('shutil').rmtree,evidence,True)
  (evidence/'test.log').write_bytes(b'previous different proof')
  with self.assertRaisesRegex(ValueError,'conflict'):self.layout().finalize(root,evidence)
  self.assertEqual((root/'test.log').read_bytes(),b'complete preserved evidence')
  self.assertEqual((evidence/'test.log').read_bytes(),b'previous different proof')
 def test_bad_signed_asset_preserves_original_evidence(self):
  root,_=self.release();(root/'UPLOAD/ASX.zip').write_bytes(b'corrupted')
  evidence=root.parent/(root.name+'-evidence')
  with self.assertRaisesRegex(ValueError,'checksum'):self.layout().finalize(root,evidence)
  self.assertTrue((root/'test.log').exists());self.assertFalse(evidence.exists())
