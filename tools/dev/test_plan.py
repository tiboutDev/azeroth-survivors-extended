"""Select current ASX checks. Never builds a DLL, deploys or launches a game.
Unknown executable changes select the full portable gate.
"""
import argparse,json,os,subprocess,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def cargo(*args):return ['cargo',*args,'--locked']
def py(pattern):return ['python3','-m','unittest','discover','-s','tests','-p',pattern,'-v']
ARCH=py('test_gameplay_boundary.py')
SOURCE=py('test_asx_source_boundary.py')
NATIVE=cargo('test','-p','asx-game','--test','client_binding','--test','jewelry','--test','enemies')
PROFILES={
 'gems':[cargo('test','-p','asx-gameplay','--lib','gems::'),NATIVE,py('test_gem_evidence.py'),ARCH],
 'enchantments':[cargo('test','-p','asx-gameplay','--lib','enchantments::'),NATIVE,py('test_enchantment_evidence.py'),ARCH],
 'glyphs':[cargo('test','-p','asx-gameplay','--lib','glyphs::'),cargo('test','-p','asx-gameplay','--lib','kept_bonus::'),NATIVE,py('test_glyphs_evidence.py'),ARCH],
 'fatigue':[cargo('test','-p','asx-gameplay','--lib','resurrection_fatigue::'),NATIVE,ARCH],
 'asx-install':[py('test_asx.py'),py('test_asx_publication.py')],
 'official-party-ui':[cargo('test','-p','asx-gameplay','--lib','menu_projection::'),cargo('test','-p','asx-gameplay','--lib','party_text::'),['uv','run','--with','lupa==2.6','python3','tests/test_official_party_lua.py'],NATIVE],
 'official':[cargo('test','-p','asx-gameplay','--test','official','--test','support','--test','immunity_aggro'),cargo('test','-p','asx-gameplay','--lib','official::menu_owner_tests'),cargo('test','-p','asx-game','--lib','official_native::'),NATIVE,py('test_extension_completion_evidence.py'),py('test_client_binding.py'),py('test_pause_evidence.py'),py('test_pause_motion.py'),py('test_reward_wallet_evidence.py'),py('test_immunity_aggro_evidence.py'),ARCH,SOURCE],
 'rules':[cargo('test','-p','asx-gameplay'),ARCH],
 'combat':[cargo('test','-p','asx-gameplay','--test','combat','--test','immunity_aggro'),NATIVE,ARCH],
 'support':[cargo('test','-p','asx-gameplay','--test','support','--test','periodic'),NATIVE,ARCH],
 'periodic':[cargo('test','-p','asx-gameplay','--test','periodic','--test','support'),ARCH],
 'tooling':[py('test_test_plan.py'),py('test_offline_comparison.py'),SOURCE,py('test_trial_limits.py'),py('test_startup_profile.py'),py('test_native_exit_evidence.py'),py('test_background_trials.py'),py('test_native_runtime_import.py'),py('test_build_only.py')],
 'full':[cargo('test','-p','asx-gameplay','-p','asx-game'),py('test_*.py'),cargo('clippy','-p','asx-gameplay','-p','asx-game','--all-features')]
}
def choose(files):
 selected=set()
 tooling={'tools/dev/compare_official.py','tests/test_offline_comparison.py','tools/dev/check.sh','tools/dev/test_plan.py','tools/dev/trial_limits.py','tools/dev/startup_profile.py','tools/dev/assess_startup_profile.py','tools/dev/native_profile_codec.py','tools/dev/native_exit_evidence.py','tools/dev/disposable115_party.py','tools/dev/background_desktop.py','tools/dev/extensions_trial.py','tools/dev/asx_install_trial.py','tests/test_test_plan.py','tests/test_asx_source_boundary.py','tests/test_trial_limits.py','tests/test_startup_profile.py','tests/test_native_exit_evidence.py','tests/test_background_trials.py','tools/dev/native_runtime_import.py','tools/dev/import_official.py','tools/dev/build-windows.sh','tools/dev/build-chest-fixture.sh','tests/test_native_runtime_import.py','tests/test_build_only.py'}
 for name in files:
  if name.endswith('.md')or name.startswith('docs/'):continue
  if name in tooling:selected.add('tooling')
  elif name in ('Cargo.toml','Cargo.lock','asx/supported-games.json')or name.startswith('crates/asx-game/src/bindings/'):return ['full']
  elif name.startswith('asx/')or name in ('tests/test_asx.py','tests/test_asx_publication.py','tools/build/asx_package.py','tools/build/asx_verify_publication.py'):selected.add('asx-install')
  elif name in ('native-ui/official-party.lua','src/runtime/party_ui.rs','src/runtime/menu_ui.rs','crates/asx-gameplay/src/party_text.rs','crates/asx-gameplay/src/menu_projection.rs','tests/test_official_party_lua.py','tools/dev/requirements-ui.txt'):selected.add('official-party-ui')
  elif name in ('crates/asx-gameplay/src/gems.rs','crates/asx-game/src/official_native/gems.rs','src/runtime/gems.rs','tools/dev/gem_evidence.py','tests/test_gem_evidence.py'):selected.add('gems')
  elif name in ('crates/asx-gameplay/src/enchantments.rs','crates/asx-game/src/official_native/enchantments.rs','crates/asx-game/tests/current/jewelry.rs','src/runtime/enchantments.rs','tools/dev/enchantment_evidence.py','tests/test_enchantment_evidence.py'):selected.add('enchantments')
  elif name.startswith(('crates/asx-game/src/official_native','src/runtime/','src/presentation/'))or name in ('crates/asx-game/src/official.rs','src/runtime.rs','tests/test_client_binding.py'):selected.add('official')
  elif name.startswith('crates/asx-gameplay/'):selected.add('rules')
  else:return ['full']
 return sorted(selected)
