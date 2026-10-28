"""Archive local build artifacts and test evidence before removing generated trees.

The default invocation only reports what would be archived or discarded.  The
explicit --apply operation is limited to this checkout's target/ and
test-results/ trees.
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import os
import secrets
import shutil
import stat
import tarfile
import tempfile
import time
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
PURGE_NAMES = ("test-results", "target")
TARGET_SUFFIXES = {".dll", ".exe", ".pdb", ".map"}
RECEIPT_NAME = "build-receipt.json"
BUFFER_SIZE = 1024 * 1024


class CleanupError(RuntimeError):
    """A safety check prevented local artifact cleanup."""


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        while chunk := stream.read(BUFFER_SIZE):
            digest.update(chunk)
    return digest.hexdigest()


def _is_target_artifact(path: Path) -> bool:
    return path.name == RECEIPT_NAME or path.suffix.lower() in TARGET_SUFFIXES


def _root_paths(root: Path) -> dict[str, Path]:
    root = root.resolve(strict=True)
    paths = {name: root / name for name in PURGE_NAMES}
    paths["evidence"] = root / ".local" / "evidence"
    for name in PURGE_NAMES:
        path = paths[name]
        if path.exists() or path.is_symlink():
            if path.is_symlink() or not path.is_dir():
                raise CleanupError(f"refusing non-directory or symlink: {path}")
    cursor = root
    for part in (".local", "evidence"):
        cursor = cursor / part
        if cursor.exists() or cursor.is_symlink():
            if cursor.is_symlink() or not cursor.is_dir():
                raise CleanupError(f"refusing redirected evidence path: {cursor}")
    return paths


def _tracked_paths(root: Path) -> list[str]:
    import subprocess

    result = subprocess.run(
        ["git", "ls-files", "-z", "--", *PURGE_NAMES],
        cwd=root,
        check=True,
        stdout=subprocess.PIPE,
    )
    return [item.decode("utf-8", "surrogateescape") for item in result.stdout.split(b"\0") if item]


def _entries(root: Path, name: str, *, all_files: bool) -> list[dict[str, Any]]:
    base = root / name
    if not base.exists():
        return []
    entries: list[dict[str, Any]] = []
    stack = [base]
    while stack:
        directory = stack.pop()
        try:
            children = sorted(os.scandir(directory), key=lambda item: item.name)
        except OSError as exc:
            raise CleanupError(f"cannot inventory {directory}: {exc}") from exc
        child_dirs: list[Path] = []
        for item in children:
            path = Path(item.path)
            relative = path.relative_to(root).as_posix()
            try:
                info = item.stat(follow_symlinks=False)
            except OSError as exc:
                raise CleanupError(f"cannot inspect {path}: {exc}") from exc
            mode = info.st_mode
            if stat.S_ISDIR(mode):
                entries.append({"path": relative, "kind": "directory", "mode": stat.S_IMODE(mode),
                                "mtime_ns": info.st_mtime_ns})
                child_dirs.append(path)
            elif stat.S_ISLNK(mode):
                if all_files or (name == "target" and _is_target_artifact(path)):
                    target = os.readlink(path)
                    entries.append({"path": relative, "kind": "symlink", "target": target,
                                    "mtime_ns": info.st_mtime_ns})
            elif stat.S_ISREG(mode):
                if all_files or (name == "target" and _is_target_artifact(path)):
                    entries.append(
                        {
                            "path": relative,
                            "kind": "file",
                            "size": info.st_size,
                            "mode": stat.S_IMODE(mode),
                            "mtime_ns": info.st_mtime_ns,
                            "ctime_ns": info.st_ctime_ns,
                            "sha256": sha256_file(path),
                        }
                    )
            else:
                raise CleanupError(f"refusing special file: {path}")
        stack.extend(reversed(child_dirs))
    entries.sort(key=lambda entry: entry["path"])
    return entries


def inventory(root: Path) -> list[dict[str, Any]]:
    """Capture entries that will be archived, plus metadata for discarded cache."""
    result: list[dict[str, Any]] = []
    result.extend(_entries(root, "test-results", all_files=True))
    # Include every target entry's metadata in the stability snapshot. Cache file
    # bytes need not be archived or hashed; retained executable artifacts are.
    result.extend(_entries(root, "target", all_files=False))
    # Track non-retained target cache files too, without reading their contents.
    target = root / "target"
    if target.exists():
        stack = [target]
        while stack:
            directory = stack.pop()
            for item in sorted(os.scandir(directory), key=lambda entry: entry.name):
                path = Path(item.path)
                relative = path.relative_to(root).as_posix()
                info = item.stat(follow_symlinks=False)
                if stat.S_ISDIR(info.st_mode):
                    stack.append(path)
                elif stat.S_ISREG(info.st_mode) and not _is_target_artifact(path):
                    result.append(
                        {
                            "path": relative,
                            "kind": "cache-file",
                            "size": info.st_size,
                            "mtime_ns": info.st_mtime_ns,
                            "ctime_ns": info.st_ctime_ns,
                            "inode": info.st_ino,
                            "mode": stat.S_IMODE(info.st_mode),
                        }
                    )
                elif stat.S_ISREG(info.st_mode):
                    # Retained artifact; _entries above records its content hash.
                    pass
                elif stat.S_ISLNK(info.st_mode):
                    result.append({"path": relative, "kind": "cache-symlink", "target": os.readlink(path)})
                elif not stat.S_ISDIR(info.st_mode):
                    raise CleanupError(f"refusing special file: {path}")
    result.sort(key=lambda entry: entry["path"])
    return result


def _archive_entries(root: Path) -> list[dict[str, Any]]:
    results = _entries(root, "test-results", all_files=True)
    target = _entries(root, "target", all_files=False)
    artifact_paths = {entry["path"] for entry in target if entry["kind"] != "directory"}
    artifact_dirs: set[str] = set()
    for path in artifact_paths:
        parent = Path(path).parent
        while parent.as_posix() not in {"target", "."}:
            artifact_dirs.add(parent.as_posix())
            parent = parent.parent
    target = [entry for entry in target if entry["kind"] != "directory" or entry["path"] in artifact_dirs]
    return sorted(results + target, key=lambda entry: entry["path"])


def _tar_add(tar: tarfile.TarFile, root: Path, entry: dict[str, Any]) -> None:
    path = root / entry["path"]
    info = tarfile.TarInfo(entry["path"] + ("/" if entry["kind"] == "directory" else ""))
    seconds, nanoseconds = divmod(entry["mtime_ns"], 1_000_000_000)
    info.mtime = seconds
    info.pax_headers["mtime"] = f"{seconds}.{nanoseconds:09d}"
    if entry["kind"] == "directory":
        info.type = tarfile.DIRTYPE
        info.mode = entry["mode"]
        tar.addfile(info)
    elif entry["kind"] == "symlink":
        info.type = tarfile.SYMTYPE
        info.linkname = entry["target"]
        tar.addfile(info)
    else:
        st = path.stat(follow_symlinks=False)
        info.type = tarfile.REGTYPE
        info.size = st.st_size
        info.mode = stat.S_IMODE(st.st_mode)
        with path.open("rb") as source:
            tar.addfile(info, source)


def _verify_archive(archive: Path, entries: list[dict[str, Any]]) -> None:
    expected = {entry["path"]: entry for entry in entries}
    seen: set[str] = set()
    try:
        with tarfile.open(archive, "r:gz") as tar:
            for member in tar:
                name = member.name.rstrip("/")
                entry = expected.get(name)
                if entry is None or name in seen:
                    raise CleanupError(f"unexpected or duplicate archive member: {name}")
                seen.add(name)
                seconds, nanoseconds = divmod(entry["mtime_ns"], 1_000_000_000)
                if member.pax_headers.get("mtime") != f"{seconds}.{nanoseconds:09d}":
                    raise CleanupError(f"archived timestamp differs: {name}")
                if entry["kind"] == "directory":
                    if not member.isdir():
                        raise CleanupError(f"archived directory has wrong type: {name}")
                    continue
                if entry["kind"] == "symlink":
                    if not member.issym() or member.linkname != entry["target"]:
                        raise CleanupError(f"archived symlink differs: {name}")
                else:
                    if not member.isfile():
                        raise CleanupError(f"archived file has wrong type: {name}")
                    stream = tar.extractfile(member)
                    if stream is None:
                        raise CleanupError(f"cannot read archived file: {name}")
                    digest = hashlib.sha256()
                    while chunk := stream.read(BUFFER_SIZE):
                        digest.update(chunk)
                    if digest.hexdigest() != entry["sha256"]:
                        raise CleanupError(f"archived file hash differs: {name}")
    except (OSError, tarfile.TarError) as exc:
        raise CleanupError(f"cannot verify archive {archive}: {exc}") from exc
    if seen != set(expected):
        missing = sorted(set(expected) - seen)
        raise CleanupError(f"archive is missing entries: {missing[:8]}")


def _new_paths(evidence: Path) -> tuple[Path, Path]:
    stamp = time.strftime("%Y%m%dT%H%M%SZ", time.gmtime())
    while True:
        stem = f"local-artifacts-{stamp}-{secrets.token_hex(4)}"
        archive = evidence / f"{stem}.tar.gz"
        index = evidence / f"{stem}.index.json"
        if not archive.exists() and not index.exists():
            return archive, index


def _atomic_create(path: Path, writer: Any) -> None:
    fd, temporary_name = tempfile.mkstemp(prefix=".local-artifacts-", dir=path.parent)
    os.close(fd)
    temporary = Path(temporary_name)
    try:
        writer(temporary)
        os.link(temporary, path)
    except FileExistsError as exc:
        raise CleanupError(f"refusing to overwrite existing file: {path}") from exc
    finally:
        temporary.unlink(missing_ok=True)


def summary(root: Path = ROOT) -> dict[str, Any]:
    paths = _root_paths(root)
    tracked = _tracked_paths(root)
    if tracked:
        raise CleanupError(f"tracked files exist under purge roots: {tracked[:8]}")
    evidence = _archive_entries(root)
    target_names = [
        entry["path"]
        for entry in evidence
        if entry["path"].startswith("target/") and entry["kind"] != "directory"
    ]
    test_entries = [entry for entry in evidence if entry["path"].startswith("test-results/")]
    target = paths["target"]
    test_results = paths["test-results"]
    return {
        "mode": "dry-run",
        "test_results_entries_to_archive": len(test_entries),
        "test_results_bytes_to_archive": sum(e.get("size", 0) for e in test_entries),
        "target_artifacts_to_archive": target_names,
        "target_bytes_to_archive": sum(e.get("size", 0) for e in evidence if e["path"].startswith("target/") and e["kind"] == "file"),
        "target_cache_present": target.exists(),
        "test_results_present": test_results.exists(),
        "archive_destination": str(paths["evidence"]),
        "deletion_requires": "explicit --apply after archive verification and unchanged inventory",
    }


def apply(root: Path = ROOT) -> dict[str, Any]:
    root = root.resolve(strict=True)
    paths = _root_paths(root)
    tracked = _tracked_paths(root)
    if tracked:
        raise CleanupError(f"tracked files exist under purge roots: {tracked[:8]}")
    before = inventory(root)
    entries = _archive_entries(root)
    evidence = paths["evidence"]
    evidence.mkdir(parents=True, exist_ok=True)
    # Recheck the full path after creation before writing anything there.
    _root_paths(root)
    archive, index = _new_paths(evidence)

    def write_archive(temporary: Path) -> None:
        with tarfile.open(temporary, "w:gz", compresslevel=6) as tar:
            for entry in entries:
                _tar_add(tar, root, entry)

    _atomic_create(archive, write_archive)
    _verify_archive(archive, entries)
    index_value = {
        "format": 1,
        "created_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "archive": archive.relative_to(root).as_posix(),
        "archive_sha256": sha256_file(archive),
        "entries": [
            {key: value for key, value in entry.items() if key not in {"ctime_ns", "inode"}}
            for entry in entries
        ],
    }
    _atomic_create(index, lambda path: path.write_text(json.dumps(index_value, indent=2) + "\n"))

    if inventory(root) != before:
        raise CleanupError("source inventory changed during archive; no purge was performed")
    # Revalidate tracked state and fixed roots immediately before bounded deletion.
    tracked = _tracked_paths(root)
    if tracked:
        raise CleanupError(f"tracked files appeared under purge roots: {tracked[:8]}")
    _root_paths(root)
    removed = []
    for name in PURGE_NAMES:
        path = root / name
        if path.exists():
            shutil.rmtree(path)
            removed.append(name)
    return {
        "mode": "applied",
        "archive": archive.relative_to(root).as_posix(),
        "index": index.relative_to(root).as_posix(),
        "archived_entries": len(entries),
        "removed_trees": removed,
        "verified": True,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true", help="archive, verify, then remove target/ and test-results/")
    args = parser.parse_args()
    try:
        result = apply() if args.apply else summary()
    except (CleanupError, OSError) as exc:
        parser.error(str(exc))
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
