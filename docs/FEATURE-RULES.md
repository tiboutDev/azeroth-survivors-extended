# Active feature rules

This summarizes the current 0.125 policy. The portable implementations in
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
installed by the selected 0.125 path.

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
Native capped rings can remain unenchantable or low-tier. Declined previews retire
when that actor accepts a validated native reward choice, before reentrant
creation of the next reward. Accepted kinds stay locked; independent draws may
still randomly repeat an enchantment.

Rules: `gems.rs`, `enchantments.rs`; coordination uses bounded reward services.
Shared retained-tooltip/card representations do not enable disabled glyphs.

## Fortunes

Sayge's Dark Fortunes grant one personal replacement per completed native boss
encounter, including endless encounters. One active fortune per player persists
until replaced or the run ends. Luck, XP and Gold
are equally likely; rarity weights are 60/25/10/4/1 and do not use player Luck.
Strength progression caps at five completed encounters; later encounters still
replace fortunes. Final percentages are rounded once and used consistently for
application, tooltip and notification.

Coop XP and Gold use the strongest current fortune in each category. Other
Luck remains personal. Rebuild native inputs before replacing bonuses.
Three economy trinkets are excluded from the selected reward pool without
removing existing inventory. The pickup-range Gnomish Universal Remote is available
again. Movement speed and pickup radius are not Fortune rewards. Rules and
coefficients: gameplay `blessings.rs`.

## Optional Treasure Goblins

One host-owned Treasure Goblin (**Venture Co. Treasurer**) may be active at a time,
in solo or coop. It uses 200% model scale, native placement 6–9 yards away
(at least 4 yards from each living participant), and a 1.5-second arrival
pause before fleeing. This pause is included in the 25-second escape window. The
first opportunity uses bad-luck protection across completed pre-final boss
encounters, never before the first boss. With five native bosses the conditional
chances are 25%, 1/3, 50% and 100% if no earlier opportunity was selected. Thus an
enabled complete normal run selects its first goblin before the final boss,
with early and late outcomes rather than a fixed first-boss spawn. Selection
schedules native placement 20–40 simulation seconds later. A confirmed failed
placement re-arms that selected first opportunity; unknown native failures stop
the optional feature rather than retrying an uncertain spawn. Short/aborted
runs and unavailable native placement/contacts cannot promise an appearance.
After an actual first appearance, later encounter chances remain 6%, then 3%.
A goblin escapes after 25 simulation seconds; group
pauses suspend its deadline. A missing native instance is abandoned without a
reward. The native horde constructor retains time/Heroic/party health scaling
over the first native guardian’s base health for that map. The event
clears guardian drop classification, attacks and rewards, while setting the
native instance’s elite target-priority flag. The event cannot attack or issue
ordinary XP, gold or elite/guardian rewards. Its native movement uses a fleeing
destination and terrain clipping. The final native navigation destination is
checked again against living pursuers and the map’s native circular arena
(center and radius), with two yards of clearance for the enlarged creature.
Spawn points outside that boundary are rejected, and flee segments are clipped
inside it. Reachable rays include lateral directions and are ranked for clearance,
distance from pursuers and heading continuity. After 0.8 simulation seconds with
no meaningful displacement, the blocked heading is discouraged for 2.5 seconds
to seek a side route. Unsafe directions hold position instead of approaching a living
pursuer. The active goblin is excluded from native distant-horde recycling, which
would otherwise teleport it back near the party. Other enemies retain recycling.
It takes spell damage but rejects ASX taunts.

A killing blow reserves personal rewards for the participants still present:
one native selection of three distinct amulets, where the eligible pool permits,
and marks equivalent to an ordinary native boss of the map. The native amulet
rarity, exclusion and card builders remain responsible; existing gems apply.
Marks are credited even while the recipient is down or choosing. Amulet requests
wait until that actor can choose and has finished any current selection rather
than overwrite it. Native selection and group-pause behavior remain in charge.

Marks use the reviewed native map, Heroic and endless coefficients, snapshotted
at the kill, then rounded down to whole marks with no minimum. A result of zero
grants no marks and does not prevent the amulet selection. Each recipient receives
a positive amount once through native pending
personal progression, with no party division or gold multiplier. No boss clear,
kill counter, equipment drop or pity reward is fabricated. Escape grants nothing;
there is no goblin gold reward. Event identity is scoped to GUID and run.
Native boss encounters and Fortunes are unaffected.

