"""Run declared targeted portable scenarios; never infer live coverage or launch games."""
import argparse,json,subprocess,sys,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(Path(__file__).resolve().parent))
from test_plan import PROFILES

def scenarios():
 data=json.loads((ROOT/'compat/feature-validation.json').read_text())['features']
 result={k:v['targeted_scenario']for k,v in data.items()if 'targeted_scenario'in v}
 for name,s in result.items():
  if s['profile']not in PROFILES or any(not s.get(k)for k in ('trigger','expected','limit')):raise ValueError('Incomplete scenario: '+name)
 return result

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--run',action='store_true');p.add_argument('--output',type=Path,required=True);a=p.parse_args()
 cases=scenarios();profiles=sorted({s['profile']for s in cases.values()});results={}
 if a.run:
  for profile in profiles:
   start=time.monotonic();r=subprocess.run(['sh','tools/dev/check.sh','--profile',profile],cwd=ROOT)
   results[profile]=dict(passed=r.returncode==0,elapsed_seconds=round(time.monotonic()-start,3))
 report=dict(schema=1,scenarios=cases,profiles=results,live_executed=False,scope='Portable scenario evidence only; live acceptance remains separate.')
 a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_text(json.dumps(report,indent=2)+'\n')
 if any(not r['passed']for r in results.values()):raise SystemExit(1)
if __name__=='__main__':main()
