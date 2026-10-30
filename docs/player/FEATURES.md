# ASX Features

[Home](../../README.md) · [Installation](INSTALLATION.md) · **Features** · [In the workshop](WORKSHOP.md)

---

ASX adds new decisions during a run and more ways to play together.

## Understanding feature status

| Status | What it means |
| --- | --- |
| **Established** | The feature’s design and behavior are more settled. Bugs and improvements are still possible. |
| **Playtesting** | Available, but more feedback from real runs is needed to assess behavior and interactions. |
| **Balance in progress** | Available, with values and progression still being tuned. |
| **Temporarily disabled** | An existing feature that is currently unavailable. |
| **Concept** | An idea being explored, with no available implementation. |

These labels describe feature maturity. They do not certify every class, build, interaction or client version. Check release notes for compatibility and known limitations.

## Build choices

These features are available in **solo and co-op**.

### 💍 Ring Enchantments

**Status: Balance in progress**

A ring’s first eligible upgrade offers an enchantment alongside its ordinary improvements.

- Each ring has its own enchantment.
- Accepting an enchanted upgrade locks that ring’s enchantment for the run.
- Later upgrades preserve its type.
- Bonuses add to your character’s stats.
- Enchantment strength increases at specific ring ranks.

| Enchantment tier | Required ring rank |
| --- | --- |
| Minor | 2 |
| Lesser | 4 |
| Standard — no prefix | 7 |
| Greater | 10 |
| Superior | 14 |

New enchantments and tier improvements display their values directly on reward cards. Unchanged enchantments appear as a compact icon and title with an **Active** label. Hover the header to inspect their values.

Some rings have low maximum ranks and may receive only a low-tier enchantment—or none. Reaching Superior is not expected in every run.

**Feedback wanted:** enchantment strength, the appeal of different bonuses, and ring ranks reached in normal and endless runs. Include the map and difficulty when sharing your results.

### 💎 Amulet Gems

**Status: Balance in progress**

Your first accepted amulet locks one of **eight gems**, each granting a single fixed stat bonus.

- Your gem stays with you until the run ends.
- Later amulet upgrades and replacements preserve it.
- Gem bonuses do not scale with amulet rank.
- All eight gems have equal chances, unaffected by Luck.

New gems display their bonus directly on reward cards. Retained gems use a compact **Active** presentation, with their value available on hover.

**Feedback wanted:** whether the bonuses make an early amulet worth taking, and whether some gems feel consistently weaker or stronger across classes.

### 🎪 Sayge’s Dark Fortunes

**Status: Balance in progress**

In co-op, completing a boss encounter grants each player an independent random fortune.

ASX instead offers **Sayge’s Trial** between bosses, never before the first. It appears throughout the playable zone with a minimap marker. Sayge appears within the rune, surrounded by candles; he is decorative and cannot be attacked. Hold its circle for 20 cumulative seconds to replace your Fortune. The trial stays at the same location until completed or the run ends. Leaving the circle suspends progress and hides the progress bar; returning resumes from the same progress. Later bosses improve its reward stage without adding more trials or resetting progress. Native pauses freeze presence accumulation. Power is rolled within a rising stage range, with a guaranteed normalized increase until its cap. Luck does not affect it. In co-op, everyone still in the run must be alive and inside for progress to advance; each player receives a personal Fortune. Disabling the optional trial restores boss Fortunes.

**Five categories:** Luck, Experience, Gold, Pickup Radius and Movement Speed.

- Only **one fortune** is active per player.
- A new fortune replaces the previous one.
- Fortunes survive death and remain until replaced or the run ends.
- Category chances are equal. Trial presentation follows its rolled power.
- Luck does not influence either roll.
- Fortune strength increases with completed boss encounters, capped at **five encounters**.
- Endless continues providing opportunities after the strength cap.
- Encounters involving several bosses grant one fortune when the encounter is completed.

Fortunes appear as buffs, with a reward notification when received. Their bonuses use whole percentage values.

In co-op, the **strongest active Experience fortune** and **strongest active Gold fortune** contribute to the group independently. They do not stack across players. Luck, Pickup Radius and Movement Speed remain personal.

The four utility trinkets associated with Luck, Experience, Gold and Pickup Radius are excluded from ASX reward pools.

**Feedback wanted:** the impact of each category and rarity, progression over the first five boss encounters, and the strength of shared Experience and Gold fortunes in co-op.

## Cooperative features

These features apply to **co-op**.

### 🤝 Shared resources and personal rewards

**Status: Established**

ASX adapts resource and reward handling for a shared adventure.

- **Shared in-run XP** helps participants progress together.
- **Shared gold** gives the party one common wallet.
- Purchased chests provide personal reward choices for eligible participants.
- Paid spell and item rerolls spend shared gold.
- Voluntary reward skips contribute their full gold award, without dividing it by player count.

