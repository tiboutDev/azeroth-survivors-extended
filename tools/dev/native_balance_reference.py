"""Human-readable native-only common-item reference and rank tables."""
from native_balance_math import RARITIES, MULTIPLIERS, AMULET_RATES, amulet_effects, amulet_scale, jewelry_history, jewelry_bonuses

SPECIAL = {
 'eye_of_hakkar': 'Critical hits apply a damage-over-time effect totaling 80% × S of the triggering damage over 4 seconds (native spell ID 12654). The duration stays fixed.',
 'enlightenment': 'Your hits leap to one additional enemy for 30% × S of the triggering damage. The extra chain count stays at one.',
 'vek_nilash': 'All applicable spells gain additional targets: min(floor(S + 0.0001), 2). This is +1 at ranks 1–6 and +2 from rank 7 for this Epic amulet. Its separate damage bonus also scales.',
 'fallen_dragon': 'A killing blow leaves you at 50% maximum health instead, once every 60 seconds. Neither parameter scales with rank. Its maximum-health and regeneration bonuses do scale.',
 'onyxia_tooth': 'Critical hits splash 30% × S of the triggering damage within 5 yards, at most 6 triggers per second. Radius and trigger limit remain fixed. Its critical-chance and critical-damage bonuses also scale.',
 'fire_lord': 'Enemies you kill explode for 15% × S of their maximum health within 6 yards, at most 4 triggers per second (native visual/spell ID 11113). Radius and trigger limit remain fixed.'}


def n(value): return f'{value:.4f}'.rstrip('0').rstrip('.')


def bonus_text(row, value):
    return '; '.join(f'{key}: {n(v)}' for key,v in jewelry_bonuses(row,value).items())


