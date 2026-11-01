# Azeroth Survivors — Native balance reference

This is the current working reference for common native items and statistics. Update its values only when explicitly requested. The observed client version is provenance, not an expiry date or a requirement for consulting this document. Class-specific spells, talents, gear and legendary spell-remaking amulets are outside scope.

Observed source: 0.135.1; executable SHA-256 `71e5f1767279db4bd428d1cd9c5d50d37759b72295386a69f9e3f6e70bc59249`.

## Reading the numbers

Values below are item contributions, before character-wide caps and interactions with other sources. Percentage-point additions to crit/echo are not multiplicative increases of the existing chance. Damage, maximum-health, regeneration and other displayed character totals also depend on level and other native sources. Proc percentages describe the native effect input; they are not a promise of final damage against every target.

The native implementation uses float32 arithmetic. Tables show four decimal places to avoid hiding small real gains behind whole-number tooltips. The JSON retains exact definition floats, source RVAs and effect bytes. The portable calculator follows float32 operation order for jewelry; amulet power uses float32-rounded exponentiation, with possible last-bit differences across powf libraries.

## Rings and trinkets: rank and rarity

**Rank and displayed rarity do not uniquely determine the bonus.** Each acquisition adds a gain determined by the rarity of that acquisition and the existing value. The displayed rarity is the highest acquired rarity; it does not retroactively rescale earlier gains. Rank is the count of acquisitions, starting at one.

| Acquisition rarity | Multiplier M |
| --- | --- |
| Common | 1 |
| Uncommon | 1.4 |
| Rare | 1.8 |
| Epic | 2.4 |
| Legendary | 3.2 |

The native PvP rarity (index 5) uses the Legendary multiplier in this calculation. It has no separate arithmetic column here; the calculator accepts PvP histories.

Let `b` be the item coefficient, `x` its stored bonus before the pick, and `r` the number already acquired (`0` for a new item). Let `q = b × M × (1.5 if r = 0 else 1)`. Add the gain below to `x`, then increment rank. Use the rarity of the new pick, not the highest displayed rarity.

| Family | Gain from the next acquisition |
| --- | --- |
| Damage, cooldown reduction, critical chance, critical damage, area | `q / (1 + 0.08 × r)`; no item-value hard cap in this function |
| Echo | Same rank formula, capped to the remaining room below 0.60 |
| Maximum health and the two compound health/reduction items | Integrate the soft-cap curve in 16 substeps, described below |
| Regeneration, heal/kill, lifesteal | `min(q × max(3b − x, 0) / (3b), max(3b − x, 0))` |
| Movement speed, pickup, XP, gold, luck, buff power | Same formula with ceiling `5b` |
| DoT haste | Same formula with ceiling `10b` |
| Soulstone | +1 available life for Common–Rare, +2 for Epic–Legendary, clipped to the remaining room below 2 unspent lives |

For the health family, start `y = x`, `T = 3b`, `h = q/16`. Repeat 16 times: if `y ≤ T`, set `w = 1 − 0.75y/T`; otherwise `w = 0.25(T/y)²`. Then `y ← y + h×w`. The gain is `y − x`. This is a soft cap: crossing `3b` does not stop health growth. The current value, including the first pick, participates in every substep.

For finite-ceiling jewelry, the choice pool stops offering another upgrade once the current value reaches 90% of the ceiling. Echo uses a 0.54 offer threshold. A pick already offered can overshoot the offer threshold, up to its arithmetic cap. Thus theoretical repeated-rank sequences must not be confused with guaranteed obtainable upgrades. Soulstone availability depends on unspent lives, which can decrease after death.

### Compound items and final contributions

- Archimtiros’ Ring of Reckoning: damage reduction `min(x, 3b)` and maximum-health bonus `4x`. Its reduction component caps at 10.5%, while its health component keeps following the soft-cap curve.
- Star of Mystaria: maximum-health bonus `x`, plus damage reduction `min(0.16x, 0.12)` (12% maximum from this item).
- Soulstone restores half maximum health on resurrection and holds at most two unspent lives. A rank-only table cannot reconstruct consumed lives.
- Cooldown inputs add into H, then the native character cooldown factor is 1/(1+H). The displayed reduction is H/(1+H). An item input x changes a pre-existing factor 1/(1+H) to 1/(1+H+x): its marginal relative reduction is x/(1+H+x). For example, x=0.135 alone gives 11.8943% shorter cooldowns, not 13.5%; total H=1 gives 50%, not 100%. Jewelry tables show both the native input and the reduction with no other haste. Spell-specific modifiers remain outside that comparison.

