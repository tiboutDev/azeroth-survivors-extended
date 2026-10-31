"""ASX owned-file installation. Native assets, saves and preferences are never payloads."""
from contextlib import contextmanager
import base64,ctypes,hashlib,json,os,re,shutil,subprocess,tempfile,time,uuid
from pathlib import Path,PurePosixPath
PRODUCT="azeroth-survivors-extended"
MANIFEST="ASX/manifest.json"
STATE="ASX/State/install.json"
ASXUI_FILES=('ASXUI.toc', 'ASXUI.lua', 'Minimap.lua', 'Events.lua', 'Buffs.lua', 'Fonts/ASXUI-Numbers.ttf', 'Fonts/OFL.txt', 'LICENSE', 'README.md')
ASXUI_SOURCE="ASX/Interface/ASXUI/"
ASXUI_TARGET="benilla-config/AddOns/ASXUI/"
def ui_target(name):return name in {ASXUI_TARGET+n for n in ASXUI_FILES}
def projections(m):
    result={"avrt.dll":"ASX/Bin/avrt.dll","benilla-config/AddOns/ASX/ASX.toc":"ASX/Interface/ASX.toc","benilla-config/AddOns/ASX/Presentation.lua":"ASX/Interface/Presentation.lua"}
    result.update({ASXUI_TARGET+n:ASXUI_SOURCE+n for n in ASXUI_FILES if ASXUI_SOURCE+n in m["files"]})
    return result
PRIVATE={"state","logs","backups","updates","settings"}
RESERVED={"con","prn","aux","nul",*(f"com{i}" for i in range(1,10)),*(f"lpt{i}" for i in range(1,10))}
VERSION=re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{0,63}\Z")
SHA=re.compile(r"[0-9a-f]{64}\Z")
def encoded(value):return (json.dumps(value,sort_keys=True,separators=(",",":"))+"\n").encode()
def read_json(data):
    def pairs(rows):
        out={}
        for k,v in rows:
            if k in out:raise ValueError("Duplicate JSON field")
            out[k]=v
        return out
    return json.loads(data,object_pairs_hook=pairs)
def digest(data):return hashlib.sha256(data).hexdigest()
def sha(path):
    h=hashlib.sha256()
    with Path(path).open("rb")as f:
        while b:=f.read(1<<20):h.update(b)
    return h.hexdigest()
def payload_name(name):
    if not isinstance(name,str)or len(name)>220 or not name.isascii():raise ValueError("Invalid ASX file path")
    parts=name.split("/")
    if any(not p or p in(".","..")or p[-1]in". "or any(ord(c)<32 or c in '\\:*?"<>|' for c in p)or p.split(".")[0].lower()in RESERVED for p in parts):raise ValueError("Unsafe ASX file path")
    if name not in("start-asx.cmd","README-ASX.txt",MANIFEST):
        if len(parts)<3 or parts[0]!="ASX"or parts[1].lower()in PRIVATE:raise ValueError("File is outside the ASX payload")
        if parts[1]not in("Client","Bin","Runtime","Interface","Defaults"):raise ValueError("Unknown ASX component")
    if name.lower().endswith((".mpq",".pem",".key",".log",".jsonl"))or "characters.txt"in name.lower()or "azerothsurvivors.exe"in name.lower():raise ValueError("Native or private files cannot be ASX payloads")
    return name
def owned_name(name):
    if ui_target(name)or name=="avrt.dll"or name in("benilla-config/AddOns/ASX/ASX.toc","benilla-config/AddOns/ASX/Presentation.lua"):return name
    return payload_name(name)
def path(root,name):
    root=Path(root).resolve();out=root.joinpath(*name.split("/"))
    for p in(out,*out.parents):
        if p==root:break
        try:link=p.is_symlink()or bool(getattr(p.lstat(),"st_file_attributes",0)&0x400)
        except FileNotFoundError:link=False
        if link:raise ValueError("ASX will not follow links: "+name)
    if root not in out.resolve().parents:raise ValueError("ASX path escaped the game folder")
    return out
def atomic(target,data):
    target=Path(target);target.parent.mkdir(parents=True,exist_ok=True)
    fd,tmp=tempfile.mkstemp(prefix=".asx-write-",dir=target.parent)
    try:
        with os.fdopen(fd,"wb")as f:f.write(data);f.flush();os.fsync(f.fileno())
        os.replace(tmp,target)
    finally:Path(tmp).unlink(missing_ok=True)
