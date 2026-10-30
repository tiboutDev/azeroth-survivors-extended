"""Read-only 0.130 image/contact preflight; semantic runtime validation is separate."""
import hashlib,json,os,re,sys,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"tools/dev"))
def descriptor_source():
 # Formatting does not alter the selected native contracts. Keep comments and
 # identifiers intact, but normalize punctuation spacing before extracting tables.
 source=(ROOT/"crates/asx-game/src/bindings/official1301.rs").read_text()
 source=re.sub(r"\s*([,:=&\[\]()])\s*",r"\1",source)
 return re.sub(r",(?=[)\]])","",source)

class Native130(unittest.TestCase):
 def test_registry_pins_distinct_exact_adapter(self):
  source=descriptor_source()
  sha=re.search(r'SHA256\s*:\s*&str\s*=\s*"([0-9a-f]{64})"',source)[1]
  self.assertEqual(json.loads((ROOT/"asx/supported-games.json").read_text())["games"]["0.130.1"]["sha256"],sha)
  self.assertIn("Revision::V130",(ROOT/"crates/asx-game/src/official_native.rs").read_text())
 def test_native_contacts_guards_and_interior_branches(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1301",ROOT/"runtime-official-01301/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private pinned 0.130 image unavailable")
  from compare_official import Image,MD
  from capstone import CS_GRP_JUMP,CS_GRP_CALL
  image=Image(path);source=descriptor_source()
  self.assertEqual(image.sha,re.search(r'SHA256\s*:\s*&str\s*=\s*"([0-9a-f]{64})"',source)[1])
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

 def test_named_hooks_have_safe_complete_detour_spans(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1301",ROOT/"runtime-official-01301/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private pinned 0.130 image unavailable")
  from compare_official import Image,MD
  from capstone import CS_GRP_JUMP,CS_GRP_CALL
  image=Image(path);source=descriptor_source()
  source=source.split("// Only reviewed ASX-consumed",1)[1]
  rows=re.findall(r"\((0x[0-9a-f]+),&\[(.*?)\]\)",source,re.S)
  self.assertGreater(len(rows),35)
  for addr,data in rows:
   addr=int(addr,16);expected=bytes(int(x,0)for x in data.split(',')if x.strip())
   self.assertEqual(image.pe.get_data(addr,len(expected)),expected)
   owner=image.owner(addr);ins=image.ins(owner)if owner else list(MD.disasm(image.pe.get_data(addr,128),addr))
   self.assertIn(addr,{i.address for i in ins});self.assertIn(addr+len(expected),{i.address+i.size for i in ins})
   boundary=next(i.address+i.size for i in ins if i.address>=addr and i.address+i.size>=addr+5)
   for i in ins:
    if (i.group(CS_GRP_JUMP)or i.group(CS_GRP_CALL))and i.operands and i.operands[0].type==2:
     target=i.operands[0].imm
     self.assertFalse(addr<target<boundary and not(addr<=i.address<boundary),f"branch enters displaced native instructions at {addr:x}")

 def test_retained_tooltip_contacts_and_native_row_contract(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1301",ROOT/"runtime-official-01301/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private 0.130 image unavailable")
  from compare_official import Image
  image=Image(path);source=descriptor_source()
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
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1301",ROOT/"runtime-official-01301/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private pinned 0.120 image unavailable")
  from compare_official import Image
  image=Image(path);source=descriptor_source()
  rva=int(re.search(r'ITEM_CHOICE_RVA:u64=(0x[0-9a-f]+)',source)[1],16)
  data=re.search(r'ITEM_CHOICE_BYTES:&\[u8\]=&\[(.*?)\]',source)[1]
  expected=bytes(int(x,0)for x in data.split(',') if x.strip())
  self.assertEqual(image.pe.get_data(rva,len(expected)),expected)
  self.assertEqual(image.owner(rva),rva)
  self.assertIn(rva+len(expected),{i.address+i.size for i in image.ins(rva)})

 def test_native_ui_receiver_vtable_has_complete_layout(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1301",ROOT/"runtime-official-01301/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private exact image unavailable")
  import struct
  from compare_official import Image
  image=Image(path);source=descriptor_source()
  at=int(re.search(r"\(0x5ede960,(0x[0-9a-f]+)\)",source)[1],16)
  words=struct.unpack("<9Q",image.pe.get_data(at,72))
  self.assertEqual(words[1:3],(32,8))
  base=image.pe.OPTIONAL_HEADER.ImageBase
  for pointer in (words[0],*words[3:]):
   rva=pointer-base
   self.assertTrue(image.text<=rva<image.text+len(image.code),hex(pointer))

 def test_native_encounter_helper_guard(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1301",ROOT/"runtime-official-01301/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private 0.130 image unavailable")
  from compare_official import Image
  image=Image(path);source=descriptor_source();at=int(re.search(r'\(0x5bd2d40,(0x[0-9a-f]+)\)',source)[1],16);data=re.search(r'BLESSING_SPAWN_BYTES:&\[u8\]=&\[(.*?)\]',source)[1];expected=bytes(int(x,0)for x in data.split(',')if x.strip());self.assertEqual(image.pe.get_data(at,len(expected)),expected);self.assertIn(at+len(expected),{i.address+i.size for i in image.ins(at)})

 def test_zone_table_identity_and_encounter_extent(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1301",ROOT/"runtime-official-01301/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private 0.130 image unavailable")
  import struct
  from compare_official import Image
  image=Image(path);source=descriptor_source();table=int(re.search(r'\(0x729b010,(0x[0-9a-f]+)\)',source)[1],16)
  names=[]
  for index in range(12):
   row=image.pe.get_data(table+index*0xf0,0xf0);ptr,n=struct.unpack_from('<QQ',row,8);names.append(image.pe.get_data(ptr-image.pe.OPTIONAL_HEADER.ImageBase,n).decode())
   self.assertEqual(struct.unpack_from('<Q',row,0x90)[0],5)
  self.assertEqual(names[:3],["elwynn","duskwood","wsg"]);self.assertEqual(len(set(names)),12)

 def test_native_glyph_spell_ids_and_keys_are_preserved(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1301",ROOT/"runtime-official-01301/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private pinned 0.130.1 image unavailable")
  import struct
  from compare_official import Image
  image=Image(path)
  source=descriptor_source()
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
class Official130(unittest.TestCase):
 def test_exact_import_inventory(self):
  from import_official import NAMES,SHA256
  self.assertEqual(len(NAMES),12);self.assertEqual(NAMES[-1],'Data/patch-B.MPQ')
  self.assertEqual(SHA256,'e8ff033544095e81d74087ff400d50286b26af02291e649f4c5e4d76cbcf8863')
 def test_shared_rendering_has_no_glyph_owner_dependency(self):
  root=Path(__file__).resolve().parents[1]
  for name in ['bonus_ui.rs','run_stats.rs']:
   text=(root/'src/runtime'/name).read_text()
   self.assertNotIn('glyphs::',text)
  self.assertFalse((root/'src/runtime/glyph_hover.rs').exists())


class ContactInventory130(unittest.TestCase):
 def test_previous_descriptor_includes_tables_and_shared_tooltip_seams(self):
  try:
   from compare_official import descriptor_inventory
  except ImportError:
   self.skipTest("Offline PE dependencies unavailable")
  rows=descriptor_inventory(ROOT/"crates/asx-game/src/bindings/inherited/official121.rs")
  for value in (0x619c9b8,0x7304258,0x73054b8,0x5c3fcb0,0x169780,0x2812005):self.assertIn(value,rows)
  for offset in (0xb0,0x140,0x3a8,0x9e8,0x1968):self.assertNotIn(offset,rows)