def render(data):
    lines=['# Azeroth Survivors — Native balance reference','',
      'This is the current working reference for common native items and statistics. Update its values only when explicitly requested. The observed client version is provenance, not an expiry date or a requirement for consulting this document. Class-specific spells, talents, gear and legendary spell-remaking amulets are outside scope.','',
      f"Observed source: {data['native_version']}; executable SHA-256 `{data['executable_sha256']}`.",'',
      '## Reading the numbers','',
      'Values below are item contributions, before character-wide caps and interactions with other sources. Percentage-point additions to crit/echo are not multiplicative increases of the existing chance. Damage, maximum-health, regeneration and other displayed character totals also depend on level and other native sources. Proc percentages describe the native effect input; they are not a promise of final damage against every target.','',
      'The native implementation uses float32 arithmetic. Tables show four decimal places to avoid hiding small real gains behind whole-number tooltips. The JSON retains exact definition floats, source RVAs and effect bytes. The portable calculator follows float32 operation order for jewelry; amulet power uses float32-rounded exponentiation, with possible last-bit differences across powf libraries.','',
      '## Rings and trinkets: rank and rarity','',
      '**Rank and displayed rarity do not uniquely determine the bonus.** Each acquisition adds a gain determined by the rarity of that acquisition and the existing value. The displayed rarity is the highest acquired rarity; it does not retroactively rescale earlier gains. Rank is the count of acquisitions, starting at one.','',
      '| Acquisition rarity | Multiplier M |','| --- | --- |']
    lines += [f'| {name} | {mult:g} |' for name,mult in zip(RARITIES,MULTIPLIERS)]
    lines += ['', 'The native PvP rarity (index 5) uses the Legendary multiplier in this calculation. It has no separate arithmetic column here; the calculator accepts PvP histories.', '',
      'Let `b` be the item coefficient, `x` its stored bonus before the pick, and `r` the number already acquired (`0` for a new item). Let `q = b × M × (1.5 if r = 0 else 1)`. Add the gain below to `x`, then increment rank. Use the rarity of the new pick, not the highest displayed rarity.','',
      '| Family | Gain from the next acquisition |','| --- | --- |',
      '| Damage, cooldown reduction, critical chance, critical damage, area | `q / (1 + 0.08 × r)`; no item-value hard cap in this function |',
      '| Echo | Same rank formula, capped to the remaining room below 0.60 |',
      '| Maximum health and the two compound health/reduction items | Integrate the soft-cap curve in 16 substeps, described below |',
      '| Regeneration, heal/kill, lifesteal | `min(q × max(3b − x, 0) / (3b), max(3b − x, 0))` |',
      '| Movement speed, pickup, XP, gold, luck, buff power | Same formula with ceiling `5b` |',
      '| DoT haste | Same formula with ceiling `10b` |',
      '| Soulstone | +1 available life for Common–Rare, +2 for Epic–Legendary, clipped to the remaining room below 2 unspent lives |','',
      'For the health family, start `y = x`, `T = 3b`, `h = q/16`. Repeat 16 times: if `y ≤ T`, set `w = 1 − 0.75y/T`; otherwise `w = 0.25(T/y)²`. Then `y ← y + h×w`. The gain is `y − x`. This is a soft cap: crossing `3b` does not stop health growth. The current value, including the first pick, participates in every substep.','',
      'For finite-ceiling jewelry, the choice pool stops offering another upgrade once the current value reaches 90% of the ceiling. Echo uses a 0.54 offer threshold. A pick already offered can overshoot the offer threshold, up to its arithmetic cap. Thus theoretical repeated-rank sequences must not be confused with guaranteed obtainable upgrades. Soulstone availability depends on unspent lives, which can decrease after death.','',
      '### Compound items and final contributions','',
      '- Archimtiros’ Ring of Reckoning: damage reduction `min(x, 3b)` and maximum-health bonus `4x`. Its reduction component caps at 10.5%, while its health component keeps following the soft-cap curve.',
      '- Star of Mystaria: maximum-health bonus `x`, plus damage reduction `min(0.16x, 0.12)` (12% maximum from this item).',
      '- Soulstone restores half maximum health on resurrection and holds at most two unspent lives. A rank-only table cannot reconstruct consumed lives.',
      '- Cooldown inputs add into H, then the native character cooldown factor is 1/(1+H). The displayed reduction is H/(1+H). An item input x changes a pre-existing factor 1/(1+H) to 1/(1+H+x): its marginal relative reduction is x/(1+H+x). For example, x=0.135 alone gives 11.8943% shorter cooldowns, not 13.5%; total H=1 gives 50%, not 100%. Jewelry tables show both the native input and the reduction with no other haste. Spell-specific modifiers remain outside that comparison.', '',
      '### Common item coefficients','',
      '| Slot | Item | b | Bonus at first Common acquisition |','| --- | --- | --- | --- |']
    for row in data['jewelry']:
        lines.append(f"| {row['slot']} | {row['name']} | {n(row['coefficient'])} | {bonus_text(row,jewelry_history(row,[0]))} |")
    lines += ['', 'See [rank and rarity tables](JEWELRY-RANKS.md) for all 23 items and all five rarities. These use a constant acquisition rarity; use the calculator for mixed histories.', '',
      '## Common amulets','',
      'Common amulets have an intrinsic rarity. It determines empowerment speed; it is not the separate minimum-pick-tier field. The base effect starts at its listed value, without the ring/trinket 1.5 first-pick multiplier. Reacquiring the worn amulet increases its rank.','',
      'For rank `r ≥ 1`, the empowerment factor is `S(r,R) = 3 − 2 × (1 − a[R]/2)^(r−1)`. It starts at 1 and approaches 3. There is no extra ring-style rarity multiplier.','',
      '| Intrinsic rarity | a | S at rank 1 | rank 2 | rank 3 | rank 5 | rank 10 |','| --- | --- | --- | --- | --- | --- | --- |']
    for rarity,name in enumerate(RARITIES):
        lines.append('| '+' | '.join([name,n(AMULET_RATES[rarity])]+[n(amulet_scale(rank,rarity)) for rank in (1,2,3,5,10)])+' |')
    lines += ['', 'Ordinary stat effects multiply their base values by S. Special effects scale only the fields listed below; multiplying every duration, radius or count by S would be incorrect.', '',
      '| Amulet | Intrinsic rarity | Complete rank-1 contributions |','| --- | --- | --- |']
    for row in data['amulets']:
        desc='; '.join(f'{label}: {n(value)} {unit}' for label,value,unit in amulet_effects(row,1))
        lines.append(f"| {row['name']} | {RARITIES[row['rarity']]} | {desc} |")
    lines += ['', '### Special effects','']
    for row in data['amulets']:
        if row['key'] in SPECIAL:lines.append(f"- **{row['name']}** — {SPECIAL[row['key']]}")
    lines += ['', 'See [amulet rank tables](AMULET-RANKS.md) for complete bonuses at ranks 1, 2, 3, 5, 7, 10 and 20. Class-specific legendary variants are deliberately excluded, rather than treated as missing common items.', '',
      '## Common statistics and native UI semantics','',
      '`AZS_Pct(x)` displays `100x` as a signed whole percentage. Expressions below explain display units; they are not substitutes for a full character simulation.', '',
      '| Key | Statistic | Native display expression | Description |','| --- | --- | --- | --- |']
    for row in data['stats']:
        lines.append('| '+' | '.join(str(row[k]).replace('|','\\|') for k in ('key','name','display_expression','description'))+' |')
    lines += ['', '## Calculator and maintenance','',
      'The calculator works entirely from the saved reference. No executable or client installation is required to consult it or calculate common-item contributions.','',
      '```sh',
      'python tools/dev/native_balance_calculate.py --item accuria --history Common,Rare,Epic',
      'python tools/dev/native_balance_calculate.py --item archimtiros --history Rare,Rare,Rare',
      'python tools/dev/native_balance_calculate.py --item onyxia_tooth --rank 7',
      '```','',
      'Do not refresh automatically when a client version changes. On explicit request, review changed native formulas and regenerate the reference. The raw extractor retains an exact-image guard because fixed offsets cannot safely be interpreted in an unknown executable; this is a protection for extraction, not a version restriction on the reference.','',
      '```sh',
      'uv run --no-project --with pefile python tools/dev/native_balance.py --exe /path/to/AzerothSurvivors.exe --output docs/native-balance',
      '```','',
      '## Evidence and scope','',
      '- Jewelry gain: RVA `0x5c64c10–0x5c65284`; rarity multipliers: `0x71432d8`.',
      '- Inventory acceptance: `0x5d459f0`, especially `0x5d45ff9–0x5d46379` (previous value/rank, additive gain, incremented rank, maximum displayed rarity).',
      '- Upgrade eligibility: `0x5d434e0`; compound-item formatting: `0x5c65290`; native stat reconstruction: `0x5cc4c30` (cooldown conversion at `0x5cc56c4–0x5cc5757`).',
      '- Amulet definition-to-owned conversion: `0x5c69d00`; intrinsic rarity copies from definition +72 to owned +96.',
      '- Amulet factor: `0x5c69060`; decay rates: `0x747e734`; per-effect scaling: `0x5dd34c0`, `0x5bb5f00`.',
      '- Effect descriptions: `0x5dda120`; hit filters: `0x5c7b640`; proc-limit formatter: `0x5c79e60`.',
      '- Sixteen static amulets and twenty-three rings/trinkets are covered. XP curves, cache prices, acquisition probabilities, class mechanics and full character-wide stat reconstruction are outside this item-progression reference.',
      '- These formulas describe native source arithmetic. They do not establish live proc frequency, encounter DPS or the absence of runtime bugs.', '']
    lines += ['', '## Related native world references', '', 'See [Enemies and encounters](ENEMIES.md) for all map definitions and scaling, and [Economy](ECONOMY.md) for gold, XP pickup amounts, caches, skips and rerolls. These remain native-only references, with assumptions and unresolved fields stated explicitly.', '']
    return '\n'.join(lines)


