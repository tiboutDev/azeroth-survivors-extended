"""Bounded AZSX-generated history. Never prune preferences or user-file backups."""
import re
import shutil
import sys
from pathlib import Path
import core

SESSION=re.compile(r"[0-9]{8}-[0-9]{6}-[0-9a-f]{8}\Z")
TRANSACTION=re.compile(r"[0-9a-f]{32}\Z")
NATIVE_PENDING="AZSX/Data/State/native-update.json"

def children(root,name):
    folder=core.path(root,name)
    return list(folder.iterdir())if folder.is_dir()else []

def remove(root,path):
    """Validate the complete generated tree before removing anything."""
    path=core.path(root,path.relative_to(Path(root).resolve()).as_posix())
    if path.is_dir():
        for item in path.rglob("*"):core.raw_path(root,item.relative_to(root).as_posix())
        shutil.rmtree(path)
    else:path.unlink()

def clean(root):
    # The caller holds the installation lock and has recovered applying journals.
    root=Path(root).resolve()
    saves=sorted((p for p in children(root,"AZSX/Data/Backups/Progression")if SESSION.fullmatch(p.stem)and p.suffix==".txt"),key=lambda p:p.stat().st_mtime_ns,reverse=True)
    distinct=set()
    for p in saves:
        digest=core.sha(core.path(root,p.relative_to(root).as_posix()))
        if digest in distinct or len(distinct)>=5:remove(root,p)
        else:distinct.add(digest)
    sessions=[]
    for p in children(root,"AZSX/Data/Logs"):
        if SESSION.fullmatch(p.name)and p.is_dir()and core.get(root,p.relative_to(root).as_posix()+"/session.json",{}).get("product")==core.PRODUCT:
            sessions.append(p)
    for p in sorted(sessions,key=lambda p:p.stat().st_mtime_ns,reverse=True)[5:]:remove(root,p)
    state=core.installed(root);versions={state.get("version"),core.get(root,core.MANIFEST,{}).get("version")};keep={state.get("rollback")}
    pending=core.get(root,NATIVE_PENDING,{})
    if pending:versions.add(pending.get("version"))
    # A worker may be starting, applying or still exiting after spawning the new launcher.
    for p in children(root,"AZSX/Data/Updates/Workers"):
        if not TRANSACTION.fullmatch(p.name)or not p.is_dir():continue
        job=core.get(root,p.relative_to(root).as_posix()+"/job.json",{})
        if not job or ("worker_pid"not in job and job.get("schema")!=1):continue
        if p in Path(sys.executable).resolve().parents or core.process_alive(job.get("worker_pid",job.get("parent"))):
            versions.add(job.get("version"));continue
        remove(root,p)
    for p in children(root,"AZSX/Data/Updates/Transactions"):
        if not TRANSACTION.fullmatch(p.name)or not p.is_dir():continue
        journal=core.get(root,p.relative_to(root).as_posix()+"/journal.json",{})
        if p.name in keep or journal.get("phase")=="applying":
            versions.add((journal.get("state")or{}).get("version"))
        elif journal.get("phase")in("committed","restored"):
            expected={"journal.json"}|{"files/"+n for n,h in journal.get("before",{}).items()if h is not None}
            actual={x.relative_to(p).as_posix()for x in p.rglob("*")if x.is_file()}
            if actual<=expected:remove(root,p)
    for p in children(root,"AZSX/Data/Updates/Packages"):
        if core.VERSION.fullmatch(p.name)and p.is_dir()and p.name not in versions:
            names={item.name for item in p.iterdir()}
            if names=={"feed.json","feed.sig","package.zip"}:
                feed=core.get(root,p.relative_to(root).as_posix()+"/feed.json",{})
                if feed.get("product")==core.PRODUCT and feed.get("manifest",{}).get("version")==p.name:remove(root,p)

def best_effort(root):
    try:clean(root)
    except (ValueError,OSError,KeyError,TypeError,AttributeError)as error:
        print("[ WARNING ] AZSX history cleanup deferred: "+str(error),flush=True)
