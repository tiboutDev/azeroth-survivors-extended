# Current ASX feature contracts

This is the stable engineering inventory for the only supported client,
**official 0.130.0**. Rows describe contracts to preserve, not universal runtime
validation. Source behavior, a built DLL and a distributed archive are different
evidence scopes. Follow [testing](TESTING.md) for evidence requirements and limits.

Do not delete or reuse an ID when a feature is disabled or absorbed by upstream;
record its current disposition. Glyph gameplay is disabled and native personal
marks replace duplicate-loot gold compensation.

| ID | Current contract / disposition | Mode | Reference |
| --- | --- | --- | --- |
| ASX-01 | Exact-build activation, required native entry/byte checks and process-local activation of the owned presentation addon. Unsupported builds fail closed. | Both | [Contract](CLIENT-ADAPTER.md) |
| ASX-02 | Shared in-run XP receipt normalization, strongest racial/fortune contribution, native choices and deferred native award bookkeeping. Preserve caller classification and rounding. | Coop | [Contract](CLIENT-ADAPTER.md) |
| ASX-03 | One host-owned gold wallet; native actor balances/coinage/earned counters are projections. Use the strongest racial, current Gold fortune and total permanent armory-Gold contribution instead of summing equipment across participants. | Coop | [Contract](FEATURE-RULES.md) |
| ASX-04 | Purchased caches debit once, share price/opened count and deliver a personal native item choice to every eligible participant. Deferred banks and repeated presentation must not create extra rewards. | Coop | [Contract](FEATURE-RULES.md) |
| ASX-05 | Native guardian rewards; no supplemental item grant is installed for 0.130. Preserve each native skill/item reward once. | Coop | [Contract](FEATURE-RULES.md) |
| ASX-06 | Complete group pause while any relevant spell/item/cache choice or manual pause remains open. Host simulation and guest prediction stop; movement is blocked after the local player has chosen too. Picking remains possible; one `picked` event cannot resume everyone. | Coop | [Contract](COOP-PAUSE.md) |
| ASX-07 | Allied direct heals, periodic heals and shields use player eligibility, range, missing-health/pressure rules and original-caster native coefficients/counters. Foreign periodic effects retain their producer and aura lifetime. | Coop | [Contract](FEATURE-RULES.md) |
| ASX-08 | Foreign Thorns/Lightning Shield retaliation, Holy Nova/Holy Shock allied heals and shield/healing talent interactions preserve caster attribution, native proc cooldowns and current modifiers. Ice Barrier remains caster-only. | Coop | [Contract](FEATURE-RULES.md) |
| ASX-09 | Hunter's Mark shares the strongest valid vulnerability and maintains native mark visuals without duplicate damage application. Owner death/leave and replacement must reconcile. | Coop | [Contract](FEATURE-RULES.md) |
| ASX-10 | Eligible Shield Slam, Hammer of Justice and Shred hits establish persistent pursuit/basic-melee ownership. Required talents, hit success, enemy life and player GUID are checked. Special abilities retain native targeting policy. | Coop | [Contract](FEATURE-RULES.md) |
| ASX-11 | Entering temporary immunity discards old taunts and ordinary aggro. Immune actors leave the ordinary NPC candidate pool; a fresh taunt remains possible during immunity. Candidate flags are restored after the native pass. | Coop | [Contract](FEATURE-RULES.md) |
| ASX-12 | Resurrection Sickness: +1 stack on resurrection, -5% outgoing damage/healing per stack, cap 3; one stack recovers per 60 alive simulation seconds. Native harmful-aura name/icon, stacks and remaining time; pauses/death suspend recovery. | Coop | [Contract](FEATURE-RULES.md) |
| ASX-13 | Sayge's Dark Fortunes: one persistent replacement buff per player, Sayge-trial replacement when ASX-28 is enabled, otherwise one roll per completed native boss encounter (not per entity), fixed category/rarity weights, bonuses rounded to whole percentage points before application and display, five-encounter scaling cap and continued endless replacements. XP/Gold use the strongest current fortune per category in coop; Luck/Pickup/Speed remain personal. Native buff tooltip and reward notification. | Both | [Contract](FEATURE-RULES.md) |
| ASX-14 | Exclude the four native utility trinkets (`remote`, `insignia`, `eldre_thalas`, `chosen`) from ASX reward candidates. Do not delete inventory or broadly remove other equipment/items that grant secondary stats. | Both | [Contract](FEATURE-RULES.md) |
| ASX-15 | Glyph gameplay intentionally disabled. Retain all nine class catalogs and shared bonus/card representations without installing glyph choices or spell mutations. | Both | [Contract](GLYPHS.md) |
| ASX-16 | Ring enchantment kind locks on the first accepted upgrade. Five total-value tiers at native ring ranks **2/4/7/10/14**, with Minor/Lesser/unprefixed/Greater/Superior names and the five native icons. Per-ring progression, additive stats, preview/apply/improve status and no rebuild accumulation. Capped native rings may remain unenchantable or low-tier (accepted V1 limitation). | Both | [Contract](FEATURE-RULES.md) |
| ASX-17 | Enrich existing native party frames with missing positive buffs, compact spell/item builds and owner-correct tooltips. Keep native health/name/death/resurrection widgets; preserve native five-player capacity. Glyph/fortune/fatigue presentation must keep source identities and expire stale cosmetic data. | Coop | [Contract](OFFICIAL-PARTY-UI.md) |
| ASX-18 | Native menu/Co-op decorations: participant class icons and class/character labels. Group waiting presentation names the relevant characters and separates local choice/menu from remote waiting. Shared enchantment/gem card blocks keep native gains, one native separator, a centered two-column header (32px icon beside stacked title/status), values below and compact retained-bonus hover. Glyph gameplay remains disabled. | Menu / Coop / reward cards | [Contract](OFFICIAL-PARTY-UI.md) |
| ASX-19 | ASX run-state isolation: reset on native run/session/clock rollback, rebind slot plus GUID, preserve authoritative native context swaps and owned pending deliveries. Solo extensions use solo identity without reading inactive coop contexts. | Both | [Contract](CLIENT-ADAPTER.md) |
| ASX-20 | Native-folder installer/launcher, exact supported-client registry, normal-DLL checks, owned addon deployment, shared native-save preservation/backups, signed update/rollback, report export and uninstall/restore. No game assets or legacy custom launcher are shipped as ASX gameplay dependencies. | Distribution | [Contract](ASX-INSTALLATION.md) |
| ASX-21 | Upstream-native personal marks. ASX duplicate-loot gold compensation is disabled; preserve native collection and reward ownership. | Both | [Contract](FEATURE-RULES.md) |
| ASX-22 | One fixed personal amulet gem per actor/run. First accepted amulet locks one of eight equally likely native-icon gems; all later amulet replacements/upgrades retain it. Stable card previews, no tiers or rarity/Luck influence, native stat input restoration and no permanent save writes. | Both | [Contract](FEATURE-RULES.md) |
| ASX-23 | Native enemy nameplates: 70% baseline size, confirmed normal-enemy names and level labels hidden; native special-rank names preserved. Missing classification retains the original native name rather than treating an unknown NPC as ordinary. Original health updates, friendly frames and visibility preferences retained. | Both | [Contract](NATIVE-WORLD-UI.md) |
| ASX-24 | Outgoing enemy damage text uses one third of the fresh native size. Native direct/periodic healing amounts are attached above allies through the existing world-text queue. Unreported actual periodic HP gains are accumulated into integer cosmetic messages at most once per second per source/recipient/spell, through the native healing-log UI queue, without replaying healing or meter/proc contributions. Preserve identities and native critical animation. | Both | [Contract](NATIVE-WORLD-UI.md) |
| ASX-25 | One-time host initialization of distinct random native participant markers from all eight icons. Every existing mark stays reserved, already marked players are skipped, and live rechecks preserve manual changes; late joins initialize once and new runs reset observation; no solo assignment. | Coop | [Contract](NATIVE-WORLD-UI.md) |
| ASX-26 | Optional host-owned Treasure Goblins in solo/coop: native creature creation and object transport, fleeing movement, 25 simulation-second expiry, no attacks/ordinary loot, taunt immunity, rare encounter-based appearances and personal native amulet/mark rewards. A separate launcher setting and optional contact checks preserve core ASX when disabled/unavailable. | Both | [Contract](FEATURE-RULES.md) |
| ASX-27 | Optional Lost Supplies: native cargo pile, map-wide placement/native-masked circular minimap icons, AS-styled fully opaque discovery notifications with instant removal, multiple 60-second reserves, six simultaneous consumable families with three encounter-based tiers, same-family renewal without stacking, twelve material variants and normalized whole-gold sales. Native temporary buff slots and stat-input restoration; host-owned cooperative scenes, identity-bound personal rolls/claims, individual hiding and shared material sales. | Solo / Coop | [Contract](FEATURE-RULES.md) |
| ASX-28 | Optional Sayge’s Trial: native encounter trigger only between bosses, native ground rune, decorative Classic Sayge and candles, map-wide placement with flatness preference and walkable fallback/minimap pin, persistent unfinished scene with stable identity and encounter-upgraded reward, cumulative presence, bounded randomized power, automatic Fortune replacement and AS-styled progress UI visible only during native circle presence. Host-owned cooperative scene and shared progress require every active participant inside; personal Fortunes replace cooperative boss grants, with strongest XP/gold contributions preserved. | Solo / Coop | [Contract](FEATURE-RULES.md) |
## Review requirements

A native compatibility change must account for ASX-01 through ASX-28 and all nine
glyph catalogs, including disabled and upstream-native behavior. Record the
native boundary, exact source/artifact identity, checks and limitations. Preserve
producer/recipient GUIDs, host/private layouts, native ownership, reset and
reentry semantics. Matching hook prologues do not establish complete parity.

Use [testing](TESTING.md) for relevant portable, Windows and Lua profiles.
Synthetic assessors and stand-ins do not establish gameplay or visible frames.
Exact-archive, normal-runtime and publication validation are deliberate
milestones, not implied by source cleanup.

See [Rust ownership](RUST-ARCHITECTURE.md) and [active rules](FEATURE-RULES.md).
Player-facing behavior and maturity are in [the feature guide](player/FEATURES.md).