def rank_tables(data):
    lines=['# Native rings and trinkets — Constant-rarity examples','',
      'Each column assumes every acquisition had that rarity. Cells show the full contribution, not the latest gain. These are arithmetic trajectories; the choice pool may stop offering an item before the listed rank (see the main reference). Soulstone assumes no life has been spent. Displayed values are rounded, never fed back into subsequent calculations.','']
    for row in data['jewelry']:
        lines += [f"## {row['name']}",'','| Rank | '+' | '.join(RARITIES)+' |','| --- | --- | --- | --- | --- | --- |']
        for rank in (1,2,3,5,10,20):
            lines.append('| '+str(rank)+' | '+' | '.join(bonus_text(row,jewelry_history(row,[r]*rank)) for r in range(5))+' |')
        lines.append('')
    amulets=['# Native common amulets — Rank examples','','Only the intrinsic rarity of each common amulet is used. Percentage values below are contributions; they do not bypass global caps.','']
    for row in data['amulets']:
        amulets += [f"## {row['name']} — {RARITIES[row['rarity']]}",'','| Rank | S | Complete contributions |','| --- | --- | --- |']
        for rank in (1,2,3,5,7,10,20):
            text='; '.join(f'{label}: {n(value)} {unit}' for label,value,unit in amulet_effects(row,rank))
            amulets.append(f"| {rank} | {n(amulet_scale(rank,row['rarity']))} | {text} |")
        amulets.append('')
    return {'JEWELRY-RANKS.md':'\n'.join(lines),'AMULET-RANKS.md':'\n'.join(amulets)}
