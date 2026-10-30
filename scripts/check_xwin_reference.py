"""Compare a fresh two-build candidate with the reviewed local WSL reference.

This check never builds, installs or publishes. Windows-native references are a
different builder and cannot be substituted here.
"""
import argparse
import json
import sys
from pathlib import Path


sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools/dev"))
from client_registry import selected_client

def verify(reference, candidate):
    if reference.get("status") != "technically-reviewed-local-reference":
        raise ValueError("Expected a technically reviewed local reference")
    if reference.get("builder") != "xwin-msvc" or candidate.get("builder") != "xwin-msvc":
        raise ValueError("Builder mismatch; native Windows references are separate")
    expected = reference.get("artifact", {})
    if expected.get("file") != "avrt.dll" or expected.get("game_version") != selected_client()[0]:
        raise ValueError("Reference does not identify the supported ASX artifact")
    if candidate.get("status") != "pending-review":
        raise ValueError("Expected a fresh two-build candidate")
    gates = candidate.get("verification", {})
    if gates.get("independent_fresh_builds") != 2 or any(gates.get(key) is not True for key in (
        "target_directories_distinct", "artifacts_byte_identical", "source_unchanged_before_between_after"
    )):
        raise ValueError("Candidate does not establish two unchanged independent builds")
    for field in ("artifact", "cargo_lock_sha256", "source_fingerprint", "build_recipe_sha256"):
        if field not in reference or candidate.get(field) != reference[field]:
            raise ValueError("Candidate differs from the reviewed reference: " + field)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reference", type=Path, default=Path(__file__).resolve().parents[1] / "validation/asx-reproducible-build-xwin.json")
    parser.add_argument("--candidate", type=Path, required=True)
    args = parser.parse_args()
    verify(json.loads(args.reference.read_text()), json.loads(args.candidate.read_text()))
    print("Two-build candidate matches the technically reviewed local WSL reference.")


if __name__ == "__main__":
    main()
