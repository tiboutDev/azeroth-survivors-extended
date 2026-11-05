# Azeroth Survivors Extended

### More choices. Deeper builds. Stronger teamwork.

**Home** · [Installation](docs/player/INSTALLATION.md) · [Features](docs/player/FEATURES.md) · [In the workshop](docs/player/WORKSHOP.md)

---

**AZSX builds on Azeroth Survivors with new build choices and deeper cooperative play.** Discover ring enchantments, world crystals and Sayge’s Dark Fortunes, alongside new ways to support your party.

> **Unofficial mod — original game required.**\
> AZSX does not include Azeroth Survivors and is independent of its original author.

**[Download AZSX](https://github.com/tiboutDev/azeroth-survivors-extended/releases)** · **[Report a bug](https://github.com/tiboutDev/azeroth-survivors-extended/issues)**

## Make each run your own

Available in **solo and co-op**:

💍 **Ring Enchantments** — *Balance in progress*\
Choose an enchantment when upgrading a ring, then strengthen it as that ring gains ranks.

💎 **Crystal Resonance** — *Balance in progress*\
Find colored crystals while exploring, without minimap markers. Each player collects their own bonuses, shown in a pickup notice and one run-long buff.

🎪 **Sayge’s Dark Fortunes** — *Balance in progress*\
Complete Sayge’s Trial between bosses to earn a personal Luck, Experience or Gold fortune. Keep one active at a time; a new fortune replaces it.

📦 **Explore for rewards** — *Balance in progress*\
Find Lost Supplies for temporary buffs or gold, and chase Venture Co. Treasure Goblins for personal rewards. First-goblin bad-luck protection applies after the first boss.

The [Mechanical Chicken escort](docs/features/SOLO-ESCORT.md) is available in solo and co-op. The unfinished [Banished Demon encounter](docs/features/BANISHED-DEMON.md) is disabled.

[Explore the build features →](docs/player/FEATURES.md#build-choices)

## Go further together

Cooperative features:

🤝 **Shared resources, personal choices** — *Established*\
Share in-run XP and gold while choosing your own build rewards.

✨ **Support and tanking** — *Playtesting*\
Help teammates with supported heals, shields and buffs. Eligible builds can draw enemies away from their allies.

👁️ **A clearer view of your party** — *Established*\
Inspect teammates’ spells, equipment and buffs, with clearer group pauses and waiting messages.

💀 **Resurrection Sickness** — *Established*\
Repeated resurrections temporarily reduce your damage and healing. Stay alive to recover.

[Explore the cooperative features →](docs/player/FEATURES.md#cooperative-features)

## Feature maturity

AZSX is actively evolving. **Balance in progress** means bonus values and progression are still being tuned. **Playtesting** means a feature needs more feedback from real runs. **Established** features have a more settled design and behavior, though issues can still occur.

These pages describe the current source; published releases may contain an earlier feature set. Check the notes of the release you download.

These labels describe feature maturity, not exhaustive validation of every class, interaction or game update. See the [feature guide](docs/player/FEATURES.md) for the feedback we are looking for and each release’s notes for compatibility information.

## Start playing

**Windows 64-bit · Use the game version specified by your AZSX release.**

1. Close the game.
2. Extract the compatible AZSX archive into your game folder, beside `AzerothSurvivors.exe`.
3. Run **`start-azsx.cmd`**. It installs the included AZSXUI addon and starts AZSX automatically.

AZSXUI needs no separate download. Signed AZSX updates compatible with your installed game install automatically, update the managed addon files and preserve saved preferences.

For multiplayer, use the official **Co-op** menu. All participants should use the same compatible game and AZSX versions.

**[Installation, updates and removal →](docs/player/INSTALLATION.md)**

## In the workshop

Experimental ideas are kept separate from the features available in current releases.

[See what we’re exploring →](docs/player/WORKSHOP.md)

## Co-op progression

In current source, dead co-op participants continue gaining shared XP and native whole-level rewards immediately. Level-up choices remain pending until revival. Prepared archives validate only the source recorded in their release notes.

## Help improve AZSX

Found a bug or an interesting build interaction? Open an [issue](https://github.com/tiboutDev/azeroth-survivors-extended/issues) with:

- Your game and AZSX versions.
- Whether you were playing solo or co-op, including the player count.
- The affected classes and what happened.
- A screenshot or diagnostic export when useful.

## Included AZSXUI addon

[AZSXUI](addons/AZSXUI/README.md) independently styles native enemy nameplates,
preserves the native damage font, and renders custom/native-cache
minimap pins using the AZSX projection bridge. Supplies, Fortune and crystal
notifications, plus Sayge progress, are configurable AZSXUI widgets with text
still supplied by AZSX.
AZSX installs it automatically in `benilla-config/AddOns/AZSXUI` and keeps its
managed files updated. Native SavedVariables preferences are preserved.
Use `/azsxui widget crystals show off` to hide crystal notices without changing
crystal rewards. Position, scale and opacity can be adjusted for each widget.

Nameplate styling requires neither the AZSX gameplay DLL nor its launcher;
minimap pins require the matching AZSX projection bridge. See its README for
commands and compatibility limits.

The [AZSX addon contract](docs/features/ADDON-API.md) exposes versioned structured values
and AZSX-authored text using native-style Lua globals/getters/handlers.

## Development

Start with the [daily development workflow](docs/development/DEVELOPMENT.md).
Use the [documentation index](docs/README.md) for detailed references, the
[compatibility checklist](docs/compatibility/COMPATIBILITY-CHECKLIST.md) for a native
client update, and the [release checklist](docs/release/RELEASE-CHECKLIST.md) for release work.

Source currently targets official **0.140.0**. Normal-artifact and in-game
validation of this port remain separate from source checks; published kits
declare their own supported game version. See
[client compatibility](docs/compatibility/CLIENT-COMPATIBILITY.md).

For source organization and engineering practices, see
[Development](docs/development/DEVELOPMENT.md). The maintained
[behavior and compatibility checklist](docs/compatibility/AZSX-BEHAVIOR.md) defines expected
outcomes for every AZSX feature, including disabled and native-owned behavior.

## License

Copyright (c) 2026 AZSX contributors.

AZSX is free software: you can redistribute it and/or modify it under the terms
of the [GNU General Public License, version 3](LICENSE).
AZSX is distributed without any warranty; see the license for details.
The original game, its assets and third-party dependencies retain their own licenses.

## Native balance data

The [native balance reference](docs/native-balance/NATIVE-BALANCE.md) records common native statistics and jewelry with rank/rarity formulas, an offline calculator, reproducible extraction and source provenance.

The native-only [enemy reference](docs/native-balance/ENEMIES.md) covers map definitions, guardians, bosses and spawn scaling; [economy](docs/native-balance/ECONOMY.md) covers kill rewards, gold credits, cache prices and rerolls. Recorded versions indicate provenance; refresh values only on explicit request.

Existing ASX users: follow the [AZSX migration guide](docs/player/AZSX-MIGRATION.md).
Hosted lobbies are automatically identified with `[AZSX]`.
