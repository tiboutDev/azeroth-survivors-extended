"""Opt-in real ASX HUD trial on a private desktop and disposable native profiles."""
import hashlib,json,math,os,re,secrets,shutil,socket,subprocess,sys,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools/dev'))
import background_desktop as background
from client_registry import CLIENT,SHA256
from disposable115_party import writer
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def compact_popup(popup):
 # Native widget coordinates include renderer floating-point rounding.
 width,height,icon=(popup.get(key)for key in ('width','height','icon'))
 return all(isinstance(value,(int,float))and math.isfinite(value)for value in (width,height,icon))and abs(width-320)<.1 and height>=79.9 and abs(icon-32)<.1 and popup.get('inside')is True
def native_goblin_unmarked(log):
 rows=re.findall(r'goblin native marker observation: guid=(\d+) skull=(true|false)\b',log)
 return bool(rows)and all(int(guid)>0 and marked=='false'for guid,marked in rows)
def run(args,gate):
 source=Path(os.environ['ASX_HUD_TRIAL_RUNTIME']).resolve(strict=True);dll=ROOT/'diagnostic/avrt.dll'
 receipt=json.loads((ROOT/'diagnostic/avrt.artifact.json').read_text())
 if digest(source/'AzerothSurvivors.exe')!=SHA256 or digest(dll)!=receipt['dll_sha256']or receipt['kind']!='diagnostic':raise ValueError('Artifact identity mismatch')
 out=gate.parent/'hud';out.mkdir();native=out/'native';native.mkdir()
 for filename in ['AzerothSurvivors.exe',*CLIENT['archives']]:
  target=native/filename;target.parent.mkdir(parents=True,exist_ok=True)
  if filename.endswith('.exe'):shutil.copyfile(source/filename,target)
  else:
   try:os.link(source/filename,target)
   except OSError:shutil.copyfile(source/filename,target)
 shutil.copyfile(dll,native/'avrt.dll');prepare=writer(native/'AzerothSurvivors.exe')
 with socket.socket()as sock:sock.bind(('127.0.0.1',0));port=sock.getsockname()[1]
 clean={k:v for k,v in os.environ.items()if not k.upper().startswith(('ASX','AZCOOP_','COOP_','WOW_','BENILLA_'))}
 password=secrets.token_hex(16)
 children=[];streams=[];profiles=[]
 try:
  for slot in range(args.hud_trial):
   profile=out/f'player-{slot+1}';profiles.append(profile);save=profile/'appdata/wenilla-rogue';save.mkdir(parents=True);prepare(save,1,name_override='HudHost'if slot==0 else'HudGuest',known_items=[])
   (profile/'shots').mkdir();(profile/'localappdata').mkdir();(profile/'config.toml').write_text('[cvars]\ngxWindow = "1"\ngxResolution = "1280x720"\n')
   addons=profile/'AddOns';addons.mkdir();shutil.copytree(ROOT/'addons/ASXUI',addons/'ASXUI')
   bridge=addons/'ASX';bridge.mkdir();(bridge/'ASX.toc').write_text('## Interface: 11200\nPresentation.lua\n');shutil.copyfile(ROOT/'native-ui/official-party.lua',bridge/'Presentation.lua')
   probe=addons/'ASXUIHudAudit';probe.mkdir();(probe/'ASXUIHudAudit.toc').write_text('## Interface: 11200\nProbe.lua\n');shutil.copyfile(ROOT/'tools/dev/fixtures/asxui-hud-live.lua',probe/'Probe.lua')
   env=dict(clean,APPDATA=str(profile/'appdata'),LOCALAPPDATA=str(profile/'localappdata'),BENILLA_HOME=str(profile),ASX_ACTIVE='1',AZCOOP_ENGINE='official',AZCOOP_NATIVE_LOG_DIRECTORY=str(profile),ASX_READY_FILE=str(profile/'ready.json'),WOW_NOSOUND='1',WOW_WIN='1',WOW_ROGUE_UPDATE='0',WOW_ROGUE_UPDATE_AUTO='0',WOW_ROGUE_AUTOPICK='1',WOW_ROGUE_AUTOSTART='1',WOW_ROGUE_AUTOPLAY='4',WOW_ROGUE_SHOTS=str(profile/'shots'),WOW_ROGUE_SHOTS_EVERY='10',RUST_LOG='info',ASX_TREASURE_GOBLINS='0',ASX_LOST_SUPPLIES='1',ASX_DARKMOON_TRIALS='1',ASX_TEST_LOST_SUPPLIES='1',ASX_TEST_DARKMOON_TRIAL='1',AZCOOP_TEST_ENCHANTMENTS='1',ASX_TEST_HUD_FIXTURE='1')
   if args.crystal_badges:env.update(ASX_TEST_CRYSTALS='1',ASX_LOST_SUPPLIES='0',ASX_DARKMOON_TRIALS='0',ASX_TEST_LOST_SUPPLIES='0',ASX_TEST_DARKMOON_TRIAL='0')
   if args.hud_followups:env['ASX_TEST_HUD_OWNERS']='1'
   if args.hud_goblins:env.update(ASX_TREASURE_GOBLINS="1",ASX_TEST_TREASURE_GOBLINS="1")
   if args.hud_trial==2:env.update(WOW_ROGUE_COOP_PASSWORD=password,ASX_TEST_COOP_EVENTS='1');env['WOW_ROGUE_COOP_HOST'if slot==0 else'WOW_ROGUE_COOP_JOIN']=str(port)if slot==0 else f'127.0.0.1:{port}'
   stream=(profile/'stdout.log').open('wb');streams.append(stream);children.append(subprocess.Popen([str(native/'AzerothSurvivors.exe')],cwd=native,env=env,stdout=stream,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW));time.sleep(1)
  deadline=time.monotonic()+args.seconds
  while time.monotonic()<deadline and any(c.poll()is None for c in children):
   for profile in profiles:
    text=(profile/'stdout.log').read_text(errors='replace')
    if any('ui_script: runtime error:'in line and 'ASXUI_HUD_LIVE 'not in line for line in text.splitlines()):raise RuntimeError('Unexpected Lua error; inspect '+str(profile/'stdout.log'))
   time.sleep(1)
 finally:
  for child in children:
   if child.poll()is None:child.terminate()
   try:child.wait(timeout=10)
   except subprocess.TimeoutExpired:child.kill();child.wait(timeout=5)
  for stream in streams:stream.close()
 results=[]
 for profile in profiles:
  text=(profile/'stdout.log').read_text(errors='replace');native_log=(profile/'azeroth-coop-native.log').read_text(errors='replace');samples=[]
  for line in text.splitlines():
   if 'ASXUI_HUD_LIVE 'in line:
    try:samples.append(json.loads(line.split('ASXUI_HUD_LIVE ',1)[1]))
    except ValueError:pass
  (profile/'samples.json').write_text(json.dumps(samples,indent=2))
  active=[s for s in samples if s.get('available')and s.get('hud',{}).get('enabled')]
  checks={'native_health_visible':any(s.get('frames',{}).get('vitalsShown')and s.get('frames',{}).get('health')and s.get('frames',{}).get('playerAlpha')==0 for s in active),'real_api_1_5':bool(active)and all(s.get('api',{}).get('minor',0)>=5 for s in active),'native_xp_preserved':bool(active)and all(abs(s['xp']['width']-437)<.1 and abs(s['xp']['height']-13)<.1 for s in active),'stats_opened':any(s.get('hud',{}).get('selected')=='stats'and s['hud'].get('progress')==1 for s in samples),'stats_closed':any(20<=s.get('age',0)<30 and s.get('hud',{}).get('progress')==0 for s in samples),'native_plate_defaults':any(s.get('plateLayout')=='overlap'and s.get('platesOn')==1 for s in samples),'affordable_glow_observed':any(s.get('hud',{}).get('affordable')for s in active),'toggle_restored':any(s.get('age',0)>40 and not s.get('hud',{}).get('enabled')for s in samples),'local_equipment_delivered':any(s.get('equipmentTexts')for s in samples),'asx_started':(profile/'ready.json').exists()}
  if args.hud_followups:
   checks['character_level_matches_native']=any(v.get('nativeLevel')and v.get('frames',{}).get('level')=='Lv '+str(v['nativeLevel'])for v in active)
   if args.hud_trial==2:
    checks['effective_economy_delivered']=any(v.get('economy')and all(isinstance(v['economy'].get(k),(int,float))for k in ('xp','gold'))for v in active)
   checks['native_owner_enchantment_delivered']=any('Enchantment:'in t.get('text','')for v in active for t in v.get('equipmentTexts',{}).values())
   checks['fortune_award_receipt_delivered']=any(v.get('fortune',{}).get('event',0)>0 for v in active)
   checks['item_tooltip_enchantment_rendered']=any(any('Enchantment:'in line for line in v.get('itemTooltipLines',{}).values())for v in active)
  checks['compact_popup_observed']=any(compact_popup(popup)for v in active for popup in v.get('popups',{}).values())
  if args.crystal_badges:
   checks['crystal_medallion_observed']=any(v.get('frames',{}).get('medallions',{}).get('crystal',{}).get('native')for v in active)
   checks['crystal_total_observed']=any(int(v.get('frames',{}).get('medallions',{}).get('crystal',{}).get('total')or 0)>0 for v in active)
   checks['crystal_tooltip_observed']=any(v.get('badgeTooltips',{}).get('ASXUICrystalMedallion')=='Crystal Resonance'for v in active)
  if args.hud_goblins and 'goblin native marker observation:'in native_log:checks['goblin_has_no_automatic_skull']=native_goblin_unmarked(native_log)
  if args.hud_goblins:checks['native_goblin_health_observed']=any(row.get('name')=='Venture Co. Treasurer'for v in active for row in v.get('frames',{}).get('encounters',{}).get('rows',{}).values())
  checks['guardian_objectives_observed']=any('Defeat 'in str(v.get('frames',{}).get('objectives',{}))for v in samples)
  checks['native_auras_docked']=any(v.get('frames',{}).get('auras',{}).get('helpful',0)>0 and v['frames']['auras'].get('aboveHealth')and v['frames']['auras'].get('durationsAbove')for v in active)
  checks['player_frame_restored']=any(v.get('age',0)>40 and not v.get('hud',{}).get('enabled')and v.get('frames',{}).get('playerAlpha')==1 for v in samples)
  checks['native_item_tooltip_observed']=any(v.get('tooltip')and v['tooltip']in [t.get('name')for t in v.get('equipmentTexts',{}).values()]for v in samples)
  if args.hud_trial==2:
   checks['ally_build_panel_observed']=any(v.get('frames',{}).get('allies',0)>0 for v in active)
   checks['ally_buffs_outside_observed']=any(v.get('frames',{}).get('allyBuffsOutside',0)>0 for v in active)
  observations={
   'goblin_marker_table':('observed'if 'goblin native marker observation:'in native_log else'unavailable_in_solo_host_guard'),
   'fortune_medallion':any(v.get('frames',{}).get('medallions',{}).get('fortune',{}).get('native')for v in active),
   'sayge_channel':any(v.get('channels',{}).get('ASXSaygeTrialProgress')for v in active),
   'resurrection_channel':any(v.get('channels',{}).get('ASXUIResurrectionProgress')for v in active),
   'marks_popup':any(p.get('name')=='ASXUINativeRewardToast'and p.get('title')=='Marks earned'for v in active for p in v.get('popups',{}).values()),
   'loot_popup':any(p.get('name')=='ASXUINativeRewardToast'and p.get('title')=='Loot received'for v in active for p in v.get('popups',{}).values())}
  if args.hud_trial==2:
   checks['fortune_medallion_observed']=observations['fortune_medallion']
   checks['sayge_channel_observed']=observations['sayge_channel']
  results.append({'checks':checks,'observations':observations,'samples':len(samples)})
 result={'passed':all(all(r['checks'].values())for r in results),'players':args.hud_trial,'clients':results,'native_sha256':SHA256,'dll_sha256':digest(dll),'official_installation_touched':False,'synthetic_provider':False,'ui_hashes':{str(f.relative_to(ROOT)):digest(f)for f in [ROOT/'native-ui/official-party.lua',*(ROOT/'addons/ASXUI').glob('*.lua')]}}
 (out/'result.json').write_text(json.dumps(result,indent=2));print(json.dumps(result),flush=True)
 if not result['passed']:raise SystemExit(1)
