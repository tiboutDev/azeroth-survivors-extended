"""Small checksum-verified copier for explicitly selected private game assets."""
import hashlib
import json
from pathlib import Path
import re
import shutil

SHA122 = "ec3a5b15fa39fed63ce4516c4fc159bdac92691ca44f88befc849010338d14f7"
NAMES = ("AzerothSurvivors.exe", "Data/patch.MPQ", *(f"Data/patch-{n}.MPQ" for n in range(2, 10)), "Data/patch-A.MPQ", "Data/patch-B.MPQ")


def digest(path):
    value = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def inspect(source, sha, names):
    source = Path(source).resolve(strict=True)
    names = tuple(names)
    if names != NAMES or sha != SHA122:
        raise ValueError("Only the reviewed 0.122.0 asset set is supported")
    data = source / "Data"
    if data.is_symlink() or {p.name for p in data.iterdir()} != {Path(n).name for n in names[1:]}:
        raise ValueError("Unexpected official archive inventory")
    if any((source / name).is_symlink() or not (source / name).is_file() for name in names):
        raise ValueError("Expected regular official files")
    if digest(source / names[0]) != SHA122:
        raise ValueError("Exact reviewed official executable required")
    return {name: digest(source / name) for name in names}


def import_runtime(source, receiver, hashes, version="0.122.0"):
    if version != "0.122.0":
        raise ValueError("Only the reviewed 0.122.0 private import is supported")
    if tuple(hashes) != NAMES or hashes.get(NAMES[0]) != SHA122 or any(not re.fullmatch(r"[0-9a-f]{64}", value) for value in hashes.values()):
        raise ValueError("Asset hash manifest does not match the reviewed 0.122.0 inventory")
    source = Path(source).resolve(strict=True)
    if inspect(source, SHA122, NAMES) != hashes:
        raise ValueError("Official source changed after inspection")
    receiver = Path(receiver).resolve(strict=True)
    tag = version.replace(".", "")
    target = receiver / ("runtime-official-" + tag)
    if target.is_symlink() or target.resolve().parent != receiver:
        raise ValueError("Private runtime escaped paired receiver")
    if (target / "Data").is_symlink():
        raise ValueError("Private archive directory redirected")
    receipt = target / ("IMPORT-OFFICIAL-" + tag + ".json")
    if target.exists():
        if receipt.is_symlink() or not receipt.is_file():
            raise ValueError("Existing private receipt is not a regular owned file; preserving it")
        if any((target / name).is_symlink() or not (target / name).is_file() for name in hashes):
            raise ValueError("Existing private assets are not regular owned files; preserving them")
        if json.loads(receipt.read_text()).get("files") != hashes:
            raise ValueError("Existing private runtime differs; preserving it")
        if any(digest(target / name) != expected for name, expected in hashes.items()):
            raise ValueError("Private assets changed; preserving them")
        return target

    target.mkdir()
    for name, expected in hashes.items():
        relative = Path(name)
        if relative.is_absolute() or ".." in relative.parts:
            raise ValueError("Asset name escaped private runtime")
        original = source / relative
        if original.is_symlink() or not original.is_file():
            raise ValueError("Expected regular official file: " + name)
        destination = target / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(original, destination)
        if digest(destination) != expected:
            raise ValueError("Official copy checksum mismatch: " + name)
        print("Verified " + name, flush=True)
    receipt.write_text(json.dumps(dict(version=version, engine="official", files=hashes, gameplay_parity_verified=False), indent=2) + "\n")
    return target
