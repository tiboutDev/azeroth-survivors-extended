# AZSX expected behavior and compatibility acceptance

Use the [compatibility checklist](COMPATIBILITY-CHECKLIST.md) for execution order; this catalog owns the expected behavior criteria.

Maintained acceptance reference for **current main source**, reviewed against
official **0.138.0**. Covers AZSX-01–AZSX-34, including disabled and upstream-owned behavior.
This is not proof that an installed DLL or published archive passes these requirements.
Uncommitted source fixes are not automatically in a prepared release.
The turret experiment remains outside this baseline.

## Maintenance and use

- Historical `ASX-NN.xx` IDs map one-to-one to `AZSX-NN.xx`; old evidence remains artifact-scoped. Each `AZSX-NN.xx` is a stable acceptance ID. Never recycle it; add criteria for
  new features/regressions and explicitly record retirement/upstream ownership.
- Update affected criteria in the **same change** as implementation/tests,
  `compat/features.json`, detailed rules/API and player documentation. Regenerate
  the existing AZSX-FEATURES.md index; do not edit only its generated table.
- Every compatibility review must account for every section. Map impacted rows
  to native producers, consumers, fields, callbacks and UI. Matching entry bytes
  does not prove behavior. Review grant/spend, host/guest, alive/dead and resets.
- Linked detailed rules own full coefficients/catalogs; ADDON-API.md owns all
  field types/units/version semantics. Candidate profiles below are starting
  points, not coverage claims. Resolve code/contract disagreements explicitly;
  do not redefine a regression as intended behavior to make the checklist pass.
- Follow TESTING.md: full portable gate at compatibility/release milestones,
  relevant explicit Lua 5.1, exact-image, Windows ABI and bounded live checks.
  Review all rows without blindly running one scene per row. One scenario may
  cover multiple IDs if each outcome has evidence.
- Solo precedes coop on the same artifact. At most two owned isolated clients,
  normal priority/all CPUs. Native five-player capacity is an invariant, not a
  claim of complete five-player coverage from a two-player trial.
- Record failures and unobserved cases. Optional unavailability is a limitation,
  not a pass. Disabled features require negative checks. Logs/stand-ins/assessor
  tests cannot establish visual appearance; use authorized visual review for it.

## Per-patch evidence record

Keep results in ignored local evidence or the review deliverable, not permanently
checked boxes in this contract. Start with:

| Criterion IDs | Trigger / expected outcome | Verdict | Evidence / scope | Limit / next action |
| --- | --- | --- | --- | --- |
| AZSX-02.02 | Dead host and guest gain XP | NOT RUN | No current-artifact observation | Both roles required |

Verdicts: PASS, FAIL, NOT RUN, BLOCKED, NOT APPLICABLE (with explicit reason).
Record executable SHA-256, source revision **and dirty-input fingerprint**, DLL
hash/kind, UI/tool fingerprints, mode/roles/builds, scenario, commands/duration and
receipt/log/capture paths. Prior-client passes never transfer automatically.
Report source, normal-runtime and signed-archive coverage separately. Diagnostic
evidence cannot certify normal conditionals or a release archive. Unresolved
required outcomes prevent a claim of full behavioral parity.

## Recurring regression priorities

Explicitly revisit guest Escape/overlapping choices, talents without pause, cache
and paid reroll debits from both roles, skip rounding, dead ordinary XP **and
whole-level** rewards through both revivals, shared-stat HUD, excluded trinkets,
Fortune deduplication, second-player Supplies claim/effect, Sayge leaving/dying,
smooth foreground player arrow and multiple guardian pins. These priorities do
not replace the other sections.

## Acceptance catalog

### AZSX-01 — Activation

Mode: **Both**. Disposition: **active**. Details: [Contract](CLIENT-ADAPTER.md).

Candidate profiles: `official`, `tooling`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-01.01 | Supported AZSX launch | Exact executable and required native bytes/layouts must pass before activation; a version label alone is insufficient. |
| AZSX-01.02 | Direct native launch or unsupported executable | Direct native launch stays inert; unsupported AZSX builds fail closed, with no retired engine fallback. |
| AZSX-01.03 | Addon activation | The selected native virtual reader appends the provider only to WorldStateFrame.lua; no internal addon is installed or required, and native assets/unrelated reads remain unchanged. |
| AZSX-01.04 | Optional feature failure | Stop the affected optional event safely without weakening required activation or disabling unrelated core coop. |
| AZSX-01.05 | Installed layout/migration | App/Config/Data separation; byte-verified restartable migration preserves settings, transaction history, backups, logs and unrelated files. Native addon locations remain unchanged. |
| AZSX-01.06 | Compatible signed update | Install automatically without a question, including noninteractive use; exact native match/signature/ownership/anti-loop gates remain. Runtime changes use a detached verified transaction and automatic restart. |
| AZSX-01.07 | Launch versus full check | Launch checks only activation dependencies; missing docs or optional AZSXUI do not block native-interface play. Full --check stays offline/read-only. Invalid maintenance metadata cannot force unsafe writes. |

### AZSX-02 — Shared XP and death

Mode: **Coop**. Disposition: **active**. Details: [Contract](CLIENT-ADAPTER.md).

