# Native-folder installation and distribution

ASX targets only the reviewed official **0.130.1** client. The native selector,
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

## Addon and internal-file ownership

| Location | Owner and role |
| --- | --- |
| `ASX/Bin`, `ASX/Client`, `ASX/Runtime` | Internal DLL source, launcher and bundled Python |
| `ASX/Interface` | Internal sources for the gated ASX gameplay presentation |
| `benilla-config/AddOns/ASX` | Owned deployment of that gated presentation; not a standalone UI addon |
| `benilla-config/AddOns/ASXUI` | Independently installed native addon; outside the ASX installer inventory |
| `ASX/Defaults`, `ASX/Settings` | Packaged defaults and private channel configuration |
| `ASX/State`, `ASX/Updates` | Ownership, locks, update history, signed caches and transactions |
| `ASX/Backups`, `ASX/Logs`, `ASX/Reports` | Preserved private data and local diagnostics |
| `ASX/options.json` | Current launcher event preferences; retained at this path for compatibility |

ASXUI preferences use the native addon's SavedVariables mechanism. They are not
launcher preferences. The ASX updater/uninstaller does not own, remove or silently
replace an independently installed ASXUI. The internal `ASX/Interface` source and
its native addon deployment are distinct from the optional autonomous ASXUI.

## Launcher interaction and file policy

No-argument startup launches automatically. `--menu` explicitly opens maintenance
choices. All update paths, including update-only and retry, require confirmation;
Enter defers, and noninteractive callers cannot authorize installation implicitly.

`core.file_role` is the product-owned classification: `README-ASX.txt` and `ASX/Client/LICENSE` are
noncritical documentation, `start-asx.cmd` is bootstrap, and
`ASX/Defaults/channel.json` is update metadata. Other payload files are critical.
An invalid ownership receipt blocks maintenance writes, not a launch whose
critical sources and projections are already verified intact. Classification does not come from a filename extension or a
user-editable role in the local manifest. Unsafe paths remain blocking for every role.
The manifest, compatibility registry and trust information gate relevant actions.
Preferences, private state and native data are not release payloads.

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
Runtime replacement under Windows remains a closed-launcher complete-archive step.

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
hygiene and [verification](TESTING.md#publication-checks) are separate gates. Preparation, tests and
a source change never authorize deployment, upload or publication.

ASXUI event-layout preferences migrate to a shared top-HUD anchor on next load;
`/asxui widget anchor reset` restores its centered default. No native client
settings, visibility bindings or official installation are changed by this layout.
