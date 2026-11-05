# Active feature rules

This summarizes the current supported source policy. The portable implementations in
`crates/azsx-gameplay/src/` are the precise reference for calculations; selected
native translation belongs in `azsx-game`. Stable feature identities live in
[AZSX-FEATURES.md](../compatibility/AZSX-FEATURES.md), and player explanations in
[player/FEATURES.md](../player/FEATURES.md).

## Shared rewards

Coop uses a host-owned wallet with native actor balances as projections.
Purchased caches debit once and deliver personal native choices. Voluntary
skip gold is divided by the current participant count, then rounded to the
nearest integer (halves up), before crediting the shared wallet once. Divide
before rounding: 7 gold / 2 becomes 4, and 2.99 / 2 becomes 1. Solo is unchanged.
Dead participants remain participants; departed identities do not. Automatic
conversions keep their existing floor rule. Strongest racial/fortune contributions and permanent armory-Gold rules
must not be replaced by summing every participant's equipment.

Dead participants continue receiving the same normalized shared XP as survivors.
Their native level and pending choices advance while they remain dead; choices
open through native flow after revival. There is no catch-up bank or replay.
Native personal modifiers can still differ; this does not force every actor to
the same absolute level or overwrite existing XP.

Never duplicate a native personal reward or retry an uncertain native commit.
Take pending work before native callbacks and preserve successful per-actor
acknowledgements. Current guardian rewards and personal collection marks are
native; supplemental guardian items and duplicate-loot gold compensation are not
installed by the selected native path.

## Jewelry

Amulets now retain their native rewards without an AZSX gem assignment or card
annotation. Crystal Resonance replaces the old immutable amulet gem.

Four native UnGoro crystal colors provide personal additive gains per pickup:
Red +12% damage, Blue +7% cooldown recovery, Green +15% maximum health,
Yellow +8% Luck. Color draws are independent and equally likely;
repeated colors are allowed. No rarity, pity rotation or ordering against other
events is applied. All collected bonuses reset at the next run, not on death.

After a 30-second opening grace period, independent random intervals average
174 simulation seconds, with at least 90 seconds between opportunities and the first at or after 2:00, targeting about five crystal opportunities by minute 15.
There is no per-run count ceiling or fixed timetable: some runs have fewer, some
have more than six. At most two models coexist; occupied slots defer the next
opportunity. Failed native placement consumes that opportunity, and short runs
can end before the average is reached. The same rate continues in endless.
Pauses do not advance native simulation time. Locations use
map-wide native terrain sampling and ordinary object separation, never enemy
kill drops. No minimap coordinates or pins are published for crystals.

Each scene snapshots currently eligible slot/GUID identities. Passing within
1.5 horizontal units and 3 vertical units while alive/playing grants once. The
first claim starts one shared 60-second simulation deadline; later claims do not
extend it. The model stays visible even for previous collectors, allowing them
to signal it. It disappears for everyone as soon as all remaining eligible
participants have collected, or the deadline expires. Departures retire their
old identity; a reused slot and late join cannot inherit its claim. Unclaimed
scenes have no countdown. Solo scenes disappear on their first claim.

One native positive presentation slot shows Crystal Resonance from the start of
the run, including with zero collected. Its native GameTooltip gives a discovery
hint at zero, then lists each collected color, count and actual accumulated
percentage on its own line. AZSXUI's badge uses the same authored text when
the native slot is unavailable.
No simulation aura, save mutation, amulet ownership or secondary transport is
created. AZSX addon API 1.3 exposes counts/bonus values independently of text.

