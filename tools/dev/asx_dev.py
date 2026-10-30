"""Explicit ASX compatibility workflow. Planning/checking never builds, stages or launches."""
import argparse,ast,json,os,shutil,subprocess,sys,time,uuid
from pathlib import Path
if not __package__:sys.path.insert(0,str(Path(__file__).resolve().parent))
try:
 from . import artifact_record,binding_registry,feature_catalog,compatibility,scenario_suites
 from .client_registry import ROOT,VERSION,SHA256
except ImportError:
 import artifact_record,binding_registry,feature_catalog,compatibility,scenario_suites
 from client_registry import ROOT,VERSION,SHA256

def atomic(path,value):
 path=Path(path);path.parent.mkdir(parents=True,exist_ok=True)
 temp=path.with_name(path.name+".tmp");temp.write_text(json.dumps(value,indent=2)+"\n");temp.replace(path)

def runtime(root):return Path(root)/("runtime-official-"+VERSION.replace(".",""))
def safe(root,relative):
 root=Path(root).resolve();p=root/relative
 if not p.resolve().is_relative_to(root)or any(a.is_symlink()for a in [p,*p.parents]if a!=root):raise ValueError("Redirected receiver/source path")
 return p

def stage(receiver,dll):
 receiver=Path(receiver).resolve(strict=True);dll=Path(dll).resolve(strict=True)
 if receiver==ROOT or receiver.is_relative_to(ROOT):raise ValueError("Receiver must be separate from source")
 record=json.loads(dll.with_suffix('.artifact.json').read_text());artifact_record.verify(record,dll)
 target=runtime(receiver)
 for name in ("AzerothSurvivors.exe","ASX-DEV.json","avrt.dll","IMPORT-OFFICIAL-"+VERSION.replace(".","")+".json"):safe(receiver,(target/name).relative_to(receiver))
 receipt=json.loads((target/("IMPORT-OFFICIAL-"+VERSION.replace('.','')+".json")).read_text())
 if receipt.get("version")!=VERSION or receipt.get("files",{}).get("AzerothSurvivors.exe")!=SHA256 or artifact_record.digest(target/"AzerothSurvivors.exe")!=SHA256:raise ValueError("Reviewed private import required")
 old=json.loads((target/"ASX-DEV.json").read_text())
 if artifact_record.digest(target/"avrt.dll")!=old.get("dll_sha256"):raise ValueError("Unowned/modified receiver DLL; preserving it")
 files=execution_inputs()
 # Resolve every destination before any writes; never remove unknown receiver files.
 metadata=[target.relative_to(receiver)/name for name in ('avrt.dll','ASX-DEV.json','avrt.artifact.json')]
 for relative in [*files,*metadata,'.asx-stage.json']:safe(ROOT,relative);safe(receiver,relative)
 for relative in [*metadata,'.asx-stage.json']:safe(receiver,str(relative)+'.tmp')
 copied=0;copied_bytes=0
 for relative,sha in sorted(files.items()):
  source=safe(ROOT,relative);dest=safe(receiver,relative)
  if artifact_record.digest(source)!=sha:raise ValueError('Staging input changed during preparation')
  if not dest.is_file()or artifact_record.digest(dest)!=sha:
   dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,dest)
   copied+=1;copied_bytes+=dest.stat().st_size
  if artifact_record.digest(dest)!=sha:raise ValueError('Staging copy identity mismatch')
 # Recheck provenance after copying tools, before replacing the owned DLL.
 artifact_record.verify(record,dll)
 if artifact_record.digest(target/"avrt.dll")!=record["dll_sha256"]:shutil.copy2(dll,target/"avrt.dll")
 atomic(target/"ASX-DEV.json",dict(dll_sha256=record["dll_sha256"],test_hooks=record["kind"]=="diagnostic"))
 atomic(target/"avrt.artifact.json",record)
 atomic(receiver/".asx-stage.json",dict(schema=2,version=VERSION,native_sha256=SHA256,kind=record["kind"],dll_sha256=record["dll_sha256"],source_fingerprint=record["source_fingerprint"],compilation_fingerprint=record.get("compilation_fingerprint",record["source_fingerprint"]),ui_fingerprint=artifact_record.ui_fingerprint(),artifact_fingerprint=artifact_record.fingerprint(record),files=files))
 verify_staged(receiver)
 print(json.dumps(dict(execution_files=len(files),execution_bytes=sum(safe(receiver,p).stat().st_size for p in files),copied_files=copied,copied_bytes=copied_bytes)))
 return record

