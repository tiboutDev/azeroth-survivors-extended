# Installing ASX

[Home](../../README.md) · **Installation** · [Features](FEATURES.md) · [In the workshop](WORKSHOP.md)

---

## Requirements

- Windows 64-bit.
- Your own copy of **Azeroth Survivors**, in the exact version supported by your ASX release.
- An ASX release explicitly compatible with that game version.

ASX does not include the original game. Check the release notes for compatibility information before installing.

## Install and launch

1. Download the compatible archive from the [ASX releases page](https://github.com/tiboutDev/azeroth-survivors-extended/releases).
2. Close Azeroth Survivors.
3. Extract the archive into your game folder, beside `AzerothSurvivors.exe`.
4. Run **`start-asx.cmd`**.

Use `start-asx.cmd` whenever you want to play with ASX, in solo or co-op.

Your existing progression and settings are reused. The launcher creates a progression backup before each ASX launch.

## Play with friends

Launch ASX, then use the game’s official **Co-op** menu to create or join a lobby.

All participants should use the same compatible game and ASX versions.

## Play without ASX

Close the game, then launch **`AzerothSurvivors.exe`** directly.

## Updates

The launcher checks for signed ASX updates.

After an official game update, check the ASX release notes for compatibility before playing with the mod.

To launch without checking for updates:

```bat
start-asx.cmd --offline
```

Offline mode skips the update check; it does not make an unsupported game version compatible.

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

The uninstaller removes owned, unchanged ASX files while preserving your progression and settings.

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
