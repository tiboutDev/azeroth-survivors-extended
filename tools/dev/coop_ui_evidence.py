"""Bounded real cooperative HUD regression evidence; visual review is separate."""
import json,math

def rows(text):
 result=[]
 for line in text.splitlines():
  if "AZSX_DIAG "not in line:continue
  try:row=json.loads(line.split("AZSX_DIAG ",1)[1])
  except ValueError:continue
  if row.get("kind")=="ui.coop_hud"and isinstance(row.get("data"),dict):result.append(row)
 return result

def expected(clock):
 if 2<=clock<6:return 1.4,1.4
 if 6<=clock<10:return 1.4,1.5
 if clock>=10:return 1.55,1.5

def assess(host,guest):
 streams=[rows(host),rows(guest)];checks={}
 checks["real_fortune_replacements"]=all(f"coop economy fixture: stage={i} "in host for i in(1,2,3))
 checks["native_economy_trinkets_filtered"]="native economy pool excluded: input=1152 output=1368"in host
 checks["talents_inspection_does_not_open_personal_menu"]="official inspection pause ignored: open=true"in host and "official menu request: open=true role=Host"not in host
 for slot,stream in enumerate(streams):
  stages=set();bad=[];actions=set();max_clock=0
  for row in stream:
   data=row["data"];clock=row.get("run_clock");wall=row.get("clock",0)
   if not isinstance(clock,(int,float)):continue
   max_clock=max(max_clock,clock)
   common=data.get("sharedEconomy",data.get("economy"))
   target=expected(clock)
   if target and common and data.get("available")and data.get("phase")in("playing","choosing"):
    actual=(common.get("xp"),common.get("gold"))
    if all(isinstance(v,(int,float))and math.isclose(v,t,abs_tol=0.00001)for v,t in zip(actual,target)):stages.add(target)
    elif all(not(edge<=clock<edge+.5)for edge in(2,6,10)):bad.append(dict(clock=clock,actual=actual,expected=target))
   stage=data.get("uiActions",{}).get("stage",0)
   if stage in(1,3)and data.get("pinned")=="stats":actions.add(stage)
   if stage in(2,4)and not data.get("pinned"):actions.add(stage)
  checks[f"slot_{slot}_shared_values_refresh"]=len(stages)==3 and not bad
  checks[f"slot_{slot}_stats_and_native_character_commands"]=actions=={1,2,3,4}
  checks[f"slot_{slot}_world_keeps_advancing"]=max_clock>=25
  checks[f"slot_{slot}_stats_labels_show_effective_values"]=any(r["data"].get("hud",{}).get("xp")=="+55%"and r["data"].get("hud",{}).get("gold")=="+50%"for r in stream)
  if bad:checks[f"slot_{slot}_no_wrong_projection"]=False
 return dict(checks=checks,passed=all(checks.values()),scope="Real Fortune replacements, native reward-pool filter, live API/labels and paired native commands; native overlay visibility and rendered pixels require explicit review.")
