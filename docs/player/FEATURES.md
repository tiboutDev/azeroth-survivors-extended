# ASX Features

[Home](../../README.md) · [Installation](INSTALLATION.md) · **Features** · [In the workshop](WORKSHOP.md)

---

ASX adds new decisions during a run and more ways to play together. These pages
describe the current source; check your downloaded release’s notes for its
available features and exact game compatibility.

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

### 💎 Crystal Resonance

**Status: Prototype — balance in progress**

Explore for colored crystals scattered independently around the map. There are
**no crystal minimap markers**: discovering them is part of the search. Colors
are freely drawn, so the same color can appear several times.

| Crystal | Permanent gain per pickup |
| --- | --- |
| Red | +10% damage |
| Blue | +5% cooldown recovery |
| Green | +15% maximum health |
| Yellow | +10% area of effect |

Each player present when a crystal appears can collect it once by passing close
while alive. Your pickup does not take another player's reward. It also leaves
the model visible, so you can still point it out to the group.

- Before the first pickup, the crystal has no disappearance countdown.
- The first pickup starts one shared **60-second** window; later pickups do not extend it.
- It disappears for everyone when all remaining eligible players have collected it, or the window expires.
- In solo, it disappears immediately when you collect it. Native pauses freeze the countdown.
- Players joining later cannot claim crystals that appeared before they joined.

ASXUI shows the crystal color and its bonus in a short pickup popup. Notices
queue for nearby pickups and stack below the top HUD. **Crystal Resonance**
keeps the accumulated bonuses in one buff: hover it to inspect your totals.
Bonuses add up, survive death and reset with the next run. The buff follows
Fortune at the start of the buff row, or comes first when Fortune is absent.

Amulets keep their native effects and **no longer assign an ASX gem**. The
prototype targets **4–6 opportunities before endless**, with at most two crystals
present. Discovery counts are not guaranteed: early run endings, occupied slots
or unavailable terrain can leave fewer. Endless continues independent spawns.