Ring enchantment kind locks on the first accepted upgrade. Five total-value
tiers begin at native ring ranks **2/4/7/10/14**: Minor, Lesser, unprefixed,
Greater and Superior. Alacrity totals are +3/+5/+8/+11/+15% cooldown recovery input; Renewal totals are +0.5/+1/+1.5/+2/+3 health per second. These are total tier values, not extra increments. Kind and rank remain per ring. Native additive-stat inputs
are temporarily borrowed and restored; repeated rebuilds must not accumulate
bonuses. Preview/apply/improve text must reflect the accepted native rank.
Native capped rings can remain unenchantable or low-tier. Declined previews retire
when that actor accepts a validated native reward choice, before reentrant
creation of the next reward. Accepted kinds stay locked; new draws
exclude the previous declined enchantment for the same ring. Nonconsecutive repeats remain possible.

Rules: `crystals.rs`, `enchantments.rs`; scene/stat coordination uses bounded
event/reward services. Retained old gem formats are compatibility/test helpers,
not active amulet bonuses.

## Fortunes

Sayge's Dark Fortunes grant one personal replacement per completed native boss
encounter, including endless encounters. One active fortune per player persists
until replaced or the run ends. Luck, XP and Gold
use personal category weights N:1:1 for Luck:XP:Gold, where N is the eligible participant count snapshotted at reward completion (one to five). Solo remains equal. Both the enabled ritual and the automatic boss fallback use this policy. Joining/leaving alone never rerolls a retained fortune. Fallback rarity weights remain 60/25/10/4/1 and do not use player Luck.
Strength progression caps at five completed encounters; later encounters still
replace fortunes. Final percentages are rounded once and used consistently for
application, tooltip and notification.

Coop XP and Gold use the strongest current fortune in each category. Other
Luck remains personal. Rebuild native inputs before replacing bonuses.
AZSX automatically discovers Insignia of the Horde, Royal Seal of Eldre'Thalas and Mark of the Chosen in the Codex at startup. Missing entries start at Common. Existing discoveries keep their best rarity.

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
over 150% of the first native guardian’s base health for that map. The event
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
It takes spell damage but rejects AZSX taunts.

A killing blow reserves personal rewards for the participants still present:
one native selection of three distinct amulets, where the eligible pool permits,
and marks equivalent to an ordinary native boss of the map. The native amulet
rarity, exclusion and card builders remain responsible; Crystal Resonance bonuses apply independently of amulet rewards.
Marks are credited even while the recipient is down or choosing. Amulet requests
wait until that actor can choose and has finished any current selection rather
than overwrite it. Native selection and group-pause behavior remain in charge.

Marks use the reviewed native map, Heroic and endless coefficients, snapshotted
at the kill, then rounded down to whole marks with no minimum. Map progression
retains the native table bound: later zones use its last reward row rather than
reading adjacent data. A result of zero
grants no marks and does not prevent the amulet selection. Each recipient receives
a positive amount once through native pending
personal progression, with no party division or gold multiplier. No boss clear,
kill counter, equipment drop or pity reward is fabricated. Escape grants nothing;
there is no goblin gold reward. Event identity is scoped to GUID and run.
Native boss encounters and Fortunes are unaffected.

The host launcher reads optional `AZSX/options.json`: `{"treasure_goblins":false}`
disables local event generation at the next launch. The host controls generation
for the party; guests retain the optional skin association for host-owned events. Missing settings enable them;
invalid optional settings disable them while keeping AZSX active. Optional native
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
height availability, terrain slope and existing AZSX event footprints. Large
footprints also probe the interior every 1.5 yards; a clear center and rim alone
can miss blocked cells beneath a tree. Sampling
is bounded to 32 candidates per attempt. This verifies local walkability, not a
new whole-map pathfinder or a guarantee against purely visual tree canopies. Treasure Goblin appearance rules remain separate.

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

90% of rewards are consumables. Choose one of eight families uniformly and resolve the actual Classic item for the current encounter stage. All durations use simulation time. Repeated timed effects refresh the strongest active item of that family. Different families can coexist.

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

Healing potions arm one personal charge until used, death or run end. A duplicate retains the stronger charge. At or below 35% health a living, playing owner consumes the charge and gains the listed percentage of current maximum health, including when claimed below the threshold. Potion healing is independent of spells and leech.

