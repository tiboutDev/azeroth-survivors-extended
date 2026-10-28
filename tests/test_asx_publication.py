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
  feed=json.dumps(dict(product="azeroth-survivors-extended",sequence=12,manifest=dict(version="0.122.0-asx.2"),package=dict(asset="asx-0.122.0-asx.2.zip",sha256=publication.digest(archive)))).encode()
  data={"ASX.zip":archive,"asx-0.122.0-asx.2.zip":archive,"asx-latest.json":feed,"asx-latest.json.sig":key.sign(feed)}
  if legacy:data.update({"latest.json":b"legacy feed","latest.json.sig":key.sign(b"legacy feed"),"coop-old.zip":b"legacy archive"})
  meta=dict(repository="example/asx",tag="v0.122.0-asx.2",version="0.122.0-asx.2",sequence=12,public_key=base64.b64encode(key.public_key().public_bytes(serialization.Encoding.Raw,serialization.PublicFormat.Raw)).decode(),assets={n:dict(size=len(b),sha256=publication.digest(b))for n,b in data.items()})
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
