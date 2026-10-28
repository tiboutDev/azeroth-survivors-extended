"""Audit publishable files and reachable Git blobs without printing matched values.

Reports contain repository-relative paths, blob IDs, categories and line numbers.
This tool never changes Git history, tracked evidence or ignored local files.
"""
import argparse
import json
import os
import re
import subprocess
from pathlib import Path

RULES = {
    "home_path": re.compile(rb"/(?:home|Users)/(?!runner(?:/|\b))[a-zA-Z0-9_.-]+/"),
    "windows_profile": re.compile(rb"[A-Za-z]:[/\\]+Users[/\\]+[^/\\\r\n\t\"'<>]+[/\\]", re.I),
    "local_drive": re.compile(rb"\b[D-Z]:[/\\]+[a-zA-Z0-9_.-]+[/\\]", re.I),
    "mounted_workspace": re.compile(rb"/mnt/[a-z]/(?!Windows(?:/|\b))[^\s\"'<>]+", re.I),
    "machine_name": re.compile(rb"\b(?:DESKTOP|LAPTOP)-[A-Z0-9]{4,}\b", re.I),
}


def git(root, *args):
    return subprocess.check_output(["git", "-C", str(root), *args])


def publishable_files(root):
    names = git(root, "ls-files", "--cached", "--others", "--exclude-standard", "-z")
    return sorted(set(name.decode("utf-8", "surrogateescape") for name in names.split(b"\0") if name))


def findings(raw):
    for number, line in enumerate(raw.splitlines(), 1):
        for category, pattern in RULES.items():
            if pattern.search(line):
                yield {"line": number, "category": category}


def audit(root, history=False):
    root = root.resolve()
    current, historical = [], []
    scanned = 0
    for name in publishable_files(root):
        path = root / name
        # A tracked link's target is source content; inspect the link text only.
        if path.is_symlink():
            scanned += 1
            current.extend(dict(path=name, **hit) for hit in findings(os.fsencode(os.readlink(path))))
            continue
        # Missing tracked files are pending deletions.
        if not path.is_file():
            continue
        scanned += 1
        current.extend(dict(path=name, **hit) for hit in findings(path.read_bytes()))
    blobs, commits = 0, 0
    if history:
        objects = git(root, "rev-list", "--objects", "--all").splitlines()
        process = subprocess.Popen(["git", "-C", str(root), "cat-file", "--batch"],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        try:
            for entry in objects:
                identity, _, name = entry.partition(b" ")
                process.stdin.write(identity + b"\n")
                process.stdin.flush()
                header = process.stdout.readline().split()
                if len(header) != 3:
                    raise RuntimeError("Git object could not be read")
                size = int(header[2])
                raw = process.stdout.read(size)
                if len(raw) != size or process.stdout.read(1) != b"\n":
                    raise RuntimeError("Incomplete Git object")
                if header[1] == b"commit":
                    commits += 1
                    historical.extend(dict(commit=identity.decode("ascii"), path="<commit>", **hit)
                                      for hit in findings(raw))
                    continue
                if header[1] != b"blob":
                    continue
                blobs += 1
                historical.extend(dict(blob=identity.decode("ascii"),
                    path=name.decode("utf-8", "replace"), **hit) for hit in findings(raw))
        finally:
            process.stdin.close()
            process.stdout.close()
            if process.wait() != 0:
                raise RuntimeError("Git object audit failed")
    return {"schema": 1, "current_files_scanned": scanned, "history_scanned": history,
            "reachable_blobs_scanned": blobs, "reachable_commits_scanned": commits, "current_findings": current,
            "historical_findings": historical,
            "limitations": ["Pattern-based local-information audit, not a complete secrets review",
                            "Reachable refs only; reflogs, unreachable objects and remote-only refs excluded",
                            "History is inspected, never rewritten"]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--history", action="store_true")
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    report = audit(args.root, args.history)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"current_findings": len(report["current_findings"]),
                      "historical_findings": len(report["historical_findings"])}))
    return int(bool(report["current_findings"] or report["historical_findings"]))


if __name__ == "__main__":
    raise SystemExit(main())
