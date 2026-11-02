"""Read-only current image/contact preflight; semantic runtime validation is separate."""
import hashlib,json,os,re,sys,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"tools/dev"))
def descriptor_source():
 # Formatting does not alter the selected native contracts. Keep comments and
 # identifiers intact, but normalize punctuation spacing before extracting tables.
 source=(ROOT/"crates/azsx-game/src/bindings/official138.rs").read_text()
 source=re.sub(r"\s*([,:=&\[\]()])\s*",r"\1",source)
 return re.sub(r",(?=[)\]])","",source)

class Native138(unittest.TestCase):
 def test_registry_pins_distinct_exact_adapter(self):
  source=descriptor_source()
  sha=re.search(r'SHA256\s*:\s*&str\s*=\s*"([0-9a-f]{64})"',source)[1]
  self.assertEqual(json.loads((ROOT/"azsx/supported-games.json").read_text())["games"]["0.138.0"]["sha256"],sha)
  self.assertIn("Revision::V138",(ROOT/"crates/azsx-game/src/official_native.rs").read_text())
 def test_native_contacts_guards_and_interior_branches(self):
  path=Path(os.environ.get("AZSX_REVIEWED_IMAGE_138",ROOT/"runtime-official-0138/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private pinned 0.138.0 image unavailable")
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
  self.assertEqual(count,70)  # Includes all nine retained pause UI guards.
  for retired in (0x10d95b,0x10d9e0,0x5908da8,0x594bfd9,0x594e3de,0x595f6da,0x595f91d,0x595ff10,0x5983784):
   self.assertNotIn(retired,contacts)

 def test_downed_xp_guards_rejoin_scaling_and_native_award_keeps_dead_choices_pending(self):
  path=Path(os.environ.get("AZSX_REVIEWED_IMAGE_138",ROOT/"runtime-official-0138/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private exact image unavailable")
  from compare_official import Image,MD
  from capstone.x86 import X86_OP_MEM,X86_OP_IMM,X86_REG_R13,X86_REG_R9
  from binding_registry import load_binding,parse_literal
  image=Image(path)
  guards=parse_literal(next(c["value"]for c in load_binding()[2]["constants"]if c["name"]=="DOWNED_XP_GUARDS"))
  for (entry,resume,expected,resume_bytes),register,award,end in zip(guards,[X86_REG_R13,X86_REG_R9],[0x5cc5e66,0x5ca2a7a],[0x5cc5e6c,0x5ca2a7f]):
   self.assertEqual(image.pe.get_data(entry,len(expected)),bytes(expected))
   self.assertEqual(image.pe.get_data(resume,len(resume_bytes)),bytes(resume_bytes))
   rows=list(MD.disasm(image.pe.get_data(entry,resume-entry),entry))
   cmp,jump=rows
   self.assertEqual((cmp.mnemonic,cmp.operands[0].type,cmp.operands[0].mem.base,cmp.operands[0].mem.disp,cmp.operands[1].imm),("cmp",X86_OP_MEM,register,0xf4b,4))
   self.assertEqual((jump.mnemonic,jump.operands[0].imm),("je",end))
   calc=list(MD.disasm(image.pe.get_data(resume,end-resume),resume))
   self.assertTrue(any(i.mnemonic=="mulss"and any(o.type==X86_OP_MEM and o.mem.disp==0x5e4 for o in i.operands)for i in calc))
   call=next(i for i in calc if i.address==award)
   self.assertEqual((call.mnemonic,call.operands[0].imm),("call",0x5cf5dc0))
  # Whole-level tokens fill the current threshold deficit. Consuming them
  # after revival would change the award if intervening shared XP advanced level.
  for (entry,resume,expected,resume_bytes),award,choice in zip(guards[2:],[0x5cc6fa9,0x5ca0af9,0x5cf17d9],[0x5cc705a,0x5ca0b97,0x5cf1872]):
   self.assertEqual(image.pe.get_data(entry,len(expected)),bytes(expected))
   self.assertEqual(image.pe.get_data(resume,len(resume_bytes)),bytes(resume_bytes))
   gate=list(MD.disasm(image.pe.get_data(entry,resume-entry),entry))
   self.assertEqual((gate[0].mnemonic,gate[0].operands[0].mem.disp,gate[0].operands[1].imm,gate[1].mnemonic),("cmp",0xf4b,1,"jne"))
   reset=next(MD.disasm(image.pe.get_data(resume,len(resume_bytes)),resume))
   self.assertEqual((reset.mnemonic,reset.operands[0].mem.disp,reset.operands[1].imm),("mov",0xea8,0))
   call=next(MD.disasm(image.pe.get_data(award,5),award));self.assertEqual(call.operands[0].imm,0x5cf5dc0)
   playing=next(MD.disasm(image.pe.get_data(choice,8),choice))
   self.assertEqual((playing.mnemonic,playing.operands[0].mem.disp,playing.operands[1].imm),("cmp",0xf4b,1))
  # Whole-level receipts were discarded for dead recipients before consumers.
  for (entry,resume,expected,resume_bytes),increment in zip(guards[5:],[0x5cc6594,0x5c9fc24]):
   self.assertEqual(image.pe.get_data(entry,len(expected)),bytes(expected))
   self.assertEqual(image.pe.get_data(resume,len(resume_bytes)),bytes(resume_bytes))
   gate=list(MD.disasm(image.pe.get_data(entry,resume-entry),entry))
   self.assertEqual((gate[0].mnemonic,gate[0].operands[0].mem.disp,gate[0].operands[1].imm,gate[1].mnemonic),("cmp",0xf4b,4,"je"))
   add=next(MD.disasm(image.pe.get_data(increment,8),increment))
   self.assertEqual((add.mnemonic,add.operands[0].mem.disp),("add",0xea8))
  rows=image.ins(0x5cf5dc0)
  self.assertTrue(any(i.mnemonic=="inc"and i.operands[0].type==X86_OP_MEM and i.operands[0].mem.disp==0xe9c for i in rows))
  # Native award changes level/XP immediately but only opens choices in Playing.
  gate=next(i for i in rows if i.address==0x5cf6418)
  self.assertEqual((gate.mnemonic,gate.operands[0].mem.disp,gate.operands[1].imm),("cmp",0xf4b,1))
  self.assertFalse(any(i.mnemonic.startswith("mov")and i.operands[0].type==X86_OP_MEM and i.operands[0].mem.base==X86_REG_R13 and i.operands[0].mem.disp==0xf4b for i in rows))

 def test_native_hybrid_emitter_produces_healing_not_the_neighboring_shield(self):
  path=Path(os.environ.get("AZSX_REVIEWED_IMAGE_138",ROOT/"runtime-official-0138/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private pinned 0.138.0 image unavailable")
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
  path=Path(os.environ.get("AZSX_REVIEWED_IMAGE_138",ROOT/"runtime-official-0138/AzerothSurvivors.exe"))
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
  manifest=json.loads((ROOT/"compat/clients/0.138.0.json").read_text())
  fields=parse_literal(next(x['value']for x in manifest['constants']if x['name']=='GAME_RESOLVED_FIELDS'))
  native=lambda offset:next(start+offset-low for low,high,start in fields if low<=offset<=high)
  self.assertEqual(native(0x500),0x578)
  self.assertEqual(native(0x4c0),0x5fc)

 def test_converted_reward_caller_is_a_real_gold_call_return_not_a_panic_nop(self):
  path=Path(os.environ.get("AZSX_REVIEWED_IMAGE_138",ROOT/"runtime-official-0138/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private exact image unavailable")
  from compare_official import Image
  from capstone import CS_GRP_CALL
  image=Image(path);source=descriptor_source()
  contacts={int(a,16):int(b,16)for a,b in re.findall(r'\((0x[0-9a-f]+),(0x[0-9a-f]+)\)',source)}
  address=contacts[0x597ee62];rows=image.ins(image.owner(address));index=next(j for j,i in enumerate(rows)if i.address==address)
  previous=rows[index-1]
  self.assertTrue(previous.group(CS_GRP_CALL))
  self.assertEqual(previous.operands[0].imm,contacts[0x5920470])

 def test_damage_size_reads_the_current_message_in_rbx_and_fresh_xmm8(self):
  path=Path(os.environ.get("AZSX_REVIEWED_IMAGE_138",ROOT/"runtime-official-01380/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private exact image unavailable")
  from compare_official import Image
  image=Image(path);rows={i.address:(i.mnemonic,i.op_str)for i in image.ins(image.owner(0xb57054))}
  self.assertEqual(rows[0xb56f7e],("movzx","ecx, byte ptr [rbx + 0x38]"))
  self.assertEqual(rows[0xb57054],("cvtss2sd","xmm10, xmm8"))

 def test_heal_observers_use_owning_systems_before_the_event_is_dropped(self):
  path=Path(os.environ.get("AZSX_REVIEWED_IMAGE_138",ROOT/"runtime-official-01380/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private exact image unavailable")
  from compare_official import Image
  from binding_registry import load_binding,parse_literal
  image=Image(path);c={x["name"]:parse_literal(x["value"])for x in load_binding()[2]["constants"]}
  self.assertEqual(c["HEAL_TEXT_TAG"],[0xc8,0x2d0]);self.assertEqual(c["HEAL_TEXT_PAYLOAD"],[0xd0,0x2d8])
  self.assertEqual(c["HEAL_TEXT_OBJECTS"],[0x228,0x120]);self.assertEqual(c["HEAL_TEXT_QUERY"],[0x368,0x260]);self.assertEqual(c["HEAL_TEXT_PARAMS"],[0x348,0x240])
  direct={i.address:i for i in image.ins(0xb53fd0)};periodic={i.address:i for i in image.ins(0xb54aa0)}
  self.assertEqual(direct[0xb5412c].operands[0].imm,0xf8d700)
  self.assertEqual((direct[c["DIRECT_HEAL_TEXT"][0]].mnemonic,direct[c["DIRECT_HEAL_TEXT"][0]].op_str),("lea","rcx, [rbp - 0x58]"))
  self.assertEqual(direct[0xb54136].mnemonic,"call")
  self.assertEqual(periodic[0xb54b8a].operands[0].imm,c["TEXT_QUERY"][0])
  self.assertEqual(periodic[0xb54c5d].operands[0].imm,0xf90620)
  self.assertLess(c["PERIODIC_HEAL_TEXT"][0],0xb54b8a)

 def test_enemy_iterator_uses_520_byte_rows_and_selected_elite_byte(self):
  path=Path(os.environ.get("AZSX_REVIEWED_IMAGE_138",ROOT/"runtime-official-01380/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private exact image unavailable")
  from compare_official import Image
  image=Image(path);rows={i.address:(i.mnemonic,i.op_str)for i in image.ins(0x5cfb530)}
  self.assertEqual(rows[0x5cfe2ab],("add","rcx, 0x208"))
  self.assertEqual(rows[0x5cfd6ac],("movzx","eax, byte ptr [rcx + 0x200]"))

 def test_named_hooks_have_safe_complete_detour_spans(self):
  path=Path(os.environ.get("AZSX_REVIEWED_IMAGE_138",ROOT/"runtime-official-0138/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private pinned 0.138.0 image unavailable")
  from compare_official import Image,MD
  from capstone import CS_GRP_JUMP,CS_GRP_CALL
  image=Image(path);source=descriptor_source()
  source=source.split("// Only reviewed AZSX-consumed",1)[1]
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
  path=Path(os.environ.get("AZSX_REVIEWED_IMAGE_138",ROOT/"runtime-official-0138/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private 0.138.0 image unavailable")
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
  path=Path(os.environ.get("AZSX_REVIEWED_IMAGE_138",ROOT/"runtime-official-0138/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private pinned 0.138.0 image unavailable")
  from compare_official import Image
  image=Image(path);source=descriptor_source()
  rva=int(re.search(r'ITEM_CHOICE_RVA:u64=(0x[0-9a-f]+)',source)[1],16)
  data=re.search(r'ITEM_CHOICE_BYTES:&\[u8\]=&\[(.*?)\]',source)[1]
  expected=bytes(int(x,0)for x in data.split(',') if x.strip())
  self.assertEqual(image.pe.get_data(rva,len(expected)),expected)
  self.assertEqual(image.owner(rva),rva)
  self.assertIn(rva+len(expected),{i.address+i.size for i in image.ins(rva)})

 def test_native_ui_receiver_vtable_has_complete_layout(self):
  path=Path(os.environ.get("AZSX_REVIEWED_IMAGE_138",ROOT/"runtime-official-0138/AzerothSurvivors.exe"))
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
  path=Path(os.environ.get("AZSX_REVIEWED_IMAGE_138",ROOT/"runtime-official-0138/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private exact 0.138.0 image unavailable")
  from compare_official import Image
  image=Image(path);source=descriptor_source()
  self.assertRegex(source,r"ENCOUNTER_GAME_FRAME:\s*u64\s*=\s*0x928")
  self.assertRegex(source,r"ENCOUNTER_DEFINITION_REGISTER:\s*usize\s*=\s*15")
  rows={i.address:(i.mnemonic,i.op_str)for i in image.ins(image.owner(0x5cfe52c))}
  self.assertEqual(rows[0x5cfd67b],("mov","r15, qword ptr [rcx + 0x30]"))
  self.assertEqual(rows[0x5cfd681],("xor","r15d, r15d"))
  self.assertEqual(rows[0x5cfddc9],("mov","r15, rdi"))
  self.assertEqual(rows[0x5cfe27e],("test","r15, r15"))
  self.assertEqual(rows[0x5cfddbb],("mov","r13, qword ptr [rbp + 0x928]"))
  self.assertEqual(rows[0x5cfe52c],("movss","xmm0, dword ptr [r13 + 0xe88]"))
  self.assertEqual(rows[0x5cfe535],("mov","dword ptr [r13 + 0x934], 1"))

 def test_trinket_filter_frame_and_excluded_keys_match_the_actual_reward_builder(self):
  path=Path(os.environ.get("AZSX_REVIEWED_IMAGE_138",ROOT/"runtime-official-0138/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private exact 0.138.0 image unavailable")
  import struct
  from compare_official import Image
  from binding_registry import load_binding,parse_literal
  _,_,manifest=load_binding();constants={x["name"]:parse_literal(x["value"])for x in manifest["constants"]}
  image=Image(path);contacts=dict(constants["EXTENSION_ADDRESSES"]);at=contacts[constants["BLESSING_POOL_RVA"]]
  rows={i.address:i for i in image.ins(image.owner(at))}
  self.assertEqual(rows[0x5d5f59c].op_str,"rbx, rdx")
  self.assertEqual(rows[0x5d5f81e].op_str,"qword ptr [rbp + 0x198], rbx")
  self.assertEqual(constants["BLESSING_POOL_FRAME"],0x198)
  self.assertEqual(rows[at].op_str,"r14, 0x678")
  lea=rows[0x5d5f89d];table=lea.address+lea.size+lea.operands[1].mem.disp
  keys=[]
  for index in range(15,20):
   ptr,n=struct.unpack("<QQ",image.pe.get_data(table+index*72,16))
   keys.append(image.pe.get_data(ptr-image.pe.OPTIONAL_HEADER.ImageBase,n).decode())
  self.assertEqual(keys,["remote","insignia","eldre_thalas","chosen","onyxia_blood"])

 def test_main_menu_title_lookup_has_exact_result_boundary(self):
  path=Path(os.environ.get("AZSX_REVIEWED_IMAGE_138",ROOT/"runtime-official-0138/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private exact 0.138.0 image unavailable")
  from compare_official import Image
  from binding_registry import load_binding,parse_literal
  _,_,manifest=load_binding();constants={x["name"]:parse_literal(x["value"])for x in manifest["constants"]}
  image=Image(path);hook,guard=constants["MENU_TITLE_RESULT"]
  self.assertEqual(hook,0x3d8e89)
  self.assertEqual(image.pe.get_data(hook,len(guard)),bytes(guard))
  rows={i.address:i for i in image.ins(image.owner(0x3d8e75))}
  lookup=rows[0x3d8e78]
  key=lookup.address+lookup.size+lookup.operands[1].mem.disp
  self.assertEqual(image.pe.get_data(key,10),b"game.title")
  self.assertEqual(rows[0x3d8e7f].op_str,"edx, 0xa")
  self.assertEqual(rows[0x3d8e84].mnemonic,"call")
  self.assertEqual(rows[hook].op_str,"rdi, rdx")
  self.assertEqual(rows[0x3d8edd].mnemonic,"call")

 def test_guest_pause_filter_is_observed_before_apply_and_keeps_native_consumption(self):
  path=Path(os.environ.get("AZSX_REVIEWED_IMAGE_138",ROOT/"runtime-official-0138/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private exact 0.138.0 image unavailable")
  import struct
  from compare_official import Image
  from binding_registry import load_binding,parse_literal
  _,_,manifest=load_binding();constants={x["name"]:parse_literal(x["value"])for x in manifest["constants"]}
  image=Image(path);hook,guard=constants["MENU_INPUT"];apply,apply_guard=constants["MENU_INPUT_APPLY"]
  self.assertEqual((hook,apply),(0x5c40a40,0x5c35b70))
  self.assertEqual(image.pe.get_data(hook,len(guard)),bytes(guard))
  self.assertEqual(image.pe.get_data(apply,len(apply_guard)),bytes(apply_guard))
  native={i.address:i for i in image.ins(image.owner(0x5c1f356))}
  self.assertEqual(native[0x5c1f356].operands[0].imm,hook)
  self.assertEqual((native[0x5c1f35c].mnemonic,native[0x5c1f35c].op_str),("test","al, al"))
  self.assertEqual(native[0x5c1f35e].operands[0].imm,0x5c1f300)
  self.assertEqual(native[0x5c1fe45].operands[0].imm,apply)
  queue,queue_guard=constants["MENU_UI_QUEUE"]
  self.assertEqual(queue,0x20bab0)
  self.assertEqual(image.pe.get_data(queue,len(queue_guard)),bytes(queue_guard))
  keys={i.address:i for i in image.ins(image.owner(0x21794e))}
  self.assertEqual(keys[0x21794e].operands[0].imm,queue)
  self.assertEqual(keys[0x21794e].address+keys[0x21794e].size,constants["MENU_ESCAPE_QUEUE_RETURN"])
  self.assertEqual(keys[0x217936].op_str,"dword ptr [rsp + 0x90], 7")
  self.assertEqual(keys[0x2179cb].op_str,"dword ptr [rsp + 0x90], 7")
  rows={i.address:i for i in image.ins(hook)}
  self.assertEqual(rows[0x5c40a67].op_str,"dword ptr [rcx + 0xf60], 1")
  self.assertEqual(rows[0x5c40a77].op_str,"byte ptr [rcx + 0x139d], 1")
  lea=rows[0x5c40a9d];table=lea.address+lea.size+lea.operands[1].mem.disp
  target=table+struct.unpack("<i",image.pe.get_data(table+(7-2)*4,4))[0]
  self.assertEqual(target,0x5c40d85)
  self.assertEqual((rows[target].mnemonic,rows[target].op_str),("mov","al, 1"))

 def test_native_encounter_helper_guard(self):
  path=Path(os.environ.get("AZSX_REVIEWED_IMAGE_138",ROOT/"runtime-official-0138/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private 0.138.0 image unavailable")
  from compare_official import Image
  image=Image(path);source=descriptor_source();at=int(re.search(r'\(0x5bd2d40,(0x[0-9a-f]+)\)',source)[1],16);data=re.search(r'BLESSING_SPAWN_BYTES:&\[u8\]=&\[(.*?)\]',source)[1];expected=bytes(int(x,0)for x in data.split(',')if x.strip());self.assertEqual(image.pe.get_data(at,len(expected)),expected);self.assertIn(at+len(expected),{i.address+i.size for i in image.ins(at)})

 def test_zone_table_identity_and_encounter_extent(self):
  path=Path(os.environ.get("AZSX_REVIEWED_IMAGE_138",ROOT/"runtime-official-0138/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private 0.138.0 image unavailable")
  import struct
  from compare_official import Image
  image=Image(path);source=descriptor_source();table=int(re.search(r'\(0x729b010,(0x[0-9a-f]+)\)',source)[1],16)
  names=[]
  for index in range(12):
   row=image.pe.get_data(table+index*0xf0,0xf0);ptr,n=struct.unpack_from('<QQ',row,8);names.append(image.pe.get_data(ptr-image.pe.OPTIONAL_HEADER.ImageBase,n).decode())
   self.assertEqual(struct.unpack_from('<Q',row,0x90)[0],5)
  self.assertEqual(names[:3],["elwynn","duskwood","wsg"]);self.assertEqual(len(set(names)),12)

import unittest,re,sys,os
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools/dev'))
class Official138(unittest.TestCase):
 def test_exact_import_inventory(self):
  from import_official import NAMES,SHA256
  self.assertEqual(len(NAMES),12);self.assertEqual(NAMES[-1],'Data/patch-B.MPQ')
  self.assertEqual(SHA256,'eb5fd1c1b96e6180fcf5f3d685d90a61471af28d22e5f060c02b7b7663fbf79e')
class ContactInventory130(unittest.TestCase):
 def test_previous_descriptor_includes_tables_and_shared_tooltip_seams(self):
  try:
   from compare_official import descriptor_inventory
  except ImportError:
   self.skipTest("Offline PE dependencies unavailable")
  rows=descriptor_inventory(ROOT/"crates/azsx-game/src/bindings/inherited/official121.rs")
  for value in (0x619c9b8,0x7304258,0x73054b8,0x5c3fcb0,0x169780,0x2812005):self.assertIn(value,rows)
  for offset in (0xb0,0x140,0x3a8,0x9e8,0x1968):self.assertNotIn(offset,rows)

class CurrentContactInventory(unittest.TestCase):
 def test_nested_xp_guards_and_returns_are_included_without_role_keys(self):
  from compare_official import descriptor_inventory
  from binding_registry import load_binding,parse_literal
  _,client,manifest=load_binding();c={x["name"]:parse_literal(x["value"])for x in manifest["constants"]}
  rows=descriptor_inventory(ROOT/client["descriptor"])
  for entry,resume,_,_ in c["DOWNED_XP_GUARDS"]:
   self.assertIn(entry,rows);self.assertIn(resume,rows)
  self.assertIn(c["MENU_ESCAPE_QUEUE_RETURN"],rows)
  self.assertNotIn(c["BLESSING_EVENT_RVA"],rows)
  self.assertIn(dict(c["EXTENSION_ADDRESSES"])[c["BLESSING_EVENT_RVA"]],rows)

class AddonAndLobbyNativeContracts(unittest.TestCase):
 def test_selected_reader_and_host_constructor_keep_reviewed_abis(self):
  path=Path(os.environ.get("AZSX_REVIEWED_IMAGE_138",ROOT/"runtime-official-01380/AzerothSurvivors.exe"))
  if not path.exists():self.skipTest("Private 0.138.0 image unavailable")
  from compare_official import Image
  from binding_registry import load_binding,parse_literal
  _,_,manifest=load_binding();constants={c['name']:parse_literal(c['value'])for c in manifest['constants']}
  image=Image(path)
  for name in ('ADDON_API_READER','LOBBY_SUBMIT'):
   at=constants[name];guard=bytes(constants[name+'_BYTES'])
   self.assertEqual(image.pe.get_data(at,len(guard)),guard)
   self.assertIn(at+len(guard),{i.address+i.size for i in image.ins(at)})
  reader={i.address:(i.mnemonic,i.op_str)for i in image.ins(constants['ADDON_API_READER'])}
  for at,operands in ((0x6020f67,'rdi, r9'),(0x6020f6a,'rbx, r8'),(0x6020f6d,'r14, rdx'),(0x6020f70,'rsi, rcx'),(0x6021001,'rax, rsi')):
   self.assertEqual(reader[at],('mov',operands))
  self.assertEqual(reader[0x6020f85],('call','0x6023050'))
  self.assertEqual(reader[0x6020fb8],('mov','qword ptr [rsi], 0xffffffffffffffff'))
  host=[i.op_str for i in image.ins(constants['LOBBY_SUBMIT'])]
  self.assertEqual(constants['LOBBY_NAME'],0xd0)
  self.assertTrue(any('+ 0xd0]'in op for op in host))
  self.assertIn('0x20bab0',host)