def put(root,name,value):atomic(path(root,name),encoded(value))
def get(root,name,default=None):
    p=path(root,name)
    return read_json(p.read_bytes())if p.exists()else default
def validate_native_archives(archives):
    """Native dependencies only: flat ASCII MPQ names, never mod payload paths.
    Signed releases may add archive names without updating the installed parser.
    Keep Windows device names, case aliases and traversal out on every platform.
    """
    if not isinstance(archives,list)or not 1<=len(archives)<=1024:
        raise ValueError("Invalid native archive inventory")
    seen=set()
    for name in archives:
        if not isinstance(name,str)or not re.fullmatch(r"Data/[A-Za-z0-9][A-Za-z0-9_-]{0,119}\.[Mm][Pp][Qq]",name):
            raise ValueError("Invalid native archive inventory")
        stem=name[5:].rsplit('.',1)[0].lower()
        canonical=name.casefold()
        if stem in RESERVED or canonical in seen:
            raise ValueError("Invalid native archive inventory")
        seen.add(canonical)
    return archives
def supported_native_games():
    """Load the same reviewed native registry used by the archive builder."""
    registry=read_json(Path(__file__).with_name("supported-games.json").read_bytes())
    games=registry.get("games") if isinstance(registry,dict) and registry.get("schema")==1 else None
    if not isinstance(games,dict) or not games:
        raise ValueError("Missing reviewed native game registry")
    return games
def validate(m,reviewed=True):
    if not isinstance(m,dict)or m.get("schema")!=1 or m.get("product")!=PRODUCT or not isinstance(m.get("version"),str) or not VERSION.fullmatch(m["version"]):raise ValueError("Not an ASX distribution manifest")
    game=m.get("game",{})
    if not isinstance(game,dict)or not isinstance(game.get("sha256"),str) or not SHA.fullmatch(game["sha256"])or not isinstance(game.get("version"),str):raise ValueError("Missing exact native game compatibility")
    known=supported_native_games().get(game["version"]) if reviewed else None
    if reviewed and (not isinstance(known,dict) or game["sha256"]!=known.get("sha256")):
        raise ValueError("ASX supports only an exact reviewed native game build")
    archives=game.get("archives")
    validate_native_archives(archives)
    files=m.get("files");seen=set();total=0
    if not isinstance(files,dict)or not 1<=len(files)<=6000:raise ValueError("Invalid ASX inventory")
    for name,spec in files.items():
        payload_name(name)
        if name==MANIFEST or name.lower()in seen:raise ValueError("Duplicate or self-referential ASX file")
        seen.add(name.lower())
        if not isinstance(spec,dict)or type(spec.get("size"))is not int or not 0<=spec["size"]<=64<<20 or not isinstance(spec.get("sha256"),str) or not SHA.fullmatch(spec["sha256"]):raise ValueError("Invalid ASX file size/hash")
        total+=spec["size"]
    if total>256<<20:raise ValueError("ASX payload too large")
    required={"start-asx.cmd","README-ASX.txt","ASX/Bin/avrt.dll","ASX/Client/launch.py","ASX/Client/supported-games.json","ASX/Interface/ASX.toc","ASX/Interface/Presentation.lua","ASX/Defaults/channel.json"}
    if not required<=files.keys():raise ValueError("Incomplete ASX distribution")
    ui_files={n for n in files if n.startswith(ASXUI_SOURCE)}
    if ui_files and ui_files!={ASXUI_SOURCE+n for n in ASXUI_FILES}:raise ValueError("Incomplete or unknown bundled ASXUI inventory")
    return m
def manifest(root):return validate(get(root,MANIFEST,{}))
def inspect(root,m=None):
    exe=path(root,"AzerothSurvivors.exe")
    if not exe.is_file():raise ValueError("Extract ASX beside AzerothSurvivors.exe, then run start-asx.cmd.")
    actual=sha(exe)
    if m:
        if actual!=m["game"]["sha256"]:raise ValueError("ASX requires native game "+m["game"]["version"]+" (the exact supported build). Start the official game normally or wait for a compatible ASX release.")
        for name in m["game"]["archives"]:
            if not path(root,name).is_file():raise ValueError("Required native game file missing: "+name)
    return actual
