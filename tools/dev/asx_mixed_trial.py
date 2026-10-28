"""Observe both ASX/native pairings; no protocol/gameplay changes or firewall rules.
Reuse two existing verified ASX trial EXE paths to avoid new firewall identities.
The native side launches the direct EXE with an inert proxy and no ASX flags.
"""
import hashlib,json,os,socket,subprocess,time,uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
DLL="8442dad71114e3a6f0cb693987bb714d57da3be7a52bb8c4fce7e58f97d5f9eb"
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def events(text):
    out=[]
    for line in text.splitlines():
        if line.startswith("rogue: {"):
            try:out.append(json.loads(line[7:]))
            except ValueError:pass
    return out
def run(seconds):
    if os.name!="nt":raise RuntimeError("Windows only")
    from disposable115_party import writer,SHA
    from script_diagnostics import assess
    # Exact previously validated private distribution paths. No installer/solo discovery.
    base=None;folders=[]
    for candidate in sorted((ROOT/"test-results").glob("asx-install-*"),key=lambda p:p.stat().st_mtime,reverse=True):
        pair=[next(candidate.glob("* "+str(i)),None)for i in(1,2)]
        if all(p and (p/"AzerothSurvivors.exe").is_file()and(p/"avrt.dll").is_file()and digest(p/"AzerothSurvivors.exe")==SHA and digest(p/"avrt.dll")==DLL for p in pair):base=candidate;folders=pair;break
    if not base:raise ValueError("Run the isolated ASX installation trial first")
    for folder in folders:
        if not folder.resolve().is_relative_to(base.resolve())or digest(folder/"AzerothSurvivors.exe")!=SHA or digest(folder/"avrt.dll")!=DLL:raise ValueError("Private trial identity changed")
    prepare=writer(folders[0]/"AzerothSurvivors.exe")
    output=ROOT/"test-results"/("asx-mixed-"+uuid.uuid4().hex[:8]);output.mkdir()
    clean={k:v for k,v in os.environ.items()if not k.upper().startswith(("ASX_","AZCOOP_","COOP_","WOW_","BENILLA_"))}
    clean.update(WOW_NOSOUND="1",WOW_WIN="1",WOW_ROGUE_UPDATE="0",WOW_ROGUE_UPDATE_AUTO="0",RUST_LOG="info",PYTHONUTF8="1")
    all_cases=[]
    for case,asx_host in(("native-host-asx-guest",False),("asx-host-native-guest",True)):
        target=output/case;target.mkdir();children=[];handles=[];records=[]
        with socket.socket()as probe:probe.bind(("127.0.0.1",0));port=probe.getsockname()[1]
        password=uuid.uuid4().hex
        try:
            for slot,folder in enumerate(folders,1):
                record=target/f"player-{slot}";record.mkdir();records.append(record)
                app=record/"appdata";save=app/"wenilla-rogue";save.mkdir(parents=True);prepare(save,slot)
                (record/"shots").mkdir()
                active=asx_host if slot==1 else not asx_host
                env=dict(clean,APPDATA=str(app),LOCALAPPDATA=str(record/"localappdata"),WOW_ROGUE_AUTOPICK="1",WOW_ROGUE_AUTOSTART="1",WOW_ROGUE_AUTOPLAY="18"if slot==1 else"4",WOW_ROGUE_COOP_PASSWORD=password,WOW_ROGUE_SHOTS=str(record/"shots"))
                env["WOW_ROGUE_COOP_HOST"if slot==1 else"WOW_ROGUE_COOP_JOIN"]=str(port)if slot==1 else f"127.0.0.1:{port}"
                if active:env.update(ASX_ACTIVE="1",AZCOOP_ENGINE="official",ASX_READY_FILE=str(record/"ready.json"),AZCOOP_NATIVE_LOG_DIRECTORY=str(record))
                from trial_limits import environment
                env.update(environment())
                env.pop("WOW_ROGUE_SHOTS",None)
                out=(record/"native-output.log").open("wb");handles.append(out)
                children.append(subprocess.Popen([str(folder/"AzerothSurvivors.exe")],cwd=folder,env=env,stdout=out,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW))
                time.sleep(4 if slot==1 else 2)
            until=time.monotonic()+min(105,seconds//2)
            while time.monotonic()<until and all(p.poll()is None for p in children):time.sleep(.25)
            rows=[]
            for slot,(record,child)in enumerate(zip(records,children),1):
                text=(record/"native-output.log").read_text(errors="replace");native=events(text)
                log=record/"azeroth-coop-native.log";mod=log.read_text(errors="replace")if log.exists()else""
                rules=[line for line in mod.splitlines()if any(key in line for key in("official XP:","official gold:","official support","official difficulty:","official group pause:"))]
                (record/"rule-observations.txt").write_text("\n".join(rules))
                levels=[e.get("level",0)for e in native if e.get("type")=="choices"]
                roster=[e for e in native if e.get("type")=="party"]
                rows.append(dict(slot=slot,asx_requested=asx_host if slot==1 else not asx_host,alive=child.poll()is None,exit_code=child.poll(),joined=any(e.get("type")=="coop"and e.get("state")=="joined"for e in native),largest_roster=max((len(e.get("members",[]))for e in roster),default=0),started=any(e.get("type")=="start"for e in native),resumed=any(e.get("type")=="resume"for e in native),levels=levels,asx_acknowledged=(record/"ready.json").exists(),asx_active="ASX active:"in mod,rule_observation_count=len(rules),native_errors=[line for line in mod.splitlines()if"[ ERROR ]"in line or"rejected:"in line],scripts=assess(text)))
            result=dict(case=case,players=rows,mixed_admitted=all(row["joined"]and row["largest_roster"]==2 for row in rows),both_started=all(row["started"]for row in rows),both_progressed=all(max(row["levels"],default=0)>1 and row["resumed"]for row in rows),both_alive=all(row["alive"]for row in rows),asx_remains_active_without_peer_negotiation=any(row["asx_requested"]and row["asx_active"]and row["rule_observation_count"]>0 for row in rows),script_diagnostics_clean=all(row["scripts"]["passed"]for row in rows))
            (target/"result.json").write_text(json.dumps(result,indent=2)+"\n");all_cases.append(result)
            print(json.dumps(result,indent=2),flush=True)
        finally:
            for child in children:
                if child.poll()is None:child.terminate()
            for child in children:
                try:child.wait(timeout=10)
                except subprocess.TimeoutExpired:child.kill();child.wait(timeout=5)
            for handle in handles:handle.close()
    summary=dict(normal_dll_sha256=DLL,game_sha256=SHA,muted=True,desktop_input=False,firewall_changes=False,transport="official CLI loopback fixture, not public lobby service",cases=all_cases,observation_complete=len(all_cases)==2 and all(case["mixed_admitted"]and case["both_started"]and case["both_progressed"]and case["both_alive"]for case in all_cases),compatibility_guaranteed=False)
    (output/"result.json").write_text(json.dumps(summary,indent=2)+"\n");print(output,flush=True)
    if not summary["observation_complete"]:raise SystemExit(1)