The host launcher reads optional `ASX/options.json`: `{"treasure_goblins":false}`
disables local event generation at the next launch. The host controls generation
for the party; guests retain the optional skin association for host-owned events. Missing settings enable them;
invalid optional settings disable them while keeping ASX active. Optional native
contacts are verified separately; their failure does not disable required coop
features or weaken exact-build checks. No event state is saved permanently.

Announcements use the same native event-chat builder as “A pack hunts you!”,
not the red notification channel.

The selected legacy Goblin display uses native armor composition for a leather
chest, trousers and boots, with existing shipped textures. The absent baked NPC
atlas is replaced only for this exact appearance; native cache and texture
ownership remain in charge. A narrowly matched base-atlas fallback remains for
other rendering paths. Official archives stay unchanged; there is no treasure sack.

Rules: `treasure_goblins.rs`; native creation/transport/terrain schema remains in
the selected adapter. Source implementation and live validation are distinct.

## World-event placement and minimap

Supplies and Sayge's Trial sample uniformly by surface area across the native
map circle. The player position is not a sampling input and there is no nearby
fallback. Check the full footprint against map limits, native navigation cells,
height availability, terrain slope and existing ASX event footprints. Sampling
is bounded to 32 candidates per attempt. This verifies local walkability, not a
new whole-map pathfinder. Treasure Goblin appearance rules remain separate.

Flatness is a preference, never a reason to discard all otherwise safe sites.
Within each bounded sample, prefer height variation at most 0.60 yards for the
trial or 0.70 for supplies. If none qualify, use the least uneven walkable site.
On that fallback, anchor at the center's native ground height rather than lifting
the entire rune above the highest edge. This avoids a floating ritual; the flat
native model may intersect a slope. Sayge and candles sample their own ground
heights without a flatness veto. Invalid/out-of-bounds/blocked sites remain
excluded. If no walkable ritual footprint is found, retry another bounded sample
after 10 simulation seconds, until placed, rescheduled by a boss, or run end.

Retained minimap children use selected native UI center/radius and the native
north-up coordinate transform, including native zoom and interior mode. Pins
outside the visible minimap clamp to its edge at reduced opacity and say
"Beyond minimap range" on hover. Pins retire with the corresponding event.
Circular crate/orb icons use native portrait masking and the tracking ring,
following the minimap parent. The full native
map scale applies to interior points; icon margins only clamp the circular edge.
Event/buff metadata remains at 4 Hz. Active pin geometry follows the native
minimap projection per frame; its one MINIMAP_PING UI listener only rebases the
reference. No gameplay event listeners, velocity prediction or map/entity scans.

## Lost Supplies

The reserve uses the native DeadMine Cargo Boxes pile (display 36), scale 1.
It is independently optional through `lost_supplies` in either mode. First attempt at 30 simulation seconds, then every 30–50
seconds, with fixed 50% success and no pity. Reserves may coexist and each lasts
90 simulation seconds. The native pause/choice state freezes deadlines. A living
player collects within two yards. Content and power are fixed at appearance;
claims reserve slot/GUID/event identity before native IO and never reroll.

90% of rewards are consumables, with seven equally likely families. Each lasts
60 simulation seconds. Different families coexist. The same family renews to
60 seconds without adding durations or power; a lower tier cannot downgrade a
still-active stronger effect. Tiers follow completed native encounters:
0–1 = I, 2–3 = II, 4+ = III, with no further endless growth.

| Family | Tier I | Tier II | Tier III |
| --- | --- | --- | --- |
| Cooldown recovery | +6% | +10% | +14% |
| Critical strike chance | +4 points | +7 points | +10 points |
| Damage reduction | +3 points | +5 points | +7 points |
| Maximum health | +10% | +20% | +30% |
| Critical strike damage | +20% | +35% | +50% |
| Area of effect | +15% | +25% | +35% |

Classic elixirs, flasks and wizard oils provide the names/icons. Their effects
are ASX adaptations. Giant Growth retains its real name across tiers rather
than inventing nonexistent Classic variants. Native damage/speed/restoration
runes remain unchanged. Maximum-health rebuilds never grant a free heal; current
health is limited to its prior budget and the new maximum. Temporary input
loans restore after every rebuild; reset also removes derived consumable bonuses.