def execution_inputs(root=ROOT):
 files=dict(artifact_record.ui_inputs(root));python_files=python_inputs(root)
 for name in scenario_suites.load_suites(root)['suites']:
  files.update({k:v for k,v in validation_inputs(name,root,python_files).items()if k!='scenario'})
 for relative in ('asx/supported-games.json','compat/suites.json'):
  files[relative]=artifact_record.digest(safe(root,relative))
 return files

def verify_staged(root=ROOT,kind=None):
 """Verify a trusted local staging receipt, without requiring compiler source files.

 This is local provenance, not a signed distribution or an adversarial trust root.
 Only source-side stage() issues it after verifying the compiler receipt.
 """
 root=Path(root).resolve();target=runtime(root)
 manifest=json.loads(safe(root,'.asx-stage.json').read_text())
 record=json.loads(safe(root,(target/'avrt.artifact.json').relative_to(root)).read_text())
 if manifest.get('schema')!=2 or manifest.get('version')!=VERSION or manifest.get('native_sha256')!=SHA256:raise ValueError('Current execution bundle receipt required')
 if manifest.get('artifact_fingerprint')!=artifact_record.fingerprint(record):raise ValueError('Compiler receipt changed after staging')
 if record.get('schema')not in(1,2)or record.get('version')!=VERSION or record.get('native_sha256')!=SHA256:raise ValueError('Artifact client identity mismatch')
 for key in ('dll_sha256','source_fingerprint','kind'):
  if manifest.get(key)!=record.get(key):raise ValueError('Staged artifact provenance mismatch')
 if record.get('source_fingerprint')!=artifact_record.fingerprint(record.get('inputs',{})):raise ValueError('Compiler input receipt mismatch')
 if record.get('schema')==2 and record.get('compilation_fingerprint')!=record.get('source_fingerprint'):raise ValueError('Compilation identity mismatch')
 if kind is not None and record.get('kind')!=kind:raise ValueError('Wrong artifact kind')
 dll=safe(root,(target/'avrt.dll').relative_to(root))
 if artifact_record.digest(dll)!=record['dll_sha256']:raise ValueError('Staged DLL changed')
 if (b'AZCOOP_TEST_EXTENSIONS'in dll.read_bytes())!=(record['kind']=='diagnostic'):raise ValueError('Artifact marker mismatch')
 owner=json.loads(safe(root,(target/'ASX-DEV.json').relative_to(root)).read_text())
 if owner.get('dll_sha256')!=record['dll_sha256']or owner.get('test_hooks')!=(record['kind']=='diagnostic'):raise ValueError('Receiver ownership mismatch')
 imported=json.loads(safe(root,(target/('IMPORT-OFFICIAL-'+VERSION.replace('.','')+'.json')).relative_to(root)).read_text())
 if imported.get('version')!=VERSION or imported.get('files',{}).get('AzerothSurvivors.exe')!=SHA256 or artifact_record.digest(safe(root,(target/'AzerothSurvivors.exe').relative_to(root)))!=SHA256:raise ValueError('Reviewed private import required')
 files=manifest.get('files')
 if not isinstance(files,dict)or not files:raise ValueError('Missing execution inventory')
 for relative,sha in files.items():
  if not isinstance(relative,str)or Path(relative).is_absolute()or '..'in Path(relative).parts or '\\'in relative or ':'in relative:raise ValueError('Invalid execution path')
  if artifact_record.digest(safe(root,relative))!=sha:raise ValueError('Execution file changed: '+relative)
 if execution_inputs(root)!=files:raise ValueError('Execution inventory differs from required scenario dependencies')
 if artifact_record.ui_fingerprint(root)!=manifest.get('ui_fingerprint'):raise ValueError('Staged UI inventory changed')
 return record

