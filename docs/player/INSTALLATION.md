# Installing ASX

[Home](../../README.md) · **Installation** · [Features](FEATURES.md) · [In the workshop](WORKSHOP.md)

---

## Requirements

- Windows 64-bit.
- Your own copy of **Azeroth Survivors 0.121.0**.
- An ASX release explicitly compatible with that game version.

ASX does not include the original game.

## Install and launch

1. Download the compatible archive from the [ASX releases page](https://github.com/tiboutDev/azeroth-survivors-coop/releases).
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