10% of rewards are automatically sold materials. Conditional material rarity
is 60/30/9/1% Common/Uncommon/Rare/Epic with relative values 1/2/4/8. These are ASX
discovery rarities, not claims about the original items' WoW quality. Three
variants per rarity cover herbs, cloth, ores and rare reagents.
Nominal gold is rounded from `10*(1+min(encounters,5))*200*rarityMultiplier/492`.
This normalizes the 1.64 rarity mean and compensates for faster appearance:
expected nominal gold/minute stays approximately at the previous prototype's
budget. Native personal gold modifiers still apply once; the discovery shows
actual credited gold. There are no fractional gold, crafting inventories or
permanent collection/save entries. Balance values are prototype tuning.

Discoveries queue as brief retained notifications, with no selection or pause.
Each active family gets its own cosmetic native positive-aura slot, icon,
frozen simulation countdown and tooltip showing actual values and tier. Only
owned placeholder slots are decorated; native buffs continue through their
original APIs. Cooperative personal claims/visibility remain a later pass.

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
[party UI](OFFICIAL-PARTY-UI.md) and [pause](COOP-PAUSE.md). Feature rule tests,
selected-memory fixtures and native stand-ins cover different risks. Consult
[testing](TESTING.md) rather than copying a historical live-trial recipe.

## Sayge's Trial

Enabled independently through `darkmoon_trials` in the optional event settings
(`ASX_DARKMOON_TRIALS=0` disables only this event). Boss Fortunes are replaced by this trial while enabled in solo and coop. The shared completed-encounter hook
is independent of either feature.

No trial before the first boss. A native completed encounter schedules one trial
20–40 simulation seconds later, except after the final normal encounter. Endless
continues scheduling. An unfinished trial stays at its original location until
completion or run end, with no availability deadline. Later encounters upgrade
its reward stage without resetting accumulated presence, replacing the scene or
banking extra rewards. Its event identity stays fixed. Once completed, only a
later eligible encounter schedules another trial. If an appearance is still
pending, a later encounter updates its schedule and stage. There is at most one
active trial. Placement is map-wide as defined above, with bounded scheduled
retries if no walkable location is found.

The scene contains the native purple ground rune, Classic Sayge display 491,
and three pairs of black candles placed through native terrain clipping. A
blocked candle position is omitted rather than stacked at the center. Sayge is
a presentation-only native Unit: friendly faction, non-attackable/non-selectable,
no gossip or target owner, and no row in the gameplay enemy array. He has no
simulation attack, aggro, collision actor, loot, XP or boss contribution. Native
unit serialization is shared scene presentation behind exact optional contacts;
it does not borrow the Treasure Goblin enemy/AI implementation. Partial creation
failures remove already-owned objects. Success and run teardown retire the complete scene, including its visual unit.
New encounters preserve an unfinished scene.

The circle uses the native purple ground rune at an eight-yard model radius.
Twenty cumulative seconds inside an eight-yard radius complete it; absence
suspends progress. There is no availability timer. Native choices/pauses freeze
presence accumulation. Uncompleted trials grant nothing. A retained progress bar, visible only inside the circle, and native
announcements explain the objective. Success uses the existing native
Fortune buff, modifiers and retained discovery notification, and replaces the
previous Fortune automatically. Reward reservation precedes native IO.

Prototype power ranges after encounters 1–5 are 10–18, 14–24, 18–30, 22–36 and
26–40 normalized units. Each unit is one tenth of the category base: Luck 15%,
Experience 12% and Gold 25%. Final displayed
percentages are whole numbers. The next successful trial raises normalized power
above the previous success until the cap of 40; a repeat category also advances
past rounding ties where possible. Missing trials does not hold back map stage.
Category is uniformly random; Luck does not affect the roll. Structured legacy rarity follows power rather than a separate random roll;
visual Fortune text uses green +X% values instead of rarity labels. These ranges are provisional solo tuning,
not approved final balance. At the endless cap further replacements can equal
power; growth is bounded.

## Cooperative event ownership

