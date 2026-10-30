"""One bounded packaged normal-ASX trial, private Windows profiles, no desktop input."""
import ctypes,hashlib,json,os,re,shutil,socket,subprocess,time,uuid,zipfile
from pathlib import Path
from client_registry import VERSION
ROOT=Path(__file__).resolve().parents[2]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def stop_game(folder):
    sessions=list((folder/"ASX/Logs").glob("*/session.json"))
    if not sessions:return
    meta=json.loads(max(sessions,key=lambda p:p.stat().st_mtime).read_text())
    k=ctypes.WinDLL("kernel32",use_last_error=True)
    k.OpenProcess.argtypes=[ctypes.c_ulong,ctypes.c_int,ctypes.c_ulong];k.OpenProcess.restype=ctypes.c_void_p
    k.TerminateProcess.argtypes=[ctypes.c_void_p,ctypes.c_uint];k.CloseHandle.argtypes=[ctypes.c_void_p]
    h=k.OpenProcess(1,False,meta["game_pid"])
    if h:
        try:k.TerminateProcess(h,0)
        finally:k.CloseHandle(h)
def run(seconds,game_version=VERSION,package_path=None):
    if game_version != VERSION:raise ValueError("Only the selected reviewed packaged trial is supported")
    tag=game_version.replace(".","")
    if os.name!="nt":raise RuntimeError("Windows only")
    runtime=(ROOT/("runtime-official-"+tag)).resolve();exe=runtime/"AzerothSurvivors.exe"
    from disposable115_party import writer
    expected=json.loads((ROOT/"asx/supported-games.json").read_text())["games"][game_version]["sha256"]
    if sha(exe)!=expected:raise ValueError("Wrong private native build")
    if package_path is None:raise ValueError("An explicit prepared package is required")
    package=Path(package_path)
    proof=json.loads(package.with_name("PREPARATION.json").read_text())
    if not proof["version"].startswith(game_version+"-asx."):raise ValueError("Packaged trial version differs")
    if sha(package)!=proof["archive_sha256"]:raise ValueError("Package differs")
    output=ROOT/"test-results"/("asx-install-"+uuid.uuid4().hex[:8]);output.mkdir()
    prepare=writer(exe);children=[];streams=[];folders=[];environments=[];checks={}
    clean={k:v for k,v in os.environ.items()if not k.upper().startswith(("AZCOOP_","COOP_","WOW_","BENILLA_","ASX_"))}
    clean.update(PYTHONUTF8="1",WOW_NOSOUND="1",WOW_WIN="1",WOW_ROGUE_UPDATE="0",WOW_ROGUE_UPDATE_AUTO="0",RUST_LOG="info")
    try:
        for slot in range(1,3):
            folder=output/("ASX & native é "+str(slot));folder.mkdir();folders.append(folder)
            shutil.copyfile(exe,folder/exe.name);(folder/"Data").mkdir()
            for asset in sorted((runtime/"Data").glob("*.MPQ")):os.link(asset,folder/"Data"/asset.name)
            with zipfile.ZipFile(package)as z:z.extractall(folder)
            (folder/"shots").mkdir();home=folder/"benilla-config";home.mkdir();app=folder/"profile/appdata";save=app/"wenilla-rogue";save.mkdir(parents=True)
            prepare(save,slot)
            config='[cvars]\ngxWindow = "1"\ngxResolution = "800x600"\n'+''.join(f'{key} = "0"\n'for key in("MasterVolume","MusicVolume","SoundVolume","AmbienceVolume","EnableMusic","EnableAmbience","MasterSoundEffects"))
            (home/"config.toml").write_text(config)
            env=dict(clean,APPDATA=str(app),LOCALAPPDATA=str(folder/"profile/localappdata"));environments.append(env)
            python=folder/"ASX/Runtime/python.exe";script=folder/"ASX/Client/launch.py"
            before=(sha(save/"characters.txt"),sha(home/"config.toml"))
            cmd=[str(python),"-I","-B",str(script),"--check","--offline"]
            result=subprocess.run(cmd,cwd=folder,env=env,capture_output=True,timeout=40)
            (output/f"install-{slot}.log").write_bytes(result.stdout+result.stderr)
            checks[f"install_{slot}"]=result.returncode==0 and before==(sha(save/"characters.txt"),sha(home/"config.toml")) and sha(folder/"avrt.dll")==proof["dll_sha256"]
            if not checks[f"install_{slot}"]:raise RuntimeError("Packaged Windows installation failed; see "+str(output))
        # Same native profile, direct EXE: proxy forwards, ASX stays inactive.
        folder=folders[0];env=dict(environments[0],WOW_ROGUE_AUTOPICK="1",WOW_ROGUE_AUTOSTART="1",WOW_ROGUE_AUTOPLAY="3600")
        out=(output/"vanilla.log").open("wb");streams.append(out)
        vanilla=subprocess.Popen([str(folder/"AzerothSurvivors.exe")],cwd=folder,env=env,stdout=out,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW);children.append(vanilla)
        time.sleep(10);checks["direct_native_alive"]=vanilla.poll()is None
        checks["direct_native_no_activation"]=not list((folder/"ASX/Logs").glob("*/ready.json"))
        vanilla.terminate();vanilla.wait(timeout=10)
        with socket.socket()as probe:probe.bind(("127.0.0.1",0));port=probe.getsockname()[1]
        password=uuid.uuid4().hex
        for slot,(folder,base)in enumerate(zip(folders,environments),1):
            env=dict(base,WOW_ROGUE_AUTOPICK="1",WOW_ROGUE_AUTOSTART="1",WOW_ROGUE_AUTOPLAY="18"if slot==1 else"4",WOW_ROGUE_COOP_PASSWORD=password,WOW_ROGUE_SHOTS=str(folder/"shots"))
            env["WOW_ROGUE_COOP_HOST"if slot==1 else"WOW_ROGUE_COOP_JOIN"]=str(port)if slot==1 else f"127.0.0.1:{port}"
            from trial_limits import environment
            env.update(environment());env.pop("WOW_ROGUE_SHOTS",None)
            out=(output/f"launcher-{slot}.log").open("wb");streams.append(out)
            # Exercise the exact distributed CMD, including ampersand/unicode paths.
            child=subprocess.Popen([os.environ["ComSpec"],"/d","/c","start-asx.cmd","--offline"],cwd=folder,env=env,stdout=out,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW)
            children.append(child);time.sleep(4 if slot==1 else 2)
        until=time.monotonic()+min(seconds,110)
        while time.monotonic()<until and all(p.poll()is None for p in children[1:]):
            if (ROOT/"test-results/asx-install-stop").exists():break
            time.sleep(.25)
        native=[]
        from script_diagnostics import assess
        for slot,folder in enumerate(folders,1):
            logs=list((folder/"ASX/Logs").glob("*/game-output.log"));text=logs[0].read_text(errors="replace")if logs else"";native.append(text)
            checks[f"active_{slot}"]=bool(list((folder/"ASX/Logs").glob("*/ready.json"))) and "ASX is active"in(output/f"launcher-{slot}.log").read_text(errors="replace")
            checks[f"native_joined_{slot}"]='"state":"joined"'in text
            checks[f"native_profile_{slot}"]=("uiwarrior","uipaladin","uidruid")[slot-1]in text.lower()
            checks[f"scripts_clean_{slot}"]=assess(text)["passed"]
            checks[f"backup_{slot}"]=bool(list((folder/"ASX/Backups/Progression").glob("*.txt")))
            checks[f"alive_{slot}"]=children[slot].poll()is None
            stop_game(folder)
        for p in children[1:]:p.wait(timeout=20)
        checks["gameplay_started"]=all('"type":"start"'in text and '"type":"choices"'in text and '"type":"resume"'in text for text in native)
        # Real self-uninstall on a disposable installation, preserve user files.
        folder=folders[-1];unknown=folder/"ASX/Client/user-note.txt";unknown.write_text("keep")
        save=folder/"profile/appdata/wenilla-rogue/characters.txt";home=folder/"benilla-config/config.toml";before=(sha(save),sha(home),sha(folder/"AzerothSurvivors.exe"))
        cmd=[str(folder/"ASX/Runtime/python.exe"),"-I","-B",str(folder/"ASX/Client/launch.py"),"--export-diagnostics"]
        report=subprocess.run(cmd,cwd=folder,env=environments[-1],capture_output=True,timeout=30);(output/"export.log").write_bytes(report.stdout+report.stderr)
        checks["export_local"]=report.returncode==0 and bool(list((folder/"ASX/Reports").glob("*.zip")))
        cmd[-1]="--uninstall"
        removal=subprocess.run(cmd,cwd=folder,env=environments[-1],capture_output=True,timeout=30);(output/"uninstall.log").write_bytes(removal.stdout+removal.stderr)
        match=re.search(r"Removal report: (.+result.json)",removal.stdout.decode(errors="replace"))
        result_path=Path(match[1].strip())if match else output/"missing-result"
        until=time.monotonic()+40
        while not result_path.exists()and time.monotonic()<until:time.sleep(.2)
        outcome=json.loads(result_path.read_text(encoding="utf-8-sig"))if result_path.exists()else{}
        (output/"uninstall-result.json").write_text(json.dumps(outcome,indent=2))
        checks["uninstall_completed"]=removal.returncode==0 and outcome.get("completed",False) and not(folder/"avrt.dll").exists()and not(folder/"start-asx.cmd").exists()and not(folder/"ASX/Runtime/python.exe").exists()
        checks["uninstall_preserved"]=unknown.read_text()=="keep"and before==(sha(save),sha(home),sha(folder/"AzerothSurvivors.exe"))and bool(list((folder/"ASX/Backups/Progression").glob("*.txt")))
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
