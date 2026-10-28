"""Conservation, fixed bonuses and locked-ring evidence from a bounded native trial."""
import ast,collections,re,math
NAMES=("Might","Alacrity","Fortitude","Precision","Ferocity","Warding","Renewal","Reach")
def assess(host,native,players):
 matrices={}
 for name,base,actual,repeat,restored in re.findall(r"enchantment matrix: name=(\w+) base=Ok\((\[[^\r\n]+?\])\) actual=Ok\((\[[^\r\n]+?\])\) repeat=Ok\((\[[^\r\n]+?\])\) restored=(true|false)",host):
  b,a,rr=map(ast.literal_eval,(base,actual,repeat));matrices[name]=(b,a,rr,restored=="true")
 close=lambda a,b:math.isclose(a,b,rel_tol=1e-5,abs_tol=0.0002)
 native_values=len(matrices)==8
 for name,(b,a,repeat,restored)in matrices.items():
  native_values&=all(close(x,y)for x,y in zip(a,repeat))and restored
  if name=="Fortitude":native_values&=a[2]>b[2]
  elif name=="Alacrity":native_values&=close(1/a[1]-1/b[1],.08)
  else:
   field,value={"Might":(0,.15),"Precision":(3,.05),"Ferocity":(4,.35),"Warding":(5,.04),"Renewal":(6,1.5),"Reach":(7,.15)}[name];native_values&=close(a[field]-b[field],value)
   if name=="Warding":native_values&=close(a[2],b[2])
 locks=[(int(s),int(g),int(r),int(rank),name)for s,g,r,rank,name in re.findall(r"enchantment locked: slot=(\d+) guid=(\d+) ring=(\d+) rank=(\d+) name=(\w+)",host)]
 counts=collections.Counter((s,g,r)for s,g,r,rank,name in locks)
 retained={}
 for s,g,r,name,rank in re.findall(r'\((\d+), (\d+), (\d+), "(\w+)", Ok\(Some\(\((\d+), \d+\)\)\)\)',host):retained[(int(s),int(g),int(r))]=(name,int(rank))
 # Require every lock to retain its identity, and actual later upgrades for
 # each participant. Native random pools need not offer both rings again.
 later=bool(locks)and all(retained.get((s,g,r),(None,0))[0]==name for s,g,r,rank,name in locks)and all(any(s==slot and retained.get((s,g,r),(None,0))[1]>=3 for s,g,r,rank,name in locks)for slot in range(players))
 annotated=[]
 import json
 for text in native:
  cards=[]
  for line in text.splitlines():
   if line.startswith('rogue: {'):
    try:v=json.loads(line[7:])
    except ValueError:continue
    cards.extend(c for c in v.get("choices",[])if "Enchantment:"in c.get("lines",""))
  annotated.append(any("Applies enchantment"in c["lines"]for c in cards)and any("Applies enchantment"not in c["lines"]for c in cards))
 return dict(checks=dict(all_eight_native_bonuses=native_values,one_lock_per_ring=bool(counts)and all(n==1 for n in counts.values()),two_rings_per_player=all(sum(1 for s,g,r in counts if s==slot)==2 for slot in range(players)),applied_on_first_upgrade=bool(locks)and all(rank==2 for s,g,r,rank,name in locks),later_upgrades_preserve_enchantment=later,first_and_later_card_text=all(annotated),rebuilds_do_not_accumulate="enchantment fixture rebuild: stable=true restored=true"in host and "stable=false"not in host and "restored=false"not in host),locks=locks,retained=[dict(slot=s,guid=g,ring=r,name=n,rank=rank)for(s,g,r),(n,rank)in retained.items()],matrices=matrices)