def file_role(name):
    if name in ("README-ASX.txt","ASX/Client/LICENSE") or any(name==prefix+suffix for prefix in (ASXUI_SOURCE,ASXUI_TARGET)for suffix in ("README.md","LICENSE","Fonts/OFL.txt")):return "documentation"
    if name=="ASX/Defaults/channel.json":return "update-metadata"
    if name=="start-asx.cmd":return "bootstrap"
    return "critical"

def sources(root,m,tolerant=False):
    out={}
    for name,spec in m["files"].items():
        p=path(root,name)
        if not p.is_file()or p.stat().st_size!=spec["size"]or sha(p)!=spec["sha256"]:
            if tolerant and file_role(name)!="critical":continue
            raise ValueError("ASX file is missing or changed: "+name+". Run --repair or --check.")
        out[name]=p.read_bytes()
    return out
def projection(data,m):
    out=dict(data);out[MANIFEST]=encoded(m)
    for target,source in projections(m).items():
        if source in data:out[target]=data[source]
    return out
def adopt(root,m,data):
    """Record the explicitly extracted payload without activating an unsupported game."""
    if get(root,STATE)is not None:return
    owned={n:digest(b)for n,b in data.items()};owned[MANIFEST]=sha(path(root,MANIFEST))
    for name,b in projection(data,m).items():
        target=path(root,name)
        if name not in owned and target.exists():
            if sha(target)!=digest(b):
                if ui_target(name):continue # Migration needs explicit consent in apply, never silent adoption.
                raise ValueError("Existing loader/addon belongs to another installation; preserved: "+name)
            owned[name]=digest(b)
    put(root,STATE,{"schema":1,"version":m["version"],"owned":owned,"game":m["game"]})
def installed(root):
    s=get(root,STATE,{"schema":1,"owned":{}})
    if s.get("schema")!=1 or not isinstance(s.get("owned"),dict):raise ValueError("Invalid ASX installation receipt")
    for name,h in s["owned"].items():
        owned_name(name)
        if not SHA.fullmatch(h):raise ValueError("Invalid ASX ownership hash")
    return s
@contextmanager
def locked(root):
    p=path(root,"ASX/State/installation.lock");p.parent.mkdir(parents=True,exist_ok=True)
    with p.open("a+b")as f:
        if f.tell()==0:f.write(b"0");f.flush()
        f.seek(0)
        try:
            if os.name=="nt":
                import msvcrt;msvcrt.locking(f.fileno(),msvcrt.LK_NBLCK,1)
            else:
                import fcntl;fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB)
        except OSError as e:raise ValueError("Another ASX operation is already running.")from e
        try:yield
        finally:
            f.seek(0)
            if os.name=="nt":msvcrt.locking(f.fileno(),msvcrt.LK_UNLCK,1)
            else:fcntl.flock(f,fcntl.LOCK_UN)
def restore(root,tx):
    if not re.fullmatch(r"[0-9a-f]{32}",tx):raise ValueError("Invalid ASX transaction")
    base="ASX/Updates/Transactions/"+tx
    j=get(root,base+"/journal.json")
    for name,spec in j["before"].items():
        owned_name(name);target=path(root,name)
        if spec is None:target.unlink(missing_ok=True)
        else:
            source=path(root,base+"/files/"+name)
            if sha(source)!=spec:raise ValueError("ASX rollback backup is damaged: "+name)
            atomic(target,source.read_bytes())
    if j["state"]is None:path(root,STATE).unlink(missing_ok=True)
    else:put(root,STATE,j["state"])
    j["phase"]="restored";put(root,base+"/journal.json",j)
def recover(root):
    folder=path(root,"ASX/Updates/Transactions")
    if not folder.exists():return False
    restored=False
    for d in sorted(folder.iterdir()):
        if re.fullmatch(r"[0-9a-f]{32}",d.name):
            j=get(root,"ASX/Updates/Transactions/"+d.name+"/journal.json",{})
            if j.get("phase")=="applying":
                restore(root,d.name)
                restored=True
    return restored