Eligible racial bonuses and Experience/Gold fortunes use their strongest contribution rather than stacking duplicate bonuses across participants. Permanent armory gold bonuses likewise use the strongest participant contribution. Luck remains personal.

**Known issue:** XP may remain desynchronized after a cooperative death and resurrection. This issue remains open.

### ✨ Allied support and team synergy

**Status: Playtesting**

Supported healing, shielding, buff and talent effects extend to teammates, giving support builds a stronger role in the party.

Effects retain their original caster’s attribution. **Mage Ice Barrier remains caster-only.**

**Feedback wanted:** mixed-class parties, overlapping effects from multiple casters, healing and shielding behavior, and interactions with talents and equipment sets. Include the abilities and builds involved.

### 🛡️ Tanking and aggro

**Status: Playtesting**

ASX adds talent-based taunts:

| Class | Ability | Required talent |
| --- | --- | --- |
| Warrior | Shield Slam | Fortified |
| Paladin | Hammer of Justice | Ardent Defender |
| Druid | Shred | Dire Bear Form |

Successful eligible hits make affected enemies pursue the tank and direct ordinary attacks toward them. Special abilities retain their own targeting rules.

Another eligible taunt can take over. Death or departure clears the previous assignment; returning to the fight requires a new taunt.

Temporary invincibility clears existing ordinary aggro and taunt assignments. A fresh taunt can still draw enemies while the player is invincible.

**Feedback wanted:** enemies that ignore a successful taunt, competing tanks, and targeting during or after invincibility. Distinguish ordinary attacks from special boss mechanics where possible.

### 👁️ Party information and pauses

**Status: Established**

ASX enriches the party interface with information useful for understanding your teammates’ builds.

- Compact rows show equipped spells and items.
- Additional buff icons expose supported beneficial effects.
- Spell and item tooltips reflect their owner’s build.
- Group pauses identify who is choosing a reward or holding a menu open.
- Waiting messages display character names.
- Reward selection pauses the group until everyone has finished.
- Players who finish choosing remain paused until the group is ready.

### 💀 Resurrection Sickness

**Status: Established**

Each resurrection adds one personal stack of **Resurrection Sickness**.

| Rule | Effect |
| --- | --- |
| Penalty per stack | −5% damage and healing done |
| Maximum stacks | 3 |
| Maximum penalty | −15% |
| Recovery | One stack removed per 60 seconds spent alive |

Another resurrection restarts the recovery interval. Death, pauses and reward choices suspend recovery.

The debuff display shows stacks and recovery time. The penalty affects outgoing damage and healing. It does not reduce maximum health or healing received from another player.

## Share feedback

[Report an issue or share a balance observation](https://github.com/tiboutDev/azeroth-survivors-extended/issues). Include your game and ASX versions, map, difficulty, classes and player count, plus what you expected and what happened.

## Treasure Goblins

An occasional **Venture Co. Treasurer** flees across the battlefield.
Boss encounters can trigger an appearance; most do not. Further goblins become
rarer within the same run, including endless, with no guaranteed spawn.
This larger Goblin appears nearby, with a native event announcement, and has
the base health of the map’s first guardian. Defeat it within
25 seconds for a personal choice of three amulets and class marks based on an
ordinary boss of the current map and difficulty, including native endless scaling.
Marks are rounded down to whole units with no minimum: a result of zero awards
only the amulet choice.
Each participant receives their own reward in coop. It ignores taunts. Group
pauses stop its escape timer. No reward is lost from normal boss or guardian pools.

To disable only goblins, create `ASX/options.json` in the game folder containing
`{"treasure_goblins":false}`, then restart through the ASX launcher. Other ASX
features remain enabled. The host controls event generation for the party.

## Lost Supplies

Piles of cargo can appear throughout the playable zone and remain for 60 seconds.
Their minimap markers help locate them. Reach one to discover a personal reward
without a selection screen or pause; several reserves can be present at once.

Six consumable families cover cooldown recovery, critical chance, damage
reduction, maximum health, critical damage and area of effect. Different families
can coexist for 20 seconds. Another consumable of the same family refreshes its
duration without stacking, retaining a stronger active tier. Tiers improve after
two and four completed boss encounters, then stop increasing in endless.
Classic consumable names/icons accompany effects adapted to ASX.

Materials instead sell immediately for whole gold. Discoveries have several
rarities and reagent variants; rare finds sell for more. Their average gold
budget compensates for the higher appearance frequency. The notification shows
actual credited gold, including native bonuses. Native battleground runes retain
their own effects.

Set `"lost_supplies":false` in `ASX/options.json` to disable this event at the next
launch. In co-op, each participant has a separate reward fixed when the reserve appears. Your pickup hides only your own reserve and marker; other players can still collect theirs. Materials are sold into the shared gold pool.
