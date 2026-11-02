"""Native two-player death/progression/rescue evidence; not an XP simulator."""
import json,re

def states(text):
 result=[]
 for line in text.splitlines():
  if "XP progression state: " not in line:continue
  try:result.append(json.loads(line.split("XP progression state: ",1)[1]))
  except ValueError:continue
 return result

def assess(host,guest):
 rows=states(host);checks={}
 checks["both_native_shared_guards_observed"]=all(f"official downed XP: path={i} "in host for i in(0,1))
 checks["native_whole_level_reward_consumed_while_dead"]=any(f"official downed XP: path={i} "in host for i in(2,3,4))
 checks["native_whole_level_receipt_preserved_for_dead"]=any(f"official downed XP: path={i} "in host for i in(5,6))
 checks["no_deferred_bank_or_replay"]=all(key not in host for key in("official XP deferred:","official XP replayed:"))
 for slot,stage in [(0,2),(1,6)]:
  dead=[r for r in rows if r.get("stage")==stage and len(r.get("actors",[]))==2 and r["actors"][slot].get("phase")=="Downed"]
  checks[f"slot_{slot}_levels_advance_while_dead"]=bool(dead)and max(r["actors"][slot]["level"]for r in dead)>min(r["actors"][slot]["level"]for r in dead)
  checks[f"slot_{slot}_shared_xp_and_level_converge_while_dead"]=len(dead)>=2 and all(r["actors"][0]["level"]==r["actors"][1]["level"]and isinstance(r["actors"][0].get("xp"),int)and r["actors"][0]["xp"]==r["actors"][1].get("xp")for r in dead[-2:])
  revived=[r for r in rows if r.get("stage")==stage+2 and len(r.get("actors",[]))==2 and r["actors"][slot].get("phase")=="Playing"]
  checks[f"slot_{slot}_native_revival_keeps_progress"]=bool(dead and revived)and revived[0]["actors"][slot]["level"]>=max(r["actors"][slot]["level"]for r in dead)
 settled=[r for r in rows if r.get("stage",0)>=8 and len(r.get("actors",[]))==2 and all(a.get("phase")=="Playing"for a in r["actors"])]
 checks["equal_xp_and_level_after_both_revivals"]=bool(settled)and all(r["actors"][0]["level"]==r["actors"][1]["level"]and isinstance(r["actors"][0].get("xp"),int)and r["actors"][0]["xp"]==r["actors"][1].get("xp")for r in settled[-3:])
 checks["both_cycles_completed"]="XP progression complete:"in host
 checks["guest_received_live_levels"]=bool(re.search(r'XP received HUD: .*"level":',guest))
 for slot,text in enumerate([host,guest]):
  audit=[]
  for line in text.splitlines():
   if "AZSX_DIAG "not in line:continue
   try:record=json.loads(line.split("AZSX_DIAG ",1)[1])
   except ValueError:continue
   if record.get("kind")=="ui.progression":audit.append(record.get("data",{}))
  checks[f"slot_{slot}_live_position_available"]=any(a.get("api")=="ok"and a.get("live",0)>20 for a in audit)
  checks[f"slot_{slot}_arrow_visible"]=any(a.get("visible",0)>20 for a in audit)
  checks[f"slot_{slot}_position_updates_between_snapshots"]=any(a.get("betweenSnapshots",0)>10 for a in audit)
 return dict(checks=checks,passed=all(checks.values()),scope="Real native damage/death, kills, XP pickups, immediate shared levels and proximity resurrection; passive live-position/arrow sampling. No direct XP/level writes or synthetic catch-up.")
