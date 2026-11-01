# Native-folder installation and distribution

Follow the [release preparation checklist](RELEASE-CHECKLIST.md) for ordered commands, evidence and blocking conditions.

ASX targets only the reviewed official **0.138.0** client. The native selector,
supported-game registry and manifest validator reject other builds. Player steps
are in [Installation](../player/INSTALLATION.md); this page defines ownership and
release boundaries.

## Owned layout

The archive overlays an existing, closed game folder. It contains `start-asx.cmd`,
`README-ASX.txt` and three groups under `ASX/`: distributed code in `App/`
(`Bin`, `Launcher`, `Python`, `AddOns` and `Defaults`), user preferences in
`Config/`, and mutable state/backups/logs in `Data/`. It does not include the native
EXE, game archives, saves or unrelated addons.

The archive has no root `avrt.dll`. The launcher checks for conflicting loaders
before projecting its own DLL and the two owned addon files. Unknown files and foreign loader/addon conflicts are preserved. Verified updates
back up managed modifications before replacing them. Direct native launch
without ASX activation remains inert.

Native archive inventory is checked independently of raw container hashes, so
repacking a valid native archive does not imply a different executable. The EXE,
native entry bytes and layout retain exact-build verification. Never repair a
failed check by modifying official game assets.

## Addon and internal-file ownership

| Location | Owner and role |
| --- | --- |
| `ASX/App/Bin`, `ASX/App/Launcher`, `ASX/App/Python` | Internal DLL source, launcher and bundled Python |
| `ASX/App/AddOns` | Internal sources for the gated ASX presentation and bundled standalone ASXUI |
| `benilla-config/AddOns/ASX` | Owned deployment of that gated presentation; not a standalone UI addon |
| `benilla-config/AddOns/ASXUI` | Bundled native addon, projected and updated by ASX using an exact owned-file list |
| `ASX/App/Defaults`, `ASX/Config` | Packaged defaults and private channel configuration |
| `ASX/Data/State`, `ASX/Data/Updates` | Ownership, locks, update history, signed caches and transactions |
| `ASX/Data/Backups`, `ASX/Data/Logs`, `ASX/Data/Reports` | Preserved private data and local diagnostics |
| `ASX/Config/options.json` | Current launcher event preferences; retained at this path for compatibility |

ASXUI preferences use the native addon's SavedVariables mechanism. They are not
launcher preferences. ASX installs and updates ASXUI automatically. Its Lua addon remains independent
of the native DLL, but its packaged code/font/license files are owned by ASX.
Existing matching files can be adopted; differing pre-existing code requires
explicit consent and a backup. Unknown files and native SavedVariables are never
owned or removed. Internal sources and their projected addon files are distinct.

## Launcher interaction and file policy

No-argument startup launches automatically. `--menu` opens maintenance choices.
A signed update for the installed native executable installs automatically,
including `--update-only`, retry and noninteractive use. No update confirmation
is requested. Invalid signatures, incompatible native hashes, unsafe paths and
foreign loader conflicts still prevent replacement. Foreign optional addons are
preserved without blocking the core update; explicit repair may replace them. Modified owned files are
backed up before a verified update replaces them; unowned addons require explicit
repair rather than silent takeover. Offline mode skips network discovery.

The ordinary launch path checks the exact executable, required native files,
launcher/DLL/retained presentation sources and activation projections. It no
longer hashes all bundled Python libraries or runs the exhaustive diagnostic.
Documentation and update metadata do not block launching an intact compatible
installation. A missing/incomplete ASXUI source falls back to native UI: only a
verifiably owned TOC is backed up and disabled; user addon code and SavedVariables
remain intact. Missing documentation does not disable ASXUI. Optional files can
be restored by repair. Invalid receipts/history block unsafe maintenance writes,
not verified existing activation. Backup write failures warn without blocking
play. The native activation acknowledgement may take up to 120 seconds; wrong
acknowledgements or failure to activate still close the owned game process.

`--check` is offline and read-only, collects issues rather than stopping at the
first, and never recovers or prepares the installation. Diagnostics remain
exportable with invalid manifest metadata. Optional-event errors disable affected
features rather than the entire launcher. Initial extraction is trusted bootstrap;
local hashes are consistency checks, not authentication of already imported code.