Candidate profiles: `official`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-02.01 | Ordinary shared XP | Normalize the unscaled native receipt once. Non-racial native contributions combine under the portable policy; strongest racial and strongest XP Fortune contribute in their respective channels. Preserve rounding/minimum; never rescale an already rounded receipt. |
| AZSX-02.02 | Host death, then guest death | Both continue gaining native XP and levels while dead, with dead phase/HP preserved. Observe actual XP and level on both clients. |
| AZSX-02.03 | Guardian whole-level reward while dead | Retain and consume the native receipt at receipt time using the threshold deficit; this award does not enter ordinary XP multiplier normalization. |
| AZSX-02.04 | Revive after XP and whole-level awards | Choices remain pending until native revival flow; no catch-up bank, lost receipt or duplicate award. Compare exact XP as well as levels before/after both revivals with identical profiles. |
| AZSX-02.05 | Different builds/starting progression | Do not force equal absolute XP/levels or overwrite progression to satisfy a synchronization check. |

### AZSX-03 — Shared wallet

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `official`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-03.01 | Collect gold from either role | Credit one host-owned wallet once; native balance/coinage projections converge. Keep collector-specific effects, strongest racial/Gold Fortune and best participant permanent armory contribution; do not sum party equipment. |
| AZSX-03.02 | Voluntary skip | Divide the scaled amount by current participant count before nearest-integer rounding, halves up: 7/2 = 4, 2.99/2 = 1. Dead members count, departed members do not; solo unchanged; small results may be zero. |
| AZSX-03.03 | Automatic conversion/world gold/material sale | Keep conversion floor policy distinct; world gold and Supplies sales do not acquire skip division. |
| AZSX-03.04 | Cache purchase and paid spell/item reroll on either role | Completed native debit reduces the common balance once without reducing earned-gold totals. Inline/generic observations cannot double-debit or overwrite nested wallet changes. |
| AZSX-03.05 | Free/rejected reroll | Native eligibility/cost remain authoritative; no phantom debit or stale common balance preventing an affordable action. |

### AZSX-04 — Purchased caches

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `official`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-04.01 | Open as host and guest | One debit, shared next price/opened count, one personal native item choice per eligible participant. |
| AZSX-04.02 | Overlap choices or repeated delivery/presentation | Keep successful recipient acknowledgements; no overwritten current choice, lost pending delivery or extra reward. |
| AZSX-04.03 | Finish at different times | Remaining choices keep group pause while selection stays usable. |

### AZSX-05 — Native guardian rewards

Mode: **Coop**. Disposition: **upstream**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `official`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-05.01 | Kill with two participants | Each native skill/item reward arrives once per eligible actor; no supplemental AZSX item grant. |
| AZSX-05.02 | Kill while dead or choosing | Preserve native pending choices and whole-level receipts; count completed deliveries, not just cards created. |

### AZSX-06 — Pause, Escape and inspection

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/COOP-PAUSE.md).

Candidate profiles: `official`, `official-party-ui`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-06.01 | Overlapping choices, one player finishes | Host simulation, local movement and guest prediction remain stopped until all relevant choices/menus close; selection and networking continue. |
| AZSX-06.02 | Actual guest Escape during play and waiting | Native personal menu opens before the guest filter consumes Pause; Resume/Escape closes only that participant ownership. |
| AZSX-06.03 | Close one of several menus/choices | Remaining ownership holds pause; native Options/Back/Resume work; waiting text names the relevant characters. |
| AZSX-06.04 | Pause while others move | Remote visual movement stops on the guest, without rewriting authoritative coordinates; native movement resumes afterward. |
| AZSX-06.05 | Host talents/read-only inspection in coop | Opening/closing alone does not pause gameplay. Actual Escape and actions within an owned menu keep native behavior. |
| AZSX-06.06 | Leave/reuse slot/new run | Retire old menu identity/epoch; no stranded pause. Solo/lobby forwarding stays native. |

### AZSX-07 — Allied healing and shields

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `support`, `periodic`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-07.01 | Direct heal/shield | Respect eligibility, range, missing health/pressure and native limits; do not duplicate an upstream delivery. |
| AZSX-07.02 | Foreign periodic effect | Retain original caster coefficients, talents, counters, aura lifetime and remaining budget across recipient context changes. Concurrent Rejuvenation producers keep separate ownership records; refreshing one must not evict another. Renew keeps its spell-level replacement rule. |
| AZSX-07.03 | Source/recipient dies, departs or aura replaced | Retire appropriate identities/effects; no ticks reach reused slots or another run. |
| AZSX-07.04 | Swiftmend / Tree of Life on allies | Swiftmend may select an eligible ally below 40% HP within support range; native producer cooldown remains 8 seconds. Tree of Life may heal allies; its overheal cap uses the producer maximum HP. |
| AZSX-07.05 | Ancestral Fortitude | The native Stormstrike counter casts one free half-power Healing Wave using ordinary support targeting. |
| AZSX-07.06 | Inspect an annotated talent during a cooperative run | Append the English Co-op note below the native current-rank text on host and guest. Fortified and Ardent Defender combine taunt and absorb-shield information under one Co-op label. Dire Bear Form identifies Shred as its taunt. Solo and armory display the original descriptions. |

