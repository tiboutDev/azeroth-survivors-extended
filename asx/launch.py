"""Player launcher: same native profile, exact game build, owned ASX files only."""
import argparse,json,os,shutil,subprocess,sys,threading,time,uuid,zipfile
from contextlib import contextmanager,ExitStack
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
import core,updates,maintenance,layout
INPUT_ACTIVE=threading.Event()
@contextmanager
def operation(label):
    print("[ ASX ] "+label+"...",flush=True);stop=threading.Event();started=time.monotonic()
    def beat():
        while not stop.wait(5):
            if not INPUT_ACTIVE.is_set():print(f"[ ASX ] {label}... still working ({int(time.monotonic()-started)}s)",flush=True)
    t=threading.Thread(target=beat,daemon=True);t.start()
    try:yield
    finally:stop.set();t.join()
def optional_event_enabled(root,key,label):
    """Optional gameplay settings never prevent the cooperative launcher starting."""
    try:
        path=core.path(root,"ASX/Config/options.json")
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

def disable_owned_ui(root):
    """Only disable a verifiably owned TOC; retain user code and SavedVariables."""
    name=core.ASXUI_TARGET+'ASXUI.toc';p=core.path(root,name)
    try:owned=core.installed(root)['owned']
    except (ValueError,OSError,KeyError,TypeError):return
    if p.is_file()and name in owned and core.sha(p)==owned[name]:
        target=core.path(root,'ASX/Data/Backups/DisabledUI/'+uuid.uuid4().hex+'/ASXUI.toc')
        core.atomic(target,p.read_bytes());p.unlink()

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
    target=core.path(root,"ASX/Data/Backups/Progression/"+time.strftime("%Y%m%d-%H%M%S-")+uuid.uuid4().hex[:8]+".txt")
    core.atomic(target,save.read_bytes());return target

def export(root):
    folder=core.path(root,"ASX/Data/Logs");target=core.path(root,"ASX/Data/Reports/asx-diagnostics-"+time.strftime("%Y%m%d-%H%M%S")+".zip")
    target.parent.mkdir(parents=True,exist_ok=True)
    with zipfile.ZipFile(target,"w",compression=zipfile.ZIP_DEFLATED)as z:
        try:
            m=core.manifest(root);metadata={"product":core.PRODUCT,"version":m["version"],"game":m["game"]}
        except (ValueError,OSError,KeyError,TypeError) as error:metadata={"product":core.PRODUCT,"metadata_unavailable":str(error)}
        z.writestr("ASX-version.json",core.encoded(metadata))
        z.writestr("installation-check.json",core.encoded(maintenance.diagnose(root)))
        if folder.exists():
            for session in sorted(folder.iterdir(),key=lambda p:p.stat().st_mtime,reverse=True)[:3]:
                if session.is_dir() and not session.is_symlink():
                    for f in session.iterdir():
                        if f.is_file()and not f.is_symlink()and f.suffix in(".log",".json")and f.stat().st_size<=16<<20:z.write(core.path(root,"ASX/Data/Logs/"+session.name+"/"+f.name),"Logs/"+session.name+"/"+f.name)
    print("[ ASX ] Diagnostic report created locally: "+str(target)+". Review it before sharing; nothing was uploaded.",flush=True)
    return target

def pending_recovery(root):
    folder=core.path(root,"ASX/Data/Updates/Transactions")
    if not folder.exists():return False
    for d in folder.iterdir():
        if core.re.fullmatch(r"[0-9a-f]{32}",d.name):
            if core.get(root,"ASX/Data/Updates/Transactions/"+d.name+"/journal.json",{}).get("phase")=="applying":return True
    return False


