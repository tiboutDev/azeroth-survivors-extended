"""Restore Windows SDK name lookup after copying its NTFS cache to Linux."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def prepare():
    cache = ROOT / ".dev-tools/xwin/xwin"
    if not cache.is_dir():
        raise SystemExit("Project-local SDK cache is missing")
    # rust-lld also needs lowercase import-library names on Linux.
    for path in sorted(cache.rglob("*"), key=lambda p: len(p.parts), reverse=True):
        if path.is_symlink():
            continue
        alias = path.with_name(path.name.lower())
        if alias != path and not alias.exists() and not alias.is_symlink():
            alias.symlink_to(path.name, target_is_directory=path.is_dir())

    def entry(path):
        if path.is_dir():
            return dict(type="directory", name=path.name,
                        contents=[entry(child) for child in sorted(path.iterdir()) if not child.is_symlink()])
        return dict(type="file", name=path.name, **{"external-contents": str(path)})

    tree = entry(cache)
    tree["name"] = str(cache)
    overlay = {"version": 0, "case-sensitive": False, "roots": [tree]}
    destination = ROOT / ".dev-tools/windows-sdk-vfs.json"
    destination.write_text(json.dumps(overlay))
    print("Prepared case-insensitive compiler view:", destination)


if __name__ == "__main__":
    prepare()
