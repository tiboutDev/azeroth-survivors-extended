"""Native update eligibility requires official identity and signed AZSX support."""
import json,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'azsx'))
import native_update,launch
class NativeUpdateTests(unittest.TestCase):
 def data(self,version='0.139.0',sha='a'*64):return lambda:json.dumps({'version':version,'sha256':sha,'parts':[]}).encode()
 def test_same_native_build_never_requests_an_update(self):
  self.assertIsNone(native_update.compatible(None,'a'*64,download=self.data(),offer=lambda *_:self.fail('offer must not be fetched')))
 def test_new_build_requires_matching_signed_azsx_manifest(self):
  calls=[]
  def offer(root,sha):calls.append(sha);return ({},b'',b'',{'manifest':{'game':{'version':'0.139.0','sha256':sha}}})
  self.assertEqual(native_update.compatible(None,'b'*64,download=self.data(),offer=offer),'a'*64)
  self.assertEqual(calls,['a'*64])
  self.assertIsNone(native_update.compatible(None,'b'*64,download=self.data(),offer=lambda *_:({},b'',b'',{'manifest':{'game':{'version':'0.140.0'}}})))
 def test_untrusted_announcement_is_rejected(self):
  for payload in (b'[]',b'{}',self.data(sha='a'*63)(),self.data(version='../0.139.0')(),b'x'*16385):
   with self.assertRaises((ValueError,TypeError,json.JSONDecodeError)):native_update.announcement(lambda:payload)
 def test_native_updater_is_disabled_without_verified_eligibility(self):
  from tempfile import TemporaryDirectory
  with TemporaryDirectory() as root:
   from pathlib import Path
   no=launch.child_environment(Path(root),{},Path(root),False)
   yes=launch.child_environment(Path(root),{},Path(root),True)
   self.assertEqual((no['WOW_ROGUE_UPDATE'],no['WOW_ROGUE_UPDATE_AUTO']),('0','0'))
   self.assertEqual((yes['WOW_ROGUE_UPDATE'],yes['WOW_ROGUE_UPDATE_AUTO']),('1','1'))
