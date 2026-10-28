"""Assess bounded gem trials; missing observations are failures, not acceptance."""
import ast,math,re,json
VALUES=[.12,.06,.20,.05,.35,.03,1.,.20]
NAMES=["Bold Ruby","Quick Sapphire","Solid Bloodstone","Deadly Topaz","Potent Amethyst","Enduring Diamond","Renewing Emerald","Expansive Opal"]
def assess(host,native,players):
 rows={}
 for kind,b,a,repeat,restored in re.findall(r"gem matrix: kind=(\d+) base=Ok\((\[[^\n]+?\])\) actual=Ok\((\[[^\n]+?\])\) repeat=Ok\((\[[^\n]+?\])\) restored=(true|false)",host):
  rows[int(kind)]=(*map(ast.literal_eval,(b,a,repeat)),restored=="true")
 close=lambda a,b:math.isclose(a,b,rel_tol=1e-5,abs_tol=.0003)
 matrix=set(rows)==set(range(8))
 for kind,(b,a,repeat,restored)in rows.items():
  if kind not in range(8)or any(len(v)!=8 for v in (b,a,repeat)):matrix=False;continue
  matrix&=restored and all(close(x,y)for x,y in zip(a,repeat))
  if kind==1:matrix&=a[1]>0 and b[1]>0 and close(1/a[1]-1/b[1],VALUES[kind])
  elif kind==2:matrix&=a[2]>b[2]
  else:
   field={0:0,3:3,4:4,5:5,6:6,7:7}[kind];matrix&=close(a[field]-b[field],VALUES[kind])
   if kind==5:matrix&=close(a[2],b[2])
 locks=re.findall(r"gem locked: slot=(\d+) guid=(\d+) amulet=(\d+) previous=(None|Some\(\d+\)) name=([^\r\n]+)",host)
 retained=re.findall(r"gem retained: slot=(\d+) guid=(\d+) amulet=(\d+) previous=Some\((\d+)\) name=([^\r\n]+)",host)
 upgrade_cards=[]
 for text in native:
  found=False
  for line in text.splitlines():
   if 'rogue: 'not in line:continue
   try:v=json.loads(line.split('rogue: ',1)[1])
   except ValueError:continue
   found|=any(c.get('kind')=='amulet'and c.get('rank',1)>1 and any(marker in c.get('lines','')for marker in ('Gem retained','Active gem:'))for c in v.get('choices',[]))
  upgrade_cards.append(found)
 validlocks=len(locks)==players and {int(s)for s,g,a,p,n in locks}==set(range(players))and all(n in NAMES for s,g,a,p,n in locks)
 owners={(s,g):n for s,g,a,p,n in locks}
 return dict(checks=dict(all_eight_native_bonuses=bool(matrix),one_gem_lock_per_player=validlocks,first_acquisition_grants=validlocks and all(p=="None"for s,g,a,p,n in locks),replacement_keeps_gem=bool(retained)and all(owners.get((s,g))==n for s,g,a,old,n in retained),every_player_replaces=all(any(int(s)==slot and a!=old for s,g,a,old,n in retained)for slot in range(players)),rebuilds_stable_and_inputs_restored="gem fixture rebuild: stable=true restored=true"in host and "stable=false"not in host and "restored=false"not in host,amulet_upgrade_annotation=all(upgrade_cards),native_card_annotation=all(any(marker in text for marker in ("gem native UI block:","kept bonus row: category=2")) for text in native)),locks=locks,replacements=retained,matrices=sorted(rows))