### AZSX-08 — Hybrid support and retaliation

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `support`, `periodic`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-08.01 | Foreign Thorns/Lightning Shield | Retain producer, native modifiers and proc cooldown/budget; recipient outgoing talents cannot steal attribution. |
| AZSX-08.02 | Holy Nova/Holy Shock and healing/shield talents | Holy Nova and Holy Shock retain native casts, impacts and self-healing, without AZSX group heals or target routing. Reject retired synthetic hybrid deliveries. General Resurrection Sickness remains applicable. Other supported healing/shield talent branches preserve caster ownership and native cooldown semantics. |
| AZSX-08.03 | Ice Barrier | Remains caster-only, never a group shield. |
| AZSX-08.04 | Absorption breaks | Divine Aegis applies only to the producer's Power Word: Shield, including allies. Hammer of the Righteous applies only to self-produced, self-worn shields. Rapture restores the same PW:S recipient with its producer's native cooldown. Expiry/replacement is not a fabricated break. |
| AZSX-08.05 | Conditional buffs and talent area | Lightning Shield is retaliation, never absorption; Stormguard requires that specific active effect. Shield-state talents require absorption. Tranquility and reviewed holy shield bursts use the producer's effective area factor exactly once. Allied Tranquility remains centred on the recipient. |

### AZSX-09 — Hunter's Mark

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `combat`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-09.01 | Multiple marks | Strongest valid shared vulnerability applies once, with native visuals and no multiplied duplicate damage. Weaker hunters cannot overwrite it; the selected native damage field is used without changing neighbouring state. |
| AZSX-09.02 | Replacement/death/departure/reset | Reconcile effective mark and clear invalid owner associations. |

### AZSX-10 — Talent taunts

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `combat`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-10.01 | Eligible successful hits | Shield Slam/Fortified, Hammer of Justice/Ardent Defender and Shred/Dire Bear Form establish persistent ordinary pursuit/basic-attack ownership. |
| AZSX-10.02 | Miss/no required talent/dead enemy/goblin | No taunt assignment. |
| AZSX-10.03 | Competing tanks/death/departure | New valid taunt replaces old; invalid ownership clears. Special casts/projectiles and NPC ownership remain native. |

### AZSX-11 — Immunity and aggro

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `combat`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-11.01 | Enter temporary invincibility | Discard prior ordinary aggro/taunts; exclude actor from ordinary candidates. |
| AZSX-11.02 | Fresh taunt while immune/exit immunity | Fresh eligible taunt can work; scoped candidate flags restore after native pass. |
| AZSX-11.03 | Absorb or damage reduction | Not an invincibility aggro wipe. |

### AZSX-12 — Resurrection Sickness

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `fatigue`, `support`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-12.01 | Repeated resurrection | One stack each time, cap three; -5% outgoing damage/healing each, maximum -15%. |
| AZSX-12.02 | Recovery | One stack per 60 alive simulation seconds; resurrection restarts interval, death/pause/choices suspend it. |
| AZSX-12.03 | Display and incoming healing | Native harmful aura shows stacks/time; no max-HP or incoming foreign-heal penalty. |

### AZSX-13 — Sayge's Dark Fortunes

Mode: **Both**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `rules`, `official-party-ui`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-13.01 | Trial success or trials-disabled boss encounter | One personal replacement Fortune; fallback rolls once per completed encounter, not per entity. Survives death, clears next run. |
| AZSX-13.02 | Category/power rolls | Luck/XP/Gold personal weights N:1:1 for the eligible completion-time participant count (1–5), in rituals and boss fallback; solo equal. Membership changes alone never reroll retained fortunes. No speed/pickup. Luck does not bias rolls. Whole percentages once for application/display; bounded power and five-encounter scaling cap, endless replacements continue. Detailed coefficients remain in linked rules. |
| AZSX-13.03 | Cooperative contributions | Strongest current XP and Gold Fortunes selected independently; Luck personal. Replacement/departure refreshes effective values. |
| AZSX-13.04 | Tooltip/notification | Green +X%, no visible rarity, native-style tooltip distinguishes own versus stronger party contribution. |
| AZSX-13.05 | Heartbeat/pause/membership/new ritual | Unchanged Fortune never replays acquisition; genuine new award with identical values still notifies once. |

### AZSX-14 — Reward exclusions

Mode: **Both**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `rules`, `official`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-14.01 | Solo/coop candidate pools | insignia, eldre_thalas and chosen remain excluded; their return is a regression. remote/Gnomish Universal Remote stays available. |
| AZSX-14.02 | Existing inventory/unrelated gear | No deletion of existing items or broad exclusion of other equipment. |

### AZSX-15 — Reserved removed feature

No runtime implementation or acceptance scenario remains. The ID and its former
criteria are retired permanently; never assign them to another feature. Historical
meaning remains in Git history, not in current gameplay or player documentation.

### AZSX-16 — Ring enchantments

Mode: **Both**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `enchantments`, `official-party-ui`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-16.01 | Decline preview | Retire preview at validated reward boundary before reentrant creation of next reward. The next preview for the same ring excludes its last declined kind; redraws of the current choice stay stable. |
| AZSX-16.02 | Accept first eligible upgrade | Kind locks per ring; other ring identity/progression remains independent. Alacrity totals 3/5/8/11/15% recovery; Renewal 0.5/1/1.5/2/3 HP/s. |
| AZSX-16.03 | Ranks 2/4/7/10/14 | Minor/Lesser/unprefixed/Greater/Superior total-value tiers; native capped-ring limitations retained. |
| AZSX-16.04 | Rebuild/replace/reset | Additive bonuses never accumulate through repeated temporary-input borrowing or leak across identity/run. |
| AZSX-16.05 | Cards and owner tooltips | Actual rank/kind/values; preview/apply/improve versus compact Active retained-bonus text, native gains preserved. |

### AZSX-17 — Party builds and addon contract

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/OFFICIAL-PARTY-UI.md).

