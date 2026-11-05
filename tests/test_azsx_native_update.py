"""Native update eligibility requires official identity and signed AZSX support."""
import json,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'azsx'))
import native_update,launch
class NativeUpdateTests(unittest.TestCase):
 def data(self,version='0.139.0',sha='a'*64):return lambda:json.dumps({'version':version,'sha256':sha,'parts':[]}).encode()
 def test_valid_announcement_returns_exact_native_identity(self):
  self.assertEqual(native_update.announcement(self.data()),("0.139.0","a"*64))
 def test_untrusted_announcement_is_rejected(self):
  for payload in (b'[]',b'{}',self.data(sha='a'*63)(),self.data(version='../0.139.0')(),b'x'*16385):
   with self.assertRaises((ValueError,TypeError,json.JSONDecodeError)):native_update.announcement(lambda:payload)
 def test_native_updater_is_disabled_without_verified_eligibility(self):
  from tempfile import TemporaryDirectory
  with TemporaryDirectory() as root:
   from pathlib import Path
   no=launch.child_environment(Path(root),{},Path(root))
   self.assertEqual((no['WOW_ROGUE_UPDATE'],no['WOW_ROGUE_UPDATE_AUTO']),('0','0'))
