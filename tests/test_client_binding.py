"""Read-only current image/contact preflight; semantic runtime validation is separate."""
import hashlib,json,os,re,sys,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"tools/dev"))
def descriptor_source():
 # Formatting does not alter the selected native contracts. Keep comments and
 # identifiers intact, but normalize punctuation spacing before extracting tables.
 source=(ROOT/"crates/asx-game/src/bindings/official1351.rs").read_text()
 source=re.sub(r"\s*([,:=&\[\]()])\s*",r"\1",source)
 return re.sub(r",(?=[)\]])","",source)

class Native1351(unittest.TestCase):
 def test_registry_pins_distinct_exact_adapter(self):
  source=descriptor_source()
  sha=re.search(r'SHA256\s*:\s*&str\s*=\s*"([0-9a-f]{64})"',source)[1]
  self.assertEqual(json.loads((ROOT/"asx/supported-games.json").read_text())["games"]["0.135.1"]["sha256"],sha)
  self.assertIn("Revision::V1351",(ROOT/"crates/asx-game/src/official_native.rs").read_text())
 def test_native_contacts_guards_and_interior_branches(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1351",ROOT/"runtime-official-01351/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private pinned 0.135.1 image unavailable")
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
  # Nine unconsumed/retired seams were removed from the old 71-contact set.
  self.assertEqual(count,62)
  for retired in (0x10d95b,0x10d9e0,0x5908da8,0x594bfd9,0x594e3de,0x595f6da,0x595f91d,0x595ff10,0x5983784):
   self.assertNotIn(retired,contacts)

 def test_native_hybrid_emitter_produces_healing_not_the_neighboring_shield(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1351",ROOT/"runtime-official-01351/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private pinned 0.135.1 image unavailable")
  from compare_official import Image
  from capstone import CS_GRP_CALL
  from capstone.x86 import X86_OP_MEM,X86_OP_IMM
  image=Image(path);source=descriptor_source()
  contacts={int(a,16):int(b,16)for a,b in re.findall(r'\((0x[0-9a-f]+),(0x[0-9a-f]+)\)',source)}
  emitter=contacts[0x58fed50];rows=image.ins(image.owner(emitter))
  # Both neighboring routines have the same ABI. Prolog similarity cannot
  # distinguish healing kind 14 from shield kind 15 in the 56-byte queue row.
  kinds={i.operands[1].imm for i in rows if i.mnemonic=="mov"and len(i.operands)==2 and i.operands[0].type==X86_OP_MEM and i.operands[0].mem.disp==40 and i.operands[1].type==X86_OP_IMM}
  self.assertEqual(kinds,{14})
  fields={i.operands[0].mem.disp for i in rows if i.mnemonic=="movss"and i.operands[0].type==X86_OP_MEM}
  self.assertTrue({32,36}<=fields) # Fraction and duration, not an unrelated append.
  caster=contacts[0x59317f0]
  calls={i.operands[0].imm for i in image.ins(image.owner(caster)) if i.group(CS_GRP_CALL)and i.operands and i.operands[0].type==X86_OP_IMM}
  self.assertIn(emitter,calls) # The reviewed native Renew branch uses this emitter.

 def test_native_gold_consumer_identifies_the_multiplier_and_wallet_fields(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1351",ROOT/"runtime-official-01351/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private exact image unavailable")
  from compare_official import Image
  from capstone.x86 import X86_OP_MEM,X86_REG_RSI
  from binding_registry import parse_literal
  source=descriptor_source();image=Image(path)
  contacts={int(a,16):int(b,16)for a,b in re.findall(r'\((0x[0-9a-f]+),(0x[0-9a-f]+)\)',source)}
  rows=image.ins(contacts[0x5920470])
  reads={o.mem.disp for i in rows if i.mnemonic=="mulss"for o in i.operands if o.type==X86_OP_MEM and o.mem.base==X86_REG_RSI}
  self.assertIn(0x5fc,reads)
  self.assertNotIn(0x5f8,reads)
  writes={i.operands[0].mem.disp for i in rows if i.mnemonic in ("mov","add")and i.operands and i.operands[0].type==X86_OP_MEM and i.operands[0].mem.base==X86_REG_RSI}
  self.assertIn(0x578,writes)
  manifest=json.loads((ROOT/"compat/clients/0.135.1.json").read_text())
  fields=parse_literal(next(x['value']for x in manifest['constants']if x['name']=='GAME_RESOLVED_FIELDS'))
  native=lambda offset:next(start+offset-low for low,high,start in fields if low<=offset<=high)
  self.assertEqual(native(0x500),0x578)
  self.assertEqual(native(0x4c0),0x5fc)

 def test_converted_reward_caller_is_a_real_gold_call_return_not_a_panic_nop(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1351",ROOT/"runtime-official-01351/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private exact image unavailable")
  from compare_official import Image
  from capstone import CS_GRP_CALL
  image=Image(path);source=descriptor_source()
  contacts={int(a,16):int(b,16)for a,b in re.findall(r'\((0x[0-9a-f]+),(0x[0-9a-f]+)\)',source)}
  address=contacts[0x597ee62];rows=image.ins(image.owner(address));index=next(j for j,i in enumerate(rows)if i.address==address)
  previous=rows[index-1]
  self.assertTrue(previous.group(CS_GRP_CALL))
  self.assertEqual(previous.operands[0].imm,contacts[0x5920470])

 def test_named_hooks_have_safe_complete_detour_spans(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1351",ROOT/"runtime-official-01351/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private pinned 0.135.1 image unavailable")
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
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1351",ROOT/"runtime-official-01351/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private 0.135.1 image unavailable")
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
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1351",ROOT/"runtime-official-01351/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private pinned 0.135.1 image unavailable")
  from compare_official import Image
  image=Image(path);source=descriptor_source()
  rva=int(re.search(r'ITEM_CHOICE_RVA:u64=(0x[0-9a-f]+)',source)[1],16)
  data=re.search(r'ITEM_CHOICE_BYTES:&\[u8\]=&\[(.*?)\]',source)[1]
  expected=bytes(int(x,0)for x in data.split(',') if x.strip())
  self.assertEqual(image.pe.get_data(rva,len(expected)),expected)
  self.assertEqual(image.owner(rva),rva)
  self.assertIn(rva+len(expected),{i.address+i.size for i in image.ins(rva)})

 def test_native_ui_receiver_vtable_has_complete_layout(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1351",ROOT/"runtime-official-01351/AzerothSurvivors.exe"))
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

 def test_encounter_definition_frame_is_distinct_from_the_enemy_guid(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1351",ROOT/"runtime-official-01351/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private exact 0.135.1 image unavailable")
  from compare_official import Image
  image=Image(path);source=descriptor_source()
  self.assertRegex(source,r"ENCOUNTER_GAME_FRAME:\s*u64\s*=\s*0x918")
  self.assertRegex(source,r"ENCOUNTER_DEFINITION_FRAME:\s*u64\s*=\s*0x908")
  rows={i.address:(i.mnemonic,i.op_str)for i in image.ins(image.owner(0x5ce2e53))}
  # The optional boss definition comes from the killed enemy's native +0x30.
  self.assertEqual(rows[0x5ce1faa],("mov","rax, qword ptr [rcx + 0x30]"))
  self.assertEqual(rows[0x5ce1fae],("mov","qword ptr [rbp + 0x908], rax"))
  self.assertEqual(rows[0x5ce2ba8],("cmp","qword ptr [rbp + 0x908], 0"))
  # At this later seam R13 has been reloaded with the original enemy GUID.
  self.assertEqual(rows[0x5ce26ec],("mov","r13, qword ptr [rbp + 0x8f8]"))
  self.assertEqual(rows[0x5ce03d1],("mov","qword ptr [rbp + 0x8f8], rbx"))
  self.assertEqual(rows[0x5cdff38],("mov","rbx, rdx"))
  self.assertEqual(rows[0x5ce2e53],("movss","xmm0, dword ptr [r15 + 0xe88]"))
  self.assertEqual(rows[0x5ce2e5c],("mov","dword ptr [r15 + 0x934], 1"))

 def test_native_encounter_helper_guard(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1351",ROOT/"runtime-official-01351/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private 0.135.1 image unavailable")
  from compare_official import Image
  image=Image(path);source=descriptor_source();at=int(re.search(r'\(0x5bd2d40,(0x[0-9a-f]+)\)',source)[1],16);data=re.search(r'BLESSING_SPAWN_BYTES:&\[u8\]=&\[(.*?)\]',source)[1];expected=bytes(int(x,0)for x in data.split(',')if x.strip());self.assertEqual(image.pe.get_data(at,len(expected)),expected);self.assertIn(at+len(expected),{i.address+i.size for i in image.ins(at)})

 def test_zone_table_identity_and_encounter_extent(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1351",ROOT/"runtime-official-01351/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private 0.135.1 image unavailable")
  import struct
  from compare_official import Image
  image=Image(path);source=descriptor_source();table=int(re.search(r'\(0x729b010,(0x[0-9a-f]+)\)',source)[1],16)
  names=[]
  for index in range(12):
   row=image.pe.get_data(table+index*0xf0,0xf0);ptr,n=struct.unpack_from('<QQ',row,8);names.append(image.pe.get_data(ptr-image.pe.OPTIONAL_HEADER.ImageBase,n).decode())
   self.assertEqual(struct.unpack_from('<Q',row,0x90)[0],5)
  self.assertEqual(names[:3],["elwynn","duskwood","wsg"]);self.assertEqual(len(set(names)),12)

 def test_native_glyph_spell_ids_and_keys_are_preserved(self):
  path=Path(os.environ.get("ASX_REVIEWED_IMAGE_1351",ROOT/"runtime-official-01351/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private pinned 0.135.1 image unavailable")
  import struct
  from compare_official import Image
  image=Image(path)
  source=descriptor_source()
  contacts={int(a,16):int(b,16)for a,b in re.findall(r'\((0x[0-9a-f]+),(0x[0-9a-f]+)\)',source)}
  previous=(ROOT/"crates/asx-game/src/bindings/inherited/official117.rs").read_text().split("pub const GLYPH_DEFINITIONS:",1)[1].split(";\n",1)[0]
  spells=[(int(a,16),int(b))for a,b in re.findall(r'\((0x[0-9a-f]+),(\d+)\)',previous)]
  spells=[row for row in spells if row[1]!=16038]+[(0x7108388,17364),(0x7108400,8017)]
  self.assertEqual(len(spells),103)
  self.assertNotIn(0x72a0700,contacts)
  keys=[]
  for old,spell in spells:
   row=contacts[old]
   self.assertEqual(struct.unpack("<I",image.pe.get_data(row+0x40,4))[0],spell)
   pointer,length=struct.unpack("<QQ",image.pe.get_data(row,16))
   self.assertTrue(0<length<=64)
   key=image.pe.get_data(pointer-image.pe.OPTIONAL_HEADER.ImageBase,length).decode("ascii")
   self.assertRegex(key,r"^[a-z_]+$")
   keys.append(key)
  self.assertEqual(len(set(keys)),103)
  self.assertIn("stormstrike",keys)
  self.assertIn("rockbiter",keys)
  self.assertNotIn("call_of_flame",keys)

import unittest,re,sys,os
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools/dev'))
class Official1351(unittest.TestCase):
 def test_exact_import_inventory(self):
  from import_official import NAMES,SHA256
  self.assertEqual(len(NAMES),12);self.assertEqual(NAMES[-1],'Data/patch-B.MPQ')
  self.assertEqual(SHA256,'71e5f1767279db4bd428d1cd9c5d50d37759b72295386a69f9e3f6e70bc59249')
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