def changed(base=None):
 args=['git','diff','--name-only',base or 'HEAD','--'];tracked=subprocess.check_output(args,cwd=ROOT,text=True).splitlines()
 untracked=subprocess.check_output(['git','ls-files','--others','--exclude-standard'],cwd=ROOT,text=True).splitlines()
 return sorted(set(tracked+untracked))
def plan(profiles,files):
 selected=profiles or choose(files)
 if 'full'in selected:selected=['full']
 commands=[]
 for profile in selected:
  for command in PROFILES[profile]:
   if command not in commands:commands.append(command)
 crates=sorted({cmd[i+1]for cmd in commands if cmd[0]=='cargo'for i,a in enumerate(cmd[:-1])if a=='-p'})
 if 'full'not in selected and crates:commands.append(cargo('clippy',*[a for c in crates for a in ['-p',c]]))
 return dict(profiles=selected,files=files,selection='explicit'if profiles else 'changed_files',commands=commands,live_game_launched=False,dll_built=False,
  live_guidance='One two-client scenario for changed native/shared behavior; enable recovery only if synchronization/lifecycle is affected. Full release validation is explicit.')
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--profile',action='append',choices=sorted(PROFILES));parser.add_argument('--base');parser.add_argument('--dry-run',action='store_true');parser.add_argument('--list',action='store_true');parser.add_argument('--cpu-budget',type=int,default=20,help='Owned check-runner CPU share, 5..50 percent');parser.add_argument('--reason',help='Why this scope covers the current change');a=parser.parse_args()
 if not 5<=a.cpu_budget<=50:raise SystemExit('CPU budget must be 5..50 percent')
 if a.list:print('\n'.join(PROFILES));return
 value=plan(a.profile,changed(a.base));value['reason']=a.reason or ('Explicit profiles reviewed against the changed behavior' if a.profile else 'Conservative changed-file selection; unmapped code falls back to full');print(json.dumps(value,indent=2),flush=True)
 if a.dry_run:return
 run=ROOT/'test-results/targeted'/time.strftime('%Y%m%d-%H%M%S');run.mkdir(parents=True,exist_ok=False);value['steps']=[]
 from trial_limits import portable_limits
 if value['commands']:value['resource_limits']=portable_limits(a.cpu_budget)
 env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1',CARGO_BUILD_JOBS='2');start=time.monotonic()
 for i,command in enumerate(value['commands']):
  before=time.monotonic();log=run/f'{i+1:02d}.log'
  with log.open('wb')as output:code=subprocess.run(command,cwd=ROOT,env=env,stdout=output,stderr=subprocess.STDOUT).returncode
  value['steps'].append(dict(command=command,exit_code=code,seconds=round(time.monotonic()-before,3),log=log.name));value['passed']=code==0;value['seconds']=round(time.monotonic()-start,3);(run/'result.json').write_text(json.dumps(value,indent=2)+'\n')
  print(f'[{i+1}/{len(value["commands"])}] {"PASS"if code==0 else "FAIL"} {command}; log: {log}',flush=True)
  if code:raise SystemExit(code)
 if not value['commands']:(run/'result.json').write_text(json.dumps(dict(value,passed=True,seconds=0),indent=2)+'\n');print('No executable changes selected.')
if __name__=='__main__':main()
