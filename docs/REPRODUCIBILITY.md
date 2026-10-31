# ASX build reproducibility

The [approved local WSL reference](../validation/asx-reproducible-build-xwin.json)
identifies the approved **0.131.0** normal DLL (1,998,336 bytes):

```text
baab84d20495fd990715320904e6a285a4fd8c59097adac6d2b4dd9e806eaf29
```

Two fresh isolated builds produced identical bytes with unchanged source inputs.
Receipts verify path remapping, disabled diagnostic hooks, PE timestamp 1 and no
debug directory. The exact DLL passed solo and two-player normal live trials,
plus installation/startup/coop/uninstall of its signed archive.
The user explicitly approved this replacement reference on **2026-10-31**.
The superseded 0.130.1 reference remains in Git history and preserved private
evidence. Pending candidates remain separate from the approved reference.

## Reproduce

Prepare the WSL environment in [BUILDING.md](BUILDING.md), then run:

```sh
python3 tools/build/asx_reproducibility.py --candidate test-results/repro-check/new-candidate.json
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
