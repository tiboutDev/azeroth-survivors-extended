# Historical build references

This directory preserves recorded artifact references and legacy reproducibility
inputs. It is separate from Cargo output and per-release evidence.

`azsx-reproducible-build-xwin.json` describes the historical 0.139.2 local WSL
build/reference. Preserve it unchanged. New releases do not update or compare
against that file and do not require manual reference approval.

Current production releases automatically record their own DLL hash, source,
client identity and validation under ignored `.local/releases/<version>/`.
See [artifact identity and reproducibility](../docs/release/REPRODUCIBILITY.md).
Build outputs, raw logs and session reports also stay in ignored local storage.

The separate legacy Windows-native reproducer expects its own reference
`azsx-reproducible-build.json`, which is absent. It is not part of the production
WSL release workflow, and an xwin result is not Windows-native evidence.
