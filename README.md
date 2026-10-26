# Azeroth Survivors Extended
### More choices. Deeper builds. Stronger teamwork.

**ASX expands Azeroth Survivors with new build choices, boss rewards and gameplay adjustments.** Shape your spells, discover new combinations and take on each run your way — alone or with friends.

> **Unofficial mod — original game required.**  
> ASX does not include Azeroth Survivors and is independent of its original author.

**[Download ASX](https://github.com/tiboutDev/azeroth-survivors-coop/releases)** · **[Report a bug](https://github.com/tiboutDev/azeroth-survivors-coop/issues)**

> 🚧 **Development preview:** some features described below are still being integrated. Check the release notes for availability and class coverage.

## Make each run your own

🌀 **Spell Glyphs**  
Reshape your spells with powerful benefits and meaningful drawbacks, directly on upgrade cards.

🎪 **Darkmoon Blessings**  
Earn random blessings from boss encounters, with different rarities and bonuses that evolve as you progress.

💀 **Resurrection Sickness**  
Repeated resurrections temporarily reduce your damage and healing. Stay alive to recover.

## Go further together

🤝 **Shared progression & rewards**  
Shared XP and gold, personal choices from group chests, and adapted elite rewards.

✨ **Support your party**  
Help teammates with supported heals, shields, buffs and talent effects. Talent-based taunts give eligible builds a tanking role.

👁️ **Keep your party in view**  
See teammates’ spells, equipment and buffs with build-specific tooltips, plus clearer group pauses and waiting messages.

**Native encounters and host difficulty options remain intact.**

## Start playing

**Windows 64-bit · Compatible game client: 0.117.0**

1. Close the game.
2. Extract the compatible ASX archive into your game folder, beside `AzerothSurvivors.exe`.
3. Run **`start-asx.cmd`**.

For multiplayer, use the official **Co-op** menu. All players should use the same compatible ASX version.

Your existing native progression and settings are reused, with a progression backup created before each ASX launch.

**Want to play without ASX?** Launch `AzerothSurvivors.exe` directly.

## Explore ASX

<details>
<summary><strong>🌀 Spell Glyphs — customize how your spells work</strong></summary>

Starting at **rank 2**, spell upgrade cards can offer a Glyph alongside their normal improvements.

- Each supported spell has **two Glyph variants**, combining a benefit with a drawback.
- Trade damage for additional targets, coverage for intensity, or availability for stronger effects.
- A new Glyph replaces that spell’s previous Glyph.
- Ordinary upgrades preserve the equipped Glyph.
- **Unglyphed** offers upgrade the spell while removing its Glyph’s benefit and drawback.
- Removing a Glyph preserves your spell ranks, native upgrades, talents and equipment.
- Active Glyphs appear in spell tooltips and remain specific to the current run.

**Current development coverage:** all eleven abilities for each of Warrior, Paladin and Hunter. More classes are being integrated.

</details>

<details>
<summary><strong>🎪 Darkmoon Blessings — new rewards from boss encounters</strong></summary>

Completing a boss encounter grants each player an independent random Blessing.

**Four categories:** Luck, Experience, Gold and Pickup Radius.

- One Blessing remains active until another replaces it or the run ends.
- Blessings survive player death and have no countdown.
- Category chances are equal, and rarity chances are fixed. Player Luck does not influence either roll.
- Blessing strength progresses with completed boss encounters.
- Encounters involving several boss entities award one Blessing when the encounter is completed.
- Endless encounters continue awarding replacements, while progression stops scaling beyond the map’s final normal boss.

In co-op, the strongest active **Experience** Blessing and strongest **Gold** Blessing contribute to the group independently. They do not stack across players. Luck and Pickup Radius remain personal.

The four corresponding utility trinkets are removed from cooperative reward pools.

</details>

<details>
<summary><strong>🤝 Cooperative progression, resources and rewards</strong></summary>

ASX adjusts resource and reward handling to support a shared adventure while preserving individual build choices.

- **Shared experience** keeps participants progressing together.
- **Shared gold** gives the party a common resource pool, including purchases and other gold costs.
- Eligible racial XP and gold bonuses use the strongest contribution rather than accumulating duplicate racial bonuses.
- Luck remains personal.
- Group chests provide personal reward choices.
- Elite rewards are adapted for the group.
- Reward selection pauses the action while participants make their choices.

</details>

<details>
<summary><strong>✨ Allied support, tanking and team synergy</strong></summary>

Supported healing, shielding, buff and talent effects extend to teammates, giving support-oriented builds a role in the party.

ASX also adds talent-based taunts:

- **Warrior:** Shield Slam with **Fortified**.
- **Paladin:** Hammer of Justice with **Ardent Defender**.
- **Druid:** Shred with **Dire Bear Form**.

Successful eligible hits make affected enemies pursue the tank and direct ordinary attacks toward them. Special abilities retain their native targeting.

Another eligible taunt can take over. Death or departure clears the previous assignment; returning to the fight requires a new taunt.

</details>

<details>
<summary><strong>👁️ Party information and clearer pauses</strong></summary>

ASX enriches the existing party interface with information useful for understanding your teammates’ builds.

- Compact rows show equipped spells and items.
- Additional buff icons expose supported beneficial effects.
- Spell and item tooltips reflect their owner’s build.
- Glyph information appears in supported spell tooltips.
- Group pauses identify who is choosing a reward or holding a menu open.
- Waiting messages display character names.

Native portraits, health bars and resurrection interactions remain familiar.

</details>

<details>
<summary><strong>💀 Resurrection Sickness — recover after returning to battle</strong></summary>

Each resurrection adds one personal stack of **Resurrection Sickness**.

- Each stack reduces damage and healing done by **5%**.
- The penalty caps at **three stacks**, or **15%**.
- Every **60 seconds spent alive** removes one stack.
- Another resurrection restarts the recovery interval.
- Death, pauses and reward choices suspend recovery.
- The native debuff display shows stacks and recovery time.

The penalty affects outgoing damage and healing. It does not reduce maximum health or another player’s healing received.

</details>

<details>
<summary><strong>⚙️ Native gameplay, updates and removal</strong></summary>

ASX builds on the original game’s encounters and systems.

- Enemy and boss health, enemy damage and cooperative density retain native scaling.
- Host difficulty options remain available.
- Enemy selection, placement and encounter schedules remain native.
- Existing native progression and settings are reused.
- Progression is backed up before each ASX launch.
- The launcher checks for signed mod updates.

Useful commands:

```bat
start-asx.cmd --offline
start-asx.cmd --rollback
start-asx.cmd --uninstall
start-asx.cmd --export-diagnostics
```

- **`--offline`** skips the update check.
- **`--rollback`** restores the previous mod version after a supported automatic update.
- **`--uninstall`** removes owned, unchanged ASX files while preserving native progression and settings.
- **`--export-diagnostics`** creates a local diagnostic archive.

Reports are never uploaded automatically. Review them before sharing, as logs may contain player names and run details.

</details>

## What’s next?

ASX continues to evolve. Here’s what we’re working toward:

🛠️ **Being designed — Rogue Glyphs**  
Bringing spell customization to the Rogue’s attacks, bleeds, controls and buffs.

🗺️ **Planned — Glyphs for every class**  
Expanding the system across the remaining classes, with choices tailored to each spell’s identity.

⚖️ **Ongoing — Balance and refinement**  
Fine-tuning Glyph trade-offs, Blessing progression and cooperative interactions through testing and player feedback.

*Plans may evolve as ideas are tested. Release notes confirm what’s available.*

## Help improve ASX

Found a bug or an interesting build interaction? Open an [issue](https://github.com/tiboutDev/azeroth-survivors-coop/issues) with:

- Your game and ASX versions.
- Whether you were playing solo or co-op, including the player count.
- The affected classes and what happened.
- A screenshot or diagnostic export when useful.
