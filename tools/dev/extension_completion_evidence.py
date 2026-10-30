"""Acceptance of observed native support/resurrection, never synthetic success."""
import re
def assess(host,mode):
 checks={}
 if mode=="shield":
  checks["shield_cast"]=bool(re.search(r"support121 cast: index=5 spell=17 caster=0 target=1 before=[^ ]+ result=Ok\(true\)",host))
  checks["shield_on_recipient"]=bool(re.search(r"support121 observed: index=5 spell=17 caster=0 target=1 before=[^ ]+ after=[^ ]+ slots=Ok\(\[1\]\)",host))
  checks["shield_consumed"]="support121 shield break: Ok(true)"in host
  return dict(checks=checks)
 if mode=="support":
  casts=re.findall(r"support121 cast: index=(\d+) spell=(\d+) caster=(\d+) target=(\d+) before=([^ ]+) result=Ok\(true\)",host)
  observed=re.findall(r"support121 observed: index=(\d+) spell=(\d+) caster=(\d+) target=(\d+) before=([^ ]+) after=([^ ]+) slots=Ok\((\[[^\n]*?\])\)",host)
  checks["twelve_native_casts"]={int(i)for i,*_ in casts}==set(range(12))
  checks["twelve_results_observed"]={int(i)for i,*_ in observed}==set(range(12))
  byspell={int(sp):(int(c),int(t),float(b),float(a),slots)for i,sp,c,t,b,a,slots in observed}
  checks["direct_heals_both_ways"]=all(sp in byspell and byspell[sp][3]>byspell[sp][2]for sp in (635,331))
  checks["foreign_periodics_and_buffs"]=all(sp in byspell and str(byspell[sp][1])in byspell[sp][4]for sp in (139,774,17,467,324))
  deliveries=re.findall(r"official support delivered: caster=(\d+) target=(\d+) spell=(\d+) raw=([^ ]+) effective=([^\r\n]+)",host)
  checks["direct_healing_receipts_both_ways"]=all(any(int(sp)==spell and int(c)==caster and int(t)==1-caster and float(e)>0 for c,t,sp,raw,e in deliveries)for spell,caster in ((635,0),(331,1)))
  checks["hybrid_receipts"]=all(any(int(sp)==spell and float(e)>0 for c,t,sp,raw,e in deliveries)for spell in (15237,25912))
  checks["actual_retaliation_damage"]=all(any(int(sp)==spell and float(e)>0 for sp,e in re.findall(r"official retaliation hit:.*?spell=(\d+).*?effective=([^\r\n]+)",host))for spell in (467,324))
  checks["ice_barrier_caster_only"]=11426 in byspell and byspell[11426][4]=="[0]"
  checks["shield_consumed"]="support121 shield break: Ok(true)"in host
  checks["retaliations_exercised"]=all(re.search(rf"support121 retaliation: spell={sp} hit=Ok\(Some\(\d+\)\)",host)is not None for sp in (467,324))
  return dict(checks=checks,casts=casts,observed=observed)
 if mode in ("fatigue","fatigue-self"):
  checks["two_revivals"]="native revival: stacks=1"in host and "native revival: stacks=2"in host
  checks["stack_recovers"]="fatigue fixture recovery:"in host
  checks["native_debuff_one_and_two_stacks"]=all(f"fatigue121 native aura: slot=1 stacks={i} "in host for i in (1,2))
  seen=set();stacks=0;healthy=False
  for line in host.splitlines():
   state=re.search(r"fatigue fixture (?:state:.*? |native revival: )stacks=(\d)",line)
   if state:stacks=int(state[1])
   m=re.search(r"fatigue fixture effect: kind=(\d) source=Some\(\((\d), \d+\)\) raw=([\d.e+-]+) scaled=([\d.e+-]+)",line)
   if m:
    kind,source,raw,scaled=int(m[1]),int(m[2]),float(m[3]),float(m[4])
    if raw>0 and source==1 and stacks>0 and abs(scaled/raw-(1-.05*stacks))<.001:seen.add(kind)
    if source==0 and kind==1 and abs(scaled-raw)<.001:healthy=True
  checks.update({f"scaled_effect_{i}":i in seen for i in range(4)});checks["healthy_healer_unchanged"]=healthy
  if mode=="fatigue-self":return dict(checks=dict(self_periodic_penalty=3 in seen,native_fatigue_present="fatigue121 native aura: slot=1 stacks=1 "in host))
  return dict(checks=checks)
 raise ValueError("Unknown evidence mode")