def apply(root,m,data,manual=False,fault=lambda n:None,launch=False,trusted=False,approve=None,repair=False):
    validate(m,reviewed=not trusted);inspect(root,m)
    expected_files={n for n in m["files"] if not launch or file_role(n)=="critical"}
    if not expected_files<=set(data) or (not launch and set(data)!=set(m["files"])):raise ValueError("ASX package inventory differs")
    for name,b in data.items():
        if len(b)!=m["files"][name]["size"]or digest(b)!=m["files"][name]["sha256"]:raise ValueError("ASX package file failed verification: "+name)
    new=projection(data,m);previous=get(root,STATE);old=installed(root)["owned"]
    if launch:
        new={n:b for n,b in new.items() if file_role(n)=="critical"or(ui_target(n)and not path(root,n).exists())}
    preserved={};conflicts=[]
    for name in set(old)|set(new):
        target=path(root,owned_name(name))
        if not target.exists():continue
        current=sha(target)
        expected=digest(new[name])if name in new else None
        if name in old:
            if current!=old[name]and not(manual and expected==current):
                if file_role(name)!="critical":
                    preserved[name]=current
                    if repair and file_role(name)=="documentation":new.pop(name,None)
                    continue
                if approve is None:raise ValueError("A managed ASX file was changed; preserved: "+name)
                conflicts.append(name)
        elif expected!=current:
            # Lost receipts can be recovered for internal ASX files with consent,
            # never by adopting an unknown root loader or projected addon.
            if (repair and name.startswith("ASX/") or ui_target(name)and file_role(name)=="critical") and approve is not None:conflicts.append(name)
            elif file_role(name)!="critical":
                preserved[name]=current
                if repair and file_role(name)=="documentation":new.pop(name,None)
            else:raise ValueError("Existing loader/addon/file belongs to another installation; preserved: "+name)
    if conflicts and not approve(conflicts):raise ValueError("Replacement cancelled; changed files preserved.")
    if launch:
        preserved.update({n:h for n,h in old.items() if file_role(n)!="critical"})
    for name in set(conflicts)|set(preserved):
        if name in new and path(root,name).exists() and sha(path(root,name))!=digest(new[name]):
            atomic(path(root,"ASX/Backups/Files/"+uuid.uuid4().hex+"/"+name),path(root,name).read_bytes())
    changes=[n for n,b in new.items()if not path(root,n).exists()or sha(path(root,n))!=digest(b)]
    removed=sorted(set(old)-set(new)-set(preserved))
    if os.name=="nt"and any(n.startswith("ASX/Runtime/")for n in changes+removed):raise ValueError("This update changes the launcher runtime. Close ASX and extract the complete archive manually.")
    if not changes and not removed and previous and previous.get("version")==m["version"]:return "current"
    tx=uuid.uuid4().hex;base="ASX/Updates/Transactions/"+tx;before={}
    for name in changes+removed:
        target=path(root,name)
        if target.exists():
            b=target.read_bytes();before[name]=digest(b);atomic(path(root,base+"/files/"+name),b)
        else:before[name]=None
    journal={"phase":"applying","before":before,"state":previous}
    put(root,base+"/journal.json",journal)
    try:
        for name in changes:atomic(path(root,name),new[name]);fault(name)
        for name in removed:path(root,name).unlink(missing_ok=True);fault(name)
        put(root,STATE,{"schema":1,"version":m["version"],"owned":{**preserved,**{n:digest(b)for n,b in new.items()}},"rollback":((previous or {}).get("rollback") if (previous or {}).get("version")==m["version"] else (None if manual else tx)),"game":m["game"]})
        fault(STATE);journal["phase"]="committed";put(root,base+"/journal.json",journal)
    except BaseException:
        restore(root,tx);raise
    return "installed"
