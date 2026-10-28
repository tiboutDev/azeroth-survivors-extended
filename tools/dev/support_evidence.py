"""Strict evidence for controlled native support trials, not a combat balance claim."""
import re

def events(text,name):
    rows=[]
    for line in text.splitlines():
        if f'event {name} ' in line:
            rows.append({k:float(v) for k,v in re.findall(r'(\w+)=(-?\d+(?:\.\d+)?(?:e[+-]?\d+)?)',line,re.I)})
    return rows

def assess(logs):
    native=[row for text in logs.values() for row in events(text,'ally_support_native_result')]
    results={}
    for spell in (635,331,8936,15237):
        results[str(spell)]=sorted({int(r['target']) for r in native if r.get('spell')==spell and r.get('after',0)>r.get('before',0)})
    for spell in (6673,17,139,774,12042,13750,5171,11426,642,20925,324,467):
        results[str(spell)]=sorted({int(r['target']) for r in native if r.get('spell')==spell and r.get('buff_present')==1})
    rules=[events(logs[n],'run_rules_applied') for n in (1,2)]
    options=all('options=[200.0, 200.0, 150.0, 100.0]' in logs[n] for n in (1,2))
    luck=all(rules) and rules[0][0].get('personal_luck')==.25 and rules[1][0].get('personal_luck')==.5
    faults=[line for text in logs.values() for line in text.splitlines() if '[FAILED]' in line or 'ACCESS_VIOLATION' in line]
    scaled=events(logs[1],'enemy_coop_scaled')
    boss=any(r.get('boss')==1 and abs(r.get('factor',0)-1.85)<1e-5 for r in scaled)
    ordinary=any(r.get('boss')==0 and abs(r.get('factor',0)-2.0)<1e-5 for r in scaled)
    return dict(passed=all(v==[1,2] for v in results.values()) and options and luck and boss and ordinary and not faults,
                ordinary_scaling_verified=ordinary,boss_scaling_verified=boss,
                verified_recipients_by_spell=results,host_options_received=options,individual_luck_at_start=luck,faults=faults)
