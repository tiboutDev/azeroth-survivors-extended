"""Exact native loot hooks must match the isolated image and have whole extents."""
import os,re,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class DuplicateLoot(unittest.TestCase):
 def test_exact_reviewed_boundaries(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1201",ROOT/"runtime-official-01201/AzerothSurvivors.exe"))
  if not path.is_file():self.skipTest("Reviewed isolated 0.120.1 image unavailable")
  import sys
  sys.path.insert(0,str(ROOT/"tools/dev"))
  from compare_official import Image
  image=Image(path)
  source=(ROOT/"crates/asx-game/src/bindings/inherited/official1201.rs").read_text().split("pub const LOOT_ENTRIES:")[1].split("];",1)[0]
  entries=re.findall(r"\((0x[0-9a-f]+), &\[([^]]+)\]\)",source)
  self.assertEqual(len(entries),5)
  descriptor=(ROOT/"crates/asx-game/src/bindings/inherited/official1201.rs").read_text()
  fixture=descriptor.split("pub const LOOT_FIXTURE_DISPATCH:",1)[1].split(";",1)[0]
  extra=re.findall(r"\((0x[0-9a-f]+),&\[([^]]+)\]\)",fixture)
  self.assertEqual(len(extra),1);entries+=extra
  # The native builder has already appended '}' and incremented R13 before
  # this hook. Its length must be forwarded exactly, not incremented again.
  before=[i for i in image.ins(image.owner(0x5b03078))if i.address<0x5b03078][-1]
  self.assertEqual((before.mnemonic,before.op_str),("inc","r13"))
  self.assertIn("loot_hide_drop(&io, c.rdx, c.r13)",(ROOT/"src/runtime/duplicate_loot.rs").read_text())
  for rva,raw in entries:
   at=int(rva,16);wanted=bytes(int(n.strip(),0)for n in raw.split(","))
   self.assertEqual(image.pe.get_data(at,len(wanted)),wanted)
   ins=image.ins(image.owner(at));boundaries={i.address for i in ins}|{image.ends[image.owner(at)]}
   self.assertIn(at,boundaries);self.assertIn(at+len(wanted),boundaries)
   for i in ins:
    if i.mnemonic.startswith("j")and i.operands[0].type==2:self.assertFalse(at<i.operands[0].imm<at+len(wanted),f"Branch into patched loot interior: {i.address:x}")