At the production rate of 0.75 supply opportunities per minute, a player collecting every pile expects about 1.27 healing potion rewards per 15 minutes before placement losses. Nominal timed-effect coverage is about 0.53, 0.63 or 0.82 active effects per player at the three stages before duplicate refreshes and missed pickups.

Defense uses generic native stat input 5: damage reduction only, with no maximum-health contribution. This input is distinct from jewelry kind 5 (Archimtiros), whose item-specific conversion also grants health. No compensating negative-health input is applied.

Classic elixirs, flasks and wizard oils provide the names/icons. Their effects
are AZSX adaptations. Giant Growth and Swiftness each have a single strength at all stages. Native damage/speed/restoration
runes remain unchanged. Maximum-health rebuilds never grant a free heal; current
health is limited to its prior budget and the new maximum. Temporary input
loans restore after every rebuild; reset also removes derived consumable bonuses.

10% of rewards are automatically sold materials. Conditional material rarity
is 60/30/9/1% Common/Uncommon/Rare/Epic with relative values 1/2/4/8. These are AZSX
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
original APIs. Cooperative personal claims and visibility follow the identity-bound rules below.

## Support and targeting

Allied direct/periodic healing, shields and retaliation retain
the original producer's native modifiers, counters and proc/cooldown ownership.
Rejuvenation retains independent producer identities on the same recipient;
a refresh replaces only that producer's ownership record. Renew retains its
native spell-level replacement rule and AZSX active-effect targeting exclusion.
Base Regrowth in 0.138.0 is direct healing, not a direct-plus-periodic spell.
This ownership correction does not change healing coefficients, targeting,
native self/ally duration differences or death/departure policies.
Caster-only Ice Barrier remains personal. Copy source/recipient identities and
aura budgets before native context swaps; no gameplay lock spans a native call.
Do not duplicate effects already delivered by upstream. Holy Nova and Holy Shock
use native cast/impact behavior without AZSX allied-heal emission or routing.
The general Resurrection Sickness modifier remains applicable; native behavior
does not exempt these spells from that shared rule.

Hunter's Mark uses the strongest valid shared vulnerability, without stacking.
Its selected 0.138.0 projection writes the same enemy field consumed by native
damage; neighbouring native state is preserved.

Allied eligible hits can rupture an active Polymorph belonging to another
living participant. The native explosion executes once under the original
caster's context: damage, critical rolls, talents and meter attribution stay
with that caster. The triggering hit and subsequent on-hit effects remain with
the attacker. Native eligibility, area, damage limits and visual emission remain
unchanged. Expired sheep, departed/replaced owners and ambiguous ownership do
not acquire a foreign producer. Context restoration precedes other procs and
also applies to nested explosions.

Faerie Fire retains native personal marks and trigger budgets: another player's
hit does not consume a producer's reserve or trigger that producer's damage.

 Eligible Shield
Slam, Hammer of Justice and Shred hits establish persistent ordinary pursuit/
attack ownership under their talent and successful-hit checks. Shred requires
the active bear form produced by Leader of the Pack. Special targeting
and already-cast projectiles retain native ownership. Enemy/player life changes,
departure and run reset retire old associations.

Deferred melee hits retain their original damage source and custom name through delivery or fallback, even after native storage is reused.

Temporary immunity excludes ordinary aggro candidates and discards prior taunts;
a fresh eligible taunt can still establish ownership. Restore scoped native
candidate flags. Absorb shields and ordinary damage reduction are not aggro wipes.
Rules: `support.rs`, `periodic.rs`, `marks.rs`, `taunts.rs`, `immunity_aggro.rs`.

Shield ownership follows the native replacement rule. Where PW:S retains the
stronger same-spell shield, a weaker or equal refresh does not take its producer's
proc attribution; native aura expiry can still refresh. Separate absorb families
remain separate, including PW:S in a native auxiliary pool. A failed allied cast
restores the borrowed source aura and shield; changed source identities cannot
receive that rollback.