Candidate profiles: `official-party-ui`, `support`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-17.01 | Different allied builds | Enrich native children, retain health/name/death/resurrection and native five-player capacity. Never format remote tooltips with viewer stats. |
| AZSX-17.02 | Long pause/departure/reused identity | Heartbeat preserves valid unchanged cosmetics; stale/ambiguous/missing/foreign data hides. |
| AZSX-17.03 | API without AZSXUI | Native-style AZSX_Get functions expose detached local structured values separately from AZSX text; AZSXUI consumes, not provides. No new transport/gameplay authority; private remote builds are not public v1 data. |
| AZSX-17.04 | Mutate snapshot/unsupported major/unavailable state | Provider/other consumers unaffected; unsupported major rejected; stale/outside-run/missing payload explicitly unavailable. |
| AZSX-17.05 | API 1.8 sharedEconomy | XP/gold independently optional actual award multipliers, replacing rather than adding to personal values. Missing is not zero; legacy economy keeps complete-pair guarantee. |
| AZSX-17.06 | Payload/lifecycle | Active namespace azsx, no recipient-build retransmission or unused radius field; heartbeat retained. Exact string identities and every field/unit/version rule follow ADDON-API.md. |

### AZSX-18 — Retained native presentation

Mode: **Both**. Disposition: **active**. Details: [Contract](../features/OFFICIAL-PARTY-UI.md).

Candidate profiles: `official-party-ui`, `official`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-18.01 | Lobby/party | Native lobby untouched, no retired class-icon decoration, replacement party root or custom DPS ranking. |
| AZSX-18.02 | Waiting menus/rewards | Names identify relevant characters; separate local choice/menu from remote waiting. |
| AZSX-18.03 | Reward cards | Native gains, one separator, icon/title/status header and actual values/retained-bonus hover; retained gem format is not active gameplay. |
| AZSX-18.04 | Main menu title | The native localized title renders as “Azeroth Survivors Extended” on the selected AZSX client. Other native labels and menu actions stay unchanged. |

### AZSX-19 — Run/identity isolation

Mode: **Both**. Disposition: **active**. Details: [Contract](CLIENT-ADAPTER.md).

Candidate profiles: `official`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-19.01 | New run/session/clock rollback | Reset owned state; no old bonuses, menus, rewards, events or cosmetic receipts leak. |
| AZSX-19.02 | Slot reused | Bind slot+GUID and generation; new actor cannot inherit claims, pending rewards or effect attribution. |
| AZSX-19.03 | Reentrant native calls | Restore actor/host context, preserve ordering and successful recipient acknowledgements; no held mutable guard or retry of uncertain grant. |
| AZSX-19.04 | Solo after coop | Use solo identity without inactive coop reads; native simulation/transport/recovery remains authoritative. |

### AZSX-20 — Installation and maintenance

Mode: **Distribution**. Disposition: **active**. Details: [Contract](../release/AZSX-INSTALLATION.md).

Candidate profiles: `azsx-install`, `tooling`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-20.01 | Install on supported closed client | Owned AZSX/start/docs only, no native EXE/archives. Automatically deploy AZSXUI to benilla-config/AddOns/AZSXUI; native script initialization supplies the AZSX API without an internal addon. |
| AZSX-20.02 | Startup/update | No-argument auto-launch; explicit --menu for maintenance. Compatible signed AZSX updates install automatically; repair or replacement of foreign files still requires consent. |
| AZSX-20.03 | Changed/missing README-AZSX.txt or AZSX/App/Launcher/LICENSE | Noncritical docs do not block otherwise verified launch; preserve modified docs, restore missing ones through repair, back up replacements on update. |
| AZSX-20.04 | Critical files/metadata/unsafe paths | Roles are product-owned, never inferred by extension. Verify critical sources/projections and client; unsafe paths always block. Invalid ownership blocks maintenance writes, not separately verified intact launch. |
| AZSX-20.05 | --check/report | Offline read-only check collects issues without repair/preparation; diagnostic export works with invalid manifest metadata. |
| AZSX-20.06 | Repair | Reauthenticate signed same-version cache/channel/local triple, show plan and obtain consent; preserve unknown files/settings/saves. No implicit adoption of unknown loader or addon. |
| AZSX-20.07 | Update/interruption/retry/rollback | Bounded signed inventories/hashes/journals/backups; retain sequence high-water mark and rollback pointer. No reinstall loop; failed attempts need explicit retry. |
| AZSX-20.08 | Code/runtime replacement/uninstall | Restart before using replaced launcher code; runtime uses the detached verified transaction. Remove only owned matching files, preserve progression/unrelated addons. |
| AZSX-20.09 | Official native update | Read the bounded official announcement; enable the official updater only when a signed AZSX release matches the announced version and exact executable SHA-256. Restart the launcher after the native executable changes. Offline/unavailable/unsupported announcements do not enable a native update. |
| AZSX-20.09 | Release | Normal artifact, approved reproducibility reference and exact signed archive are separate gates. No implicit commit/push/publication/installation. |

### AZSX-21 — Native marks and duplicate loot

Mode: **Both**. Disposition: **upstream**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `official`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-21.01 | Already-collected loot | Native personal marks/collection remain authoritative; no AZSX duplicate-loot gold or extra grant. |
| AZSX-21.02 | Contiguous mark gains | Sum contiguous earned values in one popup using a three-second quiet window refreshed by each receipt and the latest native total; duplicate item popup silent. Notification itself awards nothing. |

### AZSX-22 — Crystal Resonance