The host alone generates supplies and trials and fixes world positions. Native
GameObject metadata and create/destroy packets deliver scenes to participants;
Sayge uses a decorative native unit with no combat row. Native object creation
queues one packet: the adapter takes only that exact appended packet and sends
it once per participant, leaving unrelated native queue entries intact.

A supply fixes its eligible slot/GUID roster and an independent personal reward
for each member at appearance. Claiming commits before native delivery, hides the
object and pin only for that member, and preserves it for unclaimed members until
expiry or all claims. Departed identities are retired; a replacement or late
arrival cannot claim an existing reserve. Buff families are personal, use the
native rebuild scope and temporary aura rows, and expire on simulation time.
Material receipts credit the existing shared wallet once per personal claim,
using its existing strongest-contribution gold normalization without a party
size division. No selection screen or group pause is introduced.

Trials retain one shared progress counter. Every active participant, including a
downed member awaiting revival, must be alive, playing and within the circle for
progress to advance. Leaving pauses progress without resetting it; membership
changes are observed by identity. Each player's progress panel is visible only
while that player is inside. Completion commits one personal Fortune per current
participant before stat refresh; strongest XP/gold contribution rules remain.
The random power range, stage cap and guaranteed replacement apply independently
to each player's previous Fortune. Optional event failures stop further event
work for that run, without disabling core cooperative ASX features.

The host's addon payload carries personal event metadata through the existing
native channel. Cosmetic minimap projection uses the viewer's live native
GetPingPosition/GetZoom; local ping events rebase the viewer's reference. No
custom transport, client-side reward decision or simulation authority is added.

## Event presentation and cooperative validation

ASXUI stacks Sayge, Supplies and Fortune panels below the native top HUD,
centered by default. The shared anchor and each panel's offset, scale, opacity
and visibility remain configurable; layout accounts for scaled visible heights.
Sayge exposes collective readiness separately from local presence. Progress
suspends without resetting whenever a required participant is outside and resumes
when everyone returns. The local display must not interpolate progress during
that suspension.

Cooperative Supplies keep native aura projection active independently of Fortunes
and solo Supplies. Pending aura cleanup keeps that boundary active after buff
expiry, so stale native aura rows can be retired. Native stat reconstruction
continues to use each receiving actor's personal buff inputs.

ASXUI presents Fortunes with their actual +X% amount in green, including native
buff tooltips; rarity remains available as structured data, not the visible
quality label. Missing/stale snapshots or a new trial do not replay an unchanged
acquisition notification. Sayge progress uses native WoW casting-bar assets and
a separate waiting/progress line.

Automatic participant markers use the seven non-skull icons. A newly created
goblin receives the native skull through the existing native raid-target command
when that icon is free; manually assigned marks are never displaced. ASXUI
replaces the native guardian red flag with a masked native taunt icon, using
AZS_GetFlaggedLair and the same minimap projection as cache/event pins. The HUD
flag preference is restored when ASXUI pins or projection are unavailable.

Swiftness Potion is a personal 60-second Supplies effect granting +10%, +15% or
+20% movement speed. Native reconstruction resets its baseline; the bounded
post-rebuild addition must not accumulate across refreshes and disappears on
expiry. The seven aura families have unique personal slots.

Supply names use Classic item quality independently of the gameplay tier and
material sale-power bucket: all referenced potions/elixirs/flasks/oils are Common
(white); Black Lotus, Arcane Crystal and Arcanite Bar are Uncommon (green). Other
referenced materials are Common. See the reference entries for
[Swiftness Potion](https://www.wowhead.com/classic/item=2459/swiftness-potion),
[Black Lotus](https://www.wowhead.com/classic/item=13468/black-lotus),
[Arcane Crystal](https://www.wowhead.com/classic/item=12363/arcane-crystal), and
[Arcanite Bar](https://www.wowhead.com/classic/item=12360/arcanite-bar).

The catalog-review coop scenario places scenes near the group's actual native
positions. It never writes participant poses. A host claim and a guest claim on
the same object, matching native coordinates, personal speed statistics, trial
completion on both clients and continuous active API availability are separate
requirements. The legacy forced-position fixture is not proof of interactive
collectability. Its presentation delay derives from SUPPLY_SECONDS rather than
an obsolete 60-second literal.