`--repair` authenticates a same-version package from cache, the channel or an
explicit local archive/feed/signature triple. It presents a plan and requires
consent. Modified documentation is preserved; updates back it up before replacing
it. Missing documentation can be restored. Owned critical conflicts require
consent and backups. Lost receipts allow guided internal ASX reconstruction, never
implicit adoption of an unknown root loader or projected addon. Invalid channel
configuration can use the embedded official trust root for approved recovery.
Runtime-changing updates on Windows use a private copied interpreter, wait for the current launcher to close, reverify the signed cached package, acquire the installation lock, apply the transaction and restart the launcher automatically. A damaged interpreter that cannot start still requires complete-archive repair.

Maintenance transactions retain the previous-release rollback pointer. Update
history records the sequence high-water mark, installed release and attempt
status separately. Failed attempts require `--retry-update`; rollback holds the
same release sequence. Repair can restore the rolled-back version without lowering
the update high-water mark. Authenticated packages are cached for offline repair;
all archive checks run again on cache use. Native saves and settings are excluded.

A signed future release may carry its reviewed registry before the installed
registry knows that game version. The signed archive registry must agree with its
manifest. This permits updating the launcher, never activating the old DLL on an
unsupported executable. The DLL retains its exact native checks after restart.

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
hygiene and [verification](../development/TESTING.md#publication-checks) are separate gates. Preparation, tests and
a source change never authorize deployment, upload or publication.

ASXUI event-layout preferences migrate to a shared top-HUD anchor on next load;
`/asxui widget anchor reset` restores its centered default. No native client
settings, visibility bindings or official installation are changed by this layout.

Bundled ASXUI now includes the HUD, persistent frame layout and shared native
panel theme modules. The validator accepts the exact previous pre-HUD inventory
for authenticated rollback, as well as the complete current inventory; arbitrary
partial inventories and preference files remain rejected.

### Final publication-folder layout

Every prepared release must finish with exactly `UPLOAD/`,
`README-PREPARATION.md`, `RELEASE-NOTES.md` and `PUBLISH-ON-GITHUB.md` at its root.
`UPLOAD/` contains only the stable archive alias, versioned archive, signed feed
and feed signature. Avoid duplicate root archives, copied verifier scripts and
individual test reports in the publication folder.

The package builder creates an intermediate working folder. After signing and
validation, finalize it with:

```sh
uv run --with cryptography python3 tools/build/asx_release_layout.py \
  --folder /path/to/prepared-release \
  --evidence-dir .local/releases/<version>
```

The finalizer verifies signatures and hashes, copies all remaining root files
into the ignored project evidence directory, checks every copy before removing
originals, and writes a compact README and publication guide. Conflicting evidence
or unexpected directories stop finalization. Native game assets and signing keys
are never evidence inputs. Keep earlier failed trials and complete logs locally.
Use `asx_verify_publication.py --evidence-dir .local/releases/<version>` to read
publication metadata and write verification receipts there, keeping the prepared
folder minimal. Structural reorganization never changes signed asset bytes or
implies validation of new source, reproducibility approval or publication.

## Layout migration and generation boundary

The installed root has three groups: `ASX/App/` for distributed code, Python,
DLLs, addon sources, defaults and manifest; `ASX/Config/` for user options/channel;
and `ASX/Data/` for installation state, update transactions/cache, backups, logs
and reports. Native addons and saves remain in their native locations.

On first start, the new launcher acquires the current lock and a legacy lock,
rejects symlinks, copies explicitly named old trees into the new layout, verifies
every byte and records a durable migration plan before removing old copies.
Conflicting user settings/state are preserved and reported; a newly extracted
App file wins over differing legacy program content, with the old bytes retained
under Data/Backups/Layout. Migration is restartable and does not touch native
archives, progression or unrelated addons. Legacy receipt and journal paths remain
readable through bounded aliases. Existing settings, caches and diagnostic logs
are retained.

The first transition from the published .2 layout requires extracting the complete
new archive beside the closed game: the unchanged .2 launcher cannot understand
the new payload layout. This does not modify the published .2 archive. Afterwards,
compatible signed releases update automatically. Historical packages can still be
authenticated/read, but restoring old launcher code into the new App paths would
break it; cross-layout installation/rollback requires its complete original archive,
not reinterpretation of old code. Ordinary rollback remains transactional within
the new layout generation.