def rollback(root):
    state=installed(root);tx=state.get("rollback")
    if not tx:raise ValueError("There is no previous ASX version to restore.")
    for name,h in state["owned"].items():
        p=path(root,name)
        if p.exists()and sha(p)!=h:
            if file_role(name)=="critical":raise ValueError("Changed file preserved; rollback refused: "+name)
            atomic(path(root,"ASX/Backups/Files/"+uuid.uuid4().hex+"/"+name),p.read_bytes())
    j=get(root,"ASX/Updates/Transactions/"+tx+"/journal.json")
    if not j.get("state")or j["state"].get("version")==state.get("version"):raise ValueError("There is no previous ASX version to restore.")
    for name,expected in j["before"].items():
        p=path(root,owned_name(name))
        if file_role(name)!="critical" and p.is_file() and sha(p)!=expected:
            atomic(path(root,"ASX/Backups/Files/"+uuid.uuid4().hex+"/"+name),p.read_bytes())
    restore(root,tx)
    history=get(root,"ASX/State/updates.json",{"sequence":0})
    history["held_sequence"]=history["sequence"]
    put(root,"ASX/State/updates.json",history)
    return "restored"
def native_paths(root,env):
    root=Path(root).resolve()
    home=Path(env.get("BENILLA_HOME")or root/"benilla-config").resolve()
    if home!=(root/"benilla-config").resolve():raise ValueError("ASX currently requires the native benilla-config folder beside the game. Remove a custom BENILLA_HOME override before launching.")
    save=Path(env["WOW_ROGUE_SAVE"])if env.get("WOW_ROGUE_SAVE")else Path(env["APPDATA"])/"wenilla-rogue/characters.txt"
    return save.resolve(),home
@contextmanager
def profile_lock(save):
    if os.name!="nt":yield;return
    k=ctypes.WinDLL("kernel32",use_last_error=True);k.CreateMutexW.argtypes=[ctypes.c_void_p,ctypes.c_int,ctypes.c_wchar_p];k.CreateMutexW.restype=ctypes.c_void_p
    name="Local\\ASXProfile-"+digest(str(save).casefold().encode())
    handle=k.CreateMutexW(None,1,name)
    if not handle:raise ctypes.WinError(ctypes.get_last_error())
    if ctypes.get_last_error()==183:k.CloseHandle(ctypes.c_void_p(handle));raise ValueError("Another ASX game is using this progression profile.")
    try:yield
    finally:k.ReleaseMutex(ctypes.c_void_p(handle));k.CloseHandle(ctypes.c_void_p(handle))
def game_running(root):
    if os.name!="nt":return False
    # Only process identity is inspected, never another process's memory/environment.
    from ctypes import wintypes as W
    k=ctypes.WinDLL("kernel32",use_last_error=True)
    class Entry(ctypes.Structure):_fields_=[("size",W.DWORD),("usage",W.DWORD),("pid",W.DWORD),("heap",ctypes.c_size_t),("module",W.DWORD),("threads",W.DWORD),("parent",W.DWORD),("priority",W.LONG),("flags",W.DWORD),("exe",W.WCHAR*260)]
    k.CreateToolhelp32Snapshot.argtypes=[W.DWORD,W.DWORD];k.CreateToolhelp32Snapshot.restype=W.HANDLE
    k.Process32FirstW.argtypes=[W.HANDLE,ctypes.POINTER(Entry)];k.Process32NextW.argtypes=k.Process32FirstW.argtypes
    k.OpenProcess.argtypes=[W.DWORD,W.BOOL,W.DWORD];k.OpenProcess.restype=W.HANDLE
    k.QueryFullProcessImageNameW.argtypes=[W.HANDLE,W.DWORD,W.LPWSTR,ctypes.POINTER(W.DWORD)]
    k.CloseHandle.argtypes=[W.HANDLE]
    snapshot=k.CreateToolhelp32Snapshot(2,0);entry=Entry();entry.size=ctypes.sizeof(Entry)
    if snapshot==W.HANDLE(-1).value:raise ctypes.WinError(ctypes.get_last_error())
    try:
        more=k.Process32FirstW(snapshot,ctypes.byref(entry))
        while more:
            if entry.exe.lower()=="azerothsurvivors.exe":
                h=k.OpenProcess(0x1000,False,entry.pid)
                if not h:return True
                try:
                    buf=ctypes.create_unicode_buffer(32768);n=W.DWORD(len(buf))
                    if not k.QueryFullProcessImageNameW(h,0,buf,ctypes.byref(n)):return True
                    if Path(buf.value).resolve()==Path(root).resolve()/"AzerothSurvivors.exe":return True
                finally:k.CloseHandle(h)
            more=k.Process32NextW(snapshot,ctypes.byref(entry))
    finally:k.CloseHandle(snapshot)
    return False
