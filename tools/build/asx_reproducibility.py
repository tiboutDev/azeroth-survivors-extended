"""Prepare a pending-review reproducibility candidate from two normal ASX builds.

This tool writes only to an explicit path under ignored test-results/. It does not
approve a baseline or create validation/asx-reproducible-build.json.
"""
import argparse
import hashlib
import importlib.metadata
import json
import os
import shlex
from pathlib import Path, PurePosixPath
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
TARGET = "x86_64-pc-windows-msvc"
PACKAGE = "azeroth-coop-native"
ARTIFACT = "avrt.dll"
BUILD_SCRIPT = "tools/build/asx_release_binary.py"
RECIPE = {
    "command": [
        "cargo", "xwin", "build", "--release", "--locked", "--target", TARGET,
        "--target-dir", "<isolated-target-dir>", "-p", PACKAGE, "--jobs", "2",
    ],
    "release_debug": "0",
    "rustflags": [
        "--remap-path-prefix=<checkout>=/asx",
        "--remap-path-prefix=<cargo-home>=/cargo",
        "--remap-path-prefix=<rust-sysroot>=/rust",
        "-Cdebuginfo=0",
        "-Clink-arg=/DEBUG:NONE",
        "-Clink-arg=/TIMESTAMP:1",
    ],
    "cargo_build_jobs": "2",
    "test_hooks": "disabled-and-checked-absent",
}
SOURCE_SUFFIXES = {
    ".rs", ".toml", ".json", ".py", ".lua", ".xml", ".toc", ".cmd", ".ps1",
    ".rc", ".res", ".manifest", ".bin", ".png", ".webp", ".svg", ".txt",
}
SOURCE_DIRS = ("crates", "src", "resources/windows", "native-ui", "asx", "tools/build")
ROOT_INPUTS = (
    "Cargo.toml", "Cargo.lock", "build.rs", ".cargo/config.toml", "rust-toolchain.toml",
    "rust-toolchain",
)


def is_within(path: Path, directory: Path) -> bool:
    try:
        path.relative_to(directory)
        return True
    except ValueError:
        return False


def source_paths(root: Path) -> list[Path]:
    root = root.resolve()
    if any(part.is_symlink() for name in ROOT_INPUTS for part in (root / name, *(root / name).parents)
           if part == root or root in part.parents):
        raise ValueError("Build inputs must not use local symbolic links")
    selected = {root / name for name in ROOT_INPUTS if (root / name).is_file()}
    for name in SOURCE_DIRS:
        directory = root / name
        if directory.is_symlink():
            raise ValueError("Build input directories must not use local symbolic links")
        if not directory.is_dir():
            continue
        for path in directory.rglob("*"):
            relative_parts = path.relative_to(root).parts
            if "tests" in relative_parts or "test-results" in relative_parts or "target" in relative_parts:
                continue
            if path.is_symlink():
                raise ValueError("Build inputs must not use local symbolic links")
            if not path.is_file() or path.suffix.lower() not in SOURCE_SUFFIXES:
                continue
            selected.add(path)
    builder = root / BUILD_SCRIPT
    if builder.is_file():
        selected.add(builder)
    return sorted(selected, key=lambda path: path.relative_to(root).as_posix())


def source_snapshot(root: Path) -> dict:
    root = root.resolve()
    files = []
    digest = hashlib.sha256()
    for path in source_paths(root):
        relative = path.relative_to(root).as_posix()
        content = path.read_bytes()
        item_hash = hashlib.sha256(content).hexdigest()
        size = len(content)
        files.append({"path": relative, "sha256": item_hash, "size_bytes": size})
        digest.update(relative.encode("utf-8"))
        digest.update(b"\0")
        digest.update(item_hash.encode("ascii"))
        digest.update(b"\0")
        digest.update(str(size).encode("ascii"))
        digest.update(b"\n")
    if not files:
        raise ValueError("No Rust/build/resource/manifests found for source fingerprint")
    return {"algorithm": "sha256", "sha256": digest.hexdigest(), "files": files}


