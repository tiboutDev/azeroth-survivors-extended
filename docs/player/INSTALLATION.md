# Installing ASX

[Home](../../README.md) · **Installation** · [Features](FEATURES.md) · [In the workshop](WORKSHOP.md)

---

## Requirements

- Windows 64-bit.
- Your own copy of **Azeroth Survivors**, in the exact version supported by your ASX release.
- An ASX release explicitly compatible with that game version.

ASX does not include the original game. These pages describe the current source;
check the release notes for the features and exact compatibility of your download.

## Install and launch

1. Download **ASX.zip**, or the equivalent versioned ASX archive, from the [ASX releases page](https://github.com/tiboutDev/azeroth-survivors-extended/releases).
2. Close Azeroth Survivors.
3. Extract the archive into your game folder, beside `AzerothSurvivors.exe`.
4. Run **`start-asx.cmd`**. The launcher installs the included addons and starts ASX automatically after its checks.

If a signed ASX update matches your installed native executable, it installs automatically without asking. Use `start-asx.cmd --menu` to open maintenance choices;
`--play` is an explicit direct launch.

Use `start-asx.cmd` whenever you want to play with ASX, in solo or co-op.

Your existing progression and settings are reused. The launcher creates a progression backup before each ASX launch.

## Included addons and preferences

No separate ASXUI download or manual addon copy is required. The launcher manages
two complementary folders:

| Folder in your game installation | Purpose |
| --- | --- |
| `benilla-config/AddOns/ASX` | Connects the native ASX mod to Lua, exposes ASX data and enriches buff tooltips. |
| `benilla-config/AddOns/ASXUI` | Displays nameplate styling, minimap markers, notifications and configurable event widgets. |

Keep the top-level `ASX` folder beside the game executable; do not move it into
AddOns. The launcher puts the required addon files in their correct locations.
ASXUI remains an independent Lua addon, while its packaged files are installed
and updated with ASX.

Existing matching ASXUI files can be adopted. Different pre-existing addon code
is preserved until you choose an explicit repair. Modified managed code is backed up before an automatic update replaces it. Native SavedVariables
preferences and unknown files are preserved. Use the [ASXUI commands](../../addons/ASXUI/README.md)
to configure its widgets instead of editing managed source files.

## Play with friends

Launch ASX, then use the game’s official **Co-op** menu to create or join a lobby.

All participants should use the same compatible game and ASX versions.

## Play without ASX

Close the game, then launch **`AzerothSurvivors.exe`** directly. ASX gameplay
remains inactive. ASXUI is a standalone addon, so its independent nameplate
styling can still be present when the addon is enabled in the native client.

## Updates

The launcher checks for signed ASX updates before playing online. Compatible
updates install automatically, including without an interactive console. Successful
updates restart the launcher before continuing. ASXUI updates with the mod. Python
updates finish through an automatic detached helper while the launcher is closed.

To update without starting the game:

```bat
start-asx.cmd --update-only
```

A failed update is not retried automatically. After resolving its cause, use
`start-asx.cmd --retry-update`; no confirmation is required. A rollback remains
held at that release sequence, including explicit retry; a newer release can be offered.

After an official game update, check the ASX release notes for compatibility before playing with the mod.

To launch without checking for updates:

```bat
start-asx.cmd --offline
```

Offline mode skips the update check; it does not make an unsupported game version compatible.

## Check and repair

```bat
start-asx.cmd --check
```

This is an offline, read-only diagnosis. It lists all detected problems, does not
recover transactions, write installation files, download updates or start the game.
Missing or modified documentation and the CMD bootstrap are warnings; they do not
block an already running launcher. Missing or modified DLL, Python or addon
components require repair. Foreign loaders and addons are never silently replaced.

```bat
start-asx.cmd --repair
```

Repair restores the **same ASX release**, using a cached signed package or the
online release if its version matches. Review the proposed changes and confirm.
Modified documentation is preserved. Modified owned critical files are backed up
before replacement. The previous-release rollback remains available.

For an offline repair with an archive and its matching signed release metadata:

```bat
start-asx.cmd --repair --offline --archive "asx-VERSION.zip" --feed "asx-latest.json" --signature "asx-latest.json.sig"
```

These three files must belong to the same release. Repair never substitutes a
newer version silently. Unknown installation identity or a damaged ownership
receipt requires explicit recovery consent; an unknown conflicting root loader
or conflicting internal ASX bridge is still preserved and blocks replacement.
The known ASXUI addon can be migrated only with consent and a backup. An invalid channel
configuration can be restored to the official channel during an approved repair,
with its old contents backed up.

For an interrupted transaction, use `start-asx.cmd --recover`. Recovery requires
confirmation and restarts the launcher if client files were restored.

If Python or the launcher cannot start, or the Python runtime needs replacement,
close ASX and restore it from the complete trusted archive. Repair never modifies
the official game executable, game archives, native progression or settings.

## Roll back an ASX update

After a supported automatic update, you can restore the previous mod version with:

```bat
start-asx.cmd --rollback
```

This rolls back ASX, not the original game.

## Uninstall

Close the game, then run:

```bat
start-asx.cmd --uninstall
```

The uninstaller removes owned, unchanged ASX files, including the managed
ASXUI addon files. Native progression, settings, SavedVariables preferences,
unknown files and modified owned files are preserved.

## Export diagnostics

To create a local diagnostic archive:

```bat
start-asx.cmd --export-diagnostics
```

Reports are never uploaded automatically. Review the archive before sharing it: logs may contain player names and run details.

## Need help?

Open an [issue](https://github.com/tiboutDev/azeroth-survivors-extended/issues) and include:

- Your game and ASX versions.
- The step that failed and any displayed error.
- Whether the game launches directly without ASX.
- A diagnostic export when useful.

## Program folders and migration

ASX/App contains the program, Python and addon sources. ASX/Config contains your
options; ASX/Data contains state, cache, backups, logs and reports. The first
transition from the older .2 layout requires extracting the complete new archive
with the game closed; startup migrates existing settings and evidence automatically.
The old published archive remains unchanged. Later compatible updates are automatic.

An unavailable bundled ASXUI or changed README does not block the native interface.
Use --check for exhaustive read-only diagnostics and --repair for guided repair.
Installation conflicts are preserved. A failed progression backup warns; it does
not prevent playing an otherwise intact installation.