## Resurrection fatigue

Resurrection Sickness adds one stack per resurrection, caps at three, and reduces
outgoing damage/healing by 5% per stack. One stack recovers per 60 alive simulation
seconds; pause/death suspend recovery. Native harmful-aura projection shows the
name, icon, stacks and remaining time. Rules: `resurrection_fatigue.rs`.

## Presentation and validation

Keep native party/menu widgets and owner-correct tooltips. See
[party UI](OFFICIAL-PARTY-UI.md) and [pause](COOP-PAUSE.md). Feature rule tests,
selected-memory fixtures and native stand-ins cover different risks. Consult
[testing](../development/TESTING.md) rather than copying a historical live-trial recipe.

## Sayge's Trial

Enabled independently through `darkmoon_trials` in the optional event settings
(`AZSX_DARKMOON_TRIALS=0` disables only this event). Boss Fortunes are replaced by this trial while enabled in solo and coop. The shared completed-encounter hook
is independent of either feature.

No trial before the first boss. A native completed encounter schedules one trial
15–90 simulation seconds later, except after the final normal encounter. Endless
continues scheduling. An unfinished trial stays at its original location until
completion or run end, with no availability deadline. Later encounters upgrade
its reward stage without resetting accumulated presence, replacing the scene or
banking extra rewards. Its event identity stays fixed. Once completed, only a
later eligible encounter schedules another trial. If an appearance is still
pending, a later encounter updates its schedule and stage. There is at most one
active trial. Placement is map-wide as defined above, with bounded scheduled
retries if no walkable location is found.

The scene contains the native purple ground rune, Classic Sayge display 491 with a lava orb in his right hand and a glowing black book in his left,
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
Solo gives equal chances for Luck, XP and Gold. With N eligible participants, each personal draw uses weights N, 1 and 1 respectively. The Luck stat does not affect the roll. Structured legacy rarity follows power rather than a separate random roll;
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
work for that run, without disabling core cooperative AZSX features.

The host's addon payload carries personal event metadata through the existing
native channel. Cosmetic minimap projection uses the viewer's live native
GetPingPosition/GetZoom; local ping events rebase the viewer's reference. No
custom transport, client-side reward decision or simulation authority is added.

## Event presentation and cooperative validation

AZSXUI stacks Sayge, Supplies and Fortune panels below the native top HUD,
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

AZSXUI presents Fortunes with their actual +X% amount in green, including native
buff tooltips; rarity remains available as structured data, not the visible
quality label. Missing/stale snapshots or a new trial do not replay an unchanged
acquisition notification. Sayge progress uses native WoW casting-bar assets and
a separate waiting/progress line.

Automatic participant markers keep the seven non-skull icons; skull is left for
manual use. Goblins receive no automatic raid marker and are identified by their
AZSXUI encounter health bar. Native/manual marks are not cleared or reassigned.
AZSXUI replaces the native guardian red flag using all valid `AZS_GetLairs()`
entries and the same projection as cache/event pins. Explicit pin layers put
Sayge above active guardians, Supplies, caches and returning guardians. The
native HUD flag preference is restored when pins/projection are unavailable.

Swiftness Potion grants +20% personal movement speed for 15 seconds. Native reconstruction resets its baseline; the bounded
post-rebuild addition must not accumulate across refreshes and disappears on
expiry. The eight aura families have unique personal slots.

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

### Native spending receipts

The selected native client inlines coin-map writes for cache purchases and paid
rerolls. AZSX observes their completed native handlers in addition to the generic
currency setter. It commits only native decreases to the shared wallet and leaves
earned-gold totals unchanged. Seeing the same balance through both paths is
idempotent. A concurrent/nested wallet change cannot be overwritten by a stale
observation. Free and rejected rerolls retain native costs and availability.

Native queued whole-level rewards are also consumed while dead. Their native
threshold-deficit calculation runs at receipt time; only the resulting choices
wait for revival. The three native consumers retain separate Playing-only choice
gates. These level-fill awards never enter the shared-XP multiplier/receipt path.