def python_inputs(root=ROOT):
 root=Path(root)
 # Resolve local Python imports without importing/executing platform-specific modules.
 pending=['asx_dev','background_desktop','extensions_trial'];seen=set();files={}
 while pending:
  name=pending.pop()
  if name in seen:continue
  seen.add(name);path=root/'tools/dev'/(name+'.py')
  if not path.is_file():continue
  files[path.relative_to(root).as_posix()]=artifact_record.digest(path)
  for node in ast.walk(ast.parse(path.read_text())):
   if isinstance(node,ast.Import):pending.extend(a.name.split('.')[0]for a in node.names)
   elif isinstance(node,ast.ImportFrom):
    if node.module:pending.append(node.module.split('.')[0])
    elif node.level:pending.extend(a.name for a in node.names)
 return files

def validation_inputs(suite_name,root=ROOT,python_files=None):
 root=Path(root);files=dict(python_inputs(root)if python_files is None else python_files)
 suite=scenario_suites.load_suites(root)['suites'][suite_name]
 data=['tools/dev/fixtures/party-talents115.json','tools/dev/fixtures/asx-extensions-audit.lua']
 env=suite['environment']
 if env.get('ASX_TEST_LUA_DIAGNOSTICS')=='1':data.append('tools/dev/diagnostics/observer.lua')
 if env.get('ASX_TEST_ALL_EVENTS')=='1'and suite['players']==1:data.append('tools/dev/fixtures/asx-supplies-audit.lua')
 if env.get('ASX_TEST_COMPLETION121')=='support':data.extend(['tools/dev/fixtures/asx-fatigue-audit.lua','tools/dev/fixtures/asx-tooltip121-audit.lua'])
 if env.get('ASX_TEST_COMPLETION121')=='pause':data.append('tools/dev/fixtures/asx-pause-audit.lua')
 for relative in data:files[relative]=artifact_record.digest(root/relative)
 files['scenario']=suite
 return files

def validation_fingerprint(suite_name=None):
 if suite_name is None:raise ValueError('A named scenario is required for proof identity')
 return artifact_record.fingerprint(validation_inputs(suite_name))

def prerequisite(suite,record,receipts):
 required=suite["solo_prerequisite"]
 if not required:return
 for receipt in receipts:
  if(receipt.get("suite")==required and receipt.get("passed")is True and receipt.get("dll_sha256")==record["dll_sha256"]and receipt.get("source_fingerprint")==record["source_fingerprint"]and record.get("ui_fingerprint")and receipt.get("ui_fingerprint")==record["ui_fingerprint"]and record.get("validation_fingerprint")and receipt.get("validation_fingerprint")==record.get("prerequisite_fingerprint",record["validation_fingerprint"])and receipt.get("version")==VERSION):return
 raise ValueError("A passing solo receipt for this exact artifact/source is required before coop")

