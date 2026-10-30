"""Opt-in real ASX API/widget trial, one solo or two coop clients in a private desktop."""
import hashlib,json,os,re,shutil,socket,subprocess,sys,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools/dev'))
import background_desktop as background
from client_registry import CLIENT,SHA256
from disposable115_party import writer

def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def adapter_rejections(text):
 # Taunt immunity is the expected goblin policy, not a failed native contact.
 expected=re.compile(r"\[ INFO \]\s+\d+\.\d+s treasure goblin taunt rejected: enemy=\d+ spell=\d+$")
 return [line for line in text.splitlines()if any(v in line for v in ('[ WARN ]','[ ERROR ]','rejected:'))and not expected.fullmatch(line)]

def duration_checks(samples):
 buffs=[b['remaining']for sample in samples for b in sample.get('structured',{}).get('consumables',{}).values()]
 pins=[b['left']for sample in samples for b in sample.get('structured',{}).get('markers',{}).values()if b.get('kind')==0]
 # A recipient who already claimed has no personal pin. Require observed initial
 # duration across the trial, not on a client whose marker correctly disappeared.
 return {'sixty_second_buffs':bool(buffs)and 59<=max(buffs)<=60,'supply_pin_bounds':all(0<=left<=90 for left in pins)},max(pins,default=0)

def assess(samples):
 available=[s for s in samples if s.get('available')]
 widgets={}
 for kind in ('supplies','fortune','sayge'):
  rows=[s.get('widgets',{}).get(kind,{})for s in samples]
  widgets[kind]={'shown':any(x.get('shown')for x in rows),'text_matches':any(x.get('shown')and x.get('textMatches')for x in rows),'toggle_verified':any(x.get('hideWorks')and x.get('showWorks')and x.get('displayOnly')for x in rows)}
 checks={'api_v1':bool(available)and all(s.get('major')==1 for s in available),'globals':bool(available)and all(s.get('globals')for s in available),'recipient_is_string':bool(available)and all(isinstance(s.get('recipient'),str)for s in available),'copy_isolation':bool(available)and all(s.get('copyIsolated')for s in available),'deep_copy_isolation':any(s.get('deepCopyIsolated')for s in available),'callbacks':any(s.get('callbacks',0)>0 and s.get('badCallbacks',0)>0 for s in samples),'callback_revision_consistent':all(not s.get('callbackMismatch')for s in samples),'values_separate_from_texts':any(s.get('structured',{}).get('discovery')and 'name'not in s['structured']['discovery']and s.get('texts',{}).get('supplies')for s in available)}
 checks.update({f'{k}_{test}':value for k,row in widgets.items()for test,value in row.items()})
 for kind,field,expected in [('supplies','scale',.85),('fortune','alpha',.75),('sayge','scale',.9)]:
  checks[f'{kind}_{field}_configured']=any(abs(s.get('widgets',{}).get(kind,{}).get(field,-10)-expected)<.01 for s in samples)
 return checks

def assess_wow_visual(samples):
 shown=[s['buffOrder']for s in samples if s.get('structured',{}).get('fortune')and s.get('buffOrder',{}).get('fortuneShown')]
 return {'fortune_in_first_native_position':bool(shown)and all(v.get('first')for v in shown),'native_fortune_tooltip_observed':any("Sayge's Dark Fortune"in v.get('tooltip','')for v in shown)}

