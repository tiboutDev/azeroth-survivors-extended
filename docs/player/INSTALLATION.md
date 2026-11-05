# Installing AZSX

[Home](../../README.md) · **Installation** · [Features](FEATURES.md) · [In the workshop](WORKSHOP.md)

---

## Requirements

- Windows 64-bit.
- Your own copy of **Azeroth Survivors**, in the exact version supported by your AZSX release.
- An AZSX release explicitly compatible with that game version.

AZSX does not include the original game. These pages describe the current source;
check the release notes for the features and exact compatibility of your download.

GitHub release titles show the supported game version first: **0.141.0 — AZSX
Update 1** supports game **0.141.0**. Its AZSX version is `0.141.0-r1`; `r2`
would be the next AZSX update for that same game version.

## Install and launch

1. Download **AZSX.zip**, or the equivalent versioned AZSX archive, from the [AZSX releases page](https://github.com/tiboutDev/azeroth-survivors-extended/releases).
2. Close Azeroth Survivors.
3. Extract the archive into your game folder, beside `AzerothSurvivors.exe`.
4. Run **`start-azsx.cmd`**. The launcher installs the included addons and starts AZSX automatically after its checks.

If a signed AZSX update matches your installed native executable, it installs automatically without asking. Use `start-azsx.cmd --menu` to open maintenance choices;
`--play` is an explicit direct launch.

Use `start-azsx.cmd` whenever you want to play with AZSX, in solo or co-op.

Your existing progression and settings are reused. AZSX retains the five latest
distinct progression backups and the five latest session logs automatically.

## Included addons and preferences

No separate AZSXUI download or manual addon copy is required. The launcher
manages `benilla-config/AddOns/AZSXUI` for nameplate styling, minimap markers,
notifications and configurable event widgets. The AZSX Lua provider initializes
through the selected native client; there is no internal `AddOns/AZSX` addon.

Keep the top-level `AZSX` folder beside the game executable; do not move it into
AddOns. AZSXUI remains an independent Lua addon, while its packaged files are
installed and updated with AZSX.

Existing matching AZSXUI files can be adopted. Different pre-existing addon code
is preserved until you choose an explicit repair. Modified managed code is backed up before an automatic update replaces it. Native SavedVariables
preferences and unknown files are preserved. Use the [AZSXUI commands](../../addons/AZSXUI/README.md)
to configure its widgets instead of editing managed source files.

## Play with friends

Launch AZSX, then use the game’s official **Co-op** menu to create or join a lobby.

All participants should use the same compatible game and AZSX versions.

## Play without AZSX

Close the game, then launch **`AzerothSurvivors.exe`** directly. AZSX gameplay
remains inactive. AZSXUI is a standalone addon, so its independent nameplate
styling can still be present when the addon is enabled in the native client.

## Updates

The launcher checks for signed AZSX updates before playing online. Compatible
updates install automatically, including without an interactive console. Successful
updates restart the launcher before continuing. AZSXUI updates with the mod. Python
updates finish through an automatic detached helper while the launcher is closed.

To update without starting the game:

```bat
start-azsx.cmd --update-only
```

Failed updates retry automatically at the next launch, once per invocation.
A voluntary rollback stays held at that release sequence; a newer release can
still be offered. Interrupted installation transactions recover automatically.

After an official game update, check the AZSX release notes for compatibility before playing with the mod.

To launch without checking for updates:

```bat
start-azsx.cmd --offline
```

Offline mode skips the update check; it does not make an unsupported game version compatible.

## Check and repair

```bat
start-azsx.cmd --check
```

This is an offline, read-only diagnosis. It lists all detected problems, does not
recover transactions, write installation files, download updates or start the game.
Missing or modified documentation and the CMD bootstrap are warnings; they do not
block an already running launcher. Missing or modified critical DLL, launcher or
internal AZSX presentation files are repaired automatically when a signed
same-version package is available. If the bundled AZSXUI is unavailable,
AZSX can use the native interface; repair can restore the optional addon. Foreign
loaders and addons are never silently replaced.

```bat
start-azsx.cmd --repair
```

Repair restores the **same AZSX release**, using a cached signed package or the
online release if its version matches. Review the proposed changes and confirm.
Modified documentation is preserved. Modified owned critical files are backed up
before replacement. The previous-release rollback remains available.

For an offline repair with an archive and its matching signed release metadata:

```bat
start-azsx.cmd --repair --offline --archive "azsx-VERSION.zip" --feed "azsx-latest.json" --signature "azsx-latest.json.sig"
```

These three files must belong to the same release. Repair never substitutes a
newer version silently. Unknown installation identity or a damaged ownership
receipt requires explicit recovery consent; an unknown conflicting root loader
or conflicting internal AZSX bridge is still preserved and blocks replacement.
The known AZSXUI addon can be migrated only with consent and a backup. An invalid channel
configuration can be restored to the official channel during an approved repair,
with its old contents backed up.

Interrupted transactions recover automatically and restart the launcher before
using restored code. `--recover` remains available for explicit recovery.

Normal signed compatible updates can replace the Python runtime automatically
through the detached update helper. If Python or the launcher is damaged and
cannot start, close AZSX and restore it from the complete trusted archive. Repair
never modifies the official game executable, game archives, native progression
or settings.

## Roll back an AZSX update

After a supported automatic update, you can restore the previous mod version with:

```bat
start-azsx.cmd --rollback
```

This restores one previous AZSX release when it supports the installed game.
It does not roll back the original game or expose an older rollback chain.

## Uninstall

Close the game, then run:

```bat
start-azsx.cmd --uninstall
```

The uninstaller removes owned, unchanged AZSX files, including the managed
AZSXUI addon files. Native progression, settings, SavedVariables preferences,
unknown files and modified owned files are preserved.

## Export diagnostics

To create a local diagnostic archive:

```bat
start-azsx.cmd --export-diagnostics
```

Reports are never uploaded automatically. Review the archive before sharing it: logs may contain player names and run details.

## Need help?

Open an [issue](https://github.com/tiboutDev/azeroth-survivors-extended/issues) and include:

- Your game and AZSX versions.
- The step that failed and any displayed error.
- Whether the game launches directly without AZSX.
- A diagnostic export when useful.

## Program folders and migration

AZSX/App contains the program, Python and addon sources. AZSX/Config contains your
options; AZSX/Data contains state, cache, backups, logs and reports. The first
transition from the older .2 layout requires extracting the complete new archive
with the game closed; startup migrates existing settings and evidence automatically.
The old published archive remains unchanged. Later compatible updates are automatic.

An unavailable bundled AZSXUI or changed README does not block the native interface.
Ordinary owned-file repairs and history cleanup are automatic. Use --check for
exhaustive read-only diagnostics and --repair for guided recovery when needed.
Installation conflicts are preserved. A failed progression backup warns; it does
not prevent playing an otherwise intact installation.

### Native game updates

When an official game update is available, AZSX first checks that a signed AZSX
release supports the exact new game build. Only then does it let the game's own
updater install that build. AZSX downloads and verifies its complete matching
package **before** that step. The updater runs with AZSX inactive; after verifying
the updated client, the launcher installs the cached mod and continues
automatically. Completion can use the cache even if the connection is then lost.
`--update-only` performs the same AZS → AZSX sequence without starting normal play.
If AZSX support is not yet published, you can keep playing the installed build.
