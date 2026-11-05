"""Bounded official update announcement; native updater owns the game download."""
import json,re,urllib.request,urllib.error,os,subprocess,time,uuid
from pathlib import Path
import core
import updates
URL="https://wow.aethus.pl/exe/latest.json"
LIMIT=16384
SHA=re.compile(r"[0-9a-f]{64}\Z")
VERSION=re.compile(r"[0-9]+(?:\.[0-9]+){2}\Z")
def announcement(download=None):
    if download is None:
        def download():
            request=urllib.request.Request(URL,headers={"User-Agent":"AZSX","Accept":"application/json"})
            with urllib.request.urlopen(request,timeout=8) as response:return response.read(LIMIT+1)
    raw=download()
    if len(raw)>LIMIT:raise ValueError("Official update announcement is too large")
    value=json.loads(raw)
    if not isinstance(value,dict) or not isinstance(value.get("version"),str) or not VERSION.fullmatch(value["version"]) or not isinstance(value.get("sha256"),str) or not SHA.fullmatch(value["sha256"]):
        raise ValueError("Invalid official update announcement")
    return value["version"],value["sha256"]

PENDING="AZSX/Data/State/native-update.json"

def cached_offer(root,pending):
    """Resume using the staged signed package, even when discovery is offline."""
    cfg=updates.configuration(root,persist=False)
    version=pending["version"]
    if not core.VERSION.fullmatch(version):raise ValueError("Invalid pending AZSX version")
    base="AZSX/Data/Updates/Packages/"+version+"/"
    raw=core.path(root,base+"feed.json").read_bytes();sig=core.path(root,base+"feed.sig").read_bytes()
    feed=updates.verify(raw,sig,cfg["keys"],pending["target"],updates.update_state(root)["sequence"])
    if feed["manifest"]["version"]!=version:raise ValueError("Pending AZSX package identity changed")
    updates.verified_package(core.path(root,base+"package.zip").read_bytes(),feed)
    return cfg,raw,sig,feed

def synchronize(root,current,offline=False,download=None):
    """Return an AZSX update result, or 'native update ready' after full staging."""
    download=download or updates.fetch
    pending=core.get(root,PENDING)
    if pending:
        if current not in(pending["target"],pending["previous"]):
            raise ValueError("The native updater installed an unexpected client. AZSX remains inactive; a compatible release is required.")
        if current==pending["target"]:
            try:result=cached_offer(root,pending)
            except (ValueError,OSError,KeyError):
                if offline:raise
                # A damaged staged cache can be refreshed from the authenticated
                # channel without asking the user to remove internal state.
                result=updates.offer(root,current,download)
                if result is None:raise ValueError("No compatible package available to finish the native update")
                updates.prepared(root,result,download)
                pending["version"]=result[3]["manifest"]["version"];core.put(root,PENDING,pending)
            status=updates.automatic(root,current,download,offered=result)
            if status not in("handoff","deferred until next launch"):
                core.path(root,PENDING).unlink()
            return status
        if offline:return "offline"
        if pending.get("launch_id")==os.environ.get("AZSX_LAUNCH_ID")and pending.get("phase")=="failed":
            return "deferred until next launch"
        version,sha=announcement()
        if sha==pending["target"]:
            try:result=cached_offer(root,pending)
            except (ValueError,OSError,KeyError):
                result=updates.offer(root,sha,download)
                if result is None:raise ValueError("No matching package for the pending native update")
                updates.prepared(root,result,download)
                pending["version"]=result[3]["manifest"]["version"];core.put(root,PENDING,pending)
            if result[3]["manifest"]["game"]["version"]==version:return "native update ready"
        # The official target moved on: discover and stage a matching newer
        # release atomically. Keep the old pending package until that succeeds.
    if offline:return "offline"
    try:result=updates.offer(root,None,download)
    except urllib.error.HTTPError as error:
        if error.code==404:return "no AZSX release published"
        raise
    if result is None:return "disabled"
    cfg,raw,sig,feed=result;target=feed["manifest"]["game"]
    if target["sha256"]==current:return updates.automatic(root,current,download,offered=result)
    state=updates.update_state(root)
    if state.get("held_sequence")==feed["sequence"]:return "held after rollback"
    # Never ask the official updater for an older client or an unrelated latest build.
    installed=core.installed(root).get("game")or core.get(root,core.MANIFEST,{}).get("game",{})
    old=installed.get("version")
    version,sha=announcement()
    if (version,sha)!=(target["version"],target["sha256"]):return "no compatible native update"
    if old and VERSION.fullmatch(old)and tuple(map(int,version.split(".")))<=tuple(map(int,old.split("."))):
        return "no compatible native update"
    updates.prepared(root,result,download)
    core.put(root,PENDING,{"version":feed["manifest"]["version"],"previous":current,"target":sha,"phase":"prepared"})
    return "native update ready"

def run(root,environment):
    """Run only the official updater with AZSX inactive; verify its final executable."""
    pending=core.get(root,PENDING)
    if not pending:raise ValueError("No staged native update")
    cached_offer(root,pending)
    if core.inspect(root)!=pending["previous"]:raise ValueError("Native client changed before the official updater started")
    env={k:v for k,v in environment.items()if not k.upper().startswith(("AZSX_","AZCOOP_","COOP_"))}
    env.update(WOW_ROGUE_UPDATE="1",WOW_ROGUE_UPDATE_AUTO="1")
    folder=core.path(root,"AZSX/Data/Logs/"+time.strftime("%Y%m%d-%H%M%S-")+uuid.uuid4().hex[:8])
    folder.mkdir(parents=True)
    core.put(root,folder.relative_to(root).as_posix()+"/session.json",{"product":core.PRODUCT,"kind":"native-update","version":pending["version"]})
    import housekeeping
    housekeeping.best_effort(root)
    pending.update(phase="updating",launch_id=environment.get("AZSX_LAUNCH_ID"));core.put(root,PENDING,pending)
    print("[ AZSX ] Updating Azeroth Survivors with the official updater...",flush=True)
    try:
        with (folder/"game-output.log").open("wb")as out:
            child=subprocess.Popen([str(Path(root)/"AzerothSurvivors.exe")],cwd=root,env=env,stdout=out,stderr=subprocess.STDOUT)
            try:child.wait(timeout=600)
            finally:
                if child.poll()is None:
                    child.terminate()
                    try:child.wait(timeout=15)
                    except subprocess.TimeoutExpired:child.kill();child.wait(timeout=5)
        if core.inspect(root)!=pending["target"]:
            raise ValueError("The official update did not produce the expected client. AZSX was not installed; restart to retry.")
        pending["phase"]="ready";core.put(root,PENDING,pending)
        return 75
    except BaseException:
        pending["phase"]="failed";core.put(root,PENDING,pending);raise