Configure the notice with `/asxui widget crystals …`; see
[ASXUI presentation](#asxui-presentation) for complete examples.

**Feedback wanted:** discovery visibility, whether a detour feels worthwhile,
and combined power when several crystals of the same color appear.

### 🎪 Sayge’s Dark Fortunes

**Status: Balance in progress**

By default, **Sayge’s Trial** replaces automatic boss Fortunes in solo and
co-op. After the first boss, look for Sayge's rune and minimap marker between
boss encounters. Sayge and his candles are decorative; he cannot be attacked.

Hold the circle for **20 cumulative seconds** to receive a personal Fortune:

- In co-op, every active participant must be alive and inside the circle.
- If anyone leaves or is downed, progress pauses without being lost. It resumes when everyone is alive and back inside.
- Your casting-style progress bar appears only while you are inside; it explains when the group is waiting.
- Native pauses freeze progress. An unfinished trial stays in place until completed or the run ends.
- Later bosses improve the reward stage without replacing an unfinished trial or resetting its progress.

Fortunes roll one of **three equally likely categories: Luck, Experience or Gold**.
Luck does not influence the category or power roll. Only one Fortune is active
per player: a new one replaces the previous one. It survives death until
replaced or the run ends.

Power improves with encounter stage and previous completed trials, up to a cap.
Encounter-stage growth stops after five encounters; endless still offers
replacements. Bonuses use whole percentages, shown as a green **+X%** in the
notification and buff tooltip, without a visible rarity label. Different
categories have different power scales, so their percentages are not directly
comparable.

In co-op, the **strongest active Experience Fortune** and **strongest active
Gold Fortune** contribute to the party independently, without stacking copies
across players. Luck remains personal. The tooltip distinguishes your own
Fortune from a stronger effective party bonus.

Disabling Sayge's Trial restores boss-based Fortunes: one personal roll per
completed boss encounter, including encounters with several bosses. See
[optional events](#optional-events).

Fortunes no longer grant movement speed or pickup radius. The three economy
trinkets associated with Luck, Experience and Gold remain excluded; **Gnomish
Universal Remote**, which grants pickup range, is available again.

**Feedback wanted:** category strength, trial placement, collective presence,
progression over the first five encounters, and shared Experience/Gold bonuses.

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

## World events

### Treasure Goblins

An occasional **Venture Co. Treasurer** appears nearby and flees across the
battlefield. Its first appearance cannot happen before the first boss.
**Bad-luck protection** spreads the first opportunity across the remaining
pre-final boss encounters: it can arrive early or late, rather than always
after the first boss. A complete normal run with the event enabled selects a
first opportunity before the final boss. Short runs or unavailable native
placement can prevent an appearance. Further goblins become rarer, including
in endless; there are no guaranteed additional spawns.

Defeat the goblin within **25 seconds** for a personal native choice of amulets
and class marks based on an ordinary boss of the current map and difficulty,
including native endless scaling. If the eligible pool allows it, the choice
contains three distinct amulets. Whole-mark rounding can yield zero marks; the
amulet choice still applies. Each participant receives personal rewards.

The goblin ignores taunts and cannot attack. Its health uses the first native
guardian's base, with native time, difficulty and party scaling. Group pauses
freeze its escape timer. A native announcement signals its arrival, and it
receives the **skull marker when that icon is free**; manual marks are preserved.
Automatic player markers use the other seven icons. Goblin rewards do not remove
ordinary boss or guardian rewards.

### Lost Supplies

Cargo piles appear independently across the playable zone, with minimap markers,
and remain for **90 seconds**. Reach one for a personal reward, without a choice
screen or group pause. Several reserves can coexist.

In co-op, each participant present at appearance has their own fixed reward.
Claiming a pile hides it and its marker only for you; others can still collect
theirs. Late arrivals cannot claim an existing reserve. The pile ends early
once everyone eligible has collected it; otherwise its shared lifetime continues.

Seven consumable families grant temporary bonuses:

- Cooldown recovery.
- Critical strike chance.
- Damage reduction.
- Maximum health.
- Critical strike damage.
- Area of effect.
- Movement speed: **Swiftness Potion gives +10% / +15% / +20%**, depending on tier.

Buffs last **60 seconds of game time**. Different families coexist. Another
consumable of the same family refreshes the duration without stacking power or
adding durations; it does not downgrade a stronger active tier. Power tiers
improve after two and four completed boss encounters, with no further tier
growth in endless. Native pauses freeze reserve and buff timers.

Materials sell immediately for whole gold, into the shared wallet in co-op.
The pickup popup shows the amount actually credited, including applicable
bonuses. There is no separate crafting inventory.

Item names and icons refer to Classic consumables and materials, but their
effects are adapted to ASX. **Name colors are cosmetic item quality, not buff
strength or sale value**. Native battleground runes retain their own effects.

## ASXUI presentation

ASXUI is included in ASX and installed automatically. It handles nameplate
styling, event/cache minimap markers, Supplies/Fortune/crystal notices and the
Sayge progress bar. Caches and Supplies use different native icons; crystals
have no minimap markers. Fortune leads the native buff row, followed by
Crystal Resonance.

Event widgets share a centered anchor under the top HUD and stack without
overlap by default. Position, scale, opacity and visibility can be adjusted
without restarting:

```text
/asxui widget anchor position 0 -85
/asxui widget crystals scale 1.2
/asxui widget crystals opacity 0.8
/asxui widget crystals show off
/asxui widget crystals show on
/asxui widget crystals reset
```

Widget names are `crystals`, `supplies`, `fortune` and `sayge`. Hiding a widget
only changes its display; it never disables rewards or changes trial progress.
The native damage font is kept by default. The optional smaller number face
requires a client restart. See the [ASXUI guide](../../addons/ASXUI/README.md)
for all commands and nameplate limits.

## Optional events

The host controls event generation in co-op. To change event settings, close
the game and edit `ASX/options.json` in the game folder. These events are
enabled by default:

```json
{
  "treasure_goblins": true,
  "lost_supplies": true,
  "darkmoon_trials": true
}
```

Change the desired value to `false`, preserve any other existing settings, and
restart through the ASX launcher. Turning off `darkmoon_trials` restores
boss-based Fortunes. These are gameplay settings; ASXUI's widget commands only
control presentation.

## Share feedback

[Report an issue or share a balance observation](https://github.com/tiboutDev/azeroth-survivors-extended/issues).
Include your game and ASX versions, map, difficulty, classes and player count,
plus what you expected and what happened.
