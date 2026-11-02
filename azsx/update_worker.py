"""Apply a verified runtime update from a detached private copy of Python."""
import ctypes
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import uuid
sys.path.insert(0,str(Path(__file__).resolve().parent))
import core

def spawn(root,feed,keys):
    root=Path(root).resolve();work=core.path(root,'AZSX/Data/Updates/Workers/'+uuid.uuid4().hex)
    runtime=core.path(root,'AZSX/App/Python');work.mkdir(parents=True)
    # The installed interpreter cannot replace its own Windows image. Reuse a
    # private copy, never an unverified downloaded helper or system interpreter.
    for p in runtime.rglob('*'):
        core.raw_path(root,p.relative_to(root).as_posix())
    shutil.copytree(runtime,work/'Python')
    for name in ('core.py','layout.py','updates.py','update_worker.py','supported-games.json'):
        shutil.copy2(Path(__file__).with_name(name),work/name)
    job={'parent':os.getpid(),'root':str(root),'version':feed['manifest']['version'],'sequence':feed['sequence'],'keys':keys,'arguments':sys.argv[1:]}
    core.atomic(work/'job.json',core.encoded(job))
    child=subprocess.Popen([str(work/'Python/python.exe'),'-I','-B',str(work/'update_worker.py'),'--job',str(work/'job.json')],cwd=root)
    print('[ AZSX ] Finishing the compatible runtime update automatically; the launcher will restart.',flush=True)
    return child.pid

def wait_parent(pid):
    if os.name!='nt':raise RuntimeError('Detached runtime updater is Windows-only')
    k=ctypes.WinDLL('kernel32',use_last_error=True);k.OpenProcess.argtypes=[ctypes.c_ulong,ctypes.c_int,ctypes.c_ulong];k.OpenProcess.restype=ctypes.c_void_p
    h=k.OpenProcess(0x100000,False,pid)
    if h:
        try:
            k.WaitForSingleObject.argtypes=[ctypes.c_void_p,ctypes.c_ulong]
            if k.WaitForSingleObject(h,120000)!=0:raise RuntimeError('Original launcher did not close; update preserved for recovery')
        finally:k.CloseHandle.argtypes=[ctypes.c_void_p];k.CloseHandle(h)

def finish(job):
    import updates
    root=Path(job['root']).resolve();version=job['version']
    if not core.VERSION.fullmatch(version):raise ValueError('Invalid runtime update version')
    if core.path(root,'AZSX/App/Python')in Path(sys.executable).resolve().parents:raise ValueError('Updater must run outside installed Python')
    with core.locked(root):
        if core.game_running(root):raise ValueError('Game started during runtime update; update deferred')
        base='AZSX/Data/Updates/Packages/'+version+'/'
        raw=core.path(root,base+'feed.json').read_bytes();sig=core.path(root,base+'feed.sig').read_bytes()
        state=updates.update_state(root)
        feed=updates.verify(raw,sig,job['keys'],core.inspect(root),state['sequence'])
        if feed['sequence']!=job['sequence']or feed['manifest']['version']!=version:raise ValueError('Pending update identity changed')
        files=updates.verified_package(core.path(root,base+'package.zip').read_bytes(),feed)
        owned=core.installed(root)['owned'];state['attempt']={'sequence':feed['sequence'],'status':'applying'};core.put(root,'AZSX/Data/State/updates.json',state)
        try:
            core.apply(root,feed['manifest'],files,trusted=True,approve=lambda names:all(n in owned for n in names),detached_runtime=True)
            state.update(installed={'version':version,'sequence':feed['sequence']},attempt={'sequence':feed['sequence'],'status':'installed'});state.pop('held_sequence',None)
        except BaseException:
            state['attempt']['status']='failed';core.put(root,'AZSX/Data/State/updates.json',state);raise
        core.put(root,'AZSX/Data/State/updates.json',state)
    return subprocess.Popen([str(core.path(root,'AZSX/App/Python/python.exe')),'-I','-B','-X','utf8',str(core.path(root,'AZSX/App/Launcher/launch.py')),*job['arguments']],cwd=root).pid

if __name__=='__main__':
    import argparse
    p=argparse.ArgumentParser();p.add_argument('--job',type=Path,required=True);a=p.parse_args();job=json.loads(a.job.read_text());wait_parent(job['parent']);finish(job)