## Cooperative area bonuses

In coop, positive bonuses to the global spell area/range stat retain **50%**
of their nominal value: +40% becomes +20%, and +100% becomes +50%.
This covers all contributions to that global stat. Base spell radii, separate
spell-specific radius mechanics, reductions and solo play remain unchanged.
The effective stats panel shows the adjusted total; individual item descriptions
continue to describe nominal bonuses. This rule does not change enemy scaling.

### Dry world-event placement

AZSX-owned crystal, supplies, ritual, demon, escort/debris and goblin spawn
sites reject finite native liquid-grid samples in addition to navigation and
height checks. This deliberately also rejects mapped liquid beneath elevated
ground; no scene is relocated after an uncertain native creation.
Opening escort, demon and Sayge announcements describe the challenge without
revealing its reward; success receipts retain the actual reward text.


Cooperative talent conditions distinguish Lightning Shield retaliation from
absorption: Stormguard requires Lightning Shield; Fortified, Ardent Defender
and Sacred Duty require absorption. Shatter and Dirty Deeds can use another
player's control while retaining the attacker's own talent values.

Swiftmend may rescue an eligible ally below 40% health within support range;
its native producer cooldown is retained. Tree of Life may benefit allies,
with healing coefficients and overheal limits based on the producer.
Divine Aegis follows the producer's Power Word: Shield, including allied
recipients. Hammer of the Righteous requires a self-produced, self-worn
absorption. Rapture restores the original PW:S recipient and retains the
producer's native cooldown. Expiration and replacement do not fabricate a break.
Tranquility remains centred on an allied healed recipient, credited to the
producer. Its radius and reviewed holy shield bursts apply the producer's
effective area factor once; unrelated native queries remain unchanged.

Ancestral Fortitude's free Healing Wave uses normal cooperative target selection at half effectiveness and can heal a wounded ally.

Cooperative eight-piece armor effects use the wearer's equipment. An allied killing blow can spread the wearer's Serpent Sting or Curse of Agony, or trigger the wearer's Rupture or Flame Shock explosion. A reward that names the killing spell still requires the wearer's own spell kill. Devout and Divine Aegis each burst at the protected player when the priest's Power Word: Shield breaks. Devout uses an eight-yard base radius and the priest's area multiplier at both set tiers. PvP Mortal Strike and Earth Shock vulnerabilities apply to allied hits at the strongest active value. Damage bonuses against controlled enemies apply to the wearer when any player supplies the control.

A legendary necklace can trigger its explosion when a teammate finishes an enemy carrying your matching damage-over-time spell. The Power Word: Shield necklace bursts around the player you protected using your area bonus. Holy Light and Regrowth necklaces shield the player you heal for up to 40% of your maximum health. Vampiric Renew and Vampiric Rejuvenation grant the healed player 5% leech on their own damage for 10 and 12 seconds respectively.

In the Armory and during solo or cooperative runs, the Talents panel adds a short Co-op note below selected talent descriptions to explain ally targeting, shield ownership and control effects. Fortified and Ardent Defender combine their taunt trigger and allied shield condition in one note. Leader of the Pack identifies Shred as its bear-form taunt.

## Set weapon appearance

In the Armory, **Worn piece (spec)** shows the specialization variant of the equipped set weapon on the Armory character model, in the main-menu portrait and in a run. **This piece (spec)** uses the selected unlocked piece for that hand and follows specialization changes. Fury uses two one-handed weapons. Protection Warriors and Holy or Protection Paladins use a one-handed weapon and a shield. Enhancement Shamans use a two-handed weapon, including Sulfuras with Earthshatter. The tree with the most learned points determines the look. Ties, empty builds and other specializations use the native appearance. Manual Look choices apply to their hand slots. Melee strike animations follow the visible weapon grip. Gear bonuses come from the equipped pieces.
