"""Actual native self-HoT queue writes and GUID-matched special-rank names."""
import re,json,struct

def hot_text_observed(text):
 # Pure HoTs can use the native SPELL_HEAL message path. Bind the captured
 # source spell, IDs and amount to this exact successful native queue write.
 pending=None
 for line in text.splitlines():
  header=re.search(r'([\d.]+)s remaining UI heal packet: periodic=false bytes=Ok\((\[[^\n]+\])\)',line)
  if header:
   try:
    raw=bytes(json.loads(header[2]));target,caster,spell,amount=struct.unpack_from('<QQII',raw)
    pending=(float(header[1]),target,caster,spell,amount)
   except (ValueError,TypeError,struct.error):pending=None
  draw=re.search(r'([\d.]+)s native heal text: periodic=false event=HealText \{ caster: (\d+), target: (\d+), amount: (\d+), critical: (?:true|false) \} result=Ok\(true\)',line)
  if draw and pending:
   at,target,caster,spell,amount=pending;pending=None
   if 0<=float(draw[1])-at<=.05 and spell in(139,774)and amount>0 and (int(draw[2]),int(draw[3]),int(draw[4]))==(caster,target,amount):return True
 return bool(re.search(r'native heal text: periodic=true event=HealText \{ caster: \d+, target: \d+, amount: [1-9]\d*, critical: false \} result=Ok\(true\)',text))

def assess(host,native,logs):
 expected=dict((label,int(guid))for label,guid in re.findall(r'\("(elite|guardian|boss)", (\d+)\)',host))
 checks=dict(native_roles=set(expected)=={'elite','guardian','boss'},self_hots=all(f'remaining UI self HoT: caster={slot} spell={139 if slot==0 else 774} result=Ok(true)'in host for slot in(0,1)))
 names=[]
 for slot,(output,text)in enumerate(zip(native,logs)):
  rows={int(guid):dict(rank=None if rank=='None'else int(rank[5:-1]),name=name)for guid,rank,name in re.findall(r'ASX plate witness: guid=(\d+) rank=(Some\([1-4]\)|None) name=Some\("([^"\n]+)"\) retained=true',text)}
  names.append(rows)
  for role in('elite','guardian','boss'):checks[f'{role}_name_slot_{slot}']=role in expected and expected[role]in rows
  checks[f'hot_world_text_slot_{slot}']=hot_text_observed(text)
 checks['two_views']=len(native)==len(logs)==2
 return dict(checks=checks,passed=all(checks.values()),expected=expected,names=names,scope='GUID-matched native constructor roles and retained names (UI rank may be absent), plus successful periodic healing queue writes; rendered placement reviewed separately.')
