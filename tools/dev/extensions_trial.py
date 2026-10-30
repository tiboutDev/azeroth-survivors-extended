"""Bounded solo/two-player extension trial on disposable reviewed native profiles."""
import json,os,re,socket,subprocess,time,uuid
from pathlib import Path
from native_runtime_import import digest
from client_registry import VERSION
from disposable115_party import writer
from script_diagnostics import assess

ROOT=Path(__file__).resolve().parents[2]


def compatibility_scope(normal=False, enchantments=False, reward_wallet=False, immunity=False,
                        gems=False, glyph_retention=False, completion="", goblins=False, supplies=False, ritual=False):
 focused_mode=any((normal,enchantments,reward_wallet,immunity,gems,glyph_retention,goblins,supplies,ritual))
 return not focused_mode and completion in ("","tooltip")


def merge_assessor_checks(checks, evidence_checks):
 for name,value in evidence_checks.items():checks[name]=checks.get(name,True) and value
 return checks


def consumable_stat_evidence(text):
 """Compare observed native output against the unmodified rebuild in one tick."""
 import ast,math
 observed=set();full=False
 for row in re.finditer(r"supplies stacked stats: buffs=(\[[^\n]*?\]) base=Some\((\[[^\n]*?\])\) actual=(\[[^\n]*?\]) stable=(true|false)",text):
  buffs=ast.literal_eval(row[1]);base=ast.literal_eval(row[2]);actual=ast.literal_eval(row[3])
  if row[4]!="true" or len(base)!=8 or len(actual)!=8:continue
  full=full or len(buffs)==6
  for family,_,value in buffs:
   index={0:1,1:3,2:5,3:2,4:4,5:7}[family]
   expected=base[index]/(1+value) if family==0 else base[index]+value
   matches=actual[index]>base[index] if family==3 else math.isclose(actual[index],expected,rel_tol=1e-5,abs_tol=1e-5)
   if matches:observed.add(family)
 return dict(native_bonus_values=len(observed)==6 and full)