def run(root,offline=False,check=False,update_only=False,confirm_update=None,retry=False):
    root=Path(root).resolve()
    if check:
        report=maintenance.diagnose(root);maintenance.print_report(report)
        return 0 if report["ready"] else 1
    if core.game_running(root):raise ValueError("Close this native game before launching or updating ASX.")
    with ExitStack() as stack:
        stack.enter_context(core.locked(root))
        if pending_recovery(root):
            if not confirm_repair(["Recover interrupted installation"]):raise ValueError("Recovery required; use --recover.")
            if core.recover(root):
                print("[ ASX ] Restarting the recovered launcher...",flush=True)
                return 75
        callback=None # Compatible signed updates never ask for confirmation.
        if update_only:
            if offline:raise ValueError("Updates need a network connection. Use --repair with a signed local archive for offline repair.")
            try:status=updates.automatic(root,core.inspect(root),confirm=callback,retry=retry,approve=confirm_files)
            except UpdateCancelled:
                print("[ ASX ] Update cancelled.",flush=True);return 0
            print("[ ASX ] Update result: "+status+".",flush=True)
            return 0 if status=="handoff" else 75 if status=="installed" else (0 if status in ("current","deferred","disabled","no ASX release published") else 1)
        save,home=core.native_paths(root,os.environ)
        if any(os.environ.get(k)for k in("WOW_CAPTURE","WOW_UNATTENDED","WOW_PROBE","WOW_RIG")):
            print("[ WARNING ] Native unattended/capture overrides are active; game startup may differ.",flush=True)
        stack.enter_context(core.profile_lock(save))
        m=core.manifest(root)
        print("[ ASX ] ASX "+m["version"]+" | Supported game: "+m["game"]["version"],flush=True)
        actual=core.inspect(root)
        # An unsupported installed release may still discover an authenticated
        # compatible update, without projecting its old DLL into the new game.
        if not offline:
            try:
                with operation("Checking signed ASX updates"):
                    status=updates.automatic(root,actual,confirm=callback,approve=confirm_files)
                print("[ ASX ] Updates: "+status+".",flush=True)
                if status=="installed":return 75
                if status=="handoff":return 0
            except UpdateCancelled:
                print("[ ASX ] Launch cancelled. No update was installed.",flush=True);return 0
            except Exception as error:
                print("[ ASX ] Update failed or unavailable: "+str(error)+". Checking the installed version before continuing.",flush=True)
        core.inspect(root,m)
        with operation("Checking critical ASX files"):data=core.sources(root,m,tolerant=True,launch=True)
        optional={}
        for name in m['files']:
            if name.startswith(core.ASXUI_SOURCE):
                p=core.path(root,name);spec=m['files'][name]
                if not p.is_file()or p.stat().st_size!=spec['size']or core.sha(p)!=spec['sha256']:
                    if core.file_role(name)=='documentation':continue
                    disable_owned_ui(root);optional={};print('[ WARNING ] ASXUI unavailable; native interface remains enabled.',flush=True);break
                optional[name]=p.read_bytes()
        data.update(optional)
        projected=core.projection(data,m)
        needs_preparation=any((core.file_role(n)=="critical"or(core.ui_target(n)and not core.path(root,n).exists())) and
            (not core.path(root,n).is_file() or core.sha(core.path(root,n))!=core.digest(b))
            for n,b in projected.items())
        try:version_changed=core.installed(root).get("version")!=m["version"]
        except (ValueError,OSError,TypeError,KeyError):version_changed=False
        if needs_preparation or version_changed:
            core.adopt(root,m,data)
            with operation("Preparing owned ASX files"):core.apply(root,m,data,manual=True,launch=True,approve=lambda names:all(core.ui_target(n)for n in names)and confirm_files(names))
        report=maintenance.launch_report(root,m);maintenance.print_report(report)
        if not report["ready"]:raise ValueError("Repair required. Use --repair; --check shows all problems.")
        try:
            with operation("Backing up native progression"):backup(root,save)
        except OSError as error:print("[ WARNING ] Progression backup unavailable: "+str(error),flush=True)
        session=core.path(root,"ASX/Data/Logs/"+time.strftime("%Y%m%d-%H%M%S-")+uuid.uuid4().hex[:8]);session.mkdir(parents=True)
        core.atomic(session/"session.json",core.encoded({"product":core.PRODUCT,"version":m["version"],"game_sha256":actual,"profile":"native","started":time.strftime("%Y-%m-%dT%H:%M:%S%z")}))
        out=(session/"game-output.log").open("wb")
        child=None
        try:
            with operation("Starting Azeroth Survivors Extended"):
                child=subprocess.Popen([str(root/"AzerothSurvivors.exe")],cwd=root,env=child_environment(root,os.environ,session),stdout=out,stderr=subprocess.STDOUT)
                metadata=core.read_json((session/"session.json").read_bytes());metadata["game_pid"]=child.pid
                core.atomic(session/"session.json",core.encoded(metadata))
                until=time.monotonic()+120
                while time.monotonic()<until:
                    if child.poll()is not None:raise ValueError("The game closed during startup. See ASX/Data/Logs.")
                    ready=session/"ready.json"
                    if ready.exists():
                        try:ack=core.read_json(ready.read_bytes())
                        except (ValueError,OSError):time.sleep(.1);continue
                        if ack=={"product":"asx","game_sha256":actual}:break
                        raise ValueError("Unexpected ASX startup acknowledgement")
                    time.sleep(.1)
                else:raise ValueError("ASX could not activate safely. See ASX/Data/Logs. The owned game process will close.")
            print("[ ASX ] ASX is active. Progression and settings are shared with the native game.",flush=True)
            code=child.wait();print("[ ASX ] Game closed. Logs: "+str(session),flush=True);return code
        finally:
            if child and child.poll()is None:
                child.terminate()
                try:child.wait(timeout=15)
                except subprocess.TimeoutExpired:child.kill();child.wait(timeout=5)
            out.close()

