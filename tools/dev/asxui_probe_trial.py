"""Opt-in standalone Lua API probe in a disposable native game copy (Windows)."""
import hashlib,json,os,shutil,subprocess,sys,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/"tools/dev"))
import background_desktop as background
from client_registry import CLIENT,SHA256
from disposable115_party import writer

def run_probe(args,gate):
    source=Path(os.environ["ASXUI_PROBE_RUNTIME"]).resolve(strict=True)
    exe=source/"AzerothSurvivors.exe"
    if hashlib.sha256(exe.read_bytes()).hexdigest()!=SHA256:raise ValueError("Wrong native executable")
    output=gate.parent/"standalone";output.mkdir()
    native=output/"native";native.mkdir()
    shutil.copyfile(exe,native/exe.name)
    for name in CLIENT["archives"]:
        original=source/name;target=native/name;target.parent.mkdir(parents=True,exist_ok=True)
        if original.is_symlink() or not original.is_file():raise ValueError("Unsafe native archive source")
        # Same-volume links are read-only game inputs; auto-update is disabled.
        try:os.link(original,target)
        except OSError:shutil.copyfile(original,target)
    profile=output/"profile";save=profile/"appdata/wenilla-rogue";save.mkdir(parents=True)
    writer(exe)(save,1,name_override="AsxuiProbe")
    addon=profile/"AddOns/ASXUIProbe";addon.mkdir(parents=True)
    (addon/"ASXUIProbe.toc").write_text("## Interface: 11200\n## Title: ASXUI API probe\nProbe.lua\n")
    shutil.copyfile(ROOT/"tools/dev/fixtures/asxui-api-probe.lua",addon/"Probe.lua")
    if (ROOT/"addons/ASXUI").is_dir():shutil.copytree(ROOT/"addons/ASXUI",profile/"AddOns/ASXUI")
    clean={k:v for k,v in os.environ.items()if not k.upper().startswith(("ASX","AZCOOP_","COOP_","WOW_","BENILLA_"))}
    env=dict(clean,APPDATA=str(profile/"appdata"),LOCALAPPDATA=str(profile/"localappdata"),BENILLA_HOME=str(profile),WOW_NOSOUND="1",WOW_WIN="1",WOW_ROGUE_UPDATE="0",WOW_ROGUE_UPDATE_AUTO="0",WOW_ROGUE_AUTOPICK="1",WOW_ROGUE_AUTOSTART="1",WOW_ROGUE_AUTOPLAY="4",RUST_LOG="info")
    if os.environ.get("ASXUI_PROBE_VISUAL")=="1":
        (output/"shots").mkdir()
        env.update(WOW_ROGUE_SHOTS=str(output/"shots"),WOW_ROGUE_SHOTS_EVERY="15")
    with (output/"native.log").open("wb")as stream:
        child=subprocess.Popen([str(native/exe.name)],cwd=native,env=env,stdout=stream,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW)
        try:child.wait(timeout=args.seconds)
        except subprocess.TimeoutExpired:
            child.terminate()
            try:child.wait(timeout=10)
            except subprocess.TimeoutExpired:child.kill();child.wait(timeout=5)
    text=(output/"native.log").read_text(errors="replace")
    lines=[line for line in text.splitlines()if "ASXUI_API_PROBE"in line or "ASXUI_ADDON_PROBE"in line]
    result={"native_sha256":SHA256,"asx_dll_present":(native/"avrt.dll").exists(),"probe_samples":len(lines),"game_clients":1,"seconds":args.seconds,"screenshots":os.environ.get("ASXUI_PROBE_VISUAL")=="1","desktop_input":False,"synthetic_projection_bridge":True,"real_asx_transport_validated":False}
    (output/"result.json").write_text(json.dumps(result,indent=2)+"\n")
    (output/"api-samples.txt").write_text("\n".join(lines))
    print(json.dumps(result),flush=True)
    if not lines:raise SystemExit("No Lua probe samples; inspect native.log")

background.__file__=__file__
background.run_child=run_probe
if __name__=="__main__":background.main()