### Common item coefficients

| Slot | Item | b | Bonus at first Common acquisition |
| --- | --- | --- | --- |
| Ring | Band of Accuria | 0.17 | Damage (%): 25.5 |
| Ring | Quick Strike Ring | 0.09 | Cooldown recovery input (%): 13.5; Cooldown reduction alone (%): 11.8943 |
| Ring | Heavy Dark Iron Ring | 0.2 | Maximum health (%): 25.2618 |
| Ring | Master Dragonslayer's Ring | 0.04 | Critical chance (pp): 6 |
| Ring | Archimtiros' Ring of Reckoning | 0.035 | Damage reduction (%): 4.4208; Maximum health (%): 17.6832 |
| Ring | Ring of the Devoured | 1.5 | Health / sec: 2.25 |
| Ring | Ring of Spell Power | 0.14 | Area (%): 21 |
| Ring | Seal of the Damned | 0.1 | DoT haste (%): 15 |
| Ring | Blooddrinker's Band | 0.01 | Lifesteal (%): 1.5 |
| Trinket | Earthstrike | 0.13 | Damage (%): 19.5 |
| Trinket | Talisman of Ephemeral Power | 0.07 | Cooldown recovery input (%): 10.5; Cooldown reduction alone (%): 9.5023 |
| Trinket | Lifegiving Gem | 0.16 | Maximum health (%): 20.2094 |
| Trinket | Hand of Justice | 0.06 | Echo (pp): 9 |
| Trinket | Blackhand's Breadth | 0.195 | Critical damage (pp): 29.25 |
| Trinket | Carrot on a Stick | 0.08 | Movement speed (%): 12 |
| Trinket | Gnomish Universal Remote | 0.35 | Pickup radius (%): 52.5 |
| Trinket | Insignia of the Horde | 0.12 | Experience (%): 18 |
| Trinket | Royal Seal of Eldre'Thalas | 0.25 | Gold (%): 37.5 |
| Trinket | Mark of the Chosen | 0.15 | Luck (pp): 22.5 |
| Trinket | Onyxia Blood Talisman | 2 | Health / kill: 3 |
| Trinket | Horn of the Warchief | 0.15 | Buff power (%): 22.5 |
| Trinket | Major Soulstone | 1 | Unspent lives: 1 |
| Trinket | Star of Mystaria | 0.1667 | Maximum health (%): 21.0515; Damage reduction (%): 3.3682 |

See [rank and rarity tables](JEWELRY-RANKS.md) for all 23 items and all five rarities. These use a constant acquisition rarity; use the calculator for mixed histories.

## Common amulets

Common amulets have an intrinsic rarity. It determines empowerment speed; it is not the separate minimum-pick-tier field. The base effect starts at its listed value, without the ring/trinket 1.5 first-pick multiplier. Reacquiring the worn amulet increases its rank.

For rank `r ≥ 1`, the empowerment factor is `S(r,R) = 3 − 2 × (1 − a[R]/2)^(r−1)`. It starts at 1 and approaches 3. There is no extra ring-style rarity multiplier.

| Intrinsic rarity | a | S at rank 1 | rank 2 | rank 3 | rank 5 | rank 10 |
| --- | --- | --- | --- | --- | --- | --- |
| Common | 0.6 | 1 | 1.6 | 2.02 | 2.5198 | 2.9193 |
| Uncommon | 0.5 | 1 | 1.5 | 1.875 | 2.3672 | 2.8498 |
| Rare | 0.35 | 1 | 1.35 | 1.6388 | 2.0735 | 2.6459 |
| Epic | 0.25 | 1 | 1.25 | 1.4688 | 1.8276 | 2.3987 |
| Legendary | 0.2 | 1 | 1.2 | 1.38 | 1.6878 | 2.2252 |

Ordinary stat effects multiply their base values by S. Special effects scale only the fields listed below; multiplying every duration, radius or count by S would be incorrect.