def run(seconds,players=1,normal=False,enchantments=False,reward_wallet=False,immunity=False,gems=False,glyph_retention=False,goblins=False,supplies=False,ritual=False):
 all_events=os.environ.get("ASX_TEST_ALL_EVENTS")=="1"
 diagnostic_probe=os.environ.get("ASX_TEST_DIAGNOSTIC_PROBE")=="1"
 scenario=os.environ.get("ASX_TEST_SCENARIO")
 readiness=None
 if scenario:
  from scenario_suites import load_suites,Readiness
  spec=load_suites()["suites"][scenario]
  if spec["players"]!=players or(spec["kind"]=="normal")!=normal:raise ValueError("Scenario/artifact mode mismatch")
  readiness=Readiness(spec)
 if diagnostic_probe and(normal or os.environ.get("ASX_TEST_LUA_DIAGNOSTICS")!="1"):raise ValueError("Diagnostic probe requires opt-in diagnostic Lua")
 if all_events:
  if normal or any((enchantments,reward_wallet,immunity,gems,glyph_retention)):raise ValueError("Combined events require the diagnostic default mode")
  goblins=True
  if players==1:supplies=ritual=True
  else:os.environ["ASX_TEST_COOP_EVENTS"]="1"
 if not all_events and sum(map(bool,(normal,enchantments,reward_wallet,immunity,gems,glyph_retention,goblins,supplies,ritual)))>1:raise ValueError("Choose one extension trial mode")
 if ritual and players!=1:raise ValueError("Ritual prototype is solo only")
 if supplies and players!=1:raise ValueError("Supplies prototype is solo only")
 if immunity and players!=2:raise ValueError("Immunity aggro validation requires two participants")
 if reward_wallet and players!=2:raise ValueError("Shared wallet validation requires two participants")
 if normal and players not in (1,2):raise ValueError("Normal smoke has one or two clients")
 if players not in (1,2):raise ValueError("One solo client or two cooperative clients only")
 coop_events=os.environ.get("ASX_TEST_COOP_EVENTS")=="1"
 if coop_events and (players!=2 or normal):raise ValueError("Cooperative event fixture requires two diagnostic clients")
 visual=os.environ.get("ASX_TEST_VISUAL")=="1"
 polish=supplies and os.environ.get("ASX_TEST_EVENT_POLISH")=="1"
 persistence=ritual and os.environ.get("ASX_TEST_RITUAL_PERSISTENCE")=="1"
 ritual_ui=ritual and os.environ.get("ASX_TEST_RITUAL_UI")=="1"
 version=os.environ.get("ASX_TEST_GAME_VERSION",VERSION)
 if version != VERSION:raise ValueError("Only the selected reviewed extension runtime is supported")

 runtime=ROOT/("runtime-official-"+version.replace(".",""));exe=runtime/"AzerothSurvivors.exe";dll=runtime/"avrt.dll"
 sha=json.loads((ROOT/"asx/supported-games.json").read_text())["games"][version]["sha256"]
 proof=json.loads((runtime/"ASX-DEV.json").read_text())
 if digest(exe)!=sha or digest(dll)!=proof["dll_sha256"]:raise ValueError("Reviewed isolated runtime required")
 if normal and b"AZCOOP_TEST_"in dll.read_bytes()or not normal and b"AZCOOP_TEST_EXTENSIONS"not in dll.read_bytes():raise ValueError("Wrong normal/diagnostic binary")
 completion=os.environ.get("ASX_TEST_COMPLETION121","")
 if completion not in ("","support","fatigue","fatigue-self","tooltip","pause"):raise ValueError("Unknown completion mode")
 support_only=completion=="support" and os.environ.get("ASX_TEST_SUPPORT_ONLY")=="1"
 shield_only=completion in("support","pause") and os.environ.get("ASX_TEST_SUPPORT_SHIELD_ONLY")=="1"
 if completion and normal:raise ValueError("Completion requires a diagnostic binary")
 focused_mode=any((normal,enchantments,reward_wallet,immunity,gems,glyph_retention,goblins,supplies,ritual))
 if completion and focused_mode:raise ValueError("Completion scenarios cannot be combined with focused extension modes")
 owner_tips=normal and players==2 and os.environ.get("ASX_TEST_OWNER_TIPS121")=="1"
 mixed_profiles=completion in ("support","fatigue","fatigue-self") or owner_tips or shield_only
 compat121=compatibility_scope(normal=normal,enchantments=enchantments,reward_wallet=reward_wallet,
                              immunity=immunity,gems=gems,glyph_retention=glyph_retention,
                              completion=completion,goblins=goblins,supplies=supplies,ritual=ritual)
 if coop_events:compat121=False
 prepare=writer(exe);out=ROOT/"test-results"/(("glyph-retention-"if glyph_retention else"gems-"+str(players)+"-"if gems else"enchantments-"+str(players)+"-"if enchantments else"extensions-normal-"if normal else"extensions-"+str(players)+"-")+uuid.uuid4().hex[:8]);out.mkdir()
 clean={k:v for k,v in os.environ.items()if not k.upper().startswith(("ASX_","AZCOOP_","COOP_","WOW_","BENILLA_"))}
 with socket.socket()as p:p.bind(("127.0.0.1",0));port=p.getsockname()[1]
 children=[];handles=[];password=uuid.uuid4().hex;goblin_mark_baselines=[]
 try:
  for slot in range(1,players+1):
   folder=out/f"player-{slot}";save=folder/"appdata/wenilla-rogue";save.mkdir(parents=True);prepare(save,2 if mixed_profiles and slot==1 else 1,name_override="ExtensionHero"if slot==1 else"ExtensionAlly",class_override=(11 if slot==2 else None) if mixed_profiles else 3 if immunity else None)
   if goblins:goblin_mark_baselines.append(sum(int(v)for v in re.findall(r"^marks\|\d+\|(\d+)$",prepare.read(save),re.M)))
   (folder/"localappdata").mkdir();(folder/"shots").mkdir();(folder/"config.toml").write_text('[cvars]\ngxWindow = "1"\ngxResolution = "800x600"\n'+''.join(f'{k} = "0"\n'for k in("MasterVolume","MusicVolume","SoundVolume","AmbienceVolume","EnableMusic","EnableAmbience","MasterSoundEffects")))
   if os.environ.get("ASX_TEST_WORLD_UI")=="1":
    with (folder/"config.toml").open("a")as config:config.write('nameplateShowEnemies = "1"\n')
   addon=folder/"AddOns/ASX";addon.mkdir(parents=True);(addon/"ASX.toc").write_text("## Interface: 11200\n## Title: ASX\nPresentation.lua\nAudit.lua\n");(addon/"Presentation.lua").write_bytes((ROOT/"native-ui/official-party.lua").read_bytes());(addon/"Audit.lua").write_bytes((ROOT/"tools/dev/fixtures/asx-extensions-audit.lua").read_bytes())
   env=dict(clean,APPDATA=str(folder/"appdata"),LOCALAPPDATA=str(folder/"localappdata"),BENILLA_HOME=str(folder),ASX_ACTIVE="1",AZCOOP_ENGINE="official",AZCOOP_NATIVE_LOG_DIRECTORY=str(folder),ASX_READY_FILE=str(folder/"ready.json"),WOW_NOSOUND="1",WOW_WIN="1",WOW_ROGUE_UPDATE="0",WOW_ROGUE_UPDATE_AUTO="0",WOW_ROGUE_AUTOPICK="1",WOW_ROGUE_AUTOSTART="1",WOW_ROGUE_AUTOPLAY="18"if slot==1 else"4",RUST_LOG="info",AZCOOP_TEST_GLYPHS="1",AZCOOP_TEST_BLESSINGS="1")
   if not normal:env.update(ASX_LOST_SUPPLIES="0",ASX_DARKMOON_TRIALS="0")
   if supplies:
    env={k:v for k,v in env.items()if not k.startswith("AZCOOP_TEST_")}
    env.update(ASX_LOST_SUPPLIES="1",ASX_TEST_LOST_SUPPLIES="1",ASX_TREASURE_GOBLINS="0",ASX_TEST_EVENT_POLISH="1"if polish else"0",ASX_DARKMOON_TRIALS="1"if polish else"0",ASX_TEST_DARKMOON_TRIAL="1"if polish else"0")
    (addon/"SupplyAudit.lua").write_bytes((ROOT/"tools/dev/fixtures/asx-supplies-audit.lua").read_bytes())
    (addon/"ASX.toc").write_text("## Interface: 11200\n## Title: ASX\nPresentation.lua\nSupplyAudit.lua\n")
   if supplies and visual and os.environ.get("ASX_TEST_VISUAL_HOLD_DISCOVERY")=="1":
    # Disposable visual sampling only: make the short toast observable with sparse
    # screenshots. Never alter source UI, consumable timers or reward authority.
    presentation=addon/"Presentation.lua";text=presentation.read_text();needle="supplyUntil=GetTime()+3"
    if needle not in text:raise ValueError("Discovery visual fixture boundary changed")
    presentation.write_text(text.replace(needle,"supplyUntil=GetTime()+12"))
   if ritual:
    env={k:v for k,v in env.items()if not k.startswith("AZCOOP_TEST_")}
    env.update(ASX_DARKMOON_TRIALS="1",ASX_TEST_DARKMOON_TRIAL="1",ASX_TREASURE_GOBLINS="0",ASX_TEST_RITUAL_PERSISTENCE="1"if persistence else"0",ASX_TEST_RITUAL_UI="1"if ritual_ui else"0")
    (addon/"ASX.toc").write_text("## Interface: 11200\n## Title: ASX\nPresentation.lua\n")
   if ritual_ui:
    (addon/"RitualAudit.lua").write_bytes((ROOT/"tools/dev/fixtures/asx-ritual-ui-audit.lua").read_bytes())
    with (addon/"ASX.toc").open("a")as toc:toc.write("RitualAudit.lua\n")
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
     env["AZCOOP_TEST_MARKS121"]="1"
    (addon/"ASX.toc").write_text("## Interface: 11200\n## Title: ASX\nPresentation.lua\n")
   if completion in ("support","fatigue","fatigue-self"):
    env={k:v for k,v in env.items()if not k.startswith("AZCOOP_TEST_")}
    if slot==1:
     env["AZCOOP_TEST_SUPPORT121"if completion=="support"else"AZCOOP_TEST_OFFICIAL_FATIGUE"]="1"
     if shield_only:env["ASX_TEST_SUPPORT_SHIELD_ONLY"]="1"
     if completion=="support" and not support_only:env.update(AZCOOP_TEST_SUPPORT121_FULL="1",AZCOOP_TEST_OFFICIAL_FATIGUE="1")
    (addon/"ASX.toc").write_text("## Interface: 11200\n## Title: ASX\nPresentation.lua\n")
   if os.environ.get("ASX_TEST_COVERAGE")=="1":env["ASX_TEST_COVERAGE"]="1"
   if os.environ.get("ASX_TEST_WORLD_UI")=="1":env["ASX_TEST_UI"]="1"
   if os.environ.get("ASX_TEST_UI_REMAINING")=="1":env["ASX_TEST_UI_REMAINING"]="1"
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
    if shield_only and slot==1:env.update(AZCOOP_TEST_SUPPORT121="1",ASX_TEST_SUPPORT_SHIELD_ONLY="1")
    if slot==1:env["AZCOOP_TEST_OFFICIAL_PAUSE_HOST"]="1"
    (addon/"ASX.toc").write_text("## Interface: 11200\n## Title: ASX\nPresentation.lua\nPauseAudit.lua\n")
    (addon/"PauseAudit.lua").write_bytes((ROOT/"tools/dev/fixtures/asx-pause-audit.lua").read_bytes())
   if normal:
    env={k:v for k,v in env.items()if not k.startswith("AZCOOP_TEST_")}
    (addon/"ASX.toc").write_text("## Interface: 11200\n## Title: ASX\nPresentation.lua\n")
   elif slot==1 and not completion and not immunity and not enchantments and not gems and not glyph_retention and not supplies and not ritual:env["AZCOOP_TEST_EXTENSIONS"]="1"
   if owner_tips:
    (addon/"Audit.lua").write_bytes((ROOT/"tools/dev/fixtures/asx-tooltip121-audit.lua").read_bytes())
    with (addon/"ASX.toc").open("a")as toc:toc.write("Audit.lua\n")
   if goblins:
    env={k:v for k,v in env.items()if not k.startswith("AZCOOP_TEST_")}
    env.update(ASX_TEST_TREASURE_GOBLINS="1",ASX_TREASURE_GOBLINS="0"if os.environ.get("ASX_TEST_GOBLINS_OFF")=="1"else"1")
    (addon/"ASX.toc").write_text("## Interface: 11200\n## Title: ASX\nPresentation.lua\n")
   if coop_events:
    env={k:v for k,v in env.items()if not k.startswith("AZCOOP_TEST_")}
    env.update(ASX_LOST_SUPPLIES="1",ASX_DARKMOON_TRIALS="1",ASX_TREASURE_GOBLINS="1"if all_events else"0",ASX_TEST_TREASURE_GOBLINS="1"if all_events else"0",ASX_TEST_COOP_EVENTS="1",ASX_TEST_COOP_EVENTS_VISUAL="1"if os.environ.get("ASX_TEST_COOP_EVENTS_VISUAL")=="1"else"0")
    (addon/"ASX.toc").write_text("## Interface: 11200\n## Title: ASX\nPresentation.lua\n")
    if visual:
     (addon/"CoopEventsAudit.lua").write_bytes((ROOT/"tools/dev/fixtures/asx-coop-events-audit.lua").read_bytes())
     with (addon/"ASX.toc").open("a")as toc:toc.write("CoopEventsAudit.lua\n")
   if diagnostic_probe:
    env={k:v for k,v in env.items()if not k.startswith("AZCOOP_TEST_")}
   if os.environ.get("ASX_TEST_OBSERVED_STEPS")=="1":env["ASX_TEST_OBSERVED_STEPS"]="1"
   if os.environ.get("ASX_TEST_LUA_DIAGNOSTICS")=="1":
    if normal:raise ValueError("Lua diagnostics require a diagnostic trial")
    env.update(ASX_LUA_DIAGNOSTICS="1",ASX_DIAGNOSTIC_SESSION=out.name,ASX_DIAGNOSTIC_SLOT=str(slot-1))
    (addon/"DiagnosticConfig.lua").write_text("ASXDiagnosticConfig={enabled=true,session="+json.dumps(out.name)+",slot="+str(slot-1)+",duration="+str(min(seconds,300))+",max_records=256}\n")
    (addon/"Observer.lua").write_bytes((ROOT/"tools/dev/diagnostics/observer.lua").read_bytes())
    with (addon/"ASX.toc").open("a")as toc:toc.write("DiagnosticConfig.lua\nObserver.lua\n")
   if players==2:env.update(WOW_ROGUE_COOP_PASSWORD=password);env["WOW_ROGUE_COOP_HOST"if slot==1 else"WOW_ROGUE_COOP_JOIN"]=str(port)if slot==1 else f"127.0.0.1:{port}"
   from trial_limits import environment
   if visual:
    env.update(environment(visual=True));
    if os.environ.get("ASX_TEST_COVERAGE")=="1":env["WOW_ROGUE_SHOTS_EVERY"]="45"
    if os.environ.get("ASX_TEST_WORLD_UI")=="1":env["WOW_ROGUE_SHOTS_EVERY"]="12"
    env["WOW_ROGUE_SHOTS"]=str(folder/"shots")
   handle=(folder/"native-output.log").open("wb");handles.append(handle);children.append(subprocess.Popen([str(exe)],cwd=runtime,env=env,stdout=handle,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW));time.sleep(4 if slot==1 else 2)
  def evaluate():
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
    if shield_only:evidence=assess_completion(host,"shield")
    if completion=="support" and not support_only:
     from immunity_aggro_evidence import assess as assess_immunity
     evidence["checks"].update(assess_completion(host,"fatigue")["checks"])
     evidence["checks"].update(assess_immunity(host)["checks"])
    checks={k:checks[k]for k in("clients_alive","started","clean_scripts","clean_adapter","startup_ack")};merge_assessor_checks(checks,evidence["checks"])
    if os.environ.get("ASX_TEST_WORLD_UI")=="1":
     checks={k:checks[k]for k in("clients_alive","started","clean_scripts","clean_adapter","startup_ack","direct_healing_receipts_both_ways")}
     checks.update(native_plate_layout=all('Native plate layout:'in t for t in logs),native_heal_draw_direct=all('native heal text: periodic=false'in t and 'result=Ok(true)'in t for t in logs),native_heal_draw_periodic=all('native heal text: periodic=true'in t for t in logs))

    if os.environ.get("ASX_TEST_UI_REMAINING")=="1":
     checks={k:checks[k]for k in("clients_alive","started","clean_scripts","clean_adapter","startup_ack")}
     from remaining_ui_evidence import assess as remaining_assess
     remaining=remaining_assess(host,native,logs);checks.update(remaining['checks'])
     checks['screenshots_both']=all(len(list((out/f'player-{i}/shots').glob('*.png')))>=3 for i in (1,2))
     (out/'remaining-ui-evidence.json').write_text(json.dumps(remaining,indent=2)+'\n')
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
    if shield_only:
     from extension_completion_evidence import assess as assess_completion
     merge_assessor_checks(checks,assess_completion(host,"shield")["checks"])
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
   if all_events:
    checks={k:checks[k]for k in("clients_alive","started","clean_scripts","startup_ack","no_coop_session","no_shared_wallet","no_group_pause","coop_joined")if k in checks}
    # Rejected taunts are expected goblin policy decisions, not adapter failures.
    checks["clean_adapter"]=all(not any(error in line for error in("rejected:","[ ERROR ]","Mod HUD update failed"))for text in logs for line in text.splitlines()if"treasure goblin taunt rejected:"not in line)
   if coop_events:
    claims=re.findall(r"coop supplies discovered: slot=(\d+) owner=(\d+) event=(\d+) reward=(.*?) gold=(\d+)",host)
    checks=checks if all_events else {k:checks[k]for k in("clients_alive","started","clean_scripts","startup_ack")}
    checks.update(two_personal_claims={int(c[0])for c in claims if c[2]=="1"}=={0,1},different_rewards=len({c[3]for c in claims if c[2]=="1"})==2,ritual_waits_for_group="ritual=Some((1, 0.0, false))"in host,no_boss_fortune="blessing granted:"not in host,unique_claims=len(claims)==len({(c[0],c[1],c[2])for c in claims}),native_encounter="coop events fixture: native boss defeated"in host,ritual_reward="coop ritual completed: stage=1 recipients=2"in host,material_sales=sum(int(c[4])>0 for c in claims)>=2,no_event_failure="Cooperative events stopped:"not in host)
    if seconds<100 and os.environ.get("ASX_TEST_COOP_EVENTS_VISUAL")=="1":
     checks={k:v for k,v in checks.items()if k in("clients_alive","started","clean_scripts","startup_ack","two_personal_claims","different_rewards","unique_claims","no_event_failure")}
    (out/"coop-events-evidence.json").write_text(json.dumps(dict(checks=checks,claims=claims,passed=all(checks.values()),scope="Host authority and native replication; sparse UI images reviewed separately"),indent=2)+"\n")
   if ritual:
    checks=checks if all_events else {k:checks[k]for k in("clients_alive","started","clean_scripts","startup_ack")}
    rewards=re.findall(r"ritual completed: stage=(\d+) category=(\d+) amount=(\d+)",host)
    if persistence:
     import ast
     spawned=re.findall(r"ritual spawned: guid=(\d+) position=(\[[^\n]*?\]) clock=",host)
     kept=re.findall(r"ritual encounter: count=(\d+) retained=(\d+) stage=(\d+) position=(\[[^\n]*?\])",host)
     observed=re.findall(r"ritual persistence observed: id=(\d+) stage=(\d+) position=(\[[^\n]*?\]) anchor=Some\((\d+)\) older_than_60=true",host)
     identity=bool(len(spawned)==1 and kept and observed and all(int(stage)>=2 and id=="1" and anchor==spawned[0][0] and ast.literal_eval(point)==ast.literal_eval(spawned[0][1]) for id,stage,point,anchor in observed))
     checks.update(two_native_encounters=host.count("ritual fixture native boss defeated")==2,one_ritual=len(spawned)==1,
      stable_scene_after_boss=identity and all(count==stage and id=="1" and ast.literal_eval(point)==ast.literal_eval(spawned[0][1])for count,id,stage,point in kept),
      persists_beyond_60=bool(observed),no_premature_reward=not rewards,no_premature_removal="ritual scene removed:"not in host,
      native_visual_sayge="sayge_display=491 enemy_rows_unchanged=true"in host,
      no_automatic_boss_fortunes="blessing granted:"not in host,no_event_failure="Sayge's Trial stopped:"not in host)
    elif ritual_ui:
     checks.update(one_reward=len(rewards)==1 and rewards[0][0]=="1",first_cleanup=host.count("ritual scene removed:")==1,no_event_failure="Sayge's Trial stopped:"not in host)
     if seconds<150:
      # Focused visual repair: finish one complete real ritual/reward before the
      # second scene. The longer scenario separately verifies outside visibility.
      checks.update(native_encounter=host.count("ritual fixture native boss defeated")>=1,one_scene=host.count("ritual spawned:")==1,
       native_sayge="sayge_display=491 enemy_rows_unchanged=true"in host)
     else:
      checks.update(two_native_encounters=host.count("ritual fixture native boss defeated")==2,two_scenes=host.count("ritual spawned:")==2,
       second_scene_outside="ritual visual fixture: outside=true position=Some"in host,
       native_sayges=host.count("sayge_display=491 enemy_rows_unchanged=true")==2)
    elif all_events:
     checks.update(native_encounter="ritual fixture native boss defeated"in host,
      ritual_reward=bool(rewards),native_sayge="sayge_display=491 enemy_rows_unchanged=true"in host,
      completed_scene_cleanup=host.count("ritual scene removed:")==len(rewards),
      unique_ritual_rewards=len(rewards)==len({stage for stage,_,_ in rewards}),
      no_automatic_boss_fortunes="blessing granted:"not in host,
      no_ritual_failure="Sayge's Trial stopped:"not in host)
    else:
     checks.update(two_native_encounters=host.count("ritual fixture native boss defeated")==2,
      two_rituals=host.count("ritual spawned:")==2,two_rewards=len(rewards)==2,
      two_visual_sayges=host.count("sayge_display=491 enemy_rows_unchanged=true")==2,
      full_scene_cleanup=host.count("ritual scene removed:")==2,
      unique_rewards=len(rewards)==len({stage for stage,_,_ in rewards}),
      no_automatic_boss_fortunes="blessing granted:"not in host,
      no_event_failure="Sayge's Trial stopped:"not in host)
    (out/"ritual-evidence.json").write_text(json.dumps(dict(checks=checks,rewards=rewards,passed=all(checks.values()),scope="Controlled solo native integration; visual review separate"),indent=2)+"\n")
   if supplies:
    checks=checks if all_events else {k:checks[k]for k in("clients_alive","started","clean_scripts","startup_ack")}
    discoveries=re.findall(r"supplies discovered: event=(\d+) reward=(.*?) name=(.*?) gold=(\d+) clock=",host)
    checks.update(supply_spawn="supplies spawned:"in host,supply_cleanup="supplies removed:"in host,
     six_consumables=all("family: "+family in host for family in("Recovery","Critical","Defense","Vitality","Ferocity","Reach")),
     materials_sold=any(int(gold)>0 for _,_,_,gold in discoveries),unique_claims=len(discoveries)==len({event for event,_,_,_ in discoveries}),
     stats_restored="inputs_restored=true"in host and "inputs_restored=false"not in host and "stable=false"not in host,
     six_families_coexist="families=6"in host,buff_expired="supplies stats rebuilt: active=false"in host,
     multiple_reserves=any(int(v)>=2 for v in re.findall(r"supplies active reserves: count=(\d+)",host)),
     world_placement="world_wide=true display=36"in host,no_supplies_failure="Lost Supplies stopped:"not in host)
    checks.update(consumable_stat_evidence(host))
    if polish:checks.update(native_minimap="events native minimap:"in host,ritual_world_scene="sayge_display=491 enemy_rows_unchanged=true"in host,ritual_available="ritual spawned:"in host and "ritual scene removed:"not in host,no_ritual_failure="Sayge's Trial stopped:"not in host)
    (out/"supplies-evidence.json").write_text(json.dumps(dict(checks=checks,discoveries=discoveries,passed=all(checks.values()),scope="Solo controlled stacking plus production map-wide placement; native UI and stat values require separate review"),indent=2)+"\n")
   if goblins:
    from treasure_goblin_evidence import assess as assess_goblins
    checks=checks if all_events else {k:checks[k]for k in("clients_alive","started","clean_scripts","startup_ack")}
    evidence=assess_goblins(logs,disabled=os.environ.get("ASX_TEST_GOBLINS_OFF")=="1")
    if os.environ.get("ASX_TEST_GOBLINS_OFF")!="1":
     persisted=[];expected=[]
     for index in range(players):
      saved=sum(int(v)for v in re.findall(r"^marks\|\d+\|(\d+)$",prepare.read(out/f"player-{index+1}/appdata/wenilla-rogue"),re.M))
      expected.append(sum(round(float(v)*10)for v in re.findall(rf"treasure goblin amulets: event=\d+ slot={index} guid=\d+ count=\d+ marks=([\d.]+)",host)))
      persisted.append(saved-goblin_mark_baselines[index])
     native_boss_marks=[sum(round(float(amount)*10)for amount in re.findall(r"You earn ([\d.]+) marks",text))for text in native]if all_events else[0]*players
     expected=[goblin+boss for goblin,boss in zip(expected,native_boss_marks)]
     evidence["checks"]["marks_persisted_personally"]=all(a==b and a>=0 for a,b in zip(persisted,expected))
     evidence["marks_tenths"]=dict(expected=expected,persisted=persisted)
     evidence["passed"]=all(evidence["checks"].values())
    merge_assessor_checks(checks,evidence["checks"])
    (out/"goblin-evidence.json").write_text(json.dumps(evidence,indent=2)+"\n")
   if not visual:checks.pop("ui_captures_available",None)
   if diagnostic_probe:
    checks={k:checks[k]for k in("clients_alive","clean_scripts","clean_adapter","startup_ack")}
    checks["lua_transport_observed"]=all("ASX_DIAG "in text and'"kind":"observer.started"'in text for text in logs)
   if os.environ.get("ASX_TEST_OBSERVED_STEPS")=="1":env["ASX_TEST_OBSERVED_STEPS"]="1"
   if os.environ.get("ASX_TEST_LUA_DIAGNOSTICS")=="1":
    from diagnostic_records import collect,assess_events
    streams=[collect(text,out.name,index)for index,text in enumerate(logs)]
    for index,stream in enumerate(streams):
     (out/f"lua-observations-{index}.jsonl").write_text(''.join(json.dumps(row,separators=(',',':'))+"\n"for row in stream["records"]))
     evidence=assess_events(stream)if all_events else stream
     for name,value in evidence["checks"].items():checks[f"lua_{index}_{name}"]=value
     (out/f"lua-evidence-{index}.json").write_text(json.dumps(evidence,indent=2)+"\n")
   if os.environ.get("ASX_TEST_OBSERVED_STEPS")=="1":
    checks["observed_goblin_steps"]="scenario step: goblin phase=1 action=1"in host and"scenario step: goblin phase=2 action=2"in host
    if players==1:
     checks["observed_supply_steps"]=all(f"scenario step: supplies phase={phase} created={phase}"in host for phase in range(1,12))
     checks["observed_ritual_steps"]="scenario step: ritual action=0"in host and"scenario step: ritual action=1"in host
    else:checks["observed_coop_steps"]=all(f"scenario step: coop action={phase}"in host for phase in range(1,5))
   result=dict(observed_steps=os.environ.get("ASX_TEST_OBSERVED_STEPS")=="1",scenario=scenario,diagnostic_probe=diagnostic_probe,support_only=support_only,all_events=all_events,owner_tooltip_readback=owner_tips,completion121=completion,players=players,normal=normal,game_version=version,compat121=compat121,enchantments=enchantments,gems=gems,goblins=goblins,immunity=immunity,diagnostic_inputs=not normal,dll_sha256=proof["dll_sha256"],game_sha256=sha,checks=checks,passed=all(checks.values()),grants=grants,ui=ui,desktop_input=False,screenshots=visual,visual_review_required=visual,ritual_ui_single=ritual_ui and seconds<150,visual_discovery_hold=visual and supplies and os.environ.get("ASX_TEST_VISUAL_HOLD_DISCOVERY")=="1")
   return result
  until=time.monotonic()+min(seconds,260 if supplies or ritual else 280 if completion in ("support","fatigue","fatigue-self") else 210 if glyph_retention or goblins else 180 if enchantments else 150)
  completed_at=None
  while time.monotonic()<until and all(p.poll()is None for p in children):
   if (ROOT/"test-results/extensions-trial-stop").exists():break
   if readiness is not None and readiness.complete(out) and evaluate()["passed"]:break
   if readiness is None and all_events:
    live_path=out/"player-1/azeroth-coop-native.log"
    live=live_path.read_text(encoding="utf-8",errors="replace")if live_path.exists()else""
    goblin_done="treasure goblin rewarded:"in live and"treasure goblin escaped:"in live and live.count("treasure goblin fine motion:")==2
    solo_done="ritual completed:"in live and"supplies stats rebuilt: active=false"in live
    coop_done="coop ritual completed: stage=1 recipients=2"in live and"coop supplies discovered:"in live
    if goblin_done and(solo_done if players==1 else coop_done):break
   if os.environ.get("ASX_TEST_UI_REMAINING")=="1" and evaluate()["passed"]:break
   if os.environ.get("ASX_TEST_WORLD_UI")=="1" and os.environ.get("ASX_TEST_UI_REMAINING")!="1":
    path=out/"player-1/azeroth-coop-native.log"
    if path.exists() and "support121 observed: index=4 "in path.read_text(errors="replace") and evaluate()["passed"]:break
   if readiness is None and support_only:
    live_path=out/"player-1/azeroth-coop-native.log"
    if live_path.exists() and("support121 observed: index=5 "if shield_only else"support121 observed: index=11 ")in live_path.read_text(encoding="utf-8",errors="replace"):break
   if readiness is None and completion=="pause":
    local_logs=[out/f"player-{i}/azeroth-coop-native.log"for i in range(1,players+1)]
    closed=all(p.exists() and"open=false"in p.read_text(encoding="utf-8",errors="replace").split("native menu fixture:")[-1]for p in local_logs)
    if closed and completed_at is None:completed_at=time.monotonic()
    if completed_at is not None and time.monotonic()-completed_at>=6:break
   if os.environ.get("ASX_TEST_COVERAGE")=="1":
    path=out/"player-1/azeroth-coop-native.log"
    if path.exists() and "coverage native guardian killed:"in path.read_text(errors="replace") and evaluate()["passed"]:break
   if diagnostic_probe:
    native_path=out/"player-1/azeroth-coop-native.log"
    if native_path.exists() and native_path.read_text(encoding="utf-8",errors="replace").count("ASX_DIAG ")>=5:break
   time.sleep(1 if all_events or support_only or diagnostic_probe else .25)
  result=evaluate()
  (out/"result.json").write_text(json.dumps(result,indent=2)+"\n");print(out,flush=True);print(json.dumps(result,indent=2),flush=True)
 finally:
  for p in children:
   if p.poll()is None:p.terminate()
  for p in children:
   try:p.wait(timeout=10)
   except subprocess.TimeoutExpired:p.kill();p.wait(timeout=5)
  for f in handles:f.close()
 if not result["passed"]:raise SystemExit(1)
