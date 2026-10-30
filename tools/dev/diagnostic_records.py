"""Strict, passive Lua evidence collection. Model records never prove rendered UI."""
import json,math
KINDS={"observer.started","observer.stopped","ui.supply_toast","ui.sayge_progress","ui.buffs","ui.tooltip","asx.ritual_model"}
def collect(text,session,slot):
 records=[];errors=[];last_clock=-1
 for line in text.splitlines():
  if "ASX_DIAG "not in line:continue
  try:
   row=json.loads(line.split("ASX_DIAG ",1)[1])
   if row.get("schema")!=1 or row.get("session")!=session or row.get("slot")!=slot:raise ValueError("identity")
   if row.get("sequence")!=len(records)+1 or len(records)>=512:raise ValueError("sequence/budget")
   clock=row.get("clock")
   if isinstance(clock,bool)or not isinstance(clock,(int,float))or not math.isfinite(clock)or clock<last_clock:raise ValueError("clock")
   if row.get("kind")not in KINDS or not isinstance(row.get("data"),dict):raise ValueError("payload")
   last_clock=clock;records.append(row)
  except (ValueError,TypeError)as error:errors.append(str(error))
 checks=dict(valid_stream=not errors,started=bool(records)and records[0]["kind"]=="observer.started")
 return dict(records=records,errors=errors,checks=checks,passed=all(checks.values()))

def assess_events(stream):
 records=stream["records"]
 def rows(kind):return [r["data"]for r in records if r["kind"]==kind]
 def visible(row):return row.get("exists")is True and row.get("visible")in(True,1) and row.get("shown")in(True,1) and row.get("width",0)>0 and row.get("height",0)>0
 toasts=rows("ui.supply_toast");progress=rows("ui.sayge_progress");buffs=rows("ui.buffs")
 checks=dict(stream["checks"])
 checks.update(toast_rendered=any(visible(r)and r.get("title")and r.get("reward")and r.get("icon")for r in toasts),
  progress_rendered=any(visible(r)for r in progress),
  progress_hidden_observed=any(r.get("exists")and r.get("shown")in(False,0) for r in progress),
  native_buff_observed=any(r.get("available")and any(v.get("texture")and isinstance(v.get("remaining"),(int,float))and v["remaining"]>0 for v in r.get("slots",{}).values())for r in buffs))
 # Passive observation deliberately does not claim hovered tooltip coverage or pixel aesthetics.
 return dict(checks=checks,passed=all(checks.values()),limits=["No pixel layout or hover tooltip claim", "Ritual model is separate from widget observations"])