RANK_VALUES={"Might":[.05,.09,.15,.22,.30],"Alacrity":[.02,.04,.07,.10,.14],"Fortitude":[.05,.10,.17,.25,.35],"Precision":[.01,.02,.04,.07,.10],"Ferocity":[.10,.20,.32,.48,.65],"Warding":[.01,.02,.035,.05,.07],"Renewal":[.3,.6,1.2,2.,3.],"Reach":[.05,.09,.15,.22,.30]}
def assess_ranked(host,native,players):
 # Preserve historical unranked evidence and its assessor; reuse its conservation
 # checks with the new Minor input observations, then require the full matrix.
 minor=re.sub(r"enchantment ranked matrix: tier=0 name=", "enchantment matrix: name=",host)
 result=assess(minor,native,players)
 matrices={}
 for tier,name,base,actual,repeat,restored in re.findall(r"enchantment ranked matrix: tier=(\d+) name=(\w+) base=Ok\((\[[^\r\n]+?\])\) actual=Ok\((\[[^\r\n]+?\])\) repeat=Ok\((\[[^\r\n]+?\])\) restored=(true|false)",host):
  matrices[(name,int(tier))]=(*map(ast.literal_eval,(base,actual,repeat)),restored=="true")
 close=lambda a,b:math.isclose(a,b,rel_tol=1e-5,abs_tol=.0003)
 valid=set(matrices)=={(n,t)for n in NAMES for t in range(5)}
 for (name,tier),(b,a,repeat,restored)in matrices.items():
  if name not in RANK_VALUES or tier not in range(5) or (name,0)not in matrices:valid=False;continue
  valid&=restored and len(a)==8 and all(close(x,y)for x,y in zip(a,repeat));value=RANK_VALUES[name][tier]
  if name=="Alacrity":valid&=close(1/a[1]-1/b[1],value)
  elif name=="Fortitude":
   mb,ma,_,_=matrices[(name,0)];valid&=a[2]>b[2]and close((a[2]-b[2])/value,(ma[2]-mb[2])/.05)
  else:
   field={"Might":0,"Precision":3,"Ferocity":4,"Warding":5,"Renewal":6,"Reach":7}[name];valid&=close(a[field]-b[field],value)
   if name=="Warding":valid&=close(a[2],b[2])
 promotions=[(int(s),int(r),tier or "Regular",name)for s,r,tier,name in re.findall(r"enchantment promoted: slot=(\d+) guid=\d+ ring=\d+ rank=(\d+) name=(?:(Lesser|Greater|Superior) )?(\w+)",host)]
 result["checks"].pop("all_eight_native_bonuses",None)
 result["checks"].update(all_40_native_rank_values=valid,all_four_promotions_observed={(r,t)for _,r,t,_ in promotions}=={(4,"Lesser"),(7,"Regular"),(10,"Greater"),(14,"Superior")},every_player_promotes=all(any(s==slot for s,_,_,_ in promotions)for slot in range(players)),promotion_card_text=all("Improves enchantment"in text and "→"in text for text in native))
 result["ranked_matrices"]=[dict(name=n,tier=t,base=b,actual=a,repeat=rr,restored=ok)for(n,t),(b,a,rr,ok)in matrices.items()];result["promotions"]=promotions
 return result

def assess_fixture_party(host):
 """Fixture grants update the current actor before the next host projection.
 Require identical XP throughout and gold convergence at the next sampled boundary.
 """
 rows=[]
 for line in host.splitlines():
  if "official party: clock="not in line:continue
  actors=re.findall(r"phase: Playing, gold: (\d+), level: (\d+), xp: (\d+)",line)
  time=re.search(r"(\d+\.\d+)s",line)
  if len(actors)==2 and time:rows.append((float(time[1]),[tuple(map(int,a))for a in actors]))
 transients=[i for i,(_,a)in enumerate(rows)if a[0][0]!=a[1][0]]
 checks=dict(equal_progression=len(rows)>=3 and all(a[0][1:]==a[1][1:]for _,a in rows),gold_converges_after_fixture=len(rows)>=3 and all(i+1<len(rows)and rows[i+1][0]-rows[i][0]<=5 and rows[i+1][1][0][0]==rows[i+1][1][1][0]for i in transients))
 return dict(checks=checks,comparable_samples=len(rows),transient_gold_samples=[dict(time=rows[i][0],actors=rows[i][1],next_time=rows[i+1][0]if i+1<len(rows)else None)for i in transients])
