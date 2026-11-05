AZEROTH SURVIVORS EXTENDED (AZSX)
For the exact supported Azeroth Survivors {GAME_VERSION} build, Windows 64-bit.

INSTALL: close the game. Extract this archive beside AzerothSurvivors.exe.
PLAY AZSX: double-click start-azsx.cmd. Launch starts automatically. Use the same AZSX version for all players.
PLAY NATIVE: launch AzerothSurvivors.exe directly.

This archive is a mod overlay, not a standalone game. Download the compatible
official game separately before installing AZSX.

AZSX FEATURES
- Shared co-op progression and gold, allied support and party spell tooltips.
- Ring enchantments at ring ranks 2 / 4 / 7 / 10 / 14.
- Personal colored world crystals replace amulet gems; Crystal Resonance tracks run-long bonuses.
- Sayge's Trials grant Dark Fortunes: Luck, Experience or Gold.
- Solo/co-op Mechanical Chicken escort, after the second boss: guide it to extraction for a personal engineering gadget.
- Resurrection Sickness: temporary damage/healing penalties after revival.
- Native enemy difficulty, density and Heroic levels.
- Native personal marks replace AZSX in-run gold for duplicate boss drops.

AZSXUI is included and installed automatically in benilla-config/AddOns/AZSXUI
when AZSX launches. Signed AZSX updates also update its managed addon files.
AZSXUI remains a separate Lua addon with native SavedVariables preferences;
updates and uninstall preserve these settings and unknown files. Existing
independently installed addon code is preserved; use explicit repair to replace it.
No separate AZSXUI download is required.

Both launches use your existing native progression and settings. No import or
save synchronization is needed. Use one game at a time for the same profile.
The five latest distinct progression backups and five session logs are retained
automatically. Obsolete cached packages, completed transactions and exited runtime
workers are cleaned up; preferences and user-file backups are preserved.

Signed AZSX updates compatible with the installed native game install automatically.
Use --menu for maintenance choices, or --offline to skip update discovery.
No update confirmation is needed, including without an interactive console. Runtime
updates restart the launcher automatically through a private copied interpreter.
When a newer supported game is needed, AZSX verifies and caches its matching
package first, lets the official updater update AZS with the mod inactive, then
installs AZSX and continues automatically. --update-only uses the same sequence. A damaged interpreter needs complete-archive repair.

Read-only offline diagnosis: start-azsx.cmd --check
Repair the same release (signed package and confirmation required): start-azsx.cmd --repair
Interrupted operations recover automatically; --recover is also available.
Failed updates retry once at the next launch; --retry-update remains available.
Explicit launch: start-azsx.cmd --play
To skip an update check: start-azsx.cmd --offline
Restore the previous mod version: start-azsx.cmd --rollback
Remove AZSX: start-azsx.cmd --uninstall
Create a local report: start-azsx.cmd --export-diagnostics
Documentation warnings do not block play. Modified documentation is preserved
during repair and backed up before replacement during updates. Critical files
are repaired automatically from an available signed same-version package.
Python/runtime damage that prevents startup requires the complete trusted archive.
Logs: AZSX/Data/Logs. Reports may include player names and run details; review before
sharing. Nothing is uploaded automatically. Native saves/settings are preserved.

This is an unofficial modification, independent of the original game's author.

AZSX/App contains program files; AZSX/Config contains options; AZSX/Data contains
state, cache, backups, logs and reports. The new launcher migrates the legacy
layout after the first complete-archive installation. When migrating from the old pre-App/Config/Data layout,
close all clients, keep the old mod folder, and extract the complete new archive
beside AzerothSurvivors.exe, replacing start-azsx.cmd. Do not extract inside AZSX.
The older launcher cannot perform this layout transition automatically.
AZSXUI problems and missing documentation do not prevent native-interface play. Compatible signed updates
install automatically without confirmation; offline mode skips discovery.