def parse(argv=None):
 parser=background.build_parser();parser.add_argument('--hud-trial',type=int,choices=(1,2),required=True);parser.add_argument('--hud-followups',action='store_true');parser.add_argument('--crystal-badges',action='store_true');parser.add_argument('--hud-goblins',action='store_true');args=parser.parse_args(argv)
 if args.hud_goblins and args.hud_trial!=1:parser.error('Goblin health visual fixture uses one client')
 if args.crystal_badges and args.hud_trial!=1:parser.error('Crystal badge fixture uses one client and no conflicting event-position fixture')
 if not 60<=args.seconds<=150:parser.error('HUD trial requires 60..150 seconds')
 if any(getattr(args,n)for n in ('resource_probe','extensions_trial','ritual_trial','supplies_trial','goblins_trial','gems_trial','enchantments_trial','reward_wallet_trial','immunity_aggro_trial','asx_install','extensions_normal','asx_package')):parser.error('HUD trial is a separate mode')
 return args
if __name__=='__main__':
 background.__file__=__file__;background.run_child=run;background.parse_args=parse;background.child_flags=lambda a:['--hud-trial',str(a.hud_trial)]+(['--crystal-badges']if a.crystal_badges else[])+(['--hud-goblins']if a.hud_goblins else[])+(['--hud-followups']if a.hud_followups else[]);background.main()
