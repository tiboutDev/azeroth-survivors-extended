"""Create three disposable native profiles; never import user progression."""
import hashlib,json,sys
from pathlib import Path
SHA="0258b8a43d9ba3170b15e5fc07a646a606eb2972a5dd6aa0e6349a07ec390e2c"
def writer(executable):
 raw=Path(executable).read_bytes()
 actual=hashlib.sha256(raw).hexdigest()
 sys.path.insert(0,str(Path(__file__).resolve().parents[2]/"tools/dev"))
 import native_profile_codec as azs_rse1
 keys=azs_rse1.key_candidates_from_bytes(raw)
 if len(keys)!=1:raise ValueError("Ambiguous native save codec")
 fixture=json.loads((Path(__file__).parent/"fixtures/party-talents115.json").read_text())
 if fixture["sha256"]!=SHA or actual not in [SHA,*fixture.get("compatible_sha256",[])]:raise ValueError("Unreviewed talent fixture/image")
 def prepare(folder,slot,name_override=None,class_override=None,known_items=None):
  if slot not in (1,2,3):raise ValueError("This UI scenario has three profiles")
  path=Path(folder)/"characters.txt"
  if path.exists():raise ValueError("Never replace existing progression")
  cls,race,name=((1,1,"UiWarrior"),(2,1,"UiPaladin"),(11,4,"UiDruid"))[slot-1]
  if class_override is not None:
   if class_override not in (3,4,5,7,8,9,11):raise ValueError("Only reviewed glyph Hunter/Rogue/Priest/Shaman/Mage/Warlock/Druid overrides is supported")
   cls,race=class_override,4 if class_override==11 else 3 if class_override==3 else 2 if class_override==7 else 1
  if name_override is not None:
   if not name_override.isascii() or not name_override.isalpha():raise ValueError("Invalid fixture name")
   name=name_override
  text=f"pick|{cls}|{race}|0|elwynn|{name}|0,0,0,0,0|100,100,100,100\nhero|{cls}|{race}|0|0,0,0,0,0\n"
  talents={}if class_override==11 else {"demonic_familiar":1}if cls==9 else fixture["classes"].get(str(cls),{})
  ranks=",".join(f"{key}:{rank}"for key,rank in talents.items())
  text+=f"talent|{cls}|50000|{ranks}\n"
  if known_items is not None:
   if len(known_items)>64 or len(set(known_items))!=len(known_items)or any(type(v)is not int or not 0<v<2**32 for v in known_items):raise ValueError("Invalid disposable native collection")
   text+="items|"+",".join(map(str,known_items))+"\n"
  sealed=azs_rse1.seal(text,keys[0])
  if azs_rse1.open_save(sealed,keys[0])[1]!=text:raise ValueError("Native profile roundtrip failed")
  with path.open("xb")as out:out.write(sealed)
 # Same-build readback is restricted by callers to their own disposable profiles.
 prepare.read=lambda folder:azs_rse1.open_save((Path(folder)/"characters.txt").read_bytes(),keys[0])[1]
 return prepare
