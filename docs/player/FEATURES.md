# AZSX Features

[Home](../../README.md) · [Installation](INSTALLATION.md) · **Features** · [In the workshop](WORKSHOP.md)

---

AZSX adds new decisions during a run and more ways to play together. These pages
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

Holy Nova and Holy Shock keep the native game's effects; AZSX adds no cooperative
healing to either spell. Resurrection Sickness still applies normally.

## Build choices

These features are available in **solo and co-op**.

### 💍 Ring Enchantments

**Status: Balance in progress**

A ring’s first eligible upgrade offers an enchantment alongside its ordinary improvements.

Alacrity grants **3 / 5 / 8 / 11 / 15% cooldown recovery** and Renewal grants
**0.5 / 1 / 1.5 / 2 / 3 health per second** at ring ranks **2 / 4 / 7 / 10 / 14**.
Each new tier replaces the previous total; the five values are not added together.
Recovery feeds the native cooldown formula rather than directly shortening delays
by the listed percentage.

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
| Red | +12% damage |
| Blue | +7% cooldown recovery |
| Green | +15% maximum health |
| Yellow | +8% Luck |

Each player present when a crystal appears can collect it once by passing close
while alive. Your pickup does not take another player's reward. It also leaves
the model visible, so you can still point it out to the group.

- Before the first pickup, the crystal has no disappearance countdown.
- The first pickup starts one shared **60-second** window; later pickups do not extend it.
- It disappears for everyone when all remaining eligible players have collected it, or the window expires.
- In solo, it disappears immediately when you collect it. Native pauses freeze the countdown.
- Players joining later cannot claim crystals that appeared before they joined.

AZSXUI shows the crystal color and its bonus in a short pickup popup. Notices
queue for nearby pickups and stack below the top HUD. **Crystal Resonance**
appears as one buff from the start of the run, even before your first pickup: hover it for a discovery hint, then color counts and accumulated bonuses.
Bonuses add up, survive death and reset with the next run.

Amulets keep their native effects and **no longer assign an AZSX gem**. The
spawn rate targets about **five opportunities by minute 15**, without a maximum
number per run. The first crystal can appear from 2:00, with at least 90 seconds between opportunities. The random timing can produce fewer or more than six. At most two
crystals are present together; early endings or unavailable terrain can reduce
the actual number. The same rate continues in endless.