def run_trial(args,gate):
 players=args.api_trial
 if players not in (1,2):raise ValueError('One or two clients only')
 reported_fixes=os.environ.get('ASX_API_TRIAL_REPORTED_FIXES')=='1'
 catalog_review=os.environ.get('ASX_API_TRIAL_CATALOG_REVIEW')=='1'
 wow_visual=os.environ.get('ASX_API_TRIAL_WOW_VISUAL')=='1'
 source=Path(os.environ['ASX_API_TRIAL_RUNTIME']).resolve(strict=True)
 dll=ROOT/'diagnostic/avrt.dll'
 receipt=json.loads((ROOT/'diagnostic/avrt.artifact.json').read_text())
 if digest(source/'AzerothSurvivors.exe')!=SHA256 or digest(dll)!=receipt['dll_sha256']or receipt['kind']!='diagnostic':raise ValueError('Artifact identity mismatch')
 out=gate.parent/'real-api';out.mkdir();native=out/'native';native.mkdir()
 shutil.copyfile(source/'AzerothSurvivors.exe',native/'AzerothSurvivors.exe');shutil.copyfile(dll,native/'avrt.dll')
 for name in CLIENT['archives']:
  target=native/name;target.parent.mkdir(parents=True,exist_ok=True)
  try:os.link(source/name,target)
  except OSError:shutil.copyfile(source/name,target)
 prepare=writer(native/'AzerothSurvivors.exe')
 clean={k:v for k,v in os.environ.items()if not k.upper().startswith(('ASX_','AZCOOP_','COOP_','WOW_','BENILLA_'))}
 with socket.socket()as sock:sock.bind(('127.0.0.1',0));port=sock.getsockname()[1]
 password='ASX-API-private-local'
 children=[];handles=[]
 try:
  for slot in range(players):
   profile=out/f'player-{slot+1}';save=profile/'appdata/wenilla-rogue';save.mkdir(parents=True);prepare(save,1,name_override='ApiHost'if slot==0 else'ApiGuest')
   (profile/'localappdata').mkdir();(profile/'shots').mkdir()
   (profile/'config.toml').write_text('[cvars]\ngxWindow = "1"\ngxResolution = "1280x720"\n')
   addons=profile/'AddOns';addon=addons/'ASX';addon.mkdir(parents=True)
   (addon/'ASX.toc').write_text('## Interface: 11200\n## Title: ASX\nPresentation.lua\n')
   shutil.copyfile(ROOT/'native-ui/official-party.lua',addon/'Presentation.lua')
   shutil.copytree(ROOT/'addons/ASXUI',addons/'ASXUI')
   probe=addons/'ASXAPIAudit';probe.mkdir();(probe/'ASXAPIAudit.toc').write_text('## Interface: 11200\n## Title: ASX API audit\nAudit.lua\n');(probe/'Audit.lua').write_text(('ASX_WOW_VISUAL_AUDIT=true\n'if wow_visual else'')+(ROOT/'tools/dev/fixtures/asx-api-live-audit.lua').read_text())
   env=dict(clean,APPDATA=str(profile/'appdata'),LOCALAPPDATA=str(profile/'localappdata'),BENILLA_HOME=str(profile),ASX_ACTIVE='1',AZCOOP_ENGINE='official',AZCOOP_NATIVE_LOG_DIRECTORY=str(profile),ASX_READY_FILE=str(profile/'ready.json'),WOW_NOSOUND='1',WOW_WIN='1',WOW_ROGUE_UPDATE='0',WOW_ROGUE_UPDATE_AUTO='0',WOW_ROGUE_AUTOPICK='1',WOW_ROGUE_AUTOSTART='1',WOW_ROGUE_AUTOPLAY='4',WOW_ROGUE_SHOTS=str(profile/'shots'),WOW_ROGUE_SHOTS_EVERY='10',RUST_LOG='info',ASX_TREASURE_GOBLINS='0',ASX_LOST_SUPPLIES='1',ASX_DARKMOON_TRIALS='1')
   if reported_fixes:env.update(ASX_TREASURE_GOBLINS='1',ASX_TEST_COOP_BUFF_STATS='1',ASX_TEST_COOP_CIRCLE_EXIT='1')
   if catalog_review:env.update(ASX_TEST_EVENT_CATALOG_REVIEW='1',ASX_TEST_COOP_CIRCLE_EXIT='0',ASX_TREASURE_GOBLINS='1')
   if players==1:env.update(ASX_TEST_LOST_SUPPLIES='1',ASX_TEST_DARKMOON_TRIAL='1')
   else:
    env.update(ASX_TEST_COOP_EVENTS='1',WOW_ROGUE_COOP_PASSWORD=password)
    env['WOW_ROGUE_COOP_HOST'if slot==0 else'WOW_ROGUE_COOP_JOIN']=str(port)if slot==0 else f'127.0.0.1:{port}'
   stream=(profile/'native.log').open('wb');handles.append(stream)
   children.append(subprocess.Popen([str(native/'AzerothSurvivors.exe')],cwd=native,env=env,stdout=stream,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW))
   time.sleep(4 if players==2 and slot==0 else 1)
  deadline=time.monotonic()+args.seconds
  while time.monotonic()<deadline:
   if any(c.poll()is not None for c in children):raise RuntimeError('Owned client exited before trial completion')
   time.sleep(1)
 finally:
  for c in children:
   if c.poll()is None:c.terminate()
  for c in children:
   try:c.wait(timeout=10)
   except subprocess.TimeoutExpired:c.kill();c.wait(timeout=5)
  for stream in handles:stream.close()
 results=[]
 for slot in range(players):
  profile=out/f'player-{slot+1}';text=(profile/'native.log').read_text(errors='replace')
  samples=[json.loads(line.split('ASX_API_LIVE ',1)[1])for line in text.splitlines()if 'runtime error: ASX_API_LIVE 'in line]
  errors=[line for line in text.splitlines()if 'ui_script: runtime error:'in line and 'ASX_API_LIVE 'not in line]
  checks=assess(samples);checks['clean_lua']=not errors;checks['asx_started']=(profile/'ready.json').exists()
  if reported_fixes:
   asx=(profile/'azeroth-coop-native.log').read_text(errors='replace')
   if slot==0:
    if not catalog_review:checks['first_goblin_spawned']='treasure goblin spawned:'in asx
    if not catalog_review:checks['circle_exit_exercised']='coop circle exit begin:'in asx
    checks['circle_exit_stopped']='coop circle exit advanced unexpectedly'not in asx
    checks['native_buff_probe']='coop consumable native stats:'in asx
    if not catalog_review:checks['native_goblin_skull']='assigned=true'in asx and 'goblin native skull:'in asx
    checks['native_visual_stop']='pause remote visual stop: packets=2'in asx
   durations,pin_max=duration_checks(samples);checks.update(durations)
   if catalog_review:
    seen=False;gaps=[]
    for sample in samples:
     if sample.get('available'):seen=True
     elif seen and sample.get('native',{}).get('phase')in ('playing','choosing','paused'):gaps.append(sample.get('sample'))
    checks['continuous_active_api']=not gaps
    checks['real_position_discovery']=any(s.get('structured',{}).get('discovery')and isinstance(s.get('native',{}).get('x'),(int,float))for s in samples)
   checks['no_adapter_rejection']=not adapter_rejections(asx)
  if wow_visual:
   checks.update(assess_wow_visual(samples))
  (profile/'samples.json').write_text(json.dumps(samples,indent=2)+'\n')
  results.append({'slot':slot,'samples':len(samples),'checks':checks,'errors':errors,'recipients':sorted({s['recipient']for s in samples if s.get('recipient')}),'supply_pin_max_seconds':duration_checks(samples)[1]})
 checks={'each_client_passed':all(all(row['checks'].values())for row in results)}
 if reported_fixes and not catalog_review:checks['ninety_second_supply_observed']=any(row['supply_pin_max_seconds']>=85 for row in results)
 if players==2:checks['distinct_recipients']=len(set(g for row in results for g in row['recipients']))==2
 result={'passed':all(checks.values()),'checks':checks,'clients':results,'players':players,'seconds':args.seconds,'native_sha256':SHA256,'dll_sha256':digest(dll),'artifact_receipt':receipt,'synthetic_api':False,'wow_visual':wow_visual,'catalog_review':catalog_review,'official_installation_touched':False}
 (out/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items()if k!='artifact_receipt'}),flush=True)
 if not result['passed']:raise SystemExit(1)
def parse_args(argv=None):
 parser=background.build_parser()
 parser.add_argument('--api-trial',type=int,choices=(1,2),required=True,help='Explicit real API trial player count')
 args=parser.parse_args(argv)
 if not 15<=args.seconds<=300:parser.error('Use 15..300 seconds')
 if any(getattr(args,name)for name in ('resource_probe','extensions_trial','ritual_trial','supplies_trial','goblins_trial','gems_trial','enchantments_trial','reward_wallet_trial','immunity_aggro_trial','asx_install','extensions_normal','asx_package')):parser.error('API trial cannot be combined with another mode')
 return args
if __name__=='__main__':
 background.__file__=__file__
 background.run_child=run_trial
 background.parse_args=parse_args
 background.child_flags=lambda args:['--api-trial',str(args.api_trial)]
 background.main()