def tool_versions() -> dict:
    def version(command: list[str]) -> str:
        return subprocess.check_output(command, cwd=ROOT, text=True, stderr=subprocess.STDOUT).strip()

    return {
        "rustc": version(["rustc", "--version"]),
        "cargo": version(["cargo", "--version"]),
        "cargo_xwin": version(["cargo", "xwin", "--version"]),
        "clang": version(["clang", "--version"]).splitlines()[0],
        "python": sys.version.split()[0],
        "pefile": importlib.metadata.version("pefile"),
    }


def external_environment() -> dict:
    # Hash effective external configuration without publishing paths or values.
    names = {"CC", "CXX", "AR", "CFLAGS", "CXXFLAGS", "CPPFLAGS", "CARGO_INCREMENTAL",
             "CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER", "RUSTC", "RUSTC_WRAPPER"}
    values = {name: value for name, value in os.environ.items()
              if name in names or name.startswith("CARGO_PROFILE_")}
    overlays = []
    for name in ("CFLAGS", "CXXFLAGS"):
        for value in shlex.split(os.environ.get(name, "")):
            if value.endswith(".json") and Path(value).is_file():
                overlays.append(hashlib.sha256(Path(value).read_bytes()).hexdigest())
    return {"configuration_sha256": hashlib.sha256(json.dumps(values, sort_keys=True).encode()).hexdigest(),
            "sdk_overlay_sha256": sorted(set(overlays)),
            "scope": "Same reviewed external toolchain environment; no cross-host reproduction claim"}


def production_build(target_dir: Path, receipt_path: Path) -> None:
    command = [
        sys.executable,
        str(ROOT / BUILD_SCRIPT),
        "--target-dir", str(target_dir),
        "--receipt", str(receipt_path),
    ]
    subprocess.run(command, cwd=ROOT, check=True)


def read_verified_build(target_dir: Path, receipt_path: Path) -> dict:
    receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
    if receipt.get("schema_version") != 1:
        raise ValueError("Unexpected production-builder receipt schema")
    if (receipt.get("game_version"), receipt.get("target"), receipt.get("package"), receipt.get("profile")) != (
        "0.125.0", TARGET, PACKAGE, "release"
    ):
        raise ValueError("Production-builder receipt does not describe the expected ASX build")
    if receipt.get("test_hooks") != "absent":
        raise ValueError("Production-builder receipt does not prove test hooks are absent")
    if receipt.get("locked_dependencies") is not True:
        raise ValueError("Production-builder receipt does not prove locked dependencies")
    artifact = receipt.get("artifact", {})
    if artifact.get("file") != ARTIFACT:
        raise ValueError("Production-builder receipt names an unexpected artifact")
    artifact_path = target_dir / TARGET / "release" / ARTIFACT
    raw = artifact_path.read_bytes()
    actual = {"sha256": hashlib.sha256(raw).hexdigest(), "size_bytes": len(raw)}
    if (artifact.get("sha256"), artifact.get("size_bytes")) != (actual["sha256"], actual["size_bytes"]):
        raise ValueError("Production-builder receipt does not match its artifact bytes")
    if receipt.get("pe") != {"timestamp": 1, "debug_directory_size": 0}:
        raise ValueError("Production-builder receipt does not prove the expected PE metadata")
    return actual


def candidate_path(root: Path, requested: Path) -> Path:
    root = root.resolve()
    path = requested.resolve()
    if (root / "test-results").is_symlink():
        raise ValueError("Candidate output must not redirect through a local symbolic link")
    results = (root / "test-results").resolve()
    if not is_within(path, results):
        raise ValueError("Candidate output must be under ignored test-results/")
    if is_within(path, (root / "validation").resolve()):
        raise ValueError("A pending candidate cannot be written under validation/")
    if path.name == "asx-reproducible-build.json":
        raise ValueError("A pending candidate cannot create the approved reproducibility reference")
    if path.exists():
        raise FileExistsError("Refusing to replace an existing candidate report")
    return path


