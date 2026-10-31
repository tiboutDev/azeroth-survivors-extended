"""Read-only diagnosis and explicitly approved repair from authenticated packages."""
import os
import re
import uuid
from pathlib import Path
import core
import updates


def diagnose(root):
    issues=[]
    def issue(name,severity,message):issues.append({"file":name,"severity":severity,"message":message})
    manifest=None
    try:manifest=core.manifest(root)
    except (ValueError,OSError,KeyError,TypeError) as error:issue(core.MANIFEST,"critical",str(error))
    if manifest:
        inventory={**manifest["files"]}
        for target,source in core.projections(manifest).items():
            inventory[target]=manifest["files"][source]
        for name,spec in inventory.items():
            try:
                p=core.path(root,name)
                if not p.is_file():message="missing"
                elif p.stat().st_size!=spec["size"] or core.sha(p)!=spec["sha256"]:message="modified"
                else:continue
                issue(name,"critical" if core.file_role(name)=="critical" else "warning",message)
            except (ValueError,OSError) as error:issue(name,"critical",str(error))
    try:core.inspect(root,manifest)
    except (ValueError,OSError) as error:issue("AzerothSurvivors.exe","critical",str(error))
    try:
        if core.get(root,core.STATE) is None:issue(core.STATE,"warning","ownership receipt absent; first installation or guided recovery required")
        else:core.installed(root)
    except (ValueError,OSError,KeyError,TypeError,AttributeError) as error:issue(core.STATE,"warning","maintenance blocked by invalid ownership receipt: "+str(error))
    try:
        folder=core.path(root,"ASX/Updates/Transactions")
        if folder.exists():
            for d in folder.iterdir():
                if re.fullmatch(r"[0-9a-f]{32}",d.name):
                    j=core.get(root,"ASX/Updates/Transactions/"+d.name+"/journal.json",{})
                    if j.get("phase")=="applying":issue("ASX/Updates/Transactions/"+d.name,"critical","interrupted transaction; use --recover")
    except (ValueError,OSError,AttributeError) as error:issue("ASX/Updates/Transactions","critical",str(error))
    try:updates.update_state(root)
    except (ValueError,OSError,KeyError,TypeError) as error:issue("ASX/State/updates.json","warning",str(error))
    try:
        options=core.get(root,"ASX/options.json",{})
        if not isinstance(options,dict) or any(type(options.get(k,True)) is not bool for k in ("treasure_goblins","lost_supplies","darkmoon_trials")):
            raise ValueError("invalid optional settings; affected events are disabled at launch")
    except (ValueError,OSError) as error:issue("ASX/options.json","warning",str(error))
    return {"ready":not any(i["severity"]=="critical" for i in issues),"issues":issues}


def print_report(report):
    for issue in report["issues"]:
        print("[ "+issue["severity"].upper()+" ] "+issue["file"]+": "+issue["message"],flush=True)
    print("[ ASX ] "+("Ready to play" if report["ready"] else "Repair or recovery required")+
          (" — warnings present" if report["ready"] and report["issues"] else "")+".",flush=True)


def bounded_read(file,limit):
    with Path(file).open("rb") as stream:data=stream.read(limit+1)
    if len(data)>limit:raise ValueError("Repair input exceeds its size limit")
    return data