Mode: **Both**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `crystals`, `official-party-ui`, `azsxui`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-22.01 | Generation | Dry ground-checked independent map-wide native models, equal free colors, no minimap pins/kill drops. After 30 seconds, independent random intervals average 174 simulation seconds, targeting about five opportunities by minute 15 without a count ceiling; max two simultaneous scenes, with deferred opportunities and no backlog replay. The same rate continues in endless; terrain/early end can reduce discoveries. |
| AZSX-22.02 | Color pickup | Red +12% damage, Blue +7% cooldown recovery, Green +15% max HP, Yellow +2 percentage points of critical strike chance; personal additive permanent run gains. Native amulet effects remain, no AZSX amulet gem. |
| AZSX-22.03 | First/group claims | Alive proximity grants once per appearance-time identity; model stays visible even to collector. First claim starts shared 60 simulation seconds, never extended by later claims. |
| AZSX-22.04 | All claims/expiry/solo/departure | Common removal on all remaining claims or timeout, solo immediate; unclaimed has no timer. Pause freezes time, late/reused identities cannot claim. |
| AZSX-22.05 | Death/revival/UI restore | Bonuses survive death, reset next run. One permanent effect visible at zero with discovery guidance, then color counts and per-stat totals; only new pickups notify, no replay of restored totals. |

### AZSX-23 — AZSXUI HUD and minimap

Mode: **Both**. Disposition: **active**. Details: [Contract](../features/NATIVE-WORLD-UI.md).

Candidate profiles: `azsxui`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-23.01 | Enable/disable | Existing native artwork, reversible owned anchors/levels/scripts/visibility; UI settings never change gameplay. |
| AZSX-23.02 | Actions/XP/equipment | One stats extension; native XP art/top border intact. Order amulet, ring, ring, trinket, trinket, native empty silhouettes. |
| AZSX-23.03 | Stats input | Hover previews; button/character binding toggle AZSX stats and native character sheet together; checked indicator follows state, no animation. |
| AZSX-23.04 | Stats layout | 29 supported native stats, nonzero rows, four columns/three vertical separators; short labels/full-term help. Width XP+32 units, no bottom border, above items below XP art/foreground gryphons. |
| AZSX-23.05 | Shared stat changes | Refresh visible rows with actual effective coop XP/gold multipliers; missing shared value hides instead of falling back to personal stats. |
| AZSX-23.06 | HP/auras/tooltips | Raised independent compact native-textured capsule at 208 UI units, clear of a five-row stats panel, current HP and level only; aura rows above HP with durations above icons. Player/target roots hide reversibly; item tooltips show each native comma-separated effect on its own line and retain actual owner text and verified enchantments. |
| AZSX-23.07 | Permanent effects/channel bars | The experience bar centers the run level and native XP progress together, for example `Level 5 · 21 / 27`. Fortune/Crystal medallions beside HP, total crystal count and Fortune category overlay. Icon-free Sayge/native resurrection style; remaining Sayge seconds inside track. |
| AZSX-23.08 | Top strip | Kills/gold/cache icons/values vertically centered in ornamented 300-unit strip inset eight units; full border, clock below, soft center light behind affordable gold rather than a ring. |
| AZSX-23.09 | Encounter bars | Boss/awake guardian/goblin bars below top clock, centered names without percentages; multiple supported rows visible; native per-entity debuffs use TargetDebuffButtonTemplate, its count child, native target sizes and dispel colors below each row. Native target tooltips require an exact selected-entity match and a unique native aura icon; otherwise they stay hidden. Up to 16 icons and a bounded overflow count. |
| AZSX-23.10 | Nameplates | First load enables enemy/Overlapping once, preserving later choices. Half-width six-unit smooth capsules, no levels/skulls. Names only for unique highlighted elite/guardian/boss/final; ambiguous/untargeted hidden. |
| AZSX-23.11 | Objectives | Always-visible ornamented inset header below minimap; grouped Guardians and separate Sayge quest, hierarchical spacing. Upcoming guardians appear within 60 seconds; arrival timer on separate indented line only final 30 seconds. |
| AZSX-23.12 | Multiple minimap pins | All valid active lairs; priority Sayge > active guardian > Supplies > cache > returning guardian. Distinct native cache/Supplies icons, no crystal pins; retire individual scenes without blanking others. |
| AZSX-23.13 | Movement/turn/zoom/pan | Foreground native-art player arrow uses valid AZS_GetPlayerPosition each frame, not snapshots; pins use live native projection/zoom. Invalid data restores native arrow/flag fallback, no guessed center. |
| AZSX-23.14 | Minimap decoration | Sun/title/close strip hidden reversibly; map/ring/zoom remain native. |
| AZSX-23.15 | Popups | Compact native art, wrapped content/stacking without overlap. Green numeric bonuses/white labels, separate duration, sale amount/native gold coin. New item receipts use real icon; duplicates silent; no chat-parsed substitute for item receipt. |
| AZSX-23.16 | Settings/ally panels | Widget position/scale/opacity/visibility persist in native SavedVariables; hiding changes only display. Ally builds use matching theme with buffs outside right; no clipped text/texture gaps. |
| AZSX-23.17 | HUD before run | On native `ready` after world entry, show the retained bottom HUD without previous-run counters, clock, equipment or stats. Restore native elements if AZSXUI is disabled. |
| AZSX-23.18 | Camera and inspection | Right click in the world toggles persistent native mouselook. Menus, choices and talent/character panels release it; closing them leaves the cursor free until another right click. Native mouse handlers remain chained, left click keeps its native action, and a conflicting SurvivorsCombatMode addon prevents this controller from taking ownership. |

