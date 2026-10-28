# Current ASX feature contracts

This is the stable engineering inventory for the only supported client,
**official 0.125.0**. Rows describe contracts to preserve, not universal runtime
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
| ASX-05 | Native guardian rewards; no supplemental item grant is installed for 0.125. Preserve each native skill/item reward once. | Coop | [Contract](FEATURE-RULES.md) |
| ASX-06 | Complete group pause while any relevant spell/item/cache choice or manual pause remains open. Host simulation and guest prediction stop; movement is blocked after the local player has chosen too. Picking remains possible; one `picked` event cannot resume everyone. | Coop | [Contract](COOP-PAUSE.md) |
| ASX-07 | Allied direct heals, periodic heals and shields use player eligibility, range, missing-health/pressure rules and original-caster native coefficients/counters. Foreign periodic effects retain their producer and aura lifetime. | Coop | [Contract](FEATURE-RULES.md) |
| ASX-08 | Foreign Thorns/Lightning Shield retaliation, Holy Nova/Holy Shock allied heals and shield/healing talent interactions preserve caster attribution, native proc cooldowns and current modifiers. Ice Barrier remains caster-only. | Coop | [Contract](FEATURE-RULES.md) |
| ASX-09 | Hunter's Mark shares the strongest valid vulnerability and maintains native mark visuals without duplicate damage application. Owner death/leave and replacement must reconcile. | Coop | [Contract](FEATURE-RULES.md) |
| ASX-10 | Eligible Shield Slam, Hammer of Justice and Shred hits establish persistent pursuit/basic-melee ownership. Required talents, hit success, enemy life and player GUID are checked. Special abilities retain native targeting policy. | Coop | [Contract](FEATURE-RULES.md) |
| ASX-11 | Entering temporary immunity discards old taunts and ordinary aggro. Immune actors leave the ordinary NPC candidate pool; a fresh taunt remains possible during immunity. Candidate flags are restored after the native pass. | Coop | [Contract](FEATURE-RULES.md) |
| ASX-12 | Resurrection Sickness: +1 stack on resurrection, -5% outgoing damage/healing per stack, cap 3; one stack recovers per 60 alive simulation seconds. Native harmful-aura name/icon, stacks and remaining time; pauses/death suspend recovery. | Coop | [Contract](FEATURE-RULES.md) |
| ASX-13 | Sayge's Dark Fortunes: one persistent replacement buff per player, one roll per completed native boss encounter (not per entity), fixed category/rarity weights, bonuses rounded to whole percentage points before application and display, five-encounter scaling cap and continued endless replacements. XP/Gold use the strongest current fortune per category in coop; Luck/Pickup/Speed remain personal. Native buff tooltip and reward notification. | Both | [Contract](FEATURE-RULES.md) |
| ASX-14 | Exclude the four native utility trinkets (`remote`, `insignia`, `eldre_thalas`, `chosen`) from ASX reward candidates. Do not delete inventory or broadly remove other equipment/items that grant secondary stats. | Both | [Contract](FEATURE-RULES.md) |
| ASX-15 | Glyph gameplay intentionally disabled. Retain all nine class catalogs and shared bonus/card representations without installing glyph choices or spell mutations. | Both | [Contract](GLYPHS.md) |
| ASX-16 | Ring enchantment kind locks on the first accepted upgrade. Five total-value tiers at native ring ranks **2/4/7/10/14**, with Minor/Lesser/unprefixed/Greater/Superior names and the five native icons. Per-ring progression, additive stats, preview/apply/improve status and no rebuild accumulation. Capped native rings may remain unenchantable or low-tier (accepted V1 limitation). | Both | [Contract](FEATURE-RULES.md) |
| ASX-17 | Enrich existing native party frames with missing positive buffs, compact spell/item builds and owner-correct tooltips. Keep native health/name/death/resurrection widgets; preserve native five-player capacity. Glyph/fortune/fatigue presentation must keep source identities and expire stale cosmetic data. | Coop | [Contract](OFFICIAL-PARTY-UI.md) |
| ASX-18 | Native menu/Co-op decorations: participant class icons and class/character labels. Group waiting presentation names the relevant characters and separates local choice/menu from remote waiting. Glyph/enchantment card blocks keep native gains, centered icon/title, separate status and red drawback values. | Menu / Coop / reward cards | [Contract](OFFICIAL-PARTY-UI.md) |
| ASX-19 | ASX run-state isolation: reset on native run/session/clock rollback, rebind slot plus GUID, preserve authoritative native context swaps and owned pending deliveries. Solo extensions use solo identity without reading inactive coop contexts. | Both | [Contract](CLIENT-ADAPTER.md) |
| ASX-20 | Native-folder installer/launcher, exact supported-client registry, normal-DLL checks, owned addon deployment, shared native-save preservation/backups, signed update/rollback, report export and uninstall/restore. No game assets or legacy custom launcher are shipped as ASX gameplay dependencies. | Distribution | [Contract](ASX-INSTALLATION.md) |
| ASX-21 | Upstream-native personal marks. ASX duplicate-loot gold compensation is disabled; preserve native collection and reward ownership. | Both | [Contract](FEATURE-RULES.md) |
| ASX-22 | One fixed personal amulet gem per actor/run. First accepted amulet locks one of eight equally likely native-icon gems; all later amulet replacements/upgrades retain it. Stable card previews, no tiers or rarity/Luck influence, native stat input restoration and no permanent save writes. | Both | [Contract](FEATURE-RULES.md) |

## Review requirements

A native compatibility change must account for ASX-01 through ASX-22 and all nine
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
