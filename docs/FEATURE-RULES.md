# Active feature rules

This summarizes the current 0.122 policy. The portable implementations in
`crates/asx-gameplay/src/` are the precise reference for calculations; selected
native translation belongs in `asx-game`. Stable feature identities live in
[ASX-FEATURES.md](ASX-FEATURES.md), and player explanations in
[player/FEATURES.md](player/FEATURES.md).

## Shared rewards

Coop uses a host-owned wallet with native actor balances as projections.
Purchased caches debit once and deliver personal native choices. Voluntary full
skips and automatic conversions keep their distinct existing caller/rounding
rules. Strongest racial/fortune contributions and permanent armory-Gold rules
must not be replaced by summing every participant's equipment.

Never duplicate a native personal reward or retry an uncertain native commit.
Take pending work before native callbacks and preserve successful per-actor
acknowledgements. Current guardian rewards and personal collection marks are
native; supplemental guardian items and duplicate-loot gold compensation are not
installed by the selected 0.122 path.

## Jewelry

The first accepted amulet locks one of eight equally likely gems per actor/run.
Subsequent upgrades and replacements retain it. Gems have fixed values, no
rarity/Luck draw modifier and no permanent save writes.

| Gem | Bonus |
| --- | --- |
| Bold Ruby | +12% damage |
| Quick Sapphire | +6% cooldown recovery |
| Solid Bloodstone | +20% maximum health |
| Deadly Topaz | +5% critical strike chance |
| Potent Amethyst | +35% critical strike damage |
| Enduring Diamond | +3% damage reduction |
| Renewing Emerald | +1 health per second |
| Expansive Opal | +20% area of effect |

Ring enchantment kind locks on the first accepted upgrade. Five total-value
tiers begin at native ring ranks **2/4/7/10/14**: Minor, Lesser, unprefixed,
Greater and Superior. Kind and rank remain per ring. Native additive-stat inputs
are temporarily borrowed and restored; repeated rebuilds must not accumulate
bonuses. Preview/apply/improve text must reflect the accepted native rank.
Native capped rings can remain unenchantable or low-tier.

Rules: `gems.rs`, `enchantments.rs`; coordination uses bounded reward services.
Shared retained-tooltip/card representations do not enable disabled glyphs.

## Fortunes

Sayge's Dark Fortunes grant one personal replacement per completed native boss
encounter, including endless encounters. One active fortune per player persists
until replaced or the run ends. Luck, XP, Gold, Pickup Radius and Movement Speed
are equally likely; rarity weights are 60/25/10/4/1 and do not use player Luck.
Strength progression caps at five completed encounters; later encounters still
replace fortunes. Final percentages are rounded once and used consistently for
application, tooltip and notification.

Coop XP and Gold use the strongest current fortune in each category. Other
bonuses remain personal. Pickup modifies the native radius multiplicatively;
Speed adds to the native speed multiplier. Rebuild native inputs before replacing
bonuses. Four utility trinkets are excluded from the selected reward pool without
removing existing inventory. Rules and coefficients: gameplay `blessings.rs`.

## Support and targeting

Allied direct/periodic healing, shields, retaliation and hybrid effects retain
the original producer's native modifiers, counters and proc/cooldown ownership.
Caster-only Ice Barrier remains personal. Copy source/recipient identities and
aura budgets before native context swaps; no gameplay lock spans a native call.
Do not duplicate effects already delivered by upstream.

Hunter's Mark uses the strongest valid shared vulnerability. Eligible Shield
Slam, Hammer of Justice and Shred hits establish persistent ordinary pursuit/
attack ownership under their talent and successful-hit checks. Special targeting
and already-cast projectiles retain native ownership. Enemy/player life changes,
departure and run reset retire old associations.

Temporary immunity excludes ordinary aggro candidates and discards prior taunts;
a fresh eligible taunt can still establish ownership. Restore scoped native
candidate flags. Absorb shields and ordinary damage reduction are not aggro wipes.
Rules: `support.rs`, `periodic.rs`, `marks.rs`, `taunts.rs`, `immunity_aggro.rs`.

## Resurrection fatigue

Resurrection Sickness adds one stack per resurrection, caps at three, and reduces
outgoing damage/healing by 5% per stack. One stack recovers per 60 alive simulation
seconds; pause/death suspend recovery. Native harmful-aura projection shows the
name, icon, stacks and remaining time. Rules: `resurrection_fatigue.rs`.

## Presentation and validation

Keep native party/menu widgets and owner-correct tooltips. See
[party UI](OFFICIAL-PARTY-UI.md) and [pause](COOP-PAUSE-01220.md). Feature rule tests,
selected-memory fixtures and native stand-ins cover different risks. Consult
[testing](TESTING.md) rather than copying a historical live-trial recipe.