| Amulet | Intrinsic rarity | Complete rank-1 contributions |
| --- | --- | --- |
| Mark of Fordring | Common | Damage: 15 % |
| Talvash's Enhancing Necklace | Common | Cooldown recovery input: 10 % |
| Pristine Enchanted South Seas Kelp | Common | Luck: 30 %; Gold: 25 % |
| Prestor's Talisman of Connivery | Uncommon | Damage: 16 %; Critical chance: 4 % |
| Animated Chain Necklace | Uncommon | Maximum health: 25 %; Damage reduction: 8 % |
| Woven Ivy Necklace | Uncommon | Health per second: 6  HP/sec; Maximum health: 12 %; Movement speed: 12 % |
| Amulet of the Darkmoon | Uncommon | Cooldown recovery input: 12 %; Movement speed: 8 % |
| Jeklik's Opaline Talisman | Rare | Lifesteal: 3 %; Heal per kill: 1  HP/kill |
| The Eye of Hakkar | Rare | Critical chance: 4.8 %; Critical-hit DoT: 80 % of triggering damage; DoT duration: 4 sec |
| Medallion of Steadfast Might | Rare | Damage reduction: 10 %; Maximum health: 15 %; Thorns: 100 % |
| Choker of Enlightenment | Rare | Additional chain targets: 1 target; Chain damage: 30 % of triggering damage |
| Amulet of Vek'nilash | Epic | Additional spell targets: 1 targets; Damage: 5 % |
| Pendant of the Fallen Dragon | Epic | Death-prevention cooldown: 60 sec; Health after death prevention: 50 % of maximum health; Maximum health: 20 %; Health per second: 5  HP/sec |
| Malice Stone Pendant | Epic | Echo: 20 %; Cooldown recovery input: 8 % |
| Onyxia Tooth Pendant | Epic | Critical chance: 4 %; Critical damage: 19.5 %; Critical-hit splash: 30 % of triggering damage; Splash radius: 5 yd; Proc limit: 6 /sec |
| Choker of the Fire Lord | Epic | Kill explosion: 15 % of victim maximum health; Explosion radius: 6 yd; Proc limit: 4 /sec |

### Special effects

- **The Eye of Hakkar** — Critical hits apply a damage-over-time effect totaling 80% × S of the triggering damage over 4 seconds (native spell ID 12654). The duration stays fixed.
- **Choker of Enlightenment** — Your hits leap to one additional enemy for 30% × S of the triggering damage. The extra chain count stays at one.
- **Amulet of Vek'nilash** — All applicable spells gain additional targets: min(floor(S + 0.0001), 2). This is +1 at ranks 1–6 and +2 from rank 7 for this Epic amulet. Its separate damage bonus also scales.
- **Pendant of the Fallen Dragon** — A killing blow leaves you at 50% maximum health instead, once every 60 seconds. Neither parameter scales with rank. Its maximum-health and regeneration bonuses do scale.
- **Onyxia Tooth Pendant** — Critical hits splash 30% × S of the triggering damage within 5 yards, at most 6 triggers per second. Radius and trigger limit remain fixed. Its critical-chance and critical-damage bonuses also scale.
- **Choker of the Fire Lord** — Enemies you kill explode for 15% × S of their maximum health within 6 yards, at most 4 triggers per second (native visual/spell ID 11113). Radius and trigger limit remain fixed.

See [amulet rank tables](AMULET-RANKS.md) for complete bonuses at ranks 1, 2, 3, 5, 7, 10 and 20. Class-specific legendary variants are deliberately excluded, rather than treated as missing common items.

## Common statistics and native UI semantics

`AZS_Pct(x)` displays `100x` as a signed whole percentage. Expressions below explain display units; they are not substitutes for a full character simulation.

