"""Restartable migration of the installed legacy AZSX layout; never touches native data."""
import json
import uuid
import shutil
from pathlib import Path
PREFIXES={'AZSX/Client/':'AZSX/App/Launcher/','AZSX/Bin/':'AZSX/App/Bin/','AZSX/Runtime/':'AZSX/App/Python/','AZSX/Interface/':'AZSX/App/AddOns/','AZSX/Defaults/':'AZSX/App/Defaults/','AZSX/State/':'AZSX/Data/State/','AZSX/Updates/':'AZSX/Data/Updates/','AZSX/Backups/':'AZSX/Data/Backups/','AZSX/Logs/':'AZSX/Data/Logs/','AZSX/Reports/':'AZSX/Data/Reports/','AZSX/Settings/':'AZSX/Config/'}
EXACT={'AZSX/manifest.json':'AZSX/App/manifest.json','AZSX/options.json':'AZSX/Config/options.json'}
def name(value):
    if value.startswith("ASX/"):value="AZSX/"+value[4:]
    if value=="start-asx.cmd":return "start-azsx.cmd"
    if value=="README-ASX.txt":return "README-AZSX.txt"
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
    import migration
    renamed=migration.migrate(root)
    # Walk only specifically named legacy AZSX trees. Reject links before copying.
    for old,new in PREFIXES.items():
        source=core.raw_path(root,old.rstrip('/'))
        if not source.exists():continue
        for p in sorted(source.rglob('*')):
            relative=p.relative_to(root).as_posix();core.raw_path(root,relative)
            if p.is_file()and relative!='AZSX/State/installation.lock':entries.append((relative,name(relative)))
    for old,new in EXACT.items():
        p=core.raw_path(root,old)
        if p.is_file():entries.append((old,new))
    if not entries:return renamed
    if core.game_running(root):raise ValueError('Close the game before migrating AZSX')
    # A cross-layout lock prevents old and new launchers writing simultaneously.
    with core.legacy_locked(root):
        legacy_transactions=core.raw_path(root,'AZSX/Updates/Transactions')
        if legacy_transactions.is_dir():
            for p in legacy_transactions.glob('*/journal.json'):
                core.raw_path(root,p.relative_to(root).as_posix())
                if json.loads(p.read_text()).get('phase')=='applying':raise ValueError('Recover the interrupted legacy update with its original launcher before migrating')
        journal='AZSX/Data/State/layout-migration.json'
        state=core.get(root,journal,{})
        if not state:state={'schema':1,'phase':'copying','id':uuid.uuid4().hex,'files':[]}
        core.put(root,journal,state)
        copied=[]
        for old,new in entries:
            src=core.raw_path(root,old);dest=core.raw_path(root,new);raw=src.read_bytes()
            if dest.exists()and dest.read_bytes()!=raw:
                if old.startswith(('AZSX/State/','AZSX/Updates/','AZSX/Settings/'))or old=='AZSX/options.json':
                    # Preserve both versions and stop rather than guess user state.
                    raise ValueError('Conflicting legacy/current AZSX data preserved: '+old)
                new='AZSX/Data/Backups/Layout/'+state['id']+'/'+old
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
    lock=core.raw_path(root,'AZSX/State/installation.lock')
    if lock.exists():lock.unlink()
    for old in PREFIXES:
        p=core.raw_path(root,old.rstrip('/'))
        if p.exists():
            for d in sorted((x for x in p.rglob('*')if x.is_dir()),key=lambda x:len(x.parts),reverse=True):
                if not any(d.iterdir()):d.rmdir()
            if not any(p.iterdir()):p.rmdir()
    return True
