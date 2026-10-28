# Native-folder installation and distribution

ASX targets only the reviewed official **0.125.0** client. The native selector,
supported-game registry and manifest validator reject other builds. Player steps
are in [Installation](player/INSTALLATION.md); this page defines ownership and
release boundaries.

## Owned layout

The archive overlays an existing, closed game folder. It contains `start-asx.cmd`,
`README-ASX.txt` and `ASX/`: the normal DLL under `Bin/`, launcher/update code under
`Client/`, the bundled interpreter under `Runtime/`, addon sources under
`Interface/` and owned settings/state/backups/logs. It does not include the native
EXE, game archives, saves or unrelated addons.

The archive has no root `avrt.dll`. The launcher checks for conflicting loaders
before projecting its own DLL and the two owned addon files. Unknown or modified
files are preserved; they are not silently overwritten. Direct native launch
without ASX activation remains inert.

Native archive inventory is checked independently of raw container hashes, so
repacking a valid native archive does not imply a different executable. The EXE,
native entry bytes and layout retain exact-build verification. Never repair a
failed check by modifying official game assets.

## Native profile and addon activation

Preserve the native save and settings environment, including an existing native
save override. Back up progression before launch without rewriting it. Native
settings stay with the game; ASX does not recreate the retired isolated-profile
launcher. Concurrent use of the same progression path is rejected.

The addon is inactive on disk. The reviewed read gate matches owned file identity
and the fixed prefix, changes one byte in that read buffer and preserves the
original call/status/last-error contract. It is scoped to the game image, not a
global OS hook. Do not add archive scanning or arbitrary addon interception.

## Update and removal transactions

ASX uses its signed `asx-latest.json` feed and Ed25519 verification before applying
content. Bounded ZIP inventories reject duplicates, links, traversal and native
or private-file targets. Verify every payload hash and the installed executable.
Keep sequence high-water marks, ownership receipts and transaction journals.

Changed/obsolete owned files must match their receipts. Restore after interrupted
operations; preserve unknown or user-modified content. After replacing launcher
Python code, restart before relying on the new modules. Rollback must not trigger
a reinstall loop. Export and removal act on owned content only; they never delete
native progression or unrelated files.

## Release preparation

Use the current `asx/` code and `tools/build/asx_*.py` helpers. Signing requires an
explicitly supplied key whose public part matches the trust list; never discover
or bundle private keys. Public release assets are the stable `ASX.zip` alias,
versioned archive, `asx-latest.json` and its signature. Keep internal reports and
publication metadata out of those attachments.

Run the relevant `asx-install` tests and validate the exact signed archive at an
authorized release milestone. [Reproducibility](REPRODUCIBILITY.md), strict source
hygiene and [verification](TESTING.md#publication-checks) are separate gates. Preparation, tests and
a source change never authorize deployment, upload or publication.
