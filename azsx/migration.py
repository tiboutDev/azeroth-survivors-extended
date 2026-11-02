"""One-way ASX branding migration with verified backups and conflict checks."""
import json
from pathlib import Path

def migrate(root):
    import core
    root=Path(root).resolve()
    source=core.raw_path(root,"ASX")
    if not source.exists():return False
    if core.game_running(root):raise ValueError("Close the game before migrating ASX to AZSX")
    # The old launcher uses this exact lock. Hold it throughout copying/removal.
    lock=core.raw_path(root,"ASX/Data/State/installation.lock")
    lock.parent.mkdir(parents=True,exist_ok=True)
    with lock.open("a+b") as f:
        if f.tell()==0:f.write(b"0");f.flush()
        f.seek(0)
        try:
            if __import__('os').name=='nt':
                import msvcrt;msvcrt.locking(f.fileno(),msvcrt.LK_NBLCK,1)
            else:
                import fcntl;fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB)
        except OSError as e:raise ValueError("Close the previous ASX launcher before migration")from e
        changed=_copy(root)
    if changed:
        lock.unlink(missing_ok=True)
        for directory in sorted((p for p in source.rglob("*")if p.is_dir()),key=lambda p:len(p.parts),reverse=True):
            if not any(directory.iterdir()):directory.rmdir()
        if not any(source.iterdir()):source.rmdir()
    return changed

def _copy(root):
    import core,layout
    marker="AZSX/Data/State/branding-migration.json"
    previous=core.get(root,marker,{})
    if previous.get('phase')=='complete':return False
    for folder in ('ASX/Data/Updates/Transactions','ASX/Updates/Transactions'):
        source=core.raw_path(root,folder)
        if source.exists():
            for p in source.glob('*/journal.json'):
                core.raw_path(root,p.relative_to(root).as_posix())
                if json.loads(p.read_bytes()).get('phase')=='applying':
                    raise ValueError('Recover the interrupted ASX update before migration')
    plan=[]
    source=core.raw_path(root,'ASX')
    for p in sorted(source.rglob('*')):
        old=p.relative_to(root).as_posix();core.raw_path(root,old)
        if not p.is_file()or p.name=='installation.lock':continue
        new=layout.name(old)
        raw=p.read_bytes();target=core.raw_path(root,new)
        # Never replace the new extracted runtime/manifest with the old payload.
        payload=new.startswith('AZSX/App/')
        if target.exists() and target.read_bytes()!=raw and not payload:
            raise ValueError('Conflicting ASX/AZSX preferences or state preserved: '+old)
        backup='AZSX/Data/Backups/Branding/'+old
        plan.append((old,new,backup,core.digest(raw),payload))
    # Complete conflict preflight before writing a single migrated state file.
    history={item['source']:item for item in previous.get('files',[])}
    history.update({o:{'source':o,'target':n,'backup':b,'sha256':h}for o,n,b,h,_ in plan})
    journal={'schema':1,'phase':'copying','files':list(history.values())}
    core.put(root,marker,journal)
    for old,new,backup,digest,payload in plan:
        raw=core.raw_path(root,old).read_bytes()
        if core.digest(raw)!=digest:raise ValueError('ASX source changed during migration')
        dest=core.raw_path(root,backup)
        if dest.exists()and core.sha(dest)!=digest:raise ValueError('Migration backup conflict')
        if not dest.exists():core.atomic(dest,raw)
        target=core.raw_path(root,new)
        if not payload and not target.exists():core.atomic(target,raw)
    # Keep the old receipt raw: installed() translates payload paths while old
    # addon projection paths remain available for hash-checked retirement.
    saved=core.raw_path(root,'benilla-config/SavedVariables/ASXUI.lua')
    target=core.raw_path(root,'benilla-config/SavedVariables/AZSXUI.lua')
    if saved.is_file()and not target.exists():
        raw=saved.read_bytes()
        if len(raw)>4*1024*1024:raise ValueError('SavedVariables exceed migration limit')
        core.atomic(target,raw.replace(b'ASXUIDB',b'AZSXUIDB'))
    receipt=core.get(root,core.STATE,{})
    for name,expected in receipt.get('owned',{}).items():
        if not core.legacy_ui_target(name)and name not in ('start-asx.cmd','README-ASX.txt'):continue
        p=core.raw_path(root,name)
        if not p.is_file():continue
        actual=core.sha(p)
        if actual!=expected:
            if name.endswith('.toc'):raise ValueError('Modified legacy addon TOC preserved; disable it before migration: '+name)
            continue
        backup=core.raw_path(root,'AZSX/Data/Backups/Branding/'+name)
        core.atomic(backup,p.read_bytes())
        if core.sha(backup)!=actual or core.sha(p)!=actual:raise ValueError('Legacy addon changed during migration')
        p.unlink()
    journal['phase']='retiring'
    core.put(root,marker,journal)
    for old,new,backup,digest,payload in plan:
        p=core.raw_path(root,old)
        if core.sha(p)!=digest or core.sha(core.raw_path(root,backup))!=digest:raise ValueError('Migration source changed; preserved')
        p.unlink()
    journal['phase']='complete'
    core.put(root,marker,journal)
    return True
