"""Restartable migration of the installed legacy ASX layout; never touches native data."""
import json
import uuid
import shutil
from pathlib import Path
PREFIXES={'ASX/Client/':'ASX/App/Launcher/','ASX/Bin/':'ASX/App/Bin/','ASX/Runtime/':'ASX/App/Python/','ASX/Interface/':'ASX/App/AddOns/','ASX/Defaults/':'ASX/App/Defaults/','ASX/State/':'ASX/Data/State/','ASX/Updates/':'ASX/Data/Updates/','ASX/Backups/':'ASX/Data/Backups/','ASX/Logs/':'ASX/Data/Logs/','ASX/Reports/':'ASX/Data/Reports/','ASX/Settings/':'ASX/Config/'}
EXACT={'ASX/manifest.json':'ASX/App/manifest.json','ASX/options.json':'ASX/Config/options.json'}
def name(value):
    if value in EXACT:return EXACT[value]
    for old,new in PREFIXES.items():
        if value.startswith(old):return new+value[len(old):]
    return value

def manifest(value):
    if not isinstance(value,dict)or not isinstance(value.get('files'),dict):return value
    out=dict(value);out['files']={name(k):v for k,v in value['files'].items()}
    if len(out['files'])!=len(value['files']):raise ValueError('Duplicate legacy/current payload paths')
    return out

def migrate(root):
    import core
    root=Path(root).resolve();entries=[]
    # Walk only specifically named legacy ASX trees. Reject links before copying.
    for old,new in PREFIXES.items():
        source=core.raw_path(root,old.rstrip('/'))
        if not source.exists():continue
        for p in sorted(source.rglob('*')):
            relative=p.relative_to(root).as_posix();core.raw_path(root,relative)
            if p.is_file()and relative!='ASX/State/installation.lock':entries.append((relative,name(relative)))
    for old,new in EXACT.items():
        p=core.raw_path(root,old)
        if p.is_file():entries.append((old,new))
    if not entries:return False
    if core.game_running(root):raise ValueError('Close the game before migrating ASX')
    # A cross-layout lock prevents old and new launchers writing simultaneously.
    with core.legacy_locked(root):
        legacy_transactions=core.raw_path(root,'ASX/Updates/Transactions')
        if legacy_transactions.is_dir():
            for p in legacy_transactions.glob('*/journal.json'):
                core.raw_path(root,p.relative_to(root).as_posix())
                if json.loads(p.read_text()).get('phase')=='applying':raise ValueError('Recover the interrupted legacy update with its original launcher before migrating')
        journal='ASX/Data/State/layout-migration.json'
        state=core.get(root,journal,{})
        if not state:state={'schema':1,'phase':'copying','id':uuid.uuid4().hex,'files':[]}
        core.put(root,journal,state)
        copied=[]
        for old,new in entries:
            src=core.raw_path(root,old);dest=core.raw_path(root,new);raw=src.read_bytes()
            if dest.exists()and dest.read_bytes()!=raw:
                if old.startswith(('ASX/State/','ASX/Updates/','ASX/Settings/'))or old=='ASX/options.json':
                    # Preserve both versions and stop rather than guess user state.
                    raise ValueError('Conflicting legacy/current ASX data preserved: '+old)
                new='ASX/Data/Backups/Layout/'+state['id']+'/'+old
                dest=core.raw_path(root,new)
            if dest.exists()and dest.read_bytes()!=raw:raise ValueError('Migration backup conflict')
            if not dest.exists():
                core.atomic(dest,raw);shutil.copystat(src,dest,follow_symlinks=False)
            if core.sha(dest)!=core.digest(raw):raise ValueError('Migration copy verification failed')
            copied.append({'source':old,'target':new,'sha256':core.digest(raw)})
        # Save the complete verified plan before removing any source file.
        history={x['source']:x for x in state.get('files',[])};history.update({x['source']:x for x in copied})
        state.update(phase='removing',files=list(history.values()));core.put(root,journal,state)
        for item in copied:
            src=core.raw_path(root,item['source']);dest=core.raw_path(root,item['target'])
            if core.sha(src)!=item['sha256']or core.sha(dest)!=item['sha256']:raise ValueError('Migration files changed; preserved')
            src.unlink()
        state['phase']='complete';core.put(root,journal,state)
    lock=core.raw_path(root,'ASX/State/installation.lock')
    if lock.exists():lock.unlink()
    for old in PREFIXES:
        p=core.raw_path(root,old.rstrip('/'))
        if p.exists():
            for d in sorted((x for x in p.rglob('*')if x.is_dir()),key=lambda x:len(x.parts),reverse=True):
                if not any(d.iterdir()):d.rmdir()
            if not any(p.iterdir()):p.rmdir()
    return True
