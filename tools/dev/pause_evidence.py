"""Read-only asymmetric pause evidence; command probes do not inject native input."""
import ast,re
from pathlib import Path

def assess_logs(logs):
 checks={};gates=[];snapshots=[]
 for text in logs:
  rows=[]
  for m in re.finditer(r"pause fixture gate: paused=(true|false) blocked=(\d+) nonmovement=(\d+) windows=(\[[^\r\n]*\])",text):
   rows.append((m[1]=="true",int(m[2]),int(m[3]),ast.literal_eval(m[4])))
  gates.append(rows)
 for slot,kind in ((0,"level"),(1,"cache")):
  rows=[(p,b,n)for p,b,n,windows in gates[slot] if any(k==kind and delay==3 and 4<=elapsed<=9 for k,elapsed,delay in windows)]
  checks[kind+"_waiting_player_blocks_movement"]=len(rows)>=2 and all(p and b==1 and n==0 for p,b,n in rows)
 host=logs[0]
 windows=[]
 for line in host.splitlines():
  gate=re.search(r"pause fixture gate:.*windows=(\[[^\r\n]*\])",line)
  if gate:windows=ast.literal_eval(gate[1])
  sample=re.search(r"pause fixture snapshot: clock=([\d.]+) actors=(\[[^\r\n]*\])",line)
  if sample:
   actors=ast.literal_eval(re.sub(r"\b(Playing|Choosing|Paused|Downed|Inactive)\b",r'"\1"',sample[2]))
   snapshots.append((float(sample[1]),actors,windows))
 for kind in ("level","cache"):
  rows=[(clock,actors)for clock,actors,windows in snapshots if any(k==kind and 4<=elapsed<=9 for k,elapsed,delay in windows) and sorted(a[1]for a in actors)==["Choosing","Playing"]]
  stable=len(rows)>=2
  if stable:
   stable=max(c for c,_ in rows)-min(c for c,_ in rows)<0.001
   before={s:p for s,phase,p in rows[0][1]}
   stable=stable and all(set(before)=={s for s,_,_ in actors} and all(max(abs(a-b)for a,b in zip(before[s],p))<.01 for s,phase,p in actors)for clock,actors in rows)
  checks[kind+"_mixed_phase_clock_and_positions_frozen"]=stable
 cache=[clock for clock,actors,windows in snapshots if any(k=="cache" and 4<=elapsed<=9 for k,elapsed,delay in windows) and any(a[1]=="Choosing" for a in actors)]
 checks["resumes_after_last_cache_pick"]=bool(cache) and any(clock>max(cache)+1 and all(a[1]=="Playing"for a in actors)for clock,actors,windows in snapshots)
 checks["both_players_choose_cache"]=all("pause fixture held: kind=cache" in text for text in logs)
 checks["native_cache_opened_once"]=host.count("pause fixture native cache: true")==1
 return {"checks":checks,"command_probe_scope":"Real command filter and RW stub contract; no OS input, native movement packet injection or position writes.","snapshot_samples":len(snapshots)}

def assess(output):return assess_logs([(Path(output)/f"player-{i}/azeroth-coop-native.log").read_text(errors="replace")for i in (1,2)])

def resumed_motion(host,players=2,after_clock=10):
 """Require post-barrier native motion of each actor, not pre-run ROAM input."""
 samples=[]
 for clock,raw in re.findall(r"pause fixture snapshot: clock=([0-9.e+-]+) actors=(\[[^\r\n]*\])",host):
  actors=ast.literal_eval(re.sub(r"\b(Playing|Choosing|Paused|Downed|Inactive)\b",r'"\1"',raw))
  if float(clock)>after_clock and len(actors)==players and all(phase=="Playing"for _,phase,_ in actors):samples.append(actors)
 checks={}
 for slot in range(players):
  positions=[pose for actors in samples for who,phase,pose in actors if who==slot]
  checks[f"actor_{slot}_moves_after_group_resume"]=len(positions)>=2 and any(max(abs(a-b)for a,b in zip(positions[0],p))>.1 for p in positions)
 return checks

def menu_evidence(host):
 """Observe native clock and actor motion across two overlapping menu owners."""
 def times(pattern):return [float(t)for t in re.findall(r"\[ INFO \]\s+([0-9.]+)s "+pattern,host)]
 ho=times(r"official menu request: open=true role=Host")
 hc=times(r"official menu request: open=false role=Host")
 go=times(r"official peer menu: slot=1 open=true")
 gc=times(r"official peer menu: slot=1 open=false")
 hc=[t for t in hc if ho and t>ho[0]];gc=[t for t in gc if go and t>go[0]]
 checks={"menu_host_open_close":bool(ho and hc),"menu_guest_open_close_over_native_stream":bool(go and gc)}
 ordered=bool(ho and go and hc and gc) and ho[0]<go[0]<hc[0]<gc[0]
 checks["menu_owners_overlap_host_closes_first"]=ordered
 rows=[];paused=False
 for line in host.splitlines():
  gate=re.search(r"pause fixture gate: paused=(true|false)",line)
  if gate:paused=gate[1]=="true"
  snap=re.search(r"\[ INFO \]\s+([0-9.]+)s pause fixture snapshot: clock=([0-9.e+-]+) actors=(\[[^\r\n]*\])",line)
  if snap:
   actors=ast.literal_eval(re.sub(r"\b(Playing|Choosing|Paused|Downed|Inactive)\b",lambda m:chr(34)+m[1]+chr(34),snap[3]));rows.append((float(snap[1]),float(snap[2]),actors,paused))
 frozen=[row for row in rows if ordered and ho[0]+2<row[0]<gc[0]-.1]
 checks["menu_clock_and_both_positions_frozen"]=len(frozen)>=4 and all(p for _,_,_,p in frozen) and max(c for _,c,_,_ in frozen)-min(c for _,c,_,_ in frozen)<.001 and all(actors==frozen[0][2]for _,_,actors,_ in frozen)
 waiting=[row for row in frozen if ordered and row[0]>hc[0]+1]
 checks["menu_first_close_does_not_release_other_owner"]=len(waiting)>=2 and all(p for _,_,_,p in waiting)
 resumed=[row for row in rows if ordered and row[0]>gc[0]+1 and not row[3] and all(a[1]=="Playing"for a in row[2])]
 checks["menu_clock_resumes_after_last_close"]=bool(frozen and resumed) and max(c for _,c,_,_ in resumed)>frozen[-1][1]+1
 for slot in (0,1):
  poses=[pose for _,_,actors,_ in resumed for who,phase,pose in actors if who==slot]
  checks[f"menu_actor_{slot}_moves_after_last_close"]=len(poses)>=2 and any(max(abs(a-b)for a,b in zip(poses[0],pose))>.1 for pose in poses)
 return {"checks":checks,"events":{"host_open":ho,"guest_open":go,"host_close":hc,"guest_close":gc},"frozen_samples":len(frozen),"remaining_owner_samples":len(waiting)}
