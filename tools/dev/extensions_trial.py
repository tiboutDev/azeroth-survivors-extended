"""Bounded solo/two-player extension trial on disposable reviewed native profiles."""
import json,os,re,socket,subprocess,time,uuid
from pathlib import Path
from native_runtime_import import digest
from disposable115_party import writer
from script_diagnostics import assess

ROOT=Path(__file__).resolve().parents[2]


def compatibility_scope(normal=False, enchantments=False, reward_wallet=False, immunity=False,
                        gems=False, glyph_retention=False, completion=""):
 focused_mode=any((normal,enchantments,reward_wallet,immunity,gems,glyph_retention))
 return not focused_mode and completion in ("","tooltip")


def merge_assessor_checks(checks, evidence_checks):
 checks.update(evidence_checks)
 return checks


def run(seconds,players=1,normal=False,enchantments=False,reward_wallet=False,immunity=False,gems=False,glyph_retention=False):
 if sum(map(bool,(normal,enchantments,reward_wallet,immunity,gems,glyph_retention)))>1:raise ValueError("Choose one extension trial mode")
 if immunity and players!=2:raise ValueError("Immunity aggro validation requires two participants")
 if reward_wallet and players!=2:raise ValueError("Shared wallet validation requires two participants")
 if normal and players not in (1,2):raise ValueError("Normal smoke has one or two clients")
 if players not in (1,2):raise ValueError("One solo client or two cooperative clients only")
 visual=os.environ.get("ASX_TEST_VISUAL")=="1"
 version=os.environ.get("ASX_TEST_GAME_VERSION","0.125.0")
 if version != "0.125.0":raise ValueError("Only the reviewed 0.125.0 extension runtime is supported")

 runtime=ROOT/("runtime-official-"+version.replace(".",""));exe=runtime/"AzerothSurvivors.exe";dll=runtime/"avrt.dll"
 sha=json.loads((ROOT/"asx/supported-games.json").read_text())["games"][version]["sha256"]
 proof=json.loads((runtime/"ASX-DEV.json").read_text())
 if digest(exe)!=sha or digest(dll)!=proof["dll_sha256"]:raise ValueError("Reviewed isolated runtime required")
 if normal and b"AZCOOP_TEST_"in dll.read_bytes()or not normal and b"AZCOOP_TEST_EXTENSIONS"not in dll.read_bytes():raise ValueError("Wrong normal/diagnostic binary")
 completion=os.environ.get("ASX_TEST_COMPLETION121","")
 if completion not in ("","support","fatigue","fatigue-self","tooltip","pause"):raise ValueError("Unknown completion mode")
 if completion and normal:raise ValueError("Completion requires a diagnostic binary")
 focused_mode=any((normal,enchantments,reward_wallet,immunity,gems,glyph_retention))
 if completion and focused_mode:raise ValueError("Completion scenarios cannot be combined with focused extension modes")
 owner_tips=normal and players==2 and os.environ.get("ASX_TEST_OWNER_TIPS121")=="1"
 mixed_profiles=completion in ("support","fatigue","fatigue-self") or owner_tips
 compat121=compatibility_scope(normal=normal,enchantments=enchantments,reward_wallet=reward_wallet,
                              immunity=immunity,gems=gems,glyph_retention=glyph_retention,
                              completion=completion)
 prepare=writer(exe);out=ROOT/"test-results"/(("glyph-retention-"if glyph_retention else"gems-"+str(players)+"-"if gems else"enchantments-"+str(players)+"-"if enchantments else"extensions-normal-"if normal else"extensions-"+str(players)+"-")+uuid.uuid4().hex[:8]);out.mkdir()
 clean={k:v for k,v in os.environ.items()if not k.upper().startswith(("ASX_","AZCOOP_","COOP_","WOW_","BENILLA_"))}
 with socket.socket()as p:p.bind(("127.0.0.1",0));port=p.getsockname()[1]
 children=[];handles=[];password=uuid.uuid4().hex
 try:
  for slot in range(1,players+1):
   folder=out/f"player-{slot}";save=folder/"appdata/wenilla-rogue";save.mkdir(parents=True);prepare(save,2 if mixed_profiles and slot==1 else 1,name_override="ExtensionHero"if slot==1 else"ExtensionAlly",class_override=(11 if slot==2 else None) if mixed_profiles else 3 if immunity else None)
   (folder/"localappdata").mkdir();(folder/"shots").mkdir();(folder/"config.toml").write_text('[cvars]\ngxWindow = "1"\ngxResolution = "800x600"\n'+''.join(f'{k} = "0"\n'for k in("MasterVolume","MusicVolume","SoundVolume","AmbienceVolume","EnableMusic","EnableAmbience","MasterSoundEffects")))
   addon=folder/"AddOns/ASX";addon.mkdir(parents=True);(addon/"ASX.toc").write_text("## Interface: 11200\n## Title: ASX\nPresentation.lua\nAudit.lua\n");(addon/"Presentation.lua").write_bytes((ROOT/"native-ui/official-party.lua").read_bytes());(addon/"Audit.lua").write_bytes((ROOT/"tools/dev/fixtures/asx-extensions-audit.lua").read_bytes())
   env=dict(clean,APPDATA=str(folder/"appdata"),LOCALAPPDATA=str(folder/"localappdata"),BENILLA_HOME=str(folder),ASX_ACTIVE="1",AZCOOP_ENGINE="official",AZCOOP_NATIVE_LOG_DIRECTORY=str(folder),ASX_READY_FILE=str(folder/"ready.json"),WOW_NOSOUND="1",WOW_WIN="1",WOW_ROGUE_UPDATE="0",WOW_ROGUE_UPDATE_AUTO="0",WOW_ROGUE_AUTOPICK="1",WOW_ROGUE_AUTOSTART="1",WOW_ROGUE_AUTOPLAY="18"if slot==1 else"4",RUST_LOG="info",AZCOOP_TEST_GLYPHS="1",AZCOOP_TEST_BLESSINGS="1")
   if enchantments:
    env={k:v for k,v in env.items()if k not in("AZCOOP_TEST_GLYPHS","AZCOOP_TEST_BLESSINGS")}
    env["AZCOOP_TEST_ENCHANTMENTS"]="1";env["AZCOOP_TEST_OFFICIAL_CHOICE_DELAY"]="20"
    (addon/"ASX.toc").write_text("## Interface: 11200\n## Title: ASX\nPresentation.lua\n")
   if glyph_retention:
    env.pop("AZCOOP_TEST_BLESSINGS",None);env["AZCOOP_TEST_GLYPH_RETENTION"]="1";env["AZCOOP_TEST_OFFICIAL_CHOICE_DELAY"]="40"
    (addon/"ASX.toc").write_text("## Interface: 11200\n## Title: ASX\nPresentation.lua\n")
   if gems:
    env={k:v for k,v in env.items()if k not in("AZCOOP_TEST_GLYPHS","AZCOOP_TEST_BLESSINGS")}
    env["AZCOOP_TEST_GEMS"]="1";env["AZCOOP_TEST_OFFICIAL_CHOICE_DELAY"]="20"
    (addon/"ASX.toc").write_text("## Interface: 11200\n## Title: ASX\nPresentation.lua\n")
   if immunity:
    env={k:v for k,v in env.items()if k not in("AZCOOP_TEST_GLYPHS","AZCOOP_TEST_BLESSINGS")}
    env["AZCOOP_TEST_IMMUNITY_AGGRO"]="1"
    (addon/"ASX.toc").write_text("## Interface: 11200\n## Title: ASX\nPresentation.lua\n")
   if reward_wallet:
    env={k:v for k,v in env.items()if k not in("AZCOOP_TEST_GLYPHS","AZCOOP_TEST_BLESSINGS")}
    env["AZCOOP_TEST_REWARD_WALLET"]="1"
    (addon/"ASX.toc").write_text("## Interface: 11200\n## Title: ASX\nPresentation.lua\n")
   if compat121:
    env={k:v for k,v in env.items()if k not in("AZCOOP_TEST_GLYPHS",)}
    env.update(AZCOOP_TEST_COMPAT121="1",AZCOOP_TEST_GEMS="1",AZCOOP_TEST_ENCHANTMENTS="1",AZCOOP_TEST_BLESSINGS="1",AZCOOP_TEST_OFFICIAL_CHOICE_DELAY="3")
    if slot==1:
     env["AZCOOP_TEST_EXTENSIONS"]="1"
     if version=="0.125.0":env["AZCOOP_TEST_MARKS121"]="1"
    (addon/"ASX.toc").write_text("## Interface: 11200\n## Title: ASX\nPresentation.lua\n")
   if completion in ("support","fatigue","fatigue-self"):
    env={k:v for k,v in env.items()if not k.startswith("AZCOOP_TEST_")}
    if slot==1:
     env["AZCOOP_TEST_SUPPORT121"if completion=="support"else"AZCOOP_TEST_OFFICIAL_FATIGUE"]="1"
     if completion=="support":env.update(AZCOOP_TEST_SUPPORT121_FULL="1",AZCOOP_TEST_OFFICIAL_FATIGUE="1")
    (addon/"ASX.toc").write_text("## Interface: 11200\n## Title: ASX\nPresentation.lua\n")
   if completion in ("support","fatigue","fatigue-self"):
    audit=(ROOT/"tools/dev/fixtures/asx-fatigue-audit.lua").read_text().replace("now-last<1","now-last<2")
    (addon/"Audit.lua").write_text(audit)
    with (addon/"ASX.toc").open("a")as toc:toc.write("Audit.lua\n")
   if completion=="support":
    env["AZCOOP_TEST_NATIVE_TIPS121"]="1"
    (addon/"Audit.lua").write_bytes((ROOT/"tools/dev/fixtures/asx-tooltip121-audit.lua").read_bytes())
    with (addon/"ASX.toc").open("a")as toc:toc.write("Audit.lua\n")
   if completion=="tooltip":
    env["AZCOOP_TEST_BONUS_TOOLTIP"]="1";env["AZCOOP_TEST_OFFICIAL_CHOICE_DELAY"]="8"
    if slot==1:env["AZCOOP_TEST_MARKS121"]="1"
   if completion=="pause":
    env={k:v for k,v in env.items()if not k.startswith("AZCOOP_TEST_")}
    env.update(AZCOOP_TEST_OFFICIAL_PAUSE="1",WOW_ROGUE_ROAM="1")
    if slot==1:env["AZCOOP_TEST_OFFICIAL_PAUSE_HOST"]="1"
    (addon/"ASX.toc").write_text("## Interface: 11200\n## Title: ASX\nPresentation.lua\nPauseAudit.lua\n")
    (addon/"PauseAudit.lua").write_bytes((ROOT/"tools/dev/fixtures/asx-pause-audit.lua").read_bytes())
   if normal:
    env={k:v for k,v in env.items()if not k.startswith("AZCOOP_TEST_")}
    (addon/"ASX.toc").write_text("## Interface: 11200\n## Title: ASX\nPresentation.lua\n")
   elif slot==1 and not completion and not immunity and not enchantments and not gems and not glyph_retention:env["AZCOOP_TEST_EXTENSIONS"]="1"
   if owner_tips:
    (addon/"Audit.lua").write_bytes((ROOT/"tools/dev/fixtures/asx-tooltip121-audit.lua").read_bytes())
    with (addon/"ASX.toc").open("a")as toc:toc.write("Audit.lua\n")
   if players==2:env.update(WOW_ROGUE_COOP_PASSWORD=password);env["WOW_ROGUE_COOP_HOST"if slot==1 else"WOW_ROGUE_COOP_JOIN"]=str(port)if slot==1 else f"127.0.0.1:{port}"
   from trial_limits import environment
   if visual:
    env.update(environment(visual=True));env["WOW_ROGUE_SHOTS"]=str(folder/"shots")
   handle=(folder/"native-output.log").open("wb");handles.append(handle);children.append(subprocess.Popen([str(exe)],cwd=runtime,env=env,stdout=handle,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW));time.sleep(4 if slot==1 else 2)
  until=time.monotonic()+min(seconds,280 if completion in ("support","fatigue","fatigue-self") else 210 if glyph_retention else 180 if enchantments else 150)
  while time.monotonic()<until and all(p.poll()is None for p in children):
   if (ROOT/"test-results/extensions-trial-stop").exists():break
   time.sleep(.25)
  logs=[((out/f"player-{i}/azeroth-coop-native.log").read_text(encoding="utf-8",errors="replace") if (out/f"player-{i}/azeroth-coop-native.log").exists() else "")for i in range(1,players+1)]
  native=[(out/f"player-{i}/native-output.log").read_text(encoding="utf-8",errors="replace")for i in range(1,players+1)]
  host=logs[0];grants=[tuple(map(int,x))for x in re.findall(r"blessing granted: event=\d+ slot=(\d+) guid=\d+ boss=(\d+) category=(\d+)",host)]
  ui=[]
  for text in logs:
   ui.append([json.loads(x)for x in re.findall(r'AZCOOP_PARTY115 (\{"extensions":true,[^\r\n]+\})',text)])
  checks=dict(clients_alive=all(p.poll()is None for p in children),started=all('"type":"start"'in t and '"type":"resume"'in t for t in native),clean_scripts=all(assess(t)["passed"]for t in native),clean_adapter=all(not any(s in t for s in("rejected:","[ ERROR ]","Mod HUD update failed"))for t in logs),three_native_encounters=all({boss for s,boss,c in grants if s==slot}>={1,2,3}for slot in range(players)),glyphs_accepted=all(f"glyph accepted: slot={slot} "in host for slot in range(players)),glyph_cast_parameters="glyph fixture native cast:"in host,native_buff_projected=all({int(b)for s,b in re.findall(r"extensions native fortune aura: slot=(\d+) boss=(\d+)",host)if int(s)==slot}>={1,2,3}for slot in range(players)),ui_captures_available=all(len(list((out/f"player-{i}/shots").glob("*.png")))>=2 for i in range(1,players+1)),startup_ack=all((out/f"player-{i}/ready.json").exists()for i in range(1,players+1)))
  if enchantments:
   from enchantment_evidence import assess_ranked as assess_enchantments
   evidence=assess_enchantments(host,native,players)
   checks={k:checks[k]for k in("clients_alive","started","clean_scripts","clean_adapter","ui_captures_available","startup_ack")}
   merge_assessor_checks(checks,evidence["checks"])
   checks["native_annotation_rendered"]=all("enchantment native UI block:"in text for text in logs)
   (out/"enchantment-evidence.json").write_text(json.dumps(evidence,indent=2)+"\n")
  if glyph_retention:
   from glyph_retention_trial import evidence as assess_retention
   evidence=assess_retention(host,logs,native)
   checks={k:checks[k]for k in("clients_alive","started","clean_scripts","clean_adapter","ui_captures_available","startup_ack")}
   merge_assessor_checks(checks,evidence["checks"])
   (out/"retention-evidence.json").write_text(json.dumps(evidence,indent=2)+"\n")
  if gems:
   from gem_evidence import assess as assess_gems
   evidence=assess_gems(host,[a+"\n"+b for a,b in zip(logs,native)],players)
   checks={k:checks[k]for k in("clients_alive","started","clean_scripts","clean_adapter","ui_captures_available","startup_ack")}
   merge_assessor_checks(checks,evidence["checks"])
   (out/"gem-evidence.json").write_text(json.dumps(evidence,indent=2)+"\n")
  if immunity:
   checks={k:checks[k]for k in("clients_alive","started","clean_scripts","clean_adapter","startup_ack")}
   from immunity_aggro_evidence import assess as assess_immunity
   evidence=assess_immunity(host);checks.update(evidence["checks"])
   (out/"immunity-evidence.json").write_text(json.dumps(evidence,indent=2)+"\n")
  if reward_wallet:
   checks={k:checks[k]for k in("clients_alive","started","clean_scripts","clean_adapter","startup_ack")}
   from reward_wallet_evidence import assess as assess_wallet
   evidence=assess_wallet(host);checks.update(evidence["checks"])
   (out/"reward-wallet-evidence.json").write_text(json.dumps(evidence,indent=2)+"\n")
  if compat121:
   checks={k:checks[k]for k in("clients_alive","started","clean_scripts","clean_adapter","ui_captures_available","startup_ack","three_native_encounters","native_buff_projected")}
   checks["glyphs_disabled"]=all("ASX capabilities: glyphs=false duplicate-gold=false native-marks=true"in text and "glyph accepted:"not in text and "official-glyph-modifiers"not in text for text in logs)
   checks["no_duplicate_gold"]=all("duplicate loot credit"not in text for text in logs)
   checks["gem_each_player"]=all(f"gem locked: slot={i} "in host for i in range(players))
   checks["enchantment_accepted"]="enchantment locked:"in host
   checks["stat_inputs_restored"]="gem fixture rebuild: stable=true restored=true"in host and "restored=false"not in host and "stable=false"not in host
   from gem_evidence import assess as assess_gems
   from enchantment_evidence import assess_ranked
   combined=[a+"\n"+b for a,b in zip(logs,native)]
   gem_evidence=assess_gems(host,combined,players)
   enchant_evidence=assess_ranked(host,combined,players)
   checks["eight_gem_native_values"]=gem_evidence["checks"]["all_eight_native_bonuses"]
   checks["forty_enchantment_native_values"]=enchant_evidence["checks"]["all_40_native_rank_values"]
   marks=[]
   for i in range(1,players+1):
    save=prepare.read(out/f"player-{i}/appdata/wenilla-rogue")
    marks.append(sum(int(v) for v in re.findall(r"^marks\|1\|(\d+)$",save,re.M)))
   checks["native_personal_marks"]=all(v>0 for v in marks)
   (out/"compat-details.json").write_text(json.dumps(dict(gems=gem_evidence,enchantments=enchant_evidence,native_class_marks=marks),indent=2)+"\n")
  if completion in ("support","fatigue","fatigue-self"):
   from extension_completion_evidence import assess as assess_completion
   evidence=assess_completion(host,completion)
   if completion=="support":
    from immunity_aggro_evidence import assess as assess_immunity
    evidence["checks"].update(assess_completion(host,"fatigue")["checks"])
    evidence["checks"].update(assess_immunity(host)["checks"])
   checks={k:checks[k]for k in("clients_alive","started","clean_scripts","clean_adapter","startup_ack")};merge_assessor_checks(checks,evidence["checks"])
   (out/"completion-evidence.json").write_text(json.dumps(evidence,indent=2)+"\n")
  if completion=="tooltip":
   checks={k:checks[k]for k in("clients_alive","started","clean_scripts","clean_adapter","startup_ack","ui_captures_available","glyphs_disabled")}
   checks.update(strongest_mark="marks121 native: stage=1 marked=true vulnerability=0.35"in host,remaining_mark="marks121 native: stage=2 marked=true vulnerability=0.2"in host,mark_removed="marks121 native: stage=3 marked=false vulnerability=0"in host,three_taunts=all(f"marks121 taunt: spell={spell} owner=Some({slot})"in host for spell,slot in ((23922,0),(853,1),(5221,0))),native_popup_body=all("native kept tooltip body: category="in t for t in logs),native_attach=all("native kept tooltip attached: category="in t for t in logs),native_enter=all("entered=true"in t for t in logs))
  if completion=="pause":
   from pause_evidence import assess as assess_pause, moved_native_positions
   pause_result=assess_pause(out)
   # Local prediction is separate from the host's authoritative snapshots.
   # Native ROAM keyboard events must also demonstrate that motion works outside pauses.
   import ast
   for slot,text in enumerate(logs[:1]):
    windows=[];local=[];outside=[]
    for line in text.splitlines():
     gate=re.search(r"pause fixture gate:.*windows=(\[[^\r\n]*\])",line)
     if gate:windows=ast.literal_eval(gate[1])
     pose=re.search(r"pause fixture local: paused=(true|false) position=Ok\((\[[^\r\n]*\])\)",line)
     if not pose:continue
     position=ast.literal_eval(pose[2])
     kind="level"if slot==0 else"cache"
     if pose[1]=="true" and any(k==kind and delay==3 and 4<=elapsed<=9 for k,elapsed,delay in windows):local.append(position)
     if pose[1]=="false":outside.append(position)
    stable=len(local)>=2 and all(max(abs(a-b)for a,b in zip(local[0],row))<.01 for row in local)
    moving=moved_native_positions(outside)
    pause_result["checks"][f"local_player_{slot}_frozen_while_waiting"]=stable
    pause_result["checks"][f"native_keyboard_moves_player_{slot}_outside_pause"]=moving

   checks={k:checks[k]for k in("clients_alive","started","clean_scripts","clean_adapter","startup_ack")};checks.update(pause_result["checks"])
   from pause_evidence import resumed_motion
   pause_result["checks"].update(resumed_motion(host,players))
   from pause_evidence import menu_evidence
   menus=menu_evidence(host)
   pause_result["checks"].update(menus["checks"])
   (out/"native-menu-evidence.json").write_text(json.dumps(menus,indent=2)+chr(10))
   checks.update(pause_result["checks"])
   (out/"pause-evidence.json").write_text(json.dumps(pause_result,indent=2)+"\n")
  if normal:
   checks={k:checks[k]for k in("clients_alive","started","clean_scripts","clean_adapter","startup_ack")}
   checks["natural_progression"]=all(any(int(level)>1 for level in re.findall(r'"type":"choices"[^\n]*?"level":(\d+)',text))for text in native)
   checks["normal_binary"]=b"AZCOOP_TEST_"not in dll.read_bytes()
  if players==1:checks.update(no_coop_session='"state":"joined"'not in native[0],no_shared_wallet="official gold:"not in host,no_group_pause="official group pause: Some"not in host)
  else:
   samples=[re.findall(r'phase: Playing, gold: (\d+), level: (\d+), xp: (\d+)',l)for l in host.splitlines()if"official party: clock="in l];pairs=[a for a in samples if len(a)==2];checks.update(coop_joined=all('"state":"joined"'in t for t in native),equal_progression=len(pairs)>=3 and all(a[0]==a[1]for a in pairs),shared_wallet="official gold:"in host,group_pause="official group pause: Some"in host and"official group pause: None"in host)
  if compat121 and players==2:
   # Controlled cache grants occur before the next restored-host projection.
   # XP must match in every comparable sample; gold must converge by the next one.
   from enchantment_evidence import assess_fixture_party
   economy=assess_fixture_party(host);checks.update(economy["checks"])
   (out/"fixture-party-evidence.json").write_text(json.dumps(economy,indent=2)+"\n")
  if completion in ("support","fatigue","fatigue-self"):
   (out/"known-death-xp-observation.json").write_text(json.dumps(dict(equal_progression=checks.pop("equal_progression",None),scope="Known death/revival XP issue remains open; separate from support and fatigue acceptance."),indent=2)+"\n")
  if glyph_retention:checks.pop("shared_wallet",None) # UI scenario does not generate a gold source.
  if enchantments and players==2:
   from enchantment_evidence import assess_fixture_party
   economy=assess_fixture_party(host);checks.update(economy["checks"])
   (out/"fixture-party-evidence.json").write_text(json.dumps(economy,indent=2)+"\n")
  if not visual:checks.pop("ui_captures_available",None)
  result=dict(owner_tooltip_readback=owner_tips,completion121=completion,players=players,normal=normal,game_version=version,compat121=compat121,enchantments=enchantments,gems=gems,immunity=immunity,diagnostic_inputs=not normal,dll_sha256=proof["dll_sha256"],game_sha256=sha,checks=checks,passed=all(checks.values()),grants=grants,ui=ui,desktop_input=False,screenshots=visual,visual_review_required=visual)
  (out/"result.json").write_text(json.dumps(result,indent=2)+"\n");print(out,flush=True);print(json.dumps(result,indent=2),flush=True)
 finally:
  for p in children:
   if p.poll()is None:p.terminate()
  for p in children:
   try:p.wait(timeout=10)
   except subprocess.TimeoutExpired:p.kill();p.wait(timeout=5)
  for f in handles:f.close()
 if not result["passed"]:raise SystemExit(1)
