# ASX build reproducibility

The stored local reference identifies a prior 0.122.0 artifact. It remains
unchanged as evidence of its exact inputs; it is not a current 0.125.0 reference.
The current reference check must reject it until a new normal artifact is built
and reviewed at an explicitly selected milestone.

The [reviewed local WSL reference](../validation/asx-reproducible-build-xwin.json)
records two fresh, isolated normal builds of the same source fingerprint. Both
produce this 1,466,880-byte DLL:

```text
45a5a48134536a6050359cbecf6f25dcac7248541625fac69ac7bf9651aa05ba
```

This is a technical reference for the reviewed `xwin-msvc` builder and external
configuration. It is not deployment/publication approval, cross-host proof,
Windows-native reproduction or a live gameplay result. The isolated artifacts
were not installed into any game or shared kit.

The recorded reference identifies the reviewed input fingerprint, not future
working trees automatically. Source changes require a new independent candidate
and technical review; do not relabel an earlier binary as the latest source.
Exact prior references, binaries and receipts remain in private evidence archives.

## Reproduce the local recipe

Prepare the normal WSL tool environment as described in
[building](BUILDING.md). For an intentional reproducibility check:

```sh
python3 tools/build/asx_reproducibility.py --candidate test-results/repro-check/new-candidate.json
python3 scripts/check_xwin_reference.py --candidate test-results/repro-check/new-candidate.json
```

The first command performs two normal builds in separate new target directories,
checks their receipts/PE metadata, developer paths and test markers, and rejects
source changes between builds. It never reuses a prior candidate output or
approves its own result. The second compares artifact, input and recipe identity
with the technically reviewed reference; a mismatch requires investigation and
review, not replacing the old hash automatically.

The reference includes Rust/Cargo/cargo-xwin/Clang versions, lockfile and source
fingerprints, normalized compiler flags and hashes of effective external build
configuration/SDK overlays. External values and machine paths are not published.
Those hashes identify the reviewed configuration; they do not reconstruct a
Windows SDK installation or prove a different host will produce the same bytes.

Source changes legitimately make the old reference mismatch. Preserve it as
historical evidence and establish a newly reviewed reference at the next intended
stabilization milestone. Documentation, audit results and ordinary test files are
outside the artifact input inventory; committed native resources and export lists
are included.

## Separate Windows and publication gates

`scripts/reproduce.ps1` retains its independent Windows-native manifest contract.
No Windows-native compiler was available for this maintenance audit, so no such
reference was invented. The local WSL record cannot be supplied as a substitute
Windows hash. Native Windows reproduction remains unverified and requires its
own reference before the manual comparison can pass.

Source hygiene and historical local-information findings are separate release
gates. A technically reproducible DLL does not establish that Git history is
suitable to publish or that a release archive was validated in game.
