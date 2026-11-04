"""One bounded packaged normal-AZSX trial, private Windows profiles, no desktop input."""
import ctypes,hashlib,json,os,re,shutil,socket,subprocess,time,uuid,zipfile
from pathlib import Path
from client_registry import VERSION
ROOT=Path(__file__).resolve().parents[2]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def initial_check_passed(code,before,after,projected_hash,old_hash,new_hash):
    """Read-only check may report the old owned DLL pending replacement."""
    expected=0 if old_hash is not None and old_hash==new_hash else 1
    return code==expected and before==after and projected_hash==old_hash

def native_coop_controls(slot):
    """Native-only ready/Move-to-begin input for the fixed private 800x600 scene."""
    if type(slot) is not int or slot not in (1, 2):
        raise ValueError("Packaged trial requires host/guest slots 1 and 2")
    return {"WOW_ROGUE_AUTOSTART": "1", "WOW_ROGUE_TAPS": ";".join(["410,485" if slot == 1 else "306,525"] * 32),
            "AZSX_LOG_NATIVE_UI": "1", "WOW_ROGUE_ROAM": "1"}

def stop_game(folder):
    sessions=list((folder/"AZSX/Data/Logs").glob("*/session.json"))
    if not sessions:return
    meta=json.loads(max(sessions,key=lambda p:p.stat().st_mtime).read_text())
    k=ctypes.WinDLL("kernel32",use_last_error=True)
    k.OpenProcess.argtypes=[ctypes.c_ulong,ctypes.c_int,ctypes.c_ulong];k.OpenProcess.restype=ctypes.c_void_p
    k.TerminateProcess.argtypes=[ctypes.c_void_p,ctypes.c_uint];k.CloseHandle.argtypes=[ctypes.c_void_p]
    h=k.OpenProcess(1,False,meta["game_pid"])
    if h:
        try:k.TerminateProcess(h,0)
        finally:k.CloseHandle(h)