class UpdateCancelled(Exception):
    """Player cancellation before downloading or applying an update."""


def choose(prompt,choices,default=None):
    while True:
        INPUT_ACTIVE.set()
        try:answer=input(prompt).strip()
        except (EOFError,KeyboardInterrupt):
            print()
            return None
        finally:INPUT_ACTIVE.clear()
        if not answer and default is not None:return default
        if answer in choices:return answer
        print("Please enter one of: "+", ".join(choices)+".",flush=True)


def confirm_files(names):
    print("[ ASX ] Changed ASX files will be backed up before replacement:",flush=True)
    for name in names:print("  "+name,flush=True)
    return sys.stdin.isatty() and choose("Replace these files? 1=Yes, 0=Cancel [0]: ",("1","0"),"0")=="1"


def confirm_repair(changes):
    return sys.stdin.isatty() and choose("Apply this repair? 1=Yes, 0=Cancel [0]: ",("1","0"),"0")=="1"


def menu(root):
    print("\nAZEROTH SURVIVORS EXTENDED")
    try:
        m=core.manifest(root)
        print("ASX "+m["version"]+" | Supported game: "+m["game"]["version"])
    except (ValueError,OSError) as error:
        print("Installation needs attention: "+str(error))
    print("\n  1. Play ASX (Enter)\n  2. Play ASX offline (skip update check)"
          "\n  3. Update ASX without starting the game\n  4. Check installation (offline)"
          "\n  5. Export diagnostic report\n  6. Restore previous ASX version"
          "\n  7. Uninstall ASX\n  8. Repair installed ASX version\n  9. Recover interrupted operation\n  0. Exit")
    answer=choose("Choose [1]: ",("1","2","3","4","5","6","7","8","9","0"),"1")
    if answer in (None,"0"):return None
    if answer in ("6","7"):
        message=("Restore the previous ASX version? The game and progression will not be rolled back."
                 if answer=="6" else "Remove ASX? Native progression and settings will be preserved.")
        print(message+"\n  1. Continue\n  0. Cancel")
        if choose("Choose [0]: ",("1","0"),"0")!="1":return "menu"
    return {"1":"play","2":"offline","3":"update-only","4":"check",
            "5":"export-diagnostics","6":"rollback","7":"uninstall","8":"repair","9":"recover"}[answer]


