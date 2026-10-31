AZEROTH SURVIVORS EXTENDED (ASX)
For the exact supported Azeroth Survivors {GAME_VERSION} build, Windows 64-bit.

INSTALL: close the game. Extract this archive beside AzerothSurvivors.exe.
PLAY ASX: double-click start-asx.cmd. Launch starts automatically. Use the same ASX version for all players.
PLAY NATIVE: launch AzerothSurvivors.exe directly.

This archive is a mod overlay, not a standalone game. Download the compatible
official game separately before installing ASX.

ASX FEATURES
- Shared co-op progression and gold, allied support and party spell tooltips.
- Spell glyphs are disabled on this client pending review of native set interactions.
- Ring enchantments at ring ranks 2 / 4 / 7 / 10 / 14.
- Personal colored world crystals replace amulet gems; Crystal Resonance tracks run-long bonuses.
- Sayge's Trials grant Dark Fortunes: Luck, Experience or Gold.
- Resurrection Sickness: temporary damage/healing penalties after revival.
- Native enemy difficulty, density and Heroic levels.
- Native personal marks replace ASX in-run gold for duplicate boss drops.

ASXUI is included and installed automatically in benilla-config/AddOns/ASXUI
when ASX launches. Signed ASX updates also update its managed addon files.
ASXUI remains a separate Lua addon with native SavedVariables preferences;
updates and uninstall preserve these settings and unknown files. Existing
customized ASXUI code requires confirmation and a backup before replacement.
No separate ASXUI download is required.

Both launches use your existing native progression and settings. No import or
save synchronization is needed. Use one game at a time for the same profile.
A progression backup is kept in ASX/Backups before each ASX launch.

Signed ASX updates are checked when playing online. Confirmation is required before installing.
Use --menu for maintenance choices. Enter postpones an offered update.
No interactive console means no update installation. Launcher-runtime upgrades may require
extracting a complete archive while the game is closed. If the official game updates first,
ASX may need a compatible release. You can keep playing the native game.

Read-only offline diagnosis: start-asx.cmd --check
Repair the same release (signed package and confirmation required): start-asx.cmd --repair
Recover an interrupted operation: start-asx.cmd --recover
Retry a failed update with confirmation: start-asx.cmd --retry-update
Explicit launch: start-asx.cmd --play
To skip an update check: start-asx.cmd --offline
Restore the previous mod version: start-asx.cmd --rollback
Remove ASX: start-asx.cmd --uninstall
Create a local report: start-asx.cmd --export-diagnostics
Documentation warnings do not block play. Modified documentation is preserved
during repair and backed up before replacement during updates. Critical files
require repair. Python/runtime damage may require the complete trusted archive.
Logs: ASX/Logs. Reports may include player names and run details; review before
sharing. Nothing is uploaded automatically. Native saves/settings are preserved.

This is an unofficial modification, independent of the original game's author.
