"""Remove only unchanged ASX-owned files; preserve native/user data and conflicts."""
import os,subprocess,tempfile
from pathlib import Path
import core
def uninstall(root):
    state=core.installed(root)
    files=dict(state["owned"])
    for name in(core.STATE,):
        p=core.path(root,name)
        if p.exists():files[name]=core.sha(p)
    for name in files:
        if name not in(core.STATE,"ASX/State/installation.lock"):core.owned_name(name)
        core.path(root,name)
    if os.name!="nt":
        preserved=[]
        for name,h in files.items():
            p=core.path(root,name)
            if p.is_file()and core.sha(p)==h:p.unlink()
            elif p.exists():preserved.append(name)
        return preserved
    job=Path(tempfile.mkdtemp(prefix="ASX-uninstall-"))
    core.atomic(job/"job.json",core.encoded({"root":str(Path(root).resolve()),"pid":os.getpid(),"files":files}))
    source=Path(__file__).with_suffix(".ps1")
    core.atomic(job/"remove.ps1",source.read_bytes())
    ps=Path(os.environ["SystemRoot"])/"System32/WindowsPowerShell/v1.0/powershell.exe"
    subprocess.Popen([str(ps),"-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File",str(job/"remove.ps1"),"-JobPath",str(job/"job.json")],creationflags=subprocess.CREATE_NO_WINDOW,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    print("[ ASX ] ASX removal will finish after this launcher closes. Native progression, settings, logs and backups are preserved.",flush=True)
    print("[ ASX ] Removal report: "+str(job/"result.json"),flush=True)
    return job
