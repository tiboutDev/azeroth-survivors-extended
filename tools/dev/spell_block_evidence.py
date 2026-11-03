"""Require real native ally-triggered burst and mark observations."""
import re

def assess(host):
 checks={}
 for caster in (0,1):
  checks[f'polymorph_{caster}_applied_and_consumed']=bool(re.search(rf'spell block polymorph: caster={caster} result=Ok\(\(\d+, true, true\)\)',host))
  hits=re.findall(rf'spell block explosion: caster={caster} guid=\d+ enemy=\d+ raw=([^ ]+) effective=Some\(([^)]+)\)',host)
  checks[f'polymorph_{caster}_native_damage']=bool(hits) and any(float(raw)>0 and float(effective)>0 for raw,effective in hits)
 for stage,expected in ((1,.35),(2,.2),(3,0.)):
  observed=re.findall(rf'marks121 native: stage={stage} marked=(?:true|false) vulnerability=([^\s]+)',host)
  checks[f'mark_stage_{stage}']=bool(observed) and any(abs(float(v)-expected)<1e-5 for v in observed)
 native=re.search(r"spell block native hunters: Ok\((\[[^\n]+\])\)",host)
 import ast
 rows=ast.literal_eval(native[1])if native else []
 checks['two_native_hunters']=len(rows)==4 and len({e for e,_ in rows})==1 and all(abs(v-w)<1e-5 for (_,v),w in zip(rows,(.35,.35,.2,0.)))
 return checks

"""Require native talent outputs, including negative controls."""
import re

def assess_talents(host):
 rows={int(case):(float(damage),float(cooldown)) for case,damage,cooldown in re.findall(r'talent block result: case=(\d+) result=Ok\(\(([^,]+), Some\(([^)]+)\)\)\)',host)}
 checks={'talent_cases_complete':set(rows)==set(range(1,7))}
 for case in (2,5):checks[f'talent_burst_{case}']=case in rows and rows[case][0]>0
 for case in (3,4):checks[f'talent_blocked_{case}']=case in rows and rows[case][0]==0
 for case in (1,6):checks[f'talent_cooldown_{case}']=case in rows and rows[case][1]>0
 heals=re.findall(r'talent block native: case=1 source=\d+ target=\d+ hp_before=([^ ]+) hp_after=([^ ]+)',host)
 checks['swiftmend_ally_healed']=any(float(a)>float(b)for b,a in heals)
 checks['rapture_same_ally']=bool(re.search(r'talent block native: case=6 .*shields=Ok\(\[[^\]]*1[^\]]*\]\)',host))
 return checks

def assess_sets(host):
 rows={int(n):(float(a),float(b)) for n,a,b in re.findall(r'set block result (\d+): Ok\(\(([^,]+), ([^)]+)\)\)',host)}
 checks={'set_cases_complete':set(rows)==set(range(18))}
 for n,name in enumerate(('serpent','rupture','flame_shock','agony')):
  checks['foreign_'+name]=n in rows and rows[n][int(n in (0,3))]>0
 for n,name in [(4,'devout_with_aegis'),(6,'empowered_devout_annulus'),(10,'judgement_own_kill')]:
  checks[name]=n in rows and rows[n][0]>0
 for n,name in [(5,'devout_rejects_lightning'),(9,'judgement_rejects_ally_kill')]:
  checks[name]=n in rows and rows[n][0]==0
 for n,name in [(7,'mortal_strike'),(8,'earth_shock')]:
  checks[name+'_strongest']=n in rows and all(abs(v-.3)<.00001 for v in rows[n])
 traces=host.replace('\r\n','\n')
 for n,expected in [(4,[(8,12),(10,15)]),(5,[]),(6,[(8,12)])]:
  window=traces.partition(f'set block start {n}\n')[2].partition(f'set block result {n}:')[0]
  seen=[(float(a),float(b)) for a,b in re.findall(r'set shield radius: base=([\d.]+) effective=([\d.]+)',window)]
  checks[f'shield_queries_{n}']=all(seen.count(v)==1 for v in expected) and (bool(expected) or not seen)
 for n,cls in enumerate(('paladin','hunter','rogue','priest','mage','warlock','druid'),11):
  expected=60 if cls=='warlock' else 100
  checks['ally_control_'+cls]=n in rows and rows[n][0]>0 and abs(rows[n][1]-rows[n][0]-expected)<1
 return checks
