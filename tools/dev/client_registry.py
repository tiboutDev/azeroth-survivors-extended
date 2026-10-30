"""Validated client identities shared by private developer tools.
Reading this registry never authorizes or activates a native adapter.
"""
import json
import re
from pathlib import Path, PurePosixPath
ROOT = Path(__file__).resolve().parents[2]
REGISTRY = ROOT / "asx/supported-games.json"

def load_clients(path=REGISTRY):
    data = json.loads(Path(path).read_text(encoding="utf-8"))
    if data.get("schema") != 1 or not isinstance(data.get("games"), dict) or not data["games"]:
        raise ValueError("Invalid native client registry")
    for version, client in data["games"].items():
        if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version):
            raise ValueError("Invalid native client version")
        if not isinstance(client, dict) or not re.fullmatch(r"[0-9a-f]{64}", str(client.get("sha256", ""))):
            raise ValueError("Invalid native executable identity")
        archives = client.get("archives")
        if not isinstance(archives, list) or not archives or any(not isinstance(v,str) for v in archives) or len(set(archives)) != len(archives):
            raise ValueError("Invalid native archive inventory")
        for archive in archives:
            if not isinstance(archive, str) or "\\" in archive:
                raise ValueError("Invalid native archive path")
            parts = PurePosixPath(archive).parts
            if len(parts) != 2 or parts[0] != "Data" or parts[1] in (".", "..") or not parts[1].endswith(".MPQ") or ":" in archive:
                raise ValueError("Invalid native archive path")
    return data["games"]

def selected_client(path=REGISTRY):
    clients = load_clients(path)
    # A developer default must be explicit if multiple builds become supported.
    if len(clients) != 1:
        raise ValueError("Ambiguous selected native client")
    version = next(iter(clients))
    return version, clients[version]

VERSION, CLIENT = selected_client()
SHA256 = CLIENT["sha256"]
NAMES = ("AzerothSurvivors.exe", *CLIENT["archives"])