def repair(root,confirm,archive=None,feed_path=None,signature_path=None,download=updates.fetch,offline=False):
    # Caller owns installation lock and has handled interrupted transactions.
    try:installed=core.manifest(root)["version"]
    except (ValueError,OSError,KeyError,TypeError):
        try:installed=core.installed(root).get("version")
        except (ValueError,OSError,KeyError,TypeError,AttributeError):installed=None
    game_sha=core.inspect(root)
    reset_channel=False
    try:cfg=updates.configuration(root,persist=False)
    except (ValueError,OSError,KeyError,TypeError):
        cfg=dict(updates.DEFAULT_CHANNEL);reset_channel=True
    if archive:
        if not feed_path or not signature_path:raise ValueError("Local repair requires --archive, --feed and --signature.")
        raw=bounded_read(feed_path,updates.MAX_FEED);sig=bounded_read(signature_path,64)
        data=bounded_read(archive,updates.MAX_ZIP)
    else:
        base="ASX/Updates/Packages/"+installed+"/" if installed and core.VERSION.fullmatch(installed) else None
        if base and core.path(root,base+"feed.json").is_file():
            raw=bounded_read(core.path(root,base+"feed.json"),updates.MAX_FEED)
            sig=bounded_read(core.path(root,base+"feed.sig"),64)
            data=bounded_read(core.path(root,base+"package.zip"),updates.MAX_ZIP)
        else:
            if offline:raise ValueError("No cached signed package. Supply --archive, --feed and --signature for offline repair.")
            if reset_channel:
                raw=download(cfg["base"]+"asx-latest.json",updates.MAX_FEED)
                sig=download(cfg["base"]+"asx-latest.json.sig",64)
                result=(cfg,raw,sig,updates.verify(raw,sig,cfg["keys"],game_sha,0))
            else:result=updates.offer(root,game_sha,download,minimum=0)
            if result is None:raise ValueError("Update channel disabled. Supply a local signed package.")
            cfg,raw,sig,feed=result
            if installed and feed["manifest"]["version"]!=installed:
                raise ValueError("Repair will not change ASX version. Supply the signed archive for "+installed+" or choose Update ASX.")
            data=download(cfg["base"]+feed["package"]["asset"],updates.MAX_ZIP)
    feed=updates.verify(raw,sig,cfg["keys"],game_sha,minimum=0)
    m=feed["manifest"];files=updates.verified_package(data,feed)
    if installed and m["version"]!=installed:raise ValueError("Repair must restore the installed version, not another release.")
    core.inspect(root,m)
    new=core.projection(files,m);changes=[]
    receipt_missing=not core.path(root,core.STATE).is_file()
    try:receipt=core.installed(root);receipt_bad=False
    except (ValueError,OSError,KeyError,TypeError,AttributeError):receipt={"owned":{}};receipt_bad=True
    for name,b in new.items():
        p=core.path(root,name)
        if p.exists() and core.sha(p)==core.digest(b):continue
        if p.exists() and name not in receipt["owned"] and name in ("avrt.dll","benilla-config/AddOns/ASX/ASX.toc","benilla-config/AddOns/ASX/Presentation.lua"):
            raise ValueError("Foreign loader/addon conflict preserved: "+name)
        if p.exists() and core.file_role(name)=="documentation":
            changes.append("Preserve modified documentation: "+name);continue
        changes.append(("Back up and replace: " if p.exists() else "Restore: ")+name)
    if os.name=="nt" and any(name.startswith("ASX/Runtime/") and (not core.path(root,name).is_file() or core.sha(core.path(root,name))!=core.digest(b)) for name,b in new.items()):
        raise ValueError("Manual installation required: close ASX and extract the complete verified archive to restore the Python runtime.")
    print("[ ASX ] Repair ASX "+m["version"]+(" (recovering installation identity)" if installed is None else ""),flush=True)
    for line in changes:print("  "+line,flush=True)
    if receipt_bad or receipt_missing:print("  Recover ownership receipt; unknown projected files will not be adopted.",flush=True)
    if reset_channel:print("  Restore official update channel; back up invalid custom configuration.",flush=True)
    if not receipt_bad and not receipt_missing and not reset_channel and installed is not None and not any(not line.startswith("Preserve ") for line in changes):
        print("[ ASX ] Installed release is already intact; documentation warnings are nonblocking.",flush=True)
        return "current"
    if not confirm(changes):return "cancelled"
    updates.cache(root,raw,sig,data,feed)
    if reset_channel:
        channel=core.path(root,"ASX/Settings/channel.json")
        if channel.is_file():core.atomic(core.path(root,"ASX/Backups/Files/"+uuid.uuid4().hex+"/channel.json"),channel.read_bytes())
        core.put(root,"ASX/Settings/channel.json",cfg)
    old_raw=None
    if receipt_bad:
        p=core.path(root,core.STATE);old_raw=p.read_bytes()
        core.atomic(core.path(root,"ASX/Backups/Files/"+uuid.uuid4().hex+"/install.json"),old_raw)
        p.unlink()
    try:
        status=core.apply(root,m,files,trusted=True,approve=lambda names:True,repair=True)
        return status
    except BaseException:
        if old_raw is not None:core.atomic(core.path(root,core.STATE),old_raw)
        raise