def assert_machine_free(candidate: dict, root: Path) -> None:
    forbidden = {str(root.resolve()), str(Path.home().resolve())}
    if os.environ.get("CARGO_HOME"):
        forbidden.add(str(Path(os.environ["CARGO_HOME"]).resolve()))
    for value in forbidden:
        if not value:
            continue
        if value in json.dumps(candidate, sort_keys=True):
            raise ValueError("Candidate report would include a machine-specific path")
    for entry in candidate["source_fingerprint"]["files"]:
        rel = PurePosixPath(entry["path"])
        if rel.is_absolute() or ".." in rel.parts:
            raise ValueError("Candidate source inventory must use repository-relative paths")


def create_candidate(root: Path, requested: Path, builder=None, versions_provider=None) -> dict:
    root = root.resolve()
    report_path = candidate_path(root, requested)
    builder = builder or production_build
    versions_provider = versions_provider or tool_versions
    versions = versions_provider()
    environment = external_environment()
    before = source_snapshot(root)
    lock = root / "Cargo.lock"
    if not lock.is_file():
        raise ValueError("Cargo.lock is required for a reproducibility candidate")
    lock_hash = hashlib.sha256(lock.read_bytes()).hexdigest()

    target_base = root / "target"
    if target_base.is_symlink():
        raise ValueError("Refusing a reproducibility target directory redirected outside the ignored checkout target")
    target_root = target_base / "asx-reproducibility-candidate"
    if target_root.is_symlink() or not is_within(target_root.resolve(), root):
        raise ValueError("Reproducibility build directories must remain under the checkout target/")
    target_root.mkdir(parents=True, exist_ok=True)
    build_dirs = [Path(tempfile.mkdtemp(prefix="build-a-", dir=target_root)),
                  Path(tempfile.mkdtemp(prefix="build-b-", dir=target_root))]
    if build_dirs[0] == build_dirs[1]:
        raise RuntimeError("Independent build directories were not created")

    results = []
    for index, target_dir in enumerate(build_dirs):
        receipt_path = target_dir / "build-receipt.json"
        builder(target_dir, receipt_path)
        results.append(read_verified_build(target_dir, receipt_path))
        observed = source_snapshot(root)
        if observed != before:
            point = "between builds" if index == 0 else "after builds"
            raise RuntimeError(f"Source fingerprint changed {point}; no candidate was written")
        if external_environment() != environment:
            raise RuntimeError("External build configuration changed; no candidate was written")

    if results[0] != results[1]:
        raise RuntimeError("Independent ASX builds differ; no candidate was written")

    recipe = {
        **RECIPE,
        "tool_versions": versions,
        "external_environment": environment,
        "cargo_lock_sha256": lock_hash,
        "source_fingerprint_sha256": before["sha256"],
    }
    recipe_hash = hashlib.sha256(
        json.dumps(recipe, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()
    report = {
        "schema_version": 1,
        "status": "pending-review",
        "approved": False,
        "requires_human_approval": True,
        "builder": "xwin-msvc",
        "builder_script": BUILD_SCRIPT,
        "artifact": {
            "file": ARTIFACT,
            "game_version": "0.125.0",
            "target": TARGET,
            "sha256": results[0]["sha256"],
            "size_bytes": results[0]["size_bytes"],
        },
        "verification": {
            "independent_fresh_builds": 2,
            "target_directories_distinct": True,
            "artifacts_byte_identical": True,
            "source_unchanged_before_between_after": True,
        },
        "tool_versions": versions,
        "cargo_lock_sha256": lock_hash,
        "source_fingerprint": before,
        "build_recipe": recipe,
        "build_recipe_sha256": recipe_hash,
    }
    assert_machine_free(report, root)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    with report_path.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(report, stream, indent=2, sort_keys=True)
        stream.write("\n")
    return report


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--candidate", type=Path, required=True,
        help="explicit ignored test-results/ path for the pending-review JSON candidate",
    )
    args = parser.parse_args(argv)
    report = create_candidate(ROOT, args.candidate)
    print("Two independent normal builds matched; candidate remains pending review.", flush=True)
    print(f"Artifact SHA-256: {report['artifact']['sha256']}", flush=True)
    print(f"Candidate: {args.candidate}", flush=True)


if __name__ == "__main__":
    main()
