# AZSX build reproducibility

Reproducibility is one gate in the [release preparation checklist](RELEASE-CHECKLIST.md); candidate production never approves a release.

The [approved local WSL reference](../../validation/asx-reproducible-build-xwin.json)
identifies the approved **0.138.0-azsx.2** normal DLL (2,050,560 bytes)
for native **0.138.0**:

```text
7fe7937ca101a05dbd672d0718c878dc08d4a8fc97a1fb84cd1a1b4fc870ff8c
```

Two fresh isolated builds produced identical bytes with unchanged source inputs.
Receipts verify path remapping, disabled diagnostic hooks, PE timestamp 1 and no
debug directory. The exact DLL passed fresh solo and two-player normal live trials,
plus all 26 installation/startup/coop/export/uninstall checks of its
signed archive on a verified private Windows desktop. The user explicitly
approved this replacement reference on **2026-11-02**. The superseded
0.138.0-azsx.1 reference remains in Git history and preserved private evidence.
The old release assets were not modified. These bounded trials use local
native-engine loopback; they do not establish public-lobby or WAN parity.

## Reproduce

Prepare the WSL environment in [BUILDING.md](../development/BUILDING.md), then run:

```sh
python3 tools/build/azsx_reproducibility.py --candidate test-results/repro-check/new-candidate.json
python3 scripts/check_xwin_reference.py --candidate test-results/repro-check/new-candidate.json
```

The producer never approves its own candidate. Changes to source or recipe
require a fresh candidate and review, not replacement of an old hash by assumption.
The reference records source, lockfile, toolchain and external configuration
fingerprints. This establishes same-environment reproduction, not reproduction
on another host or with Windows-native tooling.

## Separate gates

`scripts/reproduce.ps1` retains its independent Windows-native reference contract.
No Windows-native reproducibility claim is made. Source hygiene, signed package
verification, archive installation and publication remain separate gates.
Reference approval does not authorize pushing, tagging or publishing a release.
