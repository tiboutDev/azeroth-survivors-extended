"""Two menu-only launches against one disposable, sealed native profile."""
import json
import os
from pathlib import Path
import subprocess
import time
import uuid

from client_registry import SHA256, VERSION
from disposable115_party import writer
from native_profile_codec import key_candidates_from_bytes, seal
from native_runtime_import import digest

ROOT = Path(__file__).resolve().parents[2]

def run(seconds):
    runtime = ROOT / ("runtime-official-" + VERSION.replace(".", ""))
    exe = runtime / "AzerothSurvivors.exe"
    proof = json.loads((runtime / "AZSX-DEV.json").read_text())
    if digest(exe) != SHA256 or proof["test_hooks"] or digest(runtime / "avrt.dll") != proof["dll_sha256"]:
        raise ValueError("Reviewed normal receiver required")
    folder = ROOT / "test-results" / ("codex-" + uuid.uuid4().hex[:8])
    save = folder / "appdata/wenilla-rogue"
    save.mkdir(parents=True)
    (folder / "localappdata").mkdir()
    prepare = writer(exe)
    fresh = os.environ.get("AZSX_TEST_CODEX_FRESH")=="1"
    path = save / "characters.txt"
    expected = {"chosen":"0","eldre_thalas":"0","insignia":"0"}
    if not fresh:
        prepare(save, 1, known_items=[101800])
        seed = prepare.read(save)
        keys = key_candidates_from_bytes(exe.read_bytes())
        path.write_bytes(seal(seed + "seen|carrot=2,chosen=3\n", keys[0]))
        expected.update(carrot="2", chosen="3")
    clean = {k:v for k,v in os.environ.items() if not k.upper().startswith(("AZSX","AZCOOP_","WOW_","BENILLA_","COOP_"))}
    visits = (1, 2, 3) if fresh else (1, 2)
    for visit in visits:
        logs = folder / str(visit)
        logs.mkdir()
        env = dict(clean, APPDATA=str(folder/"appdata"), LOCALAPPDATA=str(folder/"localappdata"),
                   BENILLA_HOME=str(folder), AZSX_ACTIVE="1", AZCOOP_ENGINE="official", WOW_WIN="1",
                   WOW_NOSOUND="1", WOW_ROGUE_UPDATE="0", WOW_ROGUE_UPDATE_AUTO="0",
                   AZCOOP_NATIVE_LOG_DIRECTORY=str(logs), AZSX_READY_FILE=str(logs/"ready.json"))
        if visit==3:
            env.update(WOW_ROGUE_AUTOSTART="1", WOW_ROGUE_AUTOPLAY="4", WOW_ROGUE_AUTOPICK="1")
        before = path.read_bytes() if path.exists() else None
        with (logs/"native.log").open("wb") as stream:
            child = subprocess.Popen([str(exe)], cwd=runtime, env=env, stdout=stream, stderr=subprocess.STDOUT,
                                     creationflags=subprocess.CREATE_NO_WINDOW)
            try:
                deadline = time.monotonic()+seconds/len(visits)
                while child.poll() is None and time.monotonic()<deadline:
                    time.sleep(.5)
                if child.poll() is not None or not (logs/"ready.json").exists():
                    raise RuntimeError("Native launch failed: "+str(logs))
            finally:
                if child.poll() is None:child.terminate()
                child.wait(timeout=10)
        if visit==3 and '"type":"start"' not in (logs/"native.log").read_text(errors="replace"):
            raise RuntimeError("First run did not start")
        text = prepare.read(save)
        seen = next((line[5:] for line in text.splitlines() if line.startswith("seen|")), "")
        entries = [entry.split("=") for entry in seen.split(",") if entry]
        values = dict(entries)
        preserved = all(values.get(key)==value for key,value in expected.items())
        if not preserved or len(entries)!=len(values) or (not fresh and (values!=expected or "items|101800" not in text.splitlines())):
            raise RuntimeError("Unexpected Codex or item collection at visit "+str(visit)+": "+str(logs))
        log = (logs/"azeroth-coop-native.log").read_text(errors="replace")
        if visit==1 and f"Codex: discovered {3 if fresh else 2} " not in log or visit==2 and ("Codex: discovered" in log or before!=path.read_bytes()):
            raise RuntimeError("Discovery must write only on the first visit: "+str(logs))
    print("Verified missing entries, preserved rarities and inventory, and byte-identical second launch:", folder, flush=True)
