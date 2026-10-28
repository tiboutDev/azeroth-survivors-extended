"""Summarize bounded startup timings without treating elapsed time as CPU time."""
import json,re,sys
from pathlib import Path

def assess(root):
 root=Path(root);result={"players":[]}
 for folder in sorted(root.glob("player-*")):
  text=(folder/"azeroth-coop-native.log").read_text(errors="replace")
  samples=[]
  for line in text.splitlines():
   if "ASX startup profile: " not in line:continue
   t=float(re.search(r"([0-9.]+)s",line)[1]);rows=json.loads(line.split("ASX startup profile: ",1)[1]);samples.append((t,rows))
  if not samples:raise ValueError("Missing native timing samples")
  peaks={};previous={}
  for t,rows in samples:
   for row in rows:
    name=row["name"];old=previous.get(name,(0,0));dt=t-old[0]
    if dt>0:
     cost=(row["ns"]-old[1])/1e6
     if name not in peaks or cost/dt>peaks[name]["ms_per_second"]:peaks[name]={"at":t,"ms_per_second":cost/dt,"window_seconds":dt}
    previous[name]=(t,row["ns"])
  result["players"].append({"player":folder.name,"seconds":samples[-1][0],"hooks":samples[-1][1],"peak_windows":peaks})
 raw=json.loads((root/"process-cpu.json").read_text());previous={};cpu=[]
 for row in raw:
  old=previous.get(row["pid"])
  if old:
   dt=row["t"]-old["t"]
   if dt>0:cpu.append({"t":row["t"],"slot":row["slot"],"cores":(row["cpu_seconds"]-old["cpu_seconds"])/dt})
  previous[row["pid"]]=row
 result["cpu_peak_cores_per_client"]={str(slot):max((r["cores"]for r in cpu if r["slot"]==slot),default=0)for slot in(1,2,3)}
 result["cpu_total_seconds_per_client"]={str(row["slot"]):row["cpu_seconds"]for row in previous.values()}
 result["cpu_samples"]=cpu
 return result
if __name__=="__main__":
 result=assess(sys.argv[1]);(Path(sys.argv[1])/"profile-summary.json").write_text(json.dumps(result,indent=2))
 print(json.dumps({k:v for k,v in result.items()if k!="cpu_samples"},indent=2))