### AZSX-24 — Combat text

Mode: **Both**. Disposition: **active**. Details: [Contract](../features/NATIVE-WORLD-UI.md).

Candidate profiles: `official-party-ui`, `support`, `azsxui`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-24.01 | Normal/critical damage | Native face, world projection and animation; each fresh size divided by three once, never compounded by reuse. |
| AZSX-24.02 | Healing/other categories | Damage seam leaves their size unchanged; allied healing retains identity/fractional accumulation. |
| AZSX-24.03 | Fonts/icons | Native face default. Critical spell icons/guessed Lua impact projection not implemented. Optional older OFL face also changes healing and compounds scaling; never enable silently. |

### AZSX-25 — Participant markers

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/NATIVE-WORLD-UI.md).

Candidate profiles: `official-party-ui`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-25.01 | Coop start/late join | Host initializes once with distinct seven non-skull icons, reserving existing marks and skipping marked actors. |
| AZSX-25.02 | Manual change/reset/solo | Preserve manual choices on live recheck; new run resets initialization, solo no assignment. Skull manual only, no automatic goblin skull. |

### AZSX-26 — Treasure Goblins

Mode: **Both**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `rules`, `official`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-26.01 | First opportunity | Never before first boss; pre-final bad-luck protection (25%, 1/3, 50%, 100% on five-boss maps), then 20–40-second placement delay. Not fixed early; short run/placement failure can prevent actual appearance. |
| AZSX-26.02 | Active goblin | Max one, native model/transport, 150% guardian-base native health scaling, no attack/ordinary loot, taunt immune. Flee within terrain/arena without distant-recycling teleport; 25 simulation seconds including arrival delay, pause freezes deadline. |
| AZSX-26.03 | Kill/escape | Kill reserves personal native amulet choice and native-scaled whole marks per present participant, distinct amulets where possible. Escape/missing instance grants nothing; no gold/fabricated boss or pity progression. |
| AZSX-26.04 | Downed/choosing recipient | Marks credit once; amulet request waits without replacing current choice; native pause remains authoritative. |
| AZSX-26.05 | Disable/failure/presentation | Host optional setting controls generation; no uncertain spawn retry. Native announcement/HP bar, no automatic skull. |

### AZSX-27 — Lost Supplies

Mode: **Solo / Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `rules`, `diagnostics`, `official-party-ui`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-27.01 | Generation | Map-wide native cargo, first attempt 30 seconds then every 30–50 with 50% chance; multiple 90-second reserves. Pause freezes time, no player-near fallback. |
| AZSX-27.02 | Host and guest claim same reserve | Fixed personal reward per appearance-time identity, once each. Hide only for collector; other can still collect. All claims/expiry remove globally, late/reused identities excluded. |
| AZSX-27.03 | Consumables | Seven 60-second personal families: recovery 8/12/16%, crit chance 4/7/10 points, reduction 3/5/7 points with no HP bonus, max HP 20/30/40%, crit damage 20/35/50%, area 15/25/35%, speed 10/15/20%. Tiers encounters 0–1/2–3/4+. |
| AZSX-27.04 | Refresh/rebuild/expiry | Different families coexist; same refreshes without stacking time/power or downgrading stronger tier. No rebuild accumulation/free max-HP heal; expire owned aura rows. |
| AZSX-27.05 | Materials | Immediate whole-gold sale once per personal claim to shared wallet, usual strongest contributions/no skip division. Show actual credited amount, no crafting inventory. |
| AZSX-27.06 | Quality/options/aura independence | Cosmetic reference item quality independent of tier/sale bucket. lost_supplies controls generation; no choice/pause. Projection works without Fortune/fatigue active. |

### AZSX-28 — Sayge's Trial

Mode: **Solo / Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `rules`, `diagnostics`, `official-party-ui`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-28.01 | Scheduling | Enabled trial replaces boss Fortunes; after first boss/between bosses, not after final normal encounter, endless continues. One map-wide walkable rune/decorative Sayge/candles; sample interior navigation cells across its large footprint, no combat actor/reward contribution. |
| AZSX-28.02 | Unfinished trial/later boss | Keep identity/location/progress, upgrade reward stage without reset/replacement/banked extras. No availability deadline or premature reward. |
| AZSX-28.03 | Presence/death/pause | Twenty cumulative seconds; every active coop actor must be alive/playing/inside. Leaving/death suspends without reset, resume when all return; native pause freezes time. |
| AZSX-28.04 | Progress UI | Visible only for local inside actor, collective waiting explicit; never interpolate suspended progress. |
| AZSX-28.05 | Success/new trial/disable | One personal replacement Fortune per current actor, reserved before native IO; no prior acquisition replay. Later eligible encounter schedules next; disabling trials restores boss Fortunes. |

## AZSX-29 — Cooperative area bonuses

