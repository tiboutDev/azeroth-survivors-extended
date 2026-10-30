# ASX build reproducibility

The [approved local WSL reference](../validation/asx-reproducible-build-xwin.json)
identifies the 0.130.0 normal DLL (1,943,552 bytes):

```text
33d8916a64f61d6e8ac2f2d1693f0b2b0cbb877a6c62d4f68695e13951f45f63
```

Two fresh isolated builds produced identical bytes with unchanged source inputs.
Receipts verify path remapping, disabled diagnostic hooks, PE timestamp 1 and no
debug directory. The exact DLL passed solo and two-player normal live trials.
The user approved this reference on 2026-10-30. The historical 0.122.0 reference
is preserved alongside it; candidates remain separate evidence.

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
verification, archive installation and publication are separate gates. This
release preparation does not test archive installation or authorize publication.
