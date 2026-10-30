# Azeroth Survivors Extended
### More choices. Deeper builds. Stronger teamwork.

**Home** · [Installation](docs/player/INSTALLATION.md) · [Features](docs/player/FEATURES.md) · [In the workshop](docs/player/WORKSHOP.md)

---

**ASX builds on Azeroth Survivors with new build choices and deeper cooperative play.** Discover ring enchantments, world crystals and Sayge’s Dark Fortunes, alongside new ways to support your party.

> **Unofficial mod — original game required.**\
> ASX does not include Azeroth Survivors and is independent of its original author.

**[Download ASX](https://github.com/tiboutDev/azeroth-survivors-extended/releases)** · **[Report a bug](https://github.com/tiboutDev/azeroth-survivors-extended/issues)**

## Make each run your own

Available in **solo and co-op**:

💍 **Ring Enchantments** — *Balance in progress*\
Choose an enchantment when upgrading a ring, then strengthen it as that ring gains ranks.

💎 **Crystal Resonance** — *Balance in progress*\
Find colored crystals while exploring. Collect personal bonuses to damage, cooldown recovery, maximum health and area of effect, tracked in one run-long buff.

🎪 **Sayge’s Dark Fortunes** — *Balance in progress*\
Receive a random fortune after each completed boss encounter. Keep one active at a time, with stronger bonuses as you progress.

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

ASX is actively evolving. **Balance in progress** means bonus values and progression are still being tuned. **Playtesting** means a feature needs more feedback from real runs. **Established** features have a more settled design and behavior, though issues can still occur.

These labels describe feature maturity, not exhaustive validation of every class, interaction or game update. See the [feature guide](docs/player/FEATURES.md) for the feedback we are looking for and each release’s notes for compatibility information.

## Start playing

**Windows 64-bit · Use the game version specified by your ASX release.**

1. Close the game.
2. Extract the compatible ASX archive into your game folder, beside `AzerothSurvivors.exe`.
3. Run **`start-asx.cmd`**.

For multiplayer, use the official **Co-op** menu. All participants should use the same compatible game and ASX versions.

**[Installation, updates and removal →](docs/player/INSTALLATION.md)**

## In the workshop

**Spell Glyphs are temporarily disabled for 0.130.1** while we revisit their interaction with the new equipment set bonuses.

[See what we’re exploring →](docs/player/WORKSHOP.md)

## Known issue

In co-op, XP may remain desynchronized after a player dies and is resurrected. This issue remains open.

## Help improve ASX

Found a bug or an interesting build interaction? Open an [issue](https://github.com/tiboutDev/azeroth-survivors-extended/issues) with:

- Your game and ASX versions.
- Whether you were playing solo or co-op, including the player count.
- The affected classes and what happened.
- A screenshot or diagnostic export when useful.

## Optional ASXUI addon

[ASXUI](addons/ASXUI/README.md) independently styles native enemy nameplates,
preserves the native damage font by default, and renders custom/native-cache
minimap pins using the ASX projection bridge. Supply/Fortune notifications and
Sayge progress are configurable ASXUI widgets, with text still supplied by ASX.
Install its source folder in
`benilla-config/AddOns/ASXUI`.
Nameplate styling requires neither the ASX gameplay DLL nor its launcher;
minimap pins require the matching ASX projection bridge. See its README for
commands, font licensing and compatibility limits.

The [ASX addon contract](docs/ADDON-API.md) exposes versioned structured values
and ASX-authored text using native-style Lua globals/getters/handlers.

## Development

Source currently targets official **0.130.1**. Normal-artifact and in-game
validation of this port remain separate from source checks; published kits
declare their own supported game version. See
[client compatibility](docs/CLIENT-COMPATIBILITY.md).

For source organization and engineering practices, see
[Development](docs/DEVELOPMENT.md).

## License

Copyright (c) 2026 ASX contributors.

ASX is free software: you can redistribute it and/or modify it under the terms
of the [GNU General Public License, version 3](LICENSE).
ASX is distributed without any warranty; see the license for details.
The original game, its assets and third-party dependencies retain their own licenses.

Developer workflow: [compatibility and diagnostic tooling](docs/COMPATIBILITY-WORKFLOW.md).
