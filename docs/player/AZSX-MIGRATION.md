# Moving from ASX to AZSX

Close the game and both launchers. Extract the complete AZSX archive beside
`AzerothSurvivors.exe`, then run `start-azsx.cmd`. Do not launch the old
`start-asx.cmd` again. The first transition requires complete extraction: the
previous launcher does not understand the renamed payload paths.

The launcher preserves preferences, state and update sequence information from
`ASX/` under `AZSX/`. It keeps verified originals under
`AZSX/Data/Backups/Branding/`. The extracted new application takes precedence;
old application binaries never overwrite it. Native saves and assets stay put.
Conflicting preferences and unfinished old updates stop migration with a clear
message so neither copy is silently lost.

AZSXUI replaces ASXUI. Existing SavedVariables are copied to its new name only
when no AZSXUI settings already exist. The original settings remain available. The native shared SavedVariables file
also remains intact; AZSXUI recognizes the old `ASXUIDB` variable on load.
Owned old addon files are retired by hash; modified code and unknown files are
preserved. A modified old addon TOC must be disabled explicitly before continuing.
The mod API now initializes with the native API and needs no internal addon.

New hosted lobbies use `[AZSX] Your name`. The native 32-character name limit
includes the prefix; long names are shortened without splitting Unicode text.
Joining another lobby does not rename it. Native public/private choices remain
unchanged. Compatible signed AZSX updates remain automatic after migration.
