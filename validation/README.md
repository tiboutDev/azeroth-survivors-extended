# Controlled build references

This directory contains versioned artifact references consumed by build checks.
It is separate from Cargo output and generated test reports. It is not a Rust
source directory or a declaration that publication gates have passed.

`asx-reproducible-build-xwin.json` identifies a technically reviewed pair of local
builds for its recorded input fingerprint. It is a technical local-builder
reference, not deployment/publication approval or Windows-native proof. Source
changes require a new candidate and review. Preserve superseded references in
private evidence. See [reproducibility](../docs/REPRODUCIBILITY.md).

The Windows-native reproducer requires its own approved
`asx-reproducible-build.json`. That reference is absent; the gate must remain
unsatisfied until a real reference is established. A local xwin hash is not a
substitute.

Pending candidates, raw logs, trial outputs, cleanup status and session reports
belong in ignored local evidence. Do not accumulate them here or automatically
replace reviewed references after a mismatch.
