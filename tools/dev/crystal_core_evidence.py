"""Real Crystal Resonance receipt/stat evidence for current native trials."""
import ast,math,re
def crystal_stat_checks(text,slot):
 rows=[]
 for found,owner,counts,base,actual,stable,restored in re.findall(r"crystal native stats: slot=(\d+) owner=(\d+) counts=(\[[^\n]+?\]) base=(\[[^\n]+?\]) actual=Ok\((\[[^\n]+?\])\) stable=(true|false) restored=(true|false)",text):
  if int(found)==slot:rows.append((ast.literal_eval(counts),ast.literal_eval(base),ast.literal_eval(actual),stable,restored))
 # The probe borrows the inner native rebuild before the outer coop AoE pass.
 # These are the approved nominal 12/7/15/12 inputs; final HUD AoE is tested separately.
 close=lambda a,b:math.isclose(a,b,rel_tol=1e-5,abs_tol=.0003)
 valid=bool(rows)
 for c,b,a,stable,restored in rows:
  valid=valid and len(a)==8 and len(b)==8 and stable==restored=='true'
  valid=valid and close(a[0]-b[0],c[0]*.12)and a[1]>0 and b[1]>0 and close(1/a[1]-1/b[1],c[1]*.07)and close(a[7]-b[7],c[3]*.12)
  valid=valid and(a[2]>b[2]if c[2]>0 else close(a[2],b[2]))
 return {'native_personal_stats':bool(valid)}

def assess_core(text,players):
 checks={"amulet_gems_retired":"amulet gems retired"in text and "gem locked:"not in text}
 for slot in range(players):
  checks[f"crystal_claim_{slot}"]=bool(re.search(r"crystal collected: event=\d+ slot="+str(slot)+r" owner=[1-9]\d* color=[0-3] counts=",text))
  counts=[]
  for found,values in re.findall(r"crystal native stats: slot=(\d+) owner=\d+ counts=(\[[^\n]+?\])",text):
   if int(found)==slot:counts.append(ast.literal_eval(values))
  checks[f"crystal_four_colors_{slot}"]=any(len(c)==4 and all(type(v)is int and v>0 for v in c)for c in counts)
  checks[f"crystal_native_stats_{slot}"]=crystal_stat_checks(text,slot)["native_personal_stats"]
 return {"checks":checks,"passed":all(checks.values())}