Mode: **Coop**. Disposition: **active**. Details: [Rules](../features/FEATURE-RULES.md).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-29.01 | Positive global area bonus | Retain 50%: +40% becomes +20%; +100% becomes +50%. Cover native and added global contributions together. |
| AZSX-29.02 | Solo, baseline, reductions | Solo unchanged; 1.0 remains 1.0; values below 1.0 remain unchanged; no changes to individual spell base radii. |
| AZSX-29.03 | Actor/rebuild/snapshot | Host and every authoritative remote actor adjusted once per complete rebuild. Nested same-game callbacks do not compound the reduction; guest snapshots are never scaled again. |
| AZSX-29.04 | Presentation | Effective global stat reflects the reduced result; item descriptions may still show their nominal bonus. Do not rewrite item history or individual spell definitions. |
| AZSX-29.05 | Cooperative information aura | Show Cooperative Rules before helpful buffs; no timer, cancellation or native slot. Tooltip gives the 50% rule, blank separation, then the two gray display notes. Remain present while dead/paused/choosing; hide in solo, outside a run or on stale/missing payload. |
| AZSX-29.06 | Independent addon presentation | API 1.9 exposes detached cooperativeRules values/texts without AZSXUI. AZSXUI borrows the bridge button for first position; disabling the HUD restores native-row fallback without duplicate frames, anchor cycles or altered native IDs/clicks. |

## Cross-cutting invariants and source map

Native engine owns enemy scaling, density, schedules, map/difficulty, transport,
movement, targeting and progression except explicitly listed AZSX rules. No retired
custom simulation, difficulty interception, global OS hooks or secondary transport.
Preserve bounded reads, caller ABI/flags/registers, ownership and callback ordering.
Use available native event/UI resources; do not silently add external game assets.
The documented optional font is distinct from the default native damage path.
UI/code/docs remain English; machine paths, private keys and mutable test status
do not belong here.

| Area | Source owners | Detailed reference |
| --- | --- | --- |
| XP/wallet/rewards/pause | gameplay `official.rs`; runtime `experience.rs`, `xp_hooks.rs`, `menu_pause.rs`; selected `official_native/` | [Rules](../features/FEATURE-RULES.md), [Pause](../features/COOP-PAUSE.md), [Adapter](CLIENT-ADAPTER.md) |
| Support/combat/sickness | gameplay `support.rs`, `periodic.rs`, `marks.rs`, `taunts.rs`, `immunity_aggro.rs`, `resurrection_fatigue.rs` and runtime/native consumers | [Rules](../features/FEATURE-RULES.md) |
| Rings/crystals/events | gameplay `enchantments.rs`, `crystals.rs`, `blessings.rs`, `supplies.rs`, `rituals.rs`, `treasure_goblins.rs`; runtime event/reward services | [Rules](../features/FEATURE-RULES.md), [Player guide](../player/FEATURES.md) |
| API/party/HUD | `native-ui/official-party.lua`, `addons/AZSXUI/`, runtime `party_ui.rs` | [API](../features/ADDON-API.md), [Party](../features/OFFICIAL-PARTY-UI.md), [World UI](../features/NATIVE-WORLD-UI.md), [AZSXUI guide](../../addons/AZSXUI/README.md) |
| Installation/release | `azsx/`, `tools/build/azsx_*.py` | [Installation](../release/AZSX-INSTALLATION.md), [Reproducibility](../release/REPRODUCIBILITY.md) |
| Coverage/native boundaries | `compat/features.json`, `feature-validation.json`, `contract-links.json`, `suites.json`, exact descriptor | [Workflow](COMPATIBILITY-WORKFLOW.md), [Testing](../development/TESTING.md) |

Integrating another branch requires adding its intended behavior, source mapping
and verification cases here. A prototype/model/helper alone is not acceptance.

### AZSX branding acceptance

- Migrating ASX keeps configuration, native saves, SavedVariables and original backups. New binaries take precedence; conflicting state stops without overwrite.
- Native host submissions use one `[AZSX] ` prefix within the 32-character limit. Join submissions, passwords and public/private choices remain native.
- The API must execute with no internal addon and without AZSXUI. Native API tables remain intact.

### AZSX-30 — Mechanical Chicken escort and engineering gadgets

Mode: **Solo, Coop**. Disposition: **active**. Details: [Contract](../features/SOLO-ESCORT.md).

Candidate profiles: `rules`, `official`, `official-party-ui`, `azsxui`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-30.01 | Solo scheduling and placement | No placement before the first completed native boss; first attempt 30–90 simulation seconds afterward. One successful reservation per run at most; route length scales with the native map radius (maximum 180 yards), with four candidate ground sites and eight headings each. Every segment and the interiors of the start and five-yard extraction footprints must be walkable and reject native liquid samples. Native Mechanical Chicken at scale 4.5 and Goblin Teleporter, plus up to four ground-checked native gnomish debris objects behind the start; missing debris never blocks the escort. All owned objects retire together, with no external assets. Failed placement may retry without spending the quota; the host creates at most one shared escort in coop. |
| AZSX-30.02 | Presence and movement | Invulnerable chicken advances by native spline only while every active participant is alive and within six yards. Leaving suspends cumulative progress without reset; native pause freezes time. |
| AZSX-30.03 | Extraction and failure | Arrival reserves one independently drawn gadget per current participant exactly once, removes chicken and portal; expiry, invalid route or cleanup grants nothing; a downed participant suspends the escort. No uncertain native commit is retried. |
| AZSX-30.04 | Engineering reward | Each participant receives one of four unranked, run-local utility/cosmetic gadgets: Net, Rocket Boots, Stun Grenade or Shrink Ray. The optional UI lists the escort beneath the minimap, marks its native position while active, and keeps the gadget beside the health/Fortune medallions; a versioned detached API works without AZSXUI. Variable route totals from 8 to 120 must never invalidate the full AZSX payload or hide unrelated UI. The requested Egg_05 icon is absent in 0.138.0, so the verified native EngGizmos icon is used. |
| AZSX-30.05 | Native effects | Net roots at most three nearest ordinary enemies within eight yards; Boots use native movement reconstruction; Grenade launches a zero-damage native projectile and controls at most three ordinary enemies within three yards at its identified impact. Native immunity/diminishing returns remain authoritative. Shrink Ray changes only visual scale (x0.5/x1/x1.5). |
| AZSX-30.06 | Retired damaging/shield gadgets | Sapper and Protection Belt are absent from reward draws and active effects. No engineering damage, backlash or shield grant remains. Parachute is unavailable until native movement/visual/replication contracts are reviewed. |
| AZSX-30.07 | Compatibility boundary | Exact 0.138.0 executable and selected entry bytes, fields, actor identity and source guard activation. Missing or changed contacts disable the escort; do not substitute guessed offsets or synthetic rewards. |


