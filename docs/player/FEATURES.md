# ASX Features

[Home](../../README.md) · [Installation](INSTALLATION.md) · **Features** · [In the workshop](WORKSHOP.md)

---

ASX adds new decisions during a run and more ways to play together.

## Build choices

These features are available in **solo and co-op**.

### 💍 Ring Enchantments

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

New enchantments and tier improvements display their values directly on reward cards.

Unchanged enchantments appear as a compact icon and title with an **Active** label. Hover the header to inspect their values.

Some rings have low maximum ranks and may receive only a low-tier enchantment—or none. Reaching Superior is not expected in every run.

### 💎 Amulet Gems

Your first accepted amulet locks one of **eight gems**, each granting a single fixed stat bonus.

- Your gem stays with you until the run ends.
- Later amulet upgrades and replacements preserve it.
- Gem bonuses do not scale with amulet rank.
- All eight gems have equal chances, unaffected by Luck.

New gems display their bonus directly on reward cards. Retained gems use a compact **Active** presentation, with their value available on hover.

### 🎪 Sayge’s Dark Fortunes

Completing a boss encounter grants each player an independent random fortune.

**Five categories:** Luck, Experience, Gold, Pickup Radius and Movement Speed.

- Only **one fortune** is active per player.
- A new fortune replaces the previous one.
- Fortunes survive death and remain until replaced or the run ends.
- Category chances are equal, and rarity chances are fixed.
- Luck does not influence either roll.
- Fortune strength increases with completed boss encounters, capped at **five encounters**.
- Endless bosses continue granting replacements after the strength cap.
- Encounters involving several bosses grant one fortune when the encounter is completed.

Fortunes appear as buffs, with a reward notification when received. Their bonuses use whole percentage values.

In co-op, the **strongest active Experience fortune** and **strongest active Gold fortune** contribute to the group independently. They do not stack across players.

Luck, Pickup Radius and Movement Speed remain personal.

The four utility trinkets associated with Luck, Experience, Gold and Pickup Radius are excluded from ASX reward pools.

## Cooperative features

These features apply to **co-op**.

### 🤝 Shared resources and personal rewards

ASX adapts resource and reward handling for a shared adventure.

- **Shared in-run XP** helps participants progress together.
- **Shared gold** gives the party one common wallet.
- Purchased chests provide personal reward choices for eligible participants.
- Paid spell and item rerolls spend shared gold.
- Voluntary reward skips contribute their full gold award, without dividing it by player count.
- Reward selection pauses the group until everyone has finished.

Eligible racial bonuses and Experience/Gold fortunes use their strongest contribution rather than stacking duplicate bonuses across participants.

Permanent armory gold bonuses likewise use the strongest participant contribution.

Luck remains personal.

### ✨ Allied support and team synergy

Supported healing, shielding, buff and talent effects extend to teammates, giving support builds a stronger role in the party.

Effects retain their original caster’s attribution.

**Mage Ice Barrier remains caster-only.**

### 🛡️ Tanking and aggro

ASX adds talent-based taunts:

| Class | Ability | Required talent |
| --- | --- | --- |
| Warrior | Shield Slam | Fortified |
| Paladin | Hammer of Justice | Ardent Defender |
| Druid | Shred | Dire Bear Form |

Successful eligible hits make affected enemies pursue the tank and direct ordinary attacks toward them. Special abilities retain their own targeting rules.

Another eligible taunt can take over. Death or departure clears the previous assignment; returning to the fight requires a new taunt.

Temporary invincibility clears existing ordinary aggro and taunt assignments. A fresh taunt can still draw enemies while the player is invincible.

### 👁️ Party information and pauses

ASX enriches the party interface with information useful for understanding your teammates’ builds.

- Compact rows show equipped spells and items.
- Additional buff icons expose supported beneficial effects.
- Spell and item tooltips reflect their owner’s build.
- Group pauses identify who is choosing a reward or holding a menu open.
- Waiting messages display character names.
- Players who finish choosing remain paused until the group is ready.

### 💀 Resurrection Sickness

Each resurrection adds one personal stack of **Resurrection Sickness**.

| Rule | Effect |
| --- | --- |
| Penalty per stack | −5% damage and healing done |
| Maximum stacks | 3 |
| Maximum penalty | −15% |
| Recovery | One stack removed per 60 seconds spent alive |

Another resurrection restarts the recovery interval. Death, pauses and reward choices suspend recovery.

The debuff display shows stacks and recovery time.

The penalty affects outgoing damage and healing. It does not reduce maximum health or healing received from another player.

## Known issue

XP may remain desynchronized after a cooperative death and resurrection. This issue remains open.

[Report a problem →](https://github.com/tiboutDev/azeroth-survivors-coop/issues)