Configure the notice with `/azsxui widget crystals …`; see
[AZSXUI presentation](#azsxui-presentation) for complete examples.

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

Fortunes roll **Luck, Experience or Gold**. Solo gives each category an equal chance.
In co-op, the personal category weights are **Luck = number of participants,
Experience = 1, Gold = 1**, using the eligible group at completion:

| Participants | Luck | Experience | Gold |
| --- | --- | --- | --- |
| 1 | 33.33% | 33.33% | 33.33% |
| 2 | 50% | 25% | 25% |
| 3 | 60% | 20% | 20% |
| 4 | 66.67% | 16.67% | 16.67% |
| 5 | 71.43% | 14.29% | 14.29% |

This applies to both trials and automatic boss Fortunes when trials are disabled.
Membership changes alone do not reroll your existing Fortune.
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

AZSX automatically discovers Insignia of the Horde, Royal Seal of Eldre'Thalas and Mark of the Chosen in the Codex at startup. Missing entries start at Common. Existing discoveries keep their best rarity.

Fortunes no longer grant movement speed or pickup radius. The three economy
trinkets associated with Luck, Experience and Gold remain excluded; **Gnomish
Universal Remote**, which grants pickup range, is available again.

**Feedback wanted:** category strength, trial placement, collective presence,
progression over the first five encounters, and shared Experience/Gold bonuses.

## Cooperative features

These features apply to **co-op**.

### 🤝 Shared resources and personal rewards

**Status: Established**

AZSX adapts resource and reward handling for a shared adventure.

- **Shared in-run XP** helps participants progress together.
- **Shared gold** gives the party one common wallet.
- Purchased chests provide personal reward choices for eligible participants.
- Paid spell and item rerolls spend shared gold.
- Voluntary reward skips contribute gold divided by the current party size, rounded to the nearest whole gold (halves up). For example, a 7-gold skip contributes 4 gold with two participants.

Eligible racial bonuses and Experience/Gold fortunes use their strongest contribution rather than stacking duplicate bonuses across participants. Permanent armory gold bonuses likewise use the strongest participant contribution. Luck remains personal.

Dead participants continue gaining shared XP and levels. Their level-up choices remain pending until they revive; resurrection does not award the same XP a second time.

### ✨ Allied support and team synergy

**Status: Playtesting**

Supported healing, shielding, buff and talent effects extend to teammates, giving support builds a stronger role in the party.

Effects retain their original caster’s attribution. **Mage Ice Barrier remains caster-only.**

**Feedback wanted:** mixed-class parties, overlapping effects from multiple casters, healing and shielding behavior, and interactions with talents and equipment sets. Include the abilities and builds involved.

### 🛡️ Tanking and aggro

**Status: Playtesting**

AZSX adds talent-based taunts:

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

AZSX enriches the party interface with information useful for understanding your teammates’ builds.

- Compact rows show equipped spells and items.
- Additional buff icons expose supported beneficial effects.
- Spell and item tooltips reflect their owner’s build.
- AZSXUI lists each item effect on its own tooltip line and shows native-template debuff icons and borders beneath boss, guardian and Treasure Goblin health bars. Hover an icon for its name, stacks and remaining time.
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
guardian's base multiplied by 1.5, with native time, difficulty and party scaling. Group pauses
freeze its escape timer. A native announcement and encounter health bar signal
its arrival. Goblins receive no automatic skull; manual markers remain available.
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

Lost Supplies can grant a personal healing potion or a timed bonus. Item names come from WoW Classic. Strength and duration are tuned for an AZSX run.

| Family | 0–1 completed bosses | 2–3 completed bosses | 4+ completed bosses |
| --- | --- | --- | --- |
| Cooldown recovery | Elixir of Wisdom, 8% · 60s | Elixir of Greater Intellect, 12% · 75s | Flask of Distilled Wisdom, 16% · 120s |
| Critical strike chance | Elixir of Lesser Agility, 4 points · 60s | Elixir of Greater Agility, 7 points · 75s | Elixir of the Mongoose, 10 points · 90s |
| Damage reduction | Elixir of Defense, 3 points · 60s | Elixir of Greater Defense, 5 points · 75s | Elixir of Superior Defense, 7 points · 90s |
| Maximum health | Elixir of Fortitude, 20% · 60s | Elixir of Brute Force, 30% · 75s | Flask of the Titans, 40% · 120s |
| Critical strike damage | Minor Wizard Oil, 20% · 60s | Wizard Oil, 35% · 75s | Brilliant Wizard Oil, 50% · 90s |
| Area of effect | Elixir of Giant Growth, 25% · 60s | Same item | Same item |
| Movement speed | Swiftness Potion, 20% · 15s | Same item | Same item |
| Emergency healing | Healing Potion, 20% max HP | Superior Healing Potion, 30% max HP | Major Healing Potion, 40% max HP |

At 35% health or less, your armed healing potion is consumed automatically. You can hold one charge and keep the stronger potion on a duplicate pickup. The charge lasts until used, death or the end of the run. Timed buffs from the same family refresh the stronger effect. Native pauses freeze timers.

Each personal reward also has an independent 8% chance to include a 60-second disguise. Noggenfogger Elixir makes the collector a skeleton or shrinks them. Savory Deviate Delight turns the collector into a pirate or ninja. A new disguise replaces the previous one. The skeleton can cover a druid's bear or cat form while its abilities remain active.

Materials sell immediately for whole gold, into the shared wallet in co-op.
The pickup popup shows the amount actually credited, including applicable
bonuses. There is no separate crafting inventory.

Item names and icons refer to Classic consumables and materials, but their
effects are adapted to AZSX. **Name colors are cosmetic item quality, not buff
strength or sale value**. Native battleground runes retain their own effects.

## AZSXUI presentation

AZSXUI is included in AZSX and installed automatically. It handles nameplate
styling, event/cache minimap markers, Supplies/Fortune/crystal notices and the
Sayge progress bar. Caches and Supplies use different native icons; crystals
have no minimap markers. Fortune leads the native buff row, followed by
Crystal Resonance.

Event widgets share a centered anchor under the top HUD and stack without
overlap by default. Position, scale, opacity and visibility can be adjusted
without restarting:

```text
/azsxui widget anchor position 0 -85
/azsxui widget crystals scale 1.2
/azsxui widget crystals opacity 0.8
/azsxui widget crystals show off
/azsxui widget crystals show on
/azsxui widget crystals reset
```

Widget names are `crystals`, `supplies`, `fortune`, `sayge`, `revive`, `loot` and `marks`. Hiding a widget
only changes its display; it never disables rewards or changes trial progress.
AZSXUI leaves the world-number font unchanged. AZSX scales outgoing normal hits to one third of native size and critical hits to one half. See the [AZSXUI guide](../../addons/AZSXUI/README.md)
for all commands and nameplate limits.

## Optional events

The host controls event generation in co-op. To change event settings, close
the game and edit `AZSX/options.json` in the game folder. These events are
enabled by default:

```json
{
  "treasure_goblins": true,
  "lost_supplies": true,
  "darkmoon_trials": true
}
```

Change the desired value to `false`, preserve any other existing settings, and
restart through the AZSX launcher. Turning off `darkmoon_trials` restores
boss-based Fortunes. These are gameplay settings; AZSXUI's widget commands only
control presentation.

## Share feedback

[Report an issue or share a balance observation](https://github.com/tiboutDev/azeroth-survivors-extended/issues).
Include your game and AZSX versions, map, difficulty, classes and player count,
plus what you expected and what happened.

### Compact AZSXUI HUD

AZSXUI adds one statistics button beside the native action bar. Hover to consult
stats without opening the character sheet. Clicking the button or pressing the
configured character-sheet key opens the native character sheet and AZSX stats
together; a second click/press closes both. All nonzero native stats fit into
four columns, with short labels and complete terms on hover.
Visible rows refresh while the panel is open. In coop, XP and gold show the
effective AZSX shared-award bonuses instead of only personal native values.
The panel opens immediately in front of equipment, beneath the native XP artwork
and foreground gryphons. The native XP texture and top border stay intact.

A compact ornamented strip sits just below the screen's top edge, showing kills,
gold and the next cache price. Gold has a soft light behind its value when a cache
is affordable. The native run clock and next-boss countdown sit below the strip.
Five circular equipment positions show the amulet, two rings, then two trinkets.
Native slot silhouettes remain for empty positions. Descriptions use the owner's
native item text, rank and any verified active ring enchantment. `/azsxui hud off` restores the native layout and previous visibility
preferences.

The raised compact player health bar shows current HP and character level, without a portrait or
resources. Native buffs and debuffs stay above it, with durations above their
icons. The old player and target frames are hidden while this HUD is enabled.
Boss, awake-guardian and goblin bars show centered names without percentages.
Enemy nameplates use smaller smooth health capsules and are visible with
overlapping layout by default. Native damage numbers keep their world positions
and font. Normal damage renders at one third of native size and critical damage at one half.

Cooperative ally builds use matching textured panels, with buffs immediately
outside their right edge. An ornamented Objectives header below the minimap
remains visible. One Guardians quest lists living guardians and upcoming arrivals;
an arrival countdown appears on its own indented line during the final 30 seconds.
Quest headings and objective entries have separate spacing. Sayge has its own quest.
Minimap pins cover all active guardians, with event and cache priorities to keep
important markers above less urgent ones. Pins fade as the native player arrow passes over them. Overlapping pins of the same kind and status show a count and separate again when zoomed apart. Goblins no longer receive an automatic
skull marker.

Reward and ally panels use native parchment and border textures. Compact pickup
panels grow for wrapped text and stack without overlap. Numeric bonuses are green,
their labels white, and duration appears on a separate line. Supplies sold for
gold show the exact earned amount beside a native WoW gold coin. New boss-item
drops use their real icons; already-collected items produce no loot popup. Marks
gained together add up in one notification, with the latest native total.
Fortune acquisition appears once per actual reward; a pause or new ritual does
not replay the previous reward. Original chat messages remain available.

The row below health shows active Crystal Resonance, Sayge's Fortune, the equipped gadget and a reserved healing potion. The potion icon clears when its charge is spent. Crystals show one total count on their icon; the tooltip lists accumulated
bonuses. Fortune has a small icon for the bonus category. The health capsule keeps
native shaded artwork. Sayge's icon-free channel bar shows remaining time inside
the bar; the same style also displays native resurrection progress.

These changes use existing client assets. Health, targeting, auras, progression,
rewards and native game authority remain unchanged. No native textures are shipped
in AZSX.

## Cooperative area bonuses

In coop, positive bonuses to the global spell area/range stat retain **50%**
of their nominal value: +40% becomes +20%, and +100% becomes +50%.
This covers all contributions to that global stat. Base spell radii, separate
spell-specific radius mechanics, reductions and solo play remain unchanged.
The effective stats panel shows the adjusted total; individual item descriptions
continue to describe nominal bonuses. This rule does not change enemy scaling.

## Cooperative Rules information

Hover the kill counter to see current party members ranked by their personal run kills. Tied players share a rank. Hover or click the circular question mark to the right of the top counter ribbon for co-op rules on shared XP, party gold, personal chest choices and global area/range bonuses. Without the custom HUD, the help button sits beside the first ally frame.

## Solo engineering escort

**Status: Playtesting**

In solo runs, a damaged Mechanical Chicken can appear somewhere on the map
after the first boss is defeated. Stay within six yards while it travels to a
Goblin Teleporter. If you leave, it waits
and your progress resumes when you return. A completed escort gives one
engineering gadget for the rest of the run. The route adapts to the map's
size; placement retries if no safe native path is found. There is at most one
escort in a run; an unsuccessful or expired escort gives no reward. The chicken
cannot be killed.

The four possible gadgets have no ranks: Net, Rocket Boots, Stun Grenade and
Shrink Ray. They trigger automatically and deal no damage. Net and Grenade
control at most three ordinary enemies; Shrink Ray changes appearance only
(x0.5, x1 or x1.5). The parachute cape remains under feasibility review.
The optional AZSXUI addon lists the escort beneath the minimap, marks the
chicken with a native `INV_Misc_EngGizmos_06` minimap icon, and keeps an equipped-gadget
badge in the effects row below health. The game rules do not require the
addon.

See [the engineering details](../features/SOLO-ESCORT.md). Check the release
notes before expecting this source feature in an installed build.


## Banished Demon

The unfinished Banished Demon event is disabled in solo and co-op.

## Cooperative effects

Allies can trigger your Polymorph explosions; their damage remains yours.
Hunter's Mark shares the strongest active bonus without stacking. Faerie Fire
keeps each player's marks and damage triggers independent.


Cooperative healing talents retain their owner. Swiftmend can rescue a nearby
ally below 40% health; Tree of Life can benefit allies using the healer's stats
and overheal cap. Divine Aegis can burst from an ally wearing your Power Word:
Shield. Hammer of the Righteous requires your own shield on yourself.
Tranquility and these holy shield bursts benefit from the producer's area bonus.
Lightning Shield remains a retaliation effect, not an absorption shield.

Ancestral Fortitude's free Healing Wave uses normal cooperative target selection at half effectiveness and can heal a wounded ally.

Your armor set can trigger when a teammate finishes an enemy carrying your Serpent Sting, Curse of Agony, Rupture or Flame Shock. Bonuses for killing with a specific spell require your own killing blow. A Devout shield explodes around the player you protected and uses your area bonus. Mortal Strike and Earth Shock set vulnerabilities help the party. Crowd-control damage bonuses apply to the wearer even when an ally controlled the enemy.

A legendary necklace can trigger its explosion when a teammate finishes an enemy carrying your matching damage-over-time spell. The Power Word: Shield necklace bursts around the player you protected using your area bonus. Holy Light and Regrowth necklaces shield the player you heal for up to 40% of your maximum health. Vampiric Renew and Vampiric Rejuvenation grant the healed player 5% leech on their own damage for 10 and 12 seconds respectively.

In the Armory and during solo or cooperative runs, the Talents panel adds a short Co-op note below selected talent descriptions to explain ally targeting, shield ownership and control effects.
