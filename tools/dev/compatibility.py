"""Named native contract inventory and conservative compatibility impact reports.
Never approves a build, modifies a descriptor, deploys files or launches a game.
"""
import argparse,ast,fnmatch,hashlib,json,re,subprocess
from pathlib import Path
try:
 from .binding_registry import load_binding,parse_literal
 from .client_registry import ROOT,selected_client
 from .feature_catalog import load_features
 from .contract_attribution import apply_attribution
except ImportError:
 from binding_registry import load_binding,parse_literal
 from client_registry import ROOT,selected_client
 from feature_catalog import load_features
 from contract_attribution import apply_attribution

def inventory():
 version,client,binding=load_binding();constants={c["name"]:parse_literal(c["value"])for c in binding["constants"]}
 addresses=dict(constants["ADDRESSES"]);extensions=dict(constants["EXTENSION_ADDRESSES"])
 roles={c["name"]:c.get("role")for c in binding["constants"]}
 domains=json.loads((ROOT/"compat/domains.json").read_text())["domains"]
 paths=sorted(set((ROOT/"crates/asx-game/src/official_native").rglob("*.rs"))|{ROOT/"crates/asx-game/src/official_native.rs"})
 manifest=ROOT/'tools/dev/native-inventory/Cargo.toml'
 parsed=json.loads(subprocess.check_output(['cargo','run','--locked','--quiet','--manifest-path',str(manifest),'--',*[str(p)for p in paths]],cwd=ROOT,text=True))
 features=load_features()
 suites=json.loads((ROOT/"compat/suites.json").read_text())["suites"]
 contracts=[]
 for path in paths:
  relative=path.relative_to(ROOT).as_posix()
  for method in parsed[str(path)]:
   body=method['body']
   refs=set()
   for old in re.findall(r"self\.rva\(\s*(0x[0-9a-f]+)\s*\)",body):
    if int(old,16)in addresses:refs.add(addresses[int(old,16)])
   for old in re.findall(r"self\.contact117\(\s*(0x[0-9a-f]+)\s*\)",body):
    if int(old,16)in extensions:refs.add(extensions[int(old,16)])
   if "self.rva(v)"in body or"self.rva(*"in body:
    refs.update(addresses[int(x,16)]for x in re.findall(r"0x[0-9a-f]+",body)if int(x,16)in addresses)
   named=sorted(set(re.findall(r"(?:crate::current|layout|native|v)::([A-Z][A-Z0-9_]*)",body)))
   for name in named:
    value=constants.get(name)
    if isinstance(value,int)and name.endswith(("_RVA","_TABLE","_CALLER")):
     refs.add(extensions.get(value,addresses.get(value,value))if roles.get(name)=="canonical-symbol-key"else value)
    elif isinstance(value,tuple)and len(value)==2 and isinstance(value[0],int)and isinstance(value[1],list):refs.add(value[0])
   contract_domains=sorted(d for d,patterns in domains.items()if any(fnmatch.fnmatch(relative,pattern)for pattern in patterns))or["unclassified"]
   contract_id=relative.removeprefix("crates/asx-game/src/").removesuffix(".rs").replace("/",".")+"."+method["name"]
   contracts.append(dict(id=contract_id,method=method["name"],source=relative,domains=contract_domains,native_addresses=sorted(refs),descriptor_constants=named,adapter_source_sha256=hashlib.sha256(body.encode()).hexdigest()))
 contracts,features=apply_attribution(contracts,features,suites)
 return dict(schema=1,client_version=version,client_sha256=client["sha256"],binding_sha256=hashlib.sha256(json.dumps(binding,sort_keys=True).encode()).hexdigest(),contracts=contracts,features=features,approval=False)

def impact(comparison,baseline=None):
 baseline=inventory()if baseline is None else baseline
 if comparison.get("old_sha256")!=baseline["client_sha256"]:raise ValueError("Comparison does not match baseline native identity")
 if comparison.get("approved")is not False or comparison.get("offline_only")is not True or comparison.get("mapping_policy")!="instruction-alignment-v2":raise ValueError("Only unapproved instruction-alignment evidence is accepted")
 new=comparison.get("new_sha256","")
 if not re.fullmatch(r"[0-9a-f]{64}",new):raise ValueError("Missing candidate image identity")
 mapping={int(k):v for k,v in comparison.get("mapping",{}).items()}
 results=[]
 for c in baseline["contracts"]:
  missing=[v for v in c["native_addresses"]if v not in mapping]
  state="unresolved"if missing else"instruction_candidates_context_review_required"if c["native_addresses"]else"layout_review_required"
  results.append(dict(id=c["id"],domains=c["domains"],features=c.get("features",[f["id"]for f in baseline["features"]if set(c["domains"])&set(f["native_domains"])]),validation_suites=c.get("validation_suites",[]),validation_profiles=c.get("validation_profiles",[]),link_basis=c.get("link_basis","legacy-conservative"),link_evidence=c.get("link_evidence",{}),state=state,unresolved=missing,candidates={str(v):mapping[v]for v in c["native_addresses"]if v in mapping}))
 features=[]
 for f in baseline["features"]:
  affected=[r for r in results if "unclassified"in r["domains"]or f["id"]in r["features"]]
  dependencies=[r['id']for r in affected]
  suites=sorted({suite for r in affected for suite in r['validation_suites']})
  features.append(dict(id=f["id"],disposition=f["disposition"],contracts=dependencies,validation_suites=suites,declared_live_suites=f.get("declared_live_suites",[]),live_suite_gap=f.get("live_suite_gap",True),validation_profiles=f.get("validation_profiles",sorted({p for r in affected for p in r["validation_profiles"]})),validation_note=f.get("validation_note","Legacy attribution; review validation coverage."),review_required=True))

 return dict(schema=1,baseline_sha256=baseline["client_sha256"],candidate_sha256=new,contracts=results,features=features,required_portable_profiles=["full","official-party-ui"],live_order=["solo","coop-two"],approved=False,scope="Candidates only. Caller ABI, registers, layouts, shared hooks and data remain explicit review requirements; no runtime activation.")

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument("--comparison",type=Path);p.add_argument("--out",type=Path,required=True);a=p.parse_args()
 if a.out.exists():p.error("Preserve existing evidence; choose a new output")
 value=impact(json.loads(a.comparison.read_text()))if a.comparison else inventory()
 a.out.parent.mkdir(parents=True,exist_ok=True);a.out.write_text(json.dumps(value,indent=2)+"\n");print(a.out)
if __name__=="__main__":main()
