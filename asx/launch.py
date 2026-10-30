"""Player launcher: same native profile, exact game build, owned ASX files only."""
import argparse,json,os,shutil,subprocess,sys,threading,time,uuid,zipfile
from contextlib import contextmanager
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
import core,updates
@contextmanager
def operation(label):
    print("[ ASX ] "+label+"...",flush=True);stop=threading.Event();started=time.monotonic()
    def beat():
        while not stop.wait(5):print(f"[ ASX ] {label}... still working ({int(time.monotonic()-started)}s)",flush=True)
    t=threading.Thread(target=beat,daemon=True);t.start()
    try:yield
    finally:stop.set();t.join()
def optional_event_enabled(root,key,label):
    """Optional gameplay settings never prevent the cooperative launcher starting."""
    try:
        path=core.path(root,"ASX/options.json")
        if not path.exists():return True
        if path.stat().st_size>4096:raise ValueError("options file is too large")
        value=core.read_json(path.read_bytes())
        if not isinstance(value,dict):raise ValueError("options must be an object")
        enabled=value.get(key,True)
        if not isinstance(enabled,bool):raise ValueError(key+" must be true or false")
        return enabled
    except (ValueError,OSError) as error:
        print("[ ASX ] "+label+" disabled: "+str(error)+". Other ASX features remain enabled.",flush=True)
        return False

def treasure_goblins_enabled(root):
    return optional_event_enabled(root,"treasure_goblins","Treasure Goblins")

def child_environment(root,env,session):
    result={k:v for k,v in env.items()if not k.upper().startswith(("AZCOOP_","COOP_","ASX_"))}
    result.update(ASX_ACTIVE="1",AZCOOP_ENGINE="official",AZCOOP_NATIVE_LOG_DIRECTORY=str(session),ASX_READY_FILE=str(session/"ready.json"),WOW_ROGUE_UPDATE="0",WOW_ROGUE_UPDATE_AUTO="0")
    result["ASX_TREASURE_GOBLINS"]="1" if treasure_goblins_enabled(root) else "0"
    result["ASX_LOST_SUPPLIES"]="1" if optional_event_enabled(root,"lost_supplies","Lost Supplies") else "0"
    result["ASX_DARKMOON_TRIALS"]="1" if optional_event_enabled(root,"darkmoon_trials","Sayge's Trial") else "0"
    return result
def backup(root,save):
    if not save.is_file():return None
    target=core.path(root,"ASX/Backups/Progression/"+time.strftime("%Y%m%d-%H%M%S-")+uuid.uuid4().hex[:8]+".txt")
    core.atomic(target,save.read_bytes());return target

def export(root):
    folder=core.path(root,"ASX/Logs");target=core.path(root,"ASX/Reports/asx-diagnostics-"+time.strftime("%Y%m%d-%H%M%S")+".zip")
    target.parent.mkdir(parents=True,exist_ok=True)
    with zipfile.ZipFile(target,"w",compression=zipfile.ZIP_DEFLATED)as z:
        z.writestr("ASX-version.json",core.encoded({"product":core.PRODUCT,"version":core.manifest(root)["version"],"game":core.manifest(root)["game"]}))
        if folder.exists():
            for session in sorted(folder.iterdir(),key=lambda p:p.stat().st_mtime,reverse=True)[:3]:
                if session.is_dir() and not session.is_symlink():
                    for f in session.iterdir():
                        if f.is_file()and not f.is_symlink()and f.suffix in(".log",".json")and f.stat().st_size<=16<<20:z.write(core.path(root,"ASX/Logs/"+session.name+"/"+f.name),"Logs/"+session.name+"/"+f.name)
    print("[ ASX ] Diagnostic report created locally: "+str(target)+". Review it before sharing; nothing was uploaded.",flush=True)
    return target