def execute(root,action,offline=False,interactive=False,archive=None,feed=None,signature=None):
    if action=="export-diagnostics":export(root);return 0
    if action=="repair":
        if core.game_running(root):raise ValueError("Close the game first.")
        with core.locked(root):
            if pending_recovery(root):raise ValueError("Recovery required before repair; use --recover.")
            result=maintenance.repair(root,confirm_repair,archive,feed,signature,offline=offline)
            print("[ ASX ] Repair result: "+result+".",flush=True)
            return 75 if result=="installed" else 0
    if action=="recover":
        if core.game_running(root):raise ValueError("Close the game first.")
        with core.locked(root):
            if not pending_recovery(root):print("[ ASX ] No interrupted operation.");return 0
            if not confirm_repair(["Restore interrupted transaction"]):return 0
            return 75 if core.recover(root) else 0
    if action in ("rollback","uninstall"):
        if core.game_running(root):raise ValueError("Close the game first.")
        with core.locked(root):
            if pending_recovery(root):raise ValueError("Recovery required first; use --recover.")
            if action=="rollback":print("[ ASX ] Previous version "+core.rollback(root)+". Automatic reinstallation is held until a newer release.",flush=True)
            else:
                import remove
                remove.uninstall(root)
        return 0
    return run(root,offline or action=="offline",action=="check",action in ("update-only","retry-update"),
               confirm_update=None,retry=action=="retry-update")


def main():
    parser=argparse.ArgumentParser(description="Azeroth Survivors Extended")
    actions=parser.add_mutually_exclusive_group()
    for flag in("play","menu","check","update-only","retry-update","repair","recover","rollback","uninstall","export-diagnostics"):
        actions.add_argument("--"+flag,action="store_true")
    parser.add_argument("--offline",action="store_true",help="Skip the update check")
    parser.add_argument("--archive",type=Path,help="Signed archive for same-version repair")
    parser.add_argument("--feed",type=Path,help="Signed release feed for local repair")
    parser.add_argument("--signature",type=Path,help="Detached signature for local repair")
    a=parser.parse_args()
    if any((a.archive,a.feed,a.signature)) and (not a.repair or not all((a.archive,a.feed,a.signature))):
        parser.error("Local repair requires --repair --archive --feed --signature together")
    root=Path(__file__).resolve().parents[3]
    if not a.check and (root/"AzerothSurvivors.exe").is_file():
        try:
            with core.locked(root):layout.migrate(root)
        except (ValueError,OSError)as error:
            print('[ ERROR ] ASX migration stopped; verified copies preserved: '+str(error),flush=True);return 1
    interactive=a.menu
    if a.menu and not sys.stdin.isatty():
        print("[ ERROR ] The menu requires an interactive console. Use --play or --check.",flush=True)
        return 1
    while True:
        try:
            action=menu(root) if interactive else next((name for name in
                ("check","update-only","retry-update","repair","recover","rollback","uninstall","export-diagnostics")
                if getattr(a,name.replace("-","_"))),"play")
            if action is None:return 0
            if action=="menu":continue
            code=execute(root,action,a.offline,interactive,a.archive,a.feed,a.signature)
            if code==75 or not interactive or action in ("play","offline","uninstall"):return code
        except Exception as error:
            print("[ ERROR ] "+str(error),flush=True)
            if not interactive:
                if action=="play" and sys.stdin.isatty():
                    print("  1. Repair installation\n  2. Open maintenance menu\n  0. Exit")
                    answer=choose("Choose [0]: ",("1","2","0"),"0")
                    if answer=="2":interactive=True;continue
                    if answer=="1":
                        try:return execute(root,"repair",a.offline)
                        except Exception as repair_error:print("[ ERROR ] "+str(repair_error),flush=True)
                return 1
            print("Use Check installation or Export diagnostic report for troubleshooting.",flush=True)
        except KeyboardInterrupt:
            print("\n[ ASX ] Cancelled.",flush=True);return 0

if __name__=="__main__":raise SystemExit(main())