def run(seconds,game_version=VERSION,package_path=None,legacy_package=None):
    if game_version != VERSION:raise ValueError("Only the selected reviewed packaged trial is supported")
    tag=game_version.replace(".","")
    if os.name!="nt":raise RuntimeError("Windows only")
    runtime=(ROOT/("runtime-official-"+tag)).resolve();exe=runtime/"AzerothSurvivors.exe"
    from disposable115_party import writer
    expected=json.loads((ROOT/"azsx/supported-games.json").read_text())["games"][game_version]["sha256"]
    if sha(exe)!=expected:raise ValueError("Wrong private native build")
    if package_path is None:raise ValueError("An explicit prepared package is required")
    package=Path(package_path)
    proof=json.loads(package.with_name("PREPARATION.json").read_text())
    if not proof["version"].startswith(game_version+"-azsx."):raise ValueError("Packaged trial version differs")
    if sha(package)!=proof["archive_sha256"]:raise ValueError("Package differs")
    output=ROOT/"test-results"/("azsx-install-"+uuid.uuid4().hex[:8]);output.mkdir()
    prepare=writer(exe);children=[];streams=[];folders=[];environments=[];checks={}
    clean={k:v for k,v in os.environ.items()if not k.upper().startswith(("AZCOOP_","COOP_","WOW_","BENILLA_","AZSX_"))}
    clean.update(PYTHONUTF8="1",WOW_NOSOUND="1",WOW_WIN="1",WOW_ROGUE_UPDATE="0",WOW_ROGUE_UPDATE_AUTO="0",RUST_LOG="info")
    try:
        for slot in range(1,3):
            folder=output/("AZSX & native é "+str(slot));folder.mkdir();folders.append(folder)
            shutil.copyfile(exe,folder/exe.name);(folder/"Data").mkdir()
            for asset in sorted((runtime/"Data").glob("*.MPQ")):os.link(asset,folder/"Data"/asset.name)
            old_hash=None
            if legacy_package and slot==1:
                # Read-only approved old archive, extracted only into this disposable copy.
                old_path=Path(legacy_package);old_proof=json.loads(old_path.with_name('PREPARATION.json').read_text())
                if sha(old_path)!=old_proof['archive_sha256']:raise ValueError('Legacy fixture archive changed')
                with zipfile.ZipFile(old_path)as old:
                    old.extractall(folder)
                    manifest_name=next(n for n in ('ASX/App/manifest.json','ASX/manifest.json')if n in old.namelist())
                    old_m=json.loads(old.read(manifest_name));owned={n:spec['sha256']for n,spec in old_m['files'].items()}
                    owned[manifest_name]=hashlib.sha256(old.read(manifest_name)).hexdigest()
                    modern=manifest_name.startswith('ASX/App/')
                    payload='ASX/App/'if modern else'ASX/'
                    ui=payload+('AddOns/'if modern else'Interface/')
                    projection={'avrt.dll':payload+'Bin/avrt.dll','benilla-config/AddOns/ASX/ASX.toc':ui+'ASX.toc','benilla-config/AddOns/ASX/Presentation.lua':ui+'Presentation.lua'}
                    projection.update({'benilla-config/AddOns/ASXUI/'+n.removeprefix(ui+'ASXUI/'):n for n in old_m['files']if n.startswith(ui+'ASXUI/')})
                    for name,source in projection.items():
                        dest=folder/name;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(old.read(source));owned[name]=old_m['files'][source]['sha256']
                old_hash=sha(folder/'avrt.dll')
                state=folder/('ASX/Data/State'if modern else'ASX/State');state.mkdir(parents=True,exist_ok=True)
                (state/'install.json').write_text(json.dumps({'schema':1,'version':old_m['version'],'owned':owned,'game':old_m['game']}))
                options=folder/('ASX/Config/options.json'if modern else'ASX/options.json');options.parent.mkdir(parents=True,exist_ok=True);options.write_text('{"lost_supplies":false}')
                previous=folder/('ASX/Data/Logs/retained/session.log'if modern else'ASX/Logs/retained/session.log');previous.parent.mkdir(parents=True,exist_ok=True);previous.write_text('retained legacy evidence')
            with zipfile.ZipFile(package)as z:z.extractall(folder)
            (folder/"shots").mkdir();home=folder/"benilla-config";home.mkdir(exist_ok=True);app=folder/"profile/appdata";save=app/"wenilla-rogue";save.mkdir(parents=True)
            prepare(save,slot)
            config='[cvars]\ngxWindow = "1"\ngxResolution = "800x600"\n'+''.join(f'{key} = "0"\n'for key in("MasterVolume","MusicVolume","SoundVolume","AmbienceVolume","EnableMusic","EnableAmbience","MasterSoundEffects"))
            (home/"config.toml").write_text(config)
            env=dict(clean,APPDATA=str(app),LOCALAPPDATA=str(folder/"profile/localappdata"));environments.append(env)
            python=folder/"AZSX/App/Python/python.exe";script=folder/"AZSX/App/Launcher/launch.py"
            before=(sha(save/"characters.txt"),sha(home/"config.toml"))
            cmd=[str(python),"-I","-B",str(script),"--check","--offline"]
            result=subprocess.run(cmd,cwd=folder,env=env,capture_output=True,timeout=40)
            (output/f"install-{slot}.log").write_bytes(result.stdout+result.stderr)
            checks[f"install_{slot}"]=initial_check_passed(result.returncode,before,(sha(save/"characters.txt"),sha(home/"config.toml")),sha(folder/"avrt.dll")if(folder/"avrt.dll").is_file()else None,old_hash,proof["dll_sha256"])
            if not checks[f"install_{slot}"]:raise RuntimeError("Packaged Windows installation failed; see "+str(output))
        # Same native profile, direct EXE: proxy forwards, AZSX stays inactive.
        folder=folders[0];env=dict(environments[0],WOW_ROGUE_AUTOPICK="1",WOW_ROGUE_AUTOSTART="1",WOW_ROGUE_AUTOPLAY="3600")
        out=(output/"vanilla.log").open("wb");streams.append(out)
        vanilla=subprocess.Popen([str(folder/"AzerothSurvivors.exe")],cwd=folder,env=env,stdout=out,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW);children.append(vanilla)
        time.sleep(10);checks["direct_native_alive"]=vanilla.poll()is None
        checks["direct_native_no_activation"]=not list((folder/"AZSX/Data/Logs").glob("*/ready.json"))
        vanilla.terminate();vanilla.wait(timeout=10)
        with socket.socket()as probe:probe.bind(("127.0.0.1",0));port=probe.getsockname()[1]
        password=uuid.uuid4().hex
        for slot,(folder,base)in enumerate(zip(folders,environments),1):
            env=dict(base,WOW_ROGUE_AUTOPICK="1",WOW_ROGUE_AUTOPLAY="18"if slot==1 else"4",WOW_ROGUE_COOP_PASSWORD=password,WOW_ROGUE_SHOTS=str(folder/"shots"))
            # Keep AUTOSTART through the 0.139.2 ready check, matching the
            # qualified extension trials; use native CLI input, never OS input.
            env.update(native_coop_controls(slot))
            env["WOW_ROGUE_COOP_HOST"if slot==1 else"WOW_ROGUE_COOP_JOIN"]=str(port)if slot==1 else f"127.0.0.1:{port}"
            from trial_limits import environment
            env.update(environment());env.pop("WOW_ROGUE_SHOTS",None)
            out=(output/f"launcher-{slot}.log").open("wb");streams.append(out)
            # Exercise the exact distributed CMD, including ampersand/unicode paths.
            child=subprocess.Popen([os.environ["ComSpec"],"/d","/c","start-azsx.cmd","--offline"],cwd=folder,env=env,stdout=out,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW)
            children.append(child)
            if slot==1:
                # Migration may legitimately take longer than the fresh guest.
                # Wait for the real native host before the guest's one-shot join.
                deadline=time.monotonic()+90
                while time.monotonic()<deadline and child.poll()is None:
                    logs=list((folder/'AZSX/Data/Logs').glob('*/game-output.log'))
                    if any('"state":"hosting"'in p.read_text(errors='replace')for p in logs):break
                    time.sleep(.2)
                else:raise RuntimeError('Native host did not become ready before guest launch')
            else:time.sleep(2)
        until=time.monotonic()+min(seconds,110)
        while time.monotonic()<until and all(p.poll()is None for p in children[1:]):
            if (ROOT/"test-results/azsx-install-stop").exists():break
            time.sleep(.25)
        native=[]
        from script_diagnostics import assess
        for slot,folder in enumerate(folders,1):
            logs=list((folder/"AZSX/Data/Logs").glob("*/game-output.log"));text=logs[0].read_text(errors="replace")if logs else"";native.append(text)
            with zipfile.ZipFile(package)as z:
                bundle={name[len("AZSX/App/AddOns/AZSXUI/"):]:z.read(name)for name in z.namelist()if name.startswith("AZSX/App/AddOns/AZSXUI/")}
            if legacy_package and slot==1:
                journal=folder/'AZSX/Data/State/branding-migration.json'
                checks['legacy_migration_complete']=journal.is_file()and json.loads(journal.read_text()).get('phase')=='complete'
                checks['legacy_preferences_preserved']=(folder/'AZSX/Config/options.json').read_text()=='{"lost_supplies":false}'
                checks['legacy_logs_preserved']=(folder/'AZSX/Data/Logs/retained/session.log').read_text()=='retained legacy evidence'
                checks['legacy_trees_retired']=not(folder/'ASX').exists()and not(folder/'benilla-config/AddOns/ASX/ASX.toc').exists()and not(folder/'benilla-config/AddOns/ASXUI/ASXUI.toc').exists()
            checks[f"bundled_ui_{slot}"]=bool(bundle)and all((folder/"benilla-config/AddOns/AZSXUI"/name).read_bytes()==data for name,data in bundle.items())
            checks[f"projected_dll_{slot}"]=sha(folder/"avrt.dll")==proof["dll_sha256"]
            checks[f"active_{slot}"]=bool(list((folder/"AZSX/Data/Logs").glob("*/ready.json"))) and "AZSX is active"in(output/f"launcher-{slot}.log").read_text(errors="replace")
            checks[f"native_joined_{slot}"]='"state":"joined"'in text
            checks[f"native_profile_{slot}"]=("uiwarrior","uipaladin","uidruid")[slot-1]in text.lower()
            checks[f"scripts_clean_{slot}"]=assess(text)["passed"]
            checks[f"backup_{slot}"]=bool(list((folder/"AZSX/Data/Backups/Progression").glob("*.txt")))
            checks[f"alive_{slot}"]=children[slot].poll()is None
            stop_game(folder)
        for p in children[1:]:p.wait(timeout=20)
        checks["gameplay_started"]=all('"type":"start"'in text and '"type":"choices"'in text and '"type":"resume"'in text for text in native)
        # Real self-uninstall on a disposable installation, preserve user files.
        folder=folders[-1];ui_note=folder/"benilla-config/AddOns/AZSXUI/user-note.txt";ui_note.write_text("preserve UI note")
        ui_settings=folder/"benilla-config/SavedVariables/AZSXUI.lua";ui_settings.parent.mkdir(parents=True,exist_ok=True);ui_settings.write_text("AZSXUIDB = {scale=1.25}")
        unknown=folder/"AZSX/App/Launcher/user-note.txt";unknown.write_text("keep")
        save=folder/"profile/appdata/wenilla-rogue/characters.txt";home=folder/"benilla-config/config.toml";before=(sha(save),sha(home),sha(folder/"AzerothSurvivors.exe"))
        cmd=[str(folder/"AZSX/App/Python/python.exe"),"-I","-B",str(folder/"AZSX/App/Launcher/launch.py"),"--export-diagnostics"]
        report=subprocess.run(cmd,cwd=folder,env=environments[-1],capture_output=True,timeout=30);(output/"export.log").write_bytes(report.stdout+report.stderr)
        checks["export_local"]=report.returncode==0 and bool(list((folder/"AZSX/Data/Reports").glob("*.zip")))
        cmd[-1]="--uninstall"
        removal=subprocess.run(cmd,cwd=folder,env=environments[-1],capture_output=True,timeout=30);(output/"uninstall.log").write_bytes(removal.stdout+removal.stderr)
        match=re.search(r"Removal report: (.+result.json)",removal.stdout.decode(errors="replace"))
        result_path=Path(match[1].strip())if match else output/"missing-result"
        until=time.monotonic()+40
        while not result_path.exists()and time.monotonic()<until:time.sleep(.2)
        outcome=json.loads(result_path.read_text(encoding="utf-8-sig"))if result_path.exists()else{}
        (output/"uninstall-result.json").write_text(json.dumps(outcome,indent=2))
        checks["uninstall_completed"]=removal.returncode==0 and outcome.get("completed",False) and not(folder/"avrt.dll").exists()and not(folder/"start-azsx.cmd").exists()and not(folder/"AZSX/App/Python/python.exe").exists()
        checks["uninstall_ui_removed"]=not(folder/"benilla-config/AddOns/AZSXUI/AZSXUI.toc").exists()
        checks["uninstall_ui_preferences_preserved"]=ui_note.read_text()=="preserve UI note"and ui_settings.read_text()=="AZSXUIDB = {scale=1.25}"
        checks["uninstall_preserved"]=unknown.read_text()=="keep"and before==(sha(save),sha(home),sha(folder/"AzerothSurvivors.exe"))and bool(list((folder/"AZSX/Data/Backups/Progression").glob("*.txt")))
        result=dict(checks=checks,passed=all(checks.values()),archive_sha256=proof["archive_sha256"],dll_sha256=proof["dll_sha256"],players=len(folders),muted=True,desktop_input=False,production_binary=True,transport="official engine loopback fixture; public lobby service not exercised")
        (output/"result.json").write_text(json.dumps(result,indent=2)+"\n");print(json.dumps(result,indent=2),flush=True);print(output,flush=True)
        if not result["passed"]:raise SystemExit(1)
    finally:
        for folder in folders:stop_game(folder)
        for p in children:
            if p.poll()is None:p.terminate()
        for p in children:
            try:p.wait(timeout=10)
            except subprocess.TimeoutExpired:p.kill();p.wait(timeout=5)
        for stream in streams:stream.close()