def validate(name):
 if os.name!='nt':raise ValueError("Live validation must run with Windows Python in the staged receiver")
 suite=scenario_suites.load_suites()["suites"][name];target=runtime(ROOT)
 record=json.loads((target/"avrt.artifact.json").read_text());verify_staged(ROOT,kind=suite["kind"])
 record=dict(record,ui_fingerprint=artifact_record.ui_fingerprint(),validation_fingerprint=validation_fingerprint(name),prerequisite_fingerprint=validation_fingerprint(suite["solo_prerequisite"])if suite["solo_prerequisite"]else None)
 receipts=[json.loads(p.read_text())for p in(ROOT/"test-results/workflow").glob('*.json')]
 prerequisite(suite,record,receipts)
 before=set((ROOT/"test-results").glob('*/result.json'))
 env={k:v for k,v in os.environ.items()if not k.startswith(('ASX_TEST_','AZCOOP_TEST_'))};env.update(suite["environment"]);env['ASX_TEST_SCENARIO']=name
 command=[sys.executable,'-I','-B',str(ROOT/'tools/dev/background_desktop.py'),*suite['args'],'--seconds',str(suite['seconds'])]
 started=time.perf_counter()
 completed=subprocess.run(command,cwd=ROOT,env=env)
 elapsed=time.perf_counter()-started
 after=set((ROOT/"test-results").glob('*/result.json'))-before
 # One owned new result, never a pre-existing/latest unrelated run.
 if len(after)!=1:raise ValueError("Expected exactly one new scenario result")
 path=after.pop();result=json.loads(path.read_text())
 passed=completed.returncode==0 and result.get('passed')is True and result.get('scenario')==name and result.get('dll_sha256')==record['dll_sha256']
 value=dict(schema=1,suite=name,version=VERSION,passed=passed,dll_sha256=record['dll_sha256'],source_fingerprint=record['source_fingerprint'],compilation_fingerprint=record.get('compilation_fingerprint',record['source_fingerprint']),ui_fingerprint=record['ui_fingerprint'],validation_fingerprint=record['validation_fingerprint'],result=path.relative_to(ROOT).as_posix(),exit_code=completed.returncode,elapsed_seconds=round(elapsed,3))
 atomic(ROOT/'test-results/workflow'/('validation-'+uuid.uuid4().hex+'.json'),value)
 print(json.dumps(value,indent=2))
 if not passed:raise SystemExit(1)
 return value

def main():
 p=argparse.ArgumentParser(description=__doc__);sub=p.add_subparsers(dest='command',required=True)
 sub.add_parser('check');sub.add_parser('inventory');sub.add_parser('preflight')
 q=sub.add_parser('impact');q.add_argument('--comparison',type=Path,required=True)
 q=sub.add_parser('plan');q.add_argument('name',choices=scenario_suites.load_suites()['plans'])
 q=sub.add_parser('stage');q.add_argument('--receiver',type=Path,required=True);q.add_argument('--dll',type=Path,required=True)
 q=sub.add_parser('validate');q.add_argument('name',choices=scenario_suites.load_suites()['suites'])
 q=sub.add_parser('retry');q.add_argument('check',choices=scenario_suites.load_suites()['focused_retries'])
 a=p.parse_args()
 if a.command=='check':
  _,client,_=binding_registry.load_binding()
  if binding_registry.owned_path(client['descriptor']).read_text()!=binding_registry.render():raise ValueError('Stale binding')
  if(ROOT/'docs/ASX-FEATURES.md').read_text()!=feature_catalog.render():raise ValueError('Stale feature documentation')
  print('Client, binding, feature inventory and scenario schemas verified; no game launched.')
 elif a.command=='preflight':verify_staged();print('Execution bundle, artifact provenance and native identity verified; no game launched.')
 elif a.command=='inventory':print(json.dumps(compatibility.inventory(),indent=2))
 elif a.command=='impact':print(json.dumps(compatibility.impact(json.loads(a.comparison.read_text())),indent=2))
 elif a.command=='plan':print(json.dumps([dict(name=n,**scenario_suites.load_suites()['suites'][n])for n in scenario_suites.load_suites()['plans'][a.name]],indent=2))
 elif a.command=='stage':stage(a.receiver,a.dll);print('Explicit receiver staging complete; no game launched.')
 elif a.command=='validate':validate(a.name)
 elif a.command=='retry':validate(scenario_suites.load_suites()['focused_retries'][a.check])
if __name__=='__main__':main()
