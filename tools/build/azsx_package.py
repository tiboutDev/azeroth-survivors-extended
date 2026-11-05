"""Prepare a native-folder AZSX archive. Never publishes or accesses a signing key implicitly."""
import argparse,base64,io,json,os,stat,sys,zipfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/"azsx"))
import core
sys.path.insert(0,str(Path(__file__).resolve().parent))
from azsx_release_identity import default_version,release_title
def zip_bytes(files,m):
    stream=io.BytesIO()
    with zipfile.ZipFile(stream,"w",compression=zipfile.ZIP_DEFLATED,compresslevel=6)as z:
        for name,data in sorted({**files,core.MANIFEST:core.encoded(m)}.items()):
            entry=zipfile.ZipInfo(name,(2026,1,1,0,0,0));entry.compress_type=zipfile.ZIP_DEFLATED;entry.external_attr=(stat.S_IFREG|0o644)<<16
            z.writestr(entry,data)
    return stream.getvalue()
def build(output,dll,runtime,version=None,key=None,sequence=1,game_version=None):
    games=json.loads((ROOT/"azsx/supported-games.json").read_text())["games"]
    game_version=game_version or max(games,key=lambda v:tuple(map(int,v.split("."))))
    version=version or default_version(game_version)
    if type(sequence)is not int or not 1<=sequence<=2**53:raise ValueError("Invalid AZSX release sequence")
    if not core.VERSION.fullmatch(version):raise ValueError("Invalid AZSX version")
    game=games.get(game_version)
    if not game:raise ValueError("No reviewed AZSX distribution for this game version")
    output=Path(output)
    if output.exists():raise ValueError("Choose a new preparation folder; existing releases are preserved")
    raw=Path(dll).read_bytes()
    if not raw.startswith(b"MZ")or b"AZCOOP_TEST_"in raw:raise ValueError("A normal Windows DLL without test hooks is required")
    signing=None
    if key is not None:
        from cryptography.hazmat.primitives import serialization
        from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
        signing=serialization.load_pem_private_key(Path(key).read_bytes(),password=None)
        if not isinstance(signing,Ed25519PrivateKey):raise ValueError("Use an Ed25519 signing key")
        public=base64.b64encode(signing.public_key().public_bytes(serialization.Encoding.Raw,serialization.PublicFormat.Raw)).decode()
        trusted=json.loads((ROOT/"azsx/channel.json").read_text())["keys"]
        if public not in trusted:raise ValueError("Signing key does not match the packaged AZSX trust keys")
    files={"AZSX/App/Bin/avrt.dll":raw,"start-azsx.cmd":(ROOT/"azsx/start-azsx.cmd").read_bytes().replace(b"\r\n",b"\n").replace(b"\n",b"\r\n"),"README-AZSX.txt":(ROOT/"azsx/README-AZSX.txt").read_bytes().replace(b"{GAME_VERSION}",game_version.encode()),"AZSX/App/Defaults/channel.json":(ROOT/"azsx/channel.json").read_bytes(),"AZSX/App/Launcher/LICENSE":(ROOT/"LICENSE").read_bytes(),"AZSX/App/Launcher/supported-games.json":(ROOT/"azsx/supported-games.json").read_bytes()}
    for name in("core.py","updates.py","maintenance.py","launch.py","remove.py","remove.ps1","layout.py","migration.py","native_update.py","update_worker.py","housekeeping.py"):files["AZSX/App/Launcher/"+name]=(ROOT/"azsx"/name).read_bytes()
    for name in core.AZSXUI_FILES:
        source=ROOT/"addons/AZSXUI"/name
        if any(part.is_symlink()for part in (source,source.parent,ROOT/"addons",ROOT/"addons/AZSXUI")):raise ValueError("Bundled addon links cannot be distributed")
        files[core.AZSXUI_SOURCE+name]=source.read_bytes()
    runtime=Path(runtime)
    for file in sorted(runtime.rglob("*")):
        relative=file.relative_to(runtime)
        if file.is_symlink():raise ValueError("Runtime links cannot be distributed")
        if not file.is_file()or any(p in("__pycache__","test","tests","ensurepip","pip","idlelib","tkinter")for p in relative.parts)or file.suffix in(".pyc",".pyo"):continue
        if relative.parts[0]not in("Lib","DLLs","python.exe","python3.dll","python312.dll","pythonw.exe","vcruntime140.dll","vcruntime140_1.dll","LICENSE.txt"):raise ValueError("Unexpected runtime file")
        files["AZSX/App/Python/"+relative.as_posix()]=file.read_bytes()
    if "AZSX/App/Python/python.exe"not in files or not any("/cryptography/"in n for n in files):raise ValueError("The bundled Windows runtime must include cryptography")
    m={"schema":1,"product":core.PRODUCT,"version":version,"bootstrap":1,"game":{"version":game_version,"sha256":game["sha256"],"archives":game.get("archives",["Data/patch.MPQ",*[f"Data/patch-{i}.MPQ"for i in range(2,10)]])},"files":{name:{"size":len(data),"sha256":core.digest(data)}for name,data in files.items()}}
    core.validate(m)
    archive=zip_bytes(files,m);output.mkdir(parents=True)
    target=output/("azsx-"+version+".zip");target.write_bytes(archive)
    (output/"SHA256SUMS.txt").write_text(core.digest(archive)+"  "+target.name+"\n")
    if signing is not None:
        feed={"schema":1,"product":core.PRODUCT,"sequence":sequence,"manifest":m,"package":{"asset":target.name,"size":len(archive),"sha256":core.digest(archive)}}
        raw=core.encoded(feed);(output/"azsx-latest.json").write_bytes(raw);(output/"azsx-latest.json.sig").write_bytes(signing.sign(raw))
    (output/"PREPARATION.json").write_bytes(core.encoded({"product":core.PRODUCT,"version":version,"title":release_title(version),"archive":target.name,"archive_sha256":core.digest(archive),"dll_sha256":core.digest(files["AZSX/App/Bin/avrt.dll"]),"file_count":len(files),"signed":key is not None,"published":False,"live_validation":False,"runtime_updates":"Compatible signed updates install automatically, including detached runtime replacement. Transition from the .2 layout requires complete archive extraction while closed."}))
    return target
if __name__=="__main__":
    p=argparse.ArgumentParser(description=__doc__);p.add_argument("--output",type=Path,required=True);p.add_argument("--dll",type=Path,required=True);p.add_argument("--runtime",type=Path,required=True);p.add_argument("--version",help="AZSX version (default: <game-version>-r1); increment rN for later releases");p.add_argument("--private-key",type=Path);p.add_argument("--sequence",type=int,default=1);p.add_argument("--game-version")
    a=p.parse_args();print(build(a.output,a.dll,a.runtime,a.version,a.private_key,a.sequence,a.game_version))