| Key | Statistic | Native display expression | Description |
| --- | --- | --- | --- |
| hp | Health | string.format("%.0f", s.hp) | The most health you can have. |
| dmg | Damage | AZS_Pct(s.dmg - 1) | Every spell you cast deals this much more damage. It grows with your level, gear, talents, jewelry and buffs. |
| cd | Haste | AZS_Pct(1 - s.cd) | Your spells come back this much sooner. |
| crit | Crit | string.format("%.1f%%", s.crit * 100) | The chance a spell strikes critically. Past 100%, the rest adds to your critical damage. |
| cm | Crit Dmg | string.format("%.0f%%", s.cm * 100) | A critical strike deals this much of a spell's damage. |
| arm | Armor | string.format("%.0f%%", s.arm * 100) | Damage you take is reduced by this much (60% at most). |
| area | Area | AZS_Pct(s.area - 1) | Your spells' areas and ranges are this much larger. |
| cnt | Targets | string.format("+%d", s.cnt) | Every spell that picks its targets picks this many more. |
| echo | Echo | string.format("%.0f%%", s.echo * 100) | The chance a spell strikes twice. Past 100%, every spell strikes twice and the rest is the chance of a third. |
| spd | Speed | AZS_Pct(s.spd - 1) | You run this much faster. |
| reg | Regen | string.format("%.1f", s.reg) | Health you regain every second. |
| pick | Pickup | string.format("%.0f yd", s.pick) | Experience and gold fly to you from this far. |
| lch | Lifesteal | string.format("%.1f%%", s.lch * 100) | Heals you for this much of all the damage you deal. |
| heal | Kill Heal | string.format("%.0f", s.heal) | Every kill heals you for this much. |
| thn | Thorns | string.format("%.0f%%", s.thn * 100) | This much of the melee damage you take is dealt back to the attacker. |
| liv | Lives | string.format("%d", s.liv) | Times you rise again instead of dying. |
| xp | Experience | AZS_Pct(s.xp - 1) | You gain this much more experience. |
| gold | Gold | AZS_Pct(s.gold - 1) | You find this much more gold. |
| luck | Luck | AZS_Pct(s.luck) | Better odds of rarer upgrades when you level up. |
| bp | Buff Power | AZS_Pct(s.bp) | Your damage, haste, shield and thorns buffs are this much stronger. |
| dots | DoT Stacks | string.format("%d", s.dots) | How many of each of your damage-over-time effects one enemy carries at once. |
| doth | DoT Haste | AZS_Pct(s.doth) | Your damage-over-time effects tick this much faster. |
| big | Vs Elites | AZS_Pct(s.big) | More damage to elites and bosses. |
| exe | Execute | AZS_Pct(s.exe) | More damage to enemies below 30% health. |
| cc | Vs Controlled | AZS_Pct(s.cc) | More damage to rooted, stunned, slowed or fleeing enemies. |
| dotd | Vs Afflicted | AZS_Pct(s.dotd) | More damage to enemies carrying one of your damage-over-time effects. |
| shd | Shielded | AZS_Pct(s.shd) | More damage while you hold an absorb shield. |
| auto | Melee | AZS_Pct(s.auto) | Your melee swings deal this much more damage. |
| pet | Pet Damage | AZS_Pct(s.pet) | Your pets deal this much more damage. |

## Calculator and maintenance

The calculator works entirely from the saved reference. No executable or client installation is required to consult it or calculate common-item contributions.

```sh
python tools/dev/native_balance_calculate.py --item accuria --history Common,Rare,Epic
python tools/dev/native_balance_calculate.py --item archimtiros --history Rare,Rare,Rare
python tools/dev/native_balance_calculate.py --item onyxia_tooth --rank 7
```

Do not refresh automatically when a client version changes. On explicit request, review changed native formulas and regenerate the reference. The raw extractor retains an exact-image guard because fixed offsets cannot safely be interpreted in an unknown executable; this is a protection for extraction, not a version restriction on the reference.

```sh
uv run --no-project --with pefile python tools/dev/native_balance.py --exe /path/to/AzerothSurvivors.exe --output docs/native-balance
```

## Evidence and scope

- Jewelry gain: RVA `0x5c64c10–0x5c65284`; rarity multipliers: `0x71432d8`.
- Inventory acceptance: `0x5d459f0`, especially `0x5d45ff9–0x5d46379` (previous value/rank, additive gain, incremented rank, maximum displayed rarity).
- Upgrade eligibility: `0x5d434e0`; compound-item formatting: `0x5c65290`; native stat reconstruction: `0x5cc4c30` (cooldown conversion at `0x5cc56c4–0x5cc5757`).
- Amulet definition-to-owned conversion: `0x5c69d00`; intrinsic rarity copies from definition +72 to owned +96.
- Amulet factor: `0x5c69060`; decay rates: `0x747e734`; per-effect scaling: `0x5dd34c0`, `0x5bb5f00`.
- Effect descriptions: `0x5dda120`; hit filters: `0x5c7b640`; proc-limit formatter: `0x5c79e60`.
- Sixteen static amulets and twenty-three rings/trinkets are covered. XP curves, cache prices, acquisition probabilities, class mechanics and full character-wide stat reconstruction are outside this item-progression reference.
- These formulas describe native source arithmetic. They do not establish live proc frequency, encounter DPS or the absence of runtime bugs.


## Related native world references

See [Enemies and encounters](ENEMIES.md) for all map definitions and scaling, and [Economy](ECONOMY.md) for gold, XP pickup amounts, caches, skips and rerolls. These remain native-only references, with assumptions and unresolved fields stated explicitly.
