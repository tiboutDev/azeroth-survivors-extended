"""Build and verify a normal Windows ASX artifact without developer paths."""
import argparse
import hashlib
import json
import os
import shlex
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TARGET = "x86_64-pc-windows-msvc"
PACKAGE = "azeroth-coop-native"
ARTIFACT = "avrt.dll"


def build(target_dir: Path, receipt_path: Path | None = None) -> Path:
    target_dir = target_dir.resolve()
    if receipt_path is not None:
        receipt_path = receipt_path.resolve()
        if receipt_path.exists():
            raise FileExistsError(f"Refusing to replace build receipt: {receipt_path.name}")

    sysroot = subprocess.check_output(["rustc", "--print", "sysroot"], text=True).strip()
    cargo_home = os.environ.get("CARGO_HOME", str(Path.home() / ".cargo"))
    prefixes = {str(ROOT): "/asx", cargo_home: "/cargo", sysroot: "/rust"}
    flags = [
        *("--remap-path-prefix=" + source + "=" + dest for source, dest in prefixes.items()),
        "-Cdebuginfo=0",
        "-Clink-arg=/DEBUG:NONE",
        "-Clink-arg=/TIMESTAMP:1",
    ]
    env = dict(
        os.environ,
        RUSTFLAGS=shlex.join(flags),
        CARGO_PROFILE_RELEASE_DEBUG="0",
        CARGO_BUILD_JOBS="2",
    )
    # Cargo gives the encoded variant precedence over RUSTFLAGS. A developer's
    # shell override must not silently replace the audited production recipe.
    env.pop("CARGO_ENCODED_RUSTFLAGS", None)
    print("Building normal ASX Windows distribution artifact", flush=True)
    subprocess.run(
        [
            "cargo", "xwin", "build", "--release", "--locked", "--target", TARGET,
            "--target-dir", str(target_dir), "-p", PACKAGE, "--jobs", "2",
        ],
        cwd=ROOT,
        env=env,
        check=True,
    )

    dll = target_dir / TARGET / "release" / ARTIFACT
    raw = dll.read_bytes()
    needles = [str(ROOT), cargo_home, sysroot, str(Path.home()), str(target_dir), "AZCOOP_TEST_"]
    for needle in filter(None, needles):
        for spelling in {needle, needle.replace("/", "\\")}:
            if spelling.encode() in raw or spelling.encode("utf-16le") in raw:
                raise ValueError("Distribution contains a development-only marker/path")

    import pefile

    image = pefile.PE(data=raw, fast_load=True)
    try:
        timestamp = image.FILE_HEADER.TimeDateStamp
        debug_directory_size = image.OPTIONAL_HEADER.DATA_DIRECTORY[6].Size
    finally:
        image.close()
    if timestamp != 1 or debug_directory_size != 0:
        raise ValueError("Distribution PE metadata/debug configuration differs")

    digest = hashlib.sha256(raw).hexdigest()
    receipt = {
        "schema_version": 1,
        "artifact": {"file": ARTIFACT, "sha256": digest, "size_bytes": len(raw)},
        "game_version": "0.122.0",
        "target": TARGET,
        "package": PACKAGE,
        "profile": "release",
        "locked_dependencies": True,
        "test_hooks": "absent",
        "pe": {"timestamp": timestamp, "debug_directory_size": debug_directory_size},
    }
    if receipt_path is not None:
        receipt_path.parent.mkdir(parents=True, exist_ok=True)
        with receipt_path.open("x", encoding="utf-8", newline="\n") as stream:
            json.dump(receipt, stream, indent=2, sort_keys=True)
            stream.write("\n")

    print("Normal distribution artifact verified: no test hooks, developer paths or PE debug directory", flush=True)
    print(dll, flush=True)
    return dll


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--target-dir",
        type=Path,
        help="Cargo target directory (default: target/asx-distribution)",
    )
    parser.add_argument(
        "--receipt",
        type=Path,
        help="write a machine-free artifact receipt without replacing an existing file",
    )
    args = parser.parse_args(argv)
    target_dir = args.target_dir if args.target_dir is not None else ROOT / "target/asx-distribution"
    build(target_dir, args.receipt)


if __name__ == "__main__":
    main()
