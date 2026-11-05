# AZSX artifact identity and reproducibility

Each release records its own DLL identity automatically. No human hash approval
and no comparison with a DLL from an older release are required. A changed hash
is expected after a code/client update; it identifies bytes, not correct behavior.

## Build and identify the candidate

Use [BUILDING.md](../development/BUILDING.md) and the
[release checklist](RELEASE-CHECKLIST.md):

```sh
python3 tools/build/azsx_reproducibility.py --candidate "$CANDIDATE"
python3 scripts/check_release_artifact.py --candidate "$CANDIDATE" --dll "$DLL"
```

The producer makes two fresh independent normal builds. Both must have identical
bytes, unchanged source/Git identity and unchanged external build configuration.
Receipts establish absent diagnostic hooks, PE timestamp 1 and no debug directory.
A mismatch remains a technical failure to diagnose, never a request to approve a
new hash. Successful output has `status: verified`, source commit/dirty state,
native version, DLL hash/size, source fingerprint and build recipe/toolchain.

This establishes reproduction in the recorded environment, without a claim about
another host or Windows-native tooling. Tests and actual runtime qualification
remain separate evidence; matching builds do not establish gameplay correctness.

## Per-release record

Retain the successful report as `REPRODUCIBILITY.json` in the intermediate release
folder, alongside `SOURCE-COMMIT.json` and honest `VALIDATION.json` results.
The finalizer automatically creates `RELEASE-IDENTITY.json` under
`.local/releases/<version>/`. It reads the DLL from the authenticated upload
archive and verifies that it matches both the signed manifest and its own
two-build report. The compact record binds:

- AZSX/native versions and exact DLL hash/size.
- Release/payload source commits, plus the DLL's recorded build source.
- Archive hash, reproducibility report and validation results.

Later publication verification checks that record again. Re-finalization is
idempotent; conflicting existing records are preserved and rejected. Old release
assets and evidence are never silently rewritten. The four public assets and
minimal preparation-folder layout stay unchanged.

For launcher/docs-only changes, an already qualified DLL may be reused when
compilation inputs and native target are verified unchanged. Carry its existing
build report and artifact-scoped validation into the new release; record the reuse
rationale and distinct build/release sources. Do not claim fresh builds or trials.

## Historical records and authorization

`validation/azsx-reproducible-build-xwin.json` remains an unchanged historical
record for **0.139.2-azsx.1**, native **0.139.2**, DLL SHA-256
`5760e27f4e873259b68b467c89b3d11c3a79ba3bf7d4e7fa6cc81b30f64f8eff`.
Its recorded approval and bounded trial scope remain historical facts; they do
not gate a new release. Historical two-build reports marked `pending-review`
can supply byte/provenance evidence without introducing a manual approval step.

The separate legacy Windows-native reproducer is not the production WSL release
workflow; an xwin result makes no Windows-native reproducibility claim.
Source hygiene, signed-package checks and required runtime validation still apply.
Commit, push, tag creation and publication continue to require explicit requests.