### AZSX-31 — Banished Demon

Mode: **Solo, Coop**. Disposition: **active**. Details: [Contract](../features/BANISHED-DEMON.md).

Candidate profiles: `rules`, `official`, `official-party-ui`, `azsxui`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-31.01 | Schedule and placement | Host draws a first opportunity at 90–150 simulation seconds and later opportunities 180–300 seconds apart; failed dry-ground placement can retry. No simultaneous Mechanical Chicken escort. Native rune, captive demon and warlock scene, with one of three coherent native demon kits and no external assets. |
| AZSX-31.02 | Group preparation | Every active participant alive and within four yards. Two cumulative seconds release one native hostile demon; departure suspends without resetting. Scene expires after four simulation minutes or on run teardown. |
| AZSX-31.03 | Reward | A confirmed native death grants exactly one shared native Banish charge, consumed by native card choice. Missing enemy row, expiry, uncertain write and duplicate callback grant nothing extra. No ordinary XP, gold or boss reward from the event definition. |
| AZSX-31.04 | Presentation | Detached API exposes the current recipient's structured event in solo and coop. Opening announcements do not reveal rewards. Optional AZSXUI shows an objective, native-icon marker and one terminal reward toast; unrelated UI state survives. |
| AZSX-31.05 | Compatibility boundary | Guard exact 0.138.0 Banish, demon definition, native spawn, world scene and broadcast contacts. Changed contacts disable the encounter rather than guessing. |


### AZSX-32 — Allied Polymorph rupture

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `combat`, `official`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-32.01 | Ally hits another player's active sheep, including without a special ability |  Eligible hit ruptures it once; native explosion power, critical rolls, talents and meter attribution belong to the caster. Original hit remains the attacker's. |
| AZSX-32.02 | Host/guest, guest/guest, nested burst | Restore the triggering actor before subsequent procs, including Faerie Fire. No synthetic hit or repeated burst. |
| AZSX-32.03 | Expired sheep, dead/departed/replaced caster, ambiguous ownership | Do not borrow an invalid producer or revive expired control. Own sheep and solo retain native behavior. |
| AZSX-32.04 | Native completion/flush failure | Attempt actor restoration before returning the failure; never replay uncertain native effects. |

### AZSX-33 — Cooperative armor sets and legendary necklaces

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `combat`, `support`, `official`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-33.01 | Ally kills a victim carrying Serpent Sting, Rupture, Flame Shock or Curse of Agony | Dispatch the wearer's matching native eight-piece bonus once with that producer's active effect. Preserve the producer of any propagated DoT. |
| AZSX-33.02 | Execute, Holy Shock, Judgement, Eviscerate or Mind Flay kill | Only the native killing spell and its producer qualify. Keep personal counters, cooldowns and unrelated kill effects. |
| AZSX-33.03 | Devout Power Word: Shield breaks on self or ally | The priest's set bursts around the recipient at 8 yards times the producer's area factor. Divine Aegis can trigger independently once. Both Devout tiers require an actual PW:S break. |
| AZSX-33.04 | PvP set debuffs and controlled targets | Mortal Strike and Earth Shock benefit allied attackers and retain the strongest active vulnerability. An ally's control satisfies the seven other PvP set conditions for the wearer only. |
| AZSX-33.05 | A teammate finishes an enemy carrying a legendary necklace DoT | Each eligible producer triggers the matching native explosion once. A qualifying set bonus can also trigger. |
| AZSX-33.06 | A shield or healing necklace targets another player | The Power Word: Shield necklace bursts around the protected player with the caster's area bonus. Holy Light and Regrowth apply an absorb pool capped at 40% of caster maximum health on cast. |
| AZSX-33.07 | Vampiric Renew or Rejuvenation heals another player | The recipient gains 5% leech on their own damage for 10 or 12 simulation seconds respectively. Recasting the same necklace on the same recipient refreshes its duration. |

### AZSX-34 — Specialization weapon appearance

Candidate profile: `official`.

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| AZSX-34.01 | Start a run with Worn (spec) | Project the equipped weapon using its set and the unique highest talent tree. Preserve actual gear and stats. |
| AZSX-34.02 | Select a native Look | Keep the explicit hand appearance. Use native appearance for tied or empty trees. |
| AZSX-34.03 | Inspect the Armory character model or main-menu portrait | Resolve both visible hands from the selected class and saved ranks using the same set mapping and Look precedence as the run. |
| AZSX-34.04 | A melee action requests a one- or two-handed strike while a specialization weapon is visible | Match the requested strike to the visible grip for the attacking entity. Preserve spell casts, off-hand hits and native equipment choices. |