def run(root,offline=False,check=False,update_only=False):
    root=Path(root).resolve();save,home=core.native_paths(root,os.environ)
    if any(os.environ.get(k)for k in("WOW_CAPTURE","WOW_UNATTENDED","WOW_PROBE","WOW_RIG")):raise ValueError("Remove unattended/capture environment overrides before launching ASX.")
    if core.game_running(root):raise ValueError("Close this native game before launching, updating or removing ASX.")
    with core.profile_lock(save),core.locked(root):
        if core.recover(root):
            # Recovery can replace this process's imported client modules.
            # Re-enter through CMD before validating or launching restored files.
            print("[ ASX ] Restarting the recovered launcher...",flush=True)
            return 75
        m=core.manifest(root)
        with operation("Checking ASX files"):data=core.sources(root,m)
        actual=core.inspect(root)
        core.adopt(root,m,data)
        if actual==m["game"]["sha256"]:
            with operation("Preparing ASX in this game folder"):core.apply(root,m,data,manual=True)
        if not offline:
            try:
                with operation("Checking signed ASX updates"):status=updates.automatic(root,actual)
                print("[ ASX ] Updates: "+status+".",flush=True)
                if status=="installed":print("[ ASX ] Restarting the updated launcher...",flush=True);return 75
            except Exception as e:print("[ ASX ] Update unavailable: "+str(e)+". Keeping the installed version.",flush=True)
            m=core.manifest(root);data=core.sources(root,m)
        with operation("Checking native game compatibility"):core.inspect(root,m)
        if check or update_only:print("[ ASX ] Ready. Native progression and settings paths are unchanged.",flush=True);return 0
        with operation("Backing up native progression"):backup(root,save)
        session=core.path(root,"ASX/Logs/"+time.strftime("%Y%m%d-%H%M%S-")+uuid.uuid4().hex[:8]);session.mkdir(parents=True)
        core.atomic(session/"session.json",core.encoded({"product":core.PRODUCT,"version":m["version"],"game_sha256":actual,"profile":"native","started":time.strftime("%Y-%m-%dT%H:%M:%S%z")}))
        out=(session/"game-output.log").open("wb")
        child=None
        try:
            with operation("Starting Azeroth Survivors Extended"):
                child=subprocess.Popen([str(root/"AzerothSurvivors.exe")],cwd=root,env=child_environment(root,os.environ,session),stdout=out,stderr=subprocess.STDOUT)
                metadata=core.read_json((session/"session.json").read_bytes());metadata["game_pid"]=child.pid
                core.atomic(session/"session.json",core.encoded(metadata))
                until=time.monotonic()+45
                while time.monotonic()<until:
                    if child.poll()is not None:raise ValueError("The game closed during startup. See ASX/Logs.")
                    ready=session/"ready.json"
                    if ready.exists():
                        try:ack=core.read_json(ready.read_bytes())
                        except (ValueError,OSError):time.sleep(.1);continue
                        if ack=={"product":"asx","game_sha256":actual}:break
                        raise ValueError("Unexpected ASX startup acknowledgement")
                    time.sleep(.1)
                else:raise ValueError("ASX could not activate safely. See ASX/Logs. The owned game process will close.")
            print("[ ASX ] ASX is active. Progression and settings are shared with the native game.",flush=True)
            code=child.wait();print("[ ASX ] Game closed. Logs: "+str(session),flush=True);return code
        finally:
            if child and child.poll()is None:
                child.terminate()
                try:child.wait(timeout=15)
                except subprocess.TimeoutExpired:child.kill();child.wait(timeout=5)
            out.close()

def main():
    parser=argparse.ArgumentParser(description="Azeroth Survivors Extended")
    actions=parser.add_mutually_exclusive_group()
    for flag in("check","update-only","rollback","uninstall","export-diagnostics"):actions.add_argument("--"+flag,action="store_true")
    parser.add_argument("--offline",action="store_true",help="Skip the update check")
    a=parser.parse_args();root=Path(__file__).resolve().parents[2]
    try:
        if a.export_diagnostics:export(root);return 0
        if a.rollback or a.uninstall:
            if core.game_running(root):raise ValueError("Close the game first.")
            with core.locked(root):
                core.recover(root)
                if a.rollback:print("[ ASX ] Previous version "+core.rollback(root)+". Automatic reinstallation is held until a newer release.",flush=True)
                else:
                    import remove
                    remove.uninstall(root)
            return 0
        return run(root,a.offline,a.check,a.update_only)
    except Exception as e:print("[ ERROR ] "+str(e),flush=True);return 1
if __name__=="__main__":raise SystemExit(main())
