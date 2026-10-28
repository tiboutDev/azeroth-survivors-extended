"""Read-only 0.122 image/contact preflight; semantic runtime validation is separate."""
import hashlib,json,os,re,sys,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"tools/dev"))
class Native122(unittest.TestCase):
 def test_registry_pins_distinct_exact_adapter(self):
  source=(ROOT/"crates/asx-game/src/bindings/official122.rs").read_text()
  sha=re.search(r'SHA256:&str="([0-9a-f]{64})"',source)[1]
  self.assertEqual(json.loads((ROOT/"asx/supported-games.json").read_text())["games"]["0.122.0"]["sha256"],sha)
  self.assertIn("Revision::V122",(ROOT/"crates/asx-game/src/official_native.rs").read_text())
 def test_native_contacts_guards_and_interior_branches(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_122",ROOT/"runtime-official-01220/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private pinned 0.122 image unavailable")
  from compare_official import Image,MD
  from capstone import CS_GRP_JUMP,CS_GRP_CALL
  image=Image(path);source=(ROOT/"crates/asx-game/src/bindings/official122.rs").read_text()
  self.assertEqual(image.sha,re.search(r'SHA256:&str="([0-9a-f]{64})"',source)[1])
  contacts={int(a,16):int(b,16)for a,b in re.findall(r'\((0x[0-9a-f]+),(0x[0-9a-f]+)\)',source)}
  count=0
  for table in ("ENTRIES","EXTENSION_ENTRIES"):
   body=source[source.index("pub const "+table):];body=body[:body.index("];")]
   for key,data in re.findall(r'\((0x[0-9a-f]+),&\[(.*?)\]\)',body,re.S):
    key=int(key,16);rva=contacts[key];expected=bytes(int(n.strip(),0)for n in data.split(',')if n.strip());self.assertEqual(image.pe.get_data(rva,len(expected)),expected)
    owner=image.owner(rva);ins=image.ins(owner)if owner else list(MD.disasm(image.pe.get_data(rva,64),rva));starts={i.address for i in ins};ends={i.address+i.size for i in ins};self.assertIn(rva,starts);self.assertIn(rva+len(expected),ends)
    # Native detours overwrite whole instructions covering the 5-byte near jump to an allocated relay.
    overwritten=[]
    for i in ins:
     if rva<=i.address<rva+max(5,len(expected)):overwritten.append(i)
    boundary=next((i.address+i.size for i in overwritten if i.address+i.size>=rva+5),rva+len(expected))
    for i in ins:
     if i.group(CS_GRP_JUMP)or i.group(CS_GRP_CALL):
      if i.operands and i.operands[0].type==2:
       target=i.operands[0].imm
       self.assertFalse(rva<target<boundary and not(rva<=i.address<boundary),f"branch into contact {key:x}: {i.address:x}->{target:x}")
    count+=1
  self.assertGreater(count,65)

 def test_retained_tooltip_contacts_and_native_row_contract(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_122",ROOT/"runtime-official-01220/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private 0.122 image unavailable")
  from compare_official import Image
  image=Image(path);source=(ROOT/"crates/asx-game/src/bindings/official122.rs").read_text()
  body=source.split("pub const KEPT_TOOLTIP_BINDINGS:",1)[1].split("];",1)[0]
  rows=re.findall(r"\((0x[0-9a-f]+),&\[(.*?)\]\)",body,re.S)
  self.assertEqual(len(rows),5)
  for at,data in rows:
   at=int(at,16);expected=bytes(int(x,0)for x in data.split(',')if x.strip());self.assertEqual(image.pe.get_data(at,len(expected)),expected)
   ins=image.ins(image.owner(at));self.assertIn(at,{i.address for i in ins});self.assertIn(at+len(expected),{i.address+i.size for i in ins})
  row=int(rows[0][0],16);self.assertEqual(image.pe.get_data(row,4),bytes.fromhex("488d4da0"))
  at,data=re.search(r"KEPT_TOOLTIP_FOCUS:.*?=\((0x[0-9a-f]+),&\[(.*?)\]\)",source).groups()
  at=int(at,16);expected=bytes(int(x,0)for x in data.split(',')if x.strip());self.assertEqual(image.pe.get_data(at,len(expected)),expected)
  self.assertIn(at,{i.address for i in image.ins(image.owner(at))})

 def test_native_earned_item_presenter_matches_complete_guard(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_122",ROOT/"runtime-official-01220/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private pinned 0.120 image unavailable")
  from compare_official import Image
  image=Image(path);source=(ROOT/"crates/asx-game/src/bindings/official122.rs").read_text()
  rva=int(re.search(r'ITEM_CHOICE_RVA:u64=(0x[0-9a-f]+)',source)[1],16)
  data=re.search(r'ITEM_CHOICE_BYTES:&\[u8\]=&\[(.*?)\]',source)[1]
  expected=bytes(int(x,0)for x in data.split(',') if x.strip())
  self.assertEqual(image.pe.get_data(rva,len(expected)),expected)
  self.assertEqual(image.owner(rva),rva)
  self.assertIn(rva+len(expected),{i.address+i.size for i in image.ins(rva)})

 def test_native_encounter_helper_guard(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_122",ROOT/"runtime-official-01220/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private 0.122 image unavailable")
  from compare_official import Image
  image=Image(path);source=(ROOT/"crates/asx-game/src/bindings/official122.rs").read_text();at=int(re.search(r'\(0x5bd2d40,(0x[0-9a-f]+)\)',source)[1],16);data=re.search(r'BLESSING_SPAWN_BYTES:&\[u8\]=&\[(.*?)\]',source)[1];expected=bytes(int(x,0)for x in data.split(',')if x.strip());self.assertEqual(image.pe.get_data(at,len(expected)),expected);self.assertIn(at+len(expected),{i.address+i.size for i in image.ins(at)})

 def test_zone_table_identity_and_encounter_extent(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_122",ROOT/"runtime-official-01220/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private 0.122 image unavailable")
  import struct
  from compare_official import Image
  image=Image(path);source=(ROOT/"crates/asx-game/src/bindings/official122.rs").read_text();table=int(re.search(r'\(0x729b010,(0x[0-9a-f]+)\)',source)[1],16)
  names=[]
  for index in range(12):
   row=image.pe.get_data(table+index*0xf0,0xf0);ptr,n=struct.unpack_from('<QQ',row,8);names.append(image.pe.get_data(ptr-image.pe.OPTIONAL_HEADER.ImageBase,n).decode())
   self.assertEqual(struct.unpack_from('<Q',row,0x90)[0],5)
  self.assertEqual(names[:3],["elwynn","duskwood","wsg"]);self.assertEqual(len(set(names)),12)

 def test_native_glyph_spell_ids_and_keys_are_preserved(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_122",ROOT/"runtime-official-01220/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private pinned 0.122.0 image unavailable")
  import struct
  from compare_official import Image
  image=Image(path)
  source=(ROOT/"crates/asx-game/src/bindings/official122.rs").read_text()
  contacts={int(a,16):int(b,16)for a,b in re.findall(r'\((0x[0-9a-f]+),(0x[0-9a-f]+)\)',source)}
  previous=(ROOT/"crates/asx-game/src/bindings/inherited/official117.rs").read_text().split("pub const GLYPH_DEFINITIONS:",1)[1].split(";\n",1)[0]
  spells=[(int(a,16),int(b))for a,b in re.findall(r'\((0x[0-9a-f]+),(\d+)\)',previous)]
  self.assertEqual(len(spells),102)
  keys=[]
  for old,spell in spells:
   row=contacts[old]
   self.assertEqual(struct.unpack("<I",image.pe.get_data(row+0x40,4))[0],spell)
   pointer,length=struct.unpack("<QQ",image.pe.get_data(row,16))
   self.assertTrue(0<length<=64)
   key=image.pe.get_data(pointer-image.pe.OPTIONAL_HEADER.ImageBase,length).decode("ascii")
   self.assertRegex(key,r"^[a-z_]+$")
   keys.append(key)
  self.assertEqual(len(set(keys)),102)

import unittest,re,sys,os
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools/dev'))
class Official122(unittest.TestCase):
 def test_exact_import_inventory(self):
  from import122 import NAMES,SHA122
  self.assertEqual(len(NAMES),12);self.assertEqual(NAMES[-1],'Data/patch-B.MPQ')
  self.assertEqual(SHA122,'ec3a5b15fa39fed63ce4516c4fc159bdac92691ca44f88befc849010338d14f7')
 def test_shared_rendering_has_no_glyph_owner_dependency(self):
  root=Path(__file__).resolve().parents[1]
  for name in ['bonus_ui.rs','run_stats.rs']:
   text=(root/'src/runtime'/name).read_text()
   self.assertNotIn('glyphs::',text)
  self.assertFalse((root/'src/runtime/glyph_hover.rs').exists())


class ContactInventory122(unittest.TestCase):
 def test_previous_descriptor_includes_tables_and_shared_tooltip_seams(self):
  try:
   from compare_official import descriptor_inventory
  except ImportError:
   self.skipTest("Offline PE dependencies unavailable")
  rows=descriptor_inventory(ROOT/"crates/asx-game/src/bindings/inherited/official121.rs")
  for value in (0x619c9b8,0x7304258,0x73054b8,0x5c3fcb0,0x169780,0x2812005):self.assertIn(value,rows)
  for offset in (0xb0,0x140,0x3a8,0x9e8,0x1968):self.assertNotIn(offset,rows)
