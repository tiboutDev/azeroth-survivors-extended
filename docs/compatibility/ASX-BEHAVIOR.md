# ASX expected behavior and compatibility acceptance

Use the [compatibility checklist](COMPATIBILITY-CHECKLIST.md) for execution order; this catalog owns the expected behavior criteria.

Maintained acceptance reference for **current main source**, reviewed against
official **0.138.0**. Covers all ASX-01–ASX-29 features, including disabled and
upstream-owned behavior. This is not proof that an installed DLL or published
archive passes these requirements. Uncommitted source fixes are not automatically
in a prepared release. Separate event-branch experiments (escort, gadgets, heroes)
are outside this baseline until deliberately integrated.

## Maintenance and use

- Each `ASX-NN.xx` is a stable acceptance ID. Never recycle it; add criteria for
  new features/regressions and explicitly record retirement/upstream ownership.
- Update affected criteria in the **same change** as implementation/tests,
  `compat/features.json`, detailed rules/API and player documentation. Regenerate
  the existing ASX-FEATURES.md index; do not edit only its generated table.
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
| ASX-02.02 | Dead host and guest gain XP | NOT RUN | No current-artifact observation | Both roles required |

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

### ASX-01 — Activation

Mode: **Both**. Disposition: **active**. Details: [Contract](CLIENT-ADAPTER.md).

Candidate profiles: `official`, `tooling`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-01.01 | Supported ASX launch | Exact executable and required native bytes/layouts must pass before activation; a version label alone is insufficient. |
| ASX-01.02 | Direct native launch or unsupported executable | Direct native launch stays inert; unsupported ASX builds fail closed, with no retired engine fallback. |
| ASX-01.03 | Addon activation | Only the owned file/read buffer is activated in this process; on-disk content and unrelated addon reads stay unchanged. |
| ASX-01.04 | Optional feature failure | Stop the affected optional event safely without weakening required activation or disabling unrelated core coop. |
| ASX-01.05 | Installed layout/migration | App/Config/Data separation; byte-verified restartable migration preserves settings, transaction history, backups, logs and unrelated files. Native addon locations remain unchanged. |
| ASX-01.06 | Compatible signed update | Install automatically without a question, including noninteractive use; exact native match/signature/ownership/anti-loop gates remain. Runtime changes use a detached verified transaction and automatic restart. |
| ASX-01.07 | Launch versus full check | Launch checks only activation dependencies; missing docs or optional ASXUI do not block native-interface play. Full --check stays offline/read-only. Invalid maintenance metadata cannot force unsafe writes. |

### ASX-02 — Shared XP and death

Mode: **Coop**. Disposition: **active**. Details: [Contract](CLIENT-ADAPTER.md).

Candidate profiles: `official`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-02.01 | Ordinary shared XP | Normalize the unscaled native receipt once. Non-racial native contributions combine under the portable policy; strongest racial and strongest XP Fortune contribute in their respective channels. Preserve rounding/minimum; never rescale an already rounded receipt. |
| ASX-02.02 | Host death, then guest death | Both continue gaining native XP and levels while dead, with dead phase/HP preserved. Observe actual XP and level on both clients. |
| ASX-02.03 | Guardian whole-level reward while dead | Retain and consume the native receipt at receipt time using the threshold deficit; this award does not enter ordinary XP multiplier normalization. |
| ASX-02.04 | Revive after XP and whole-level awards | Choices remain pending until native revival flow; no catch-up bank, lost receipt or duplicate award. Compare exact XP as well as levels before/after both revivals with identical profiles. |
| ASX-02.05 | Different builds/starting progression | Do not force equal absolute XP/levels or overwrite progression to satisfy a synchronization check. |

### ASX-03 — Shared wallet

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `official`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-03.01 | Collect gold from either role | Credit one host-owned wallet once; native balance/coinage projections converge. Keep collector-specific effects, strongest racial/Gold Fortune and best participant permanent armory contribution; do not sum party equipment. |
| ASX-03.02 | Voluntary skip | Divide the scaled amount by current participant count before nearest-integer rounding, halves up: 7/2 = 4, 2.99/2 = 1. Dead members count, departed members do not; solo unchanged; small results may be zero. |
| ASX-03.03 | Automatic conversion/world gold/material sale | Keep conversion floor policy distinct; world gold and Supplies sales do not acquire skip division. |
| ASX-03.04 | Cache purchase and paid spell/item reroll on either role | Completed native debit reduces the common balance once without reducing earned-gold totals. Inline/generic observations cannot double-debit or overwrite nested wallet changes. |
| ASX-03.05 | Free/rejected reroll | Native eligibility/cost remain authoritative; no phantom debit or stale common balance preventing an affordable action. |

### ASX-04 — Purchased caches

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `official`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-04.01 | Open as host and guest | One debit, shared next price/opened count, one personal native item choice per eligible participant. |
| ASX-04.02 | Overlap choices or repeated delivery/presentation | Keep successful recipient acknowledgements; no overwritten current choice, lost pending delivery or extra reward. |
| ASX-04.03 | Finish at different times | Remaining choices keep group pause while selection stays usable. |

### ASX-05 — Native guardian rewards

Mode: **Coop**. Disposition: **upstream**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `official`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-05.01 | Kill with two participants | Each native skill/item reward arrives once per eligible actor; no supplemental ASX item grant. |
| ASX-05.02 | Kill while dead or choosing | Preserve native pending choices and whole-level receipts; count completed deliveries, not just cards created. |

### ASX-06 — Pause, Escape and inspection

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/COOP-PAUSE.md).

Candidate profiles: `official`, `official-party-ui`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-06.01 | Overlapping choices, one player finishes | Host simulation, local movement and guest prediction remain stopped until all relevant choices/menus close; selection and networking continue. |
| ASX-06.02 | Actual guest Escape during play and waiting | Native personal menu opens before the guest filter consumes Pause; Resume/Escape closes only that participant ownership. |
| ASX-06.03 | Close one of several menus/choices | Remaining ownership holds pause; native Options/Back/Resume work; waiting text names the relevant characters. |
| ASX-06.04 | Pause while others move | Remote visual movement stops on the guest, without rewriting authoritative coordinates; native movement resumes afterward. |
| ASX-06.05 | Host talents/read-only inspection in coop | Opening/closing alone does not pause gameplay. Actual Escape and actions within an owned menu keep native behavior. |
| ASX-06.06 | Leave/reuse slot/new run | Retire old menu identity/epoch; no stranded pause. Solo/lobby forwarding stays native. |

### ASX-07 — Allied healing and shields

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `support`, `periodic`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-07.01 | Direct heal/shield | Respect eligibility, range, missing health/pressure and native limits; do not duplicate an upstream delivery. |
| ASX-07.02 | Foreign periodic effect | Retain original caster coefficients, talents, counters, aura lifetime and remaining budget across recipient context changes. |
| ASX-07.03 | Source/recipient dies, departs or aura replaced | Retire appropriate identities/effects; no ticks reach reused slots or another run. |

### ASX-08 — Hybrid support and retaliation

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `support`, `periodic`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-08.01 | Foreign Thorns/Lightning Shield | Retain producer, native modifiers and proc cooldown/budget; recipient outgoing talents cannot steal attribution. |
| ASX-08.02 | Holy Nova/Holy Shock and healing/shield talents | Supported allied branches keep caster ownership and native resource/cooldown semantics. |
| ASX-08.03 | Ice Barrier | Remains caster-only, never a group shield. |

### ASX-09 — Hunter's Mark

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `combat`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-09.01 | Multiple marks | Strongest valid shared vulnerability applies once, with native visuals and no multiplied duplicate damage. |
| ASX-09.02 | Replacement/death/departure/reset | Reconcile effective mark and clear invalid owner associations. |

### ASX-10 — Talent taunts

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `combat`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-10.01 | Eligible successful hits | Shield Slam/Fortified, Hammer of Justice/Ardent Defender and Shred/Dire Bear Form establish persistent ordinary pursuit/basic-attack ownership. |
| ASX-10.02 | Miss/no required talent/dead enemy/goblin | No taunt assignment. |
| ASX-10.03 | Competing tanks/death/departure | New valid taunt replaces old; invalid ownership clears. Special casts/projectiles and NPC ownership remain native. |

### ASX-11 — Immunity and aggro

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `combat`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-11.01 | Enter temporary invincibility | Discard prior ordinary aggro/taunts; exclude actor from ordinary candidates. |
| ASX-11.02 | Fresh taunt while immune/exit immunity | Fresh eligible taunt can work; scoped candidate flags restore after native pass. |
| ASX-11.03 | Absorb or damage reduction | Not an invincibility aggro wipe. |

### ASX-12 — Resurrection Sickness

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `fatigue`, `support`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-12.01 | Repeated resurrection | One stack each time, cap three; -5% outgoing damage/healing each, maximum -15%. |
| ASX-12.02 | Recovery | One stack per 60 alive simulation seconds; resurrection restarts interval, death/pause/choices suspend it. |
| ASX-12.03 | Display and incoming healing | Native harmful aura shows stacks/time; no max-HP or incoming foreign-heal penalty. |

### ASX-13 — Sayge's Dark Fortunes

Mode: **Both**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `rules`, `official-party-ui`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-13.01 | Trial success or trials-disabled boss encounter | One personal replacement Fortune; fallback rolls once per completed encounter, not per entity. Survives death, clears next run. |
| ASX-13.02 | Category/power rolls | Luck/XP/Gold personal weights N:1:1 for the eligible completion-time participant count (1–5), in rituals and boss fallback; solo equal. Membership changes alone never reroll retained fortunes. No speed/pickup. Luck does not bias rolls. Whole percentages once for application/display; bounded power and five-encounter scaling cap, endless replacements continue. Detailed coefficients remain in linked rules. |
| ASX-13.03 | Cooperative contributions | Strongest current XP and Gold Fortunes selected independently; Luck personal. Replacement/departure refreshes effective values. |
| ASX-13.04 | Tooltip/notification | Green +X%, no visible rarity, native-style tooltip distinguishes own versus stronger party contribution. |
| ASX-13.05 | Heartbeat/pause/membership/new ritual | Unchanged Fortune never replays acquisition; genuine new award with identical values still notifies once. |

### ASX-14 — Reward exclusions

Mode: **Both**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `rules`, `official`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-14.01 | Solo/coop candidate pools | insignia, eldre_thalas and chosen remain excluded; their return is a regression. remote/Gnomish Universal Remote stays available. |
| ASX-14.02 | Existing inventory/unrelated gear | No deletion of existing items or broad exclusion of other equipment. |

### ASX-15 — Reserved removed feature

No runtime implementation or acceptance scenario remains. The ID and its former
criteria are retired permanently; never assign them to another feature. Historical
meaning remains in Git history, not in current gameplay or player documentation.

### ASX-16 — Ring enchantments

Mode: **Both**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `enchantments`, `official-party-ui`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-16.01 | Decline preview | Retire preview at validated reward boundary before reentrant creation of next reward. Independent draws can legitimately repeat. |
| ASX-16.02 | Accept first eligible upgrade | Kind locks per ring; other ring identity/progression remains independent. Alacrity totals 3/5/8/11/15% recovery; Renewal 0.5/1/1.5/2/3 HP/s. |
| ASX-16.03 | Ranks 2/4/7/10/14 | Minor/Lesser/unprefixed/Greater/Superior total-value tiers; native capped-ring limitations retained. |
| ASX-16.04 | Rebuild/replace/reset | Additive bonuses never accumulate through repeated temporary-input borrowing or leak across identity/run. |
| ASX-16.05 | Cards and owner tooltips | Actual rank/kind/values; preview/apply/improve versus compact Active retained-bonus text, native gains preserved. |

### ASX-17 — Party builds and addon contract

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/OFFICIAL-PARTY-UI.md).

Candidate profiles: `official-party-ui`, `support`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-17.01 | Different allied builds | Enrich native children, retain health/name/death/resurrection and native five-player capacity. Never format remote tooltips with viewer stats. |
| ASX-17.02 | Long pause/departure/reused identity | Heartbeat preserves valid unchanged cosmetics; stale/ambiguous/missing/foreign data hides. |
| ASX-17.03 | API without ASXUI | Native-style ASX_Get functions expose detached local structured values separately from ASX text; ASXUI consumes, not provides. No new transport/gameplay authority; private remote builds are not public v1 data. |
| ASX-17.04 | Mutate snapshot/unsupported major/unavailable state | Provider/other consumers unaffected; unsupported major rejected; stale/outside-run/missing payload explicitly unavailable. |
| ASX-17.05 | API 1.8 sharedEconomy | XP/gold independently optional actual award multipliers, replacing rather than adding to personal values. Missing is not zero; legacy economy keeps complete-pair guarantee. |
| ASX-17.06 | Payload/lifecycle | Active namespace asx, no recipient-build retransmission or unused radius field; heartbeat retained. Exact string identities and every field/unit/version rule follow ADDON-API.md. |

### ASX-18 — Retained native presentation

Mode: **Coop / reward cards**. Disposition: **active**. Details: [Contract](../features/OFFICIAL-PARTY-UI.md).

Candidate profiles: `official-party-ui`, `official`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-18.01 | Lobby/party | Native lobby untouched, no retired class-icon decoration, replacement party root or custom DPS ranking. |
| ASX-18.02 | Waiting menus/rewards | Names identify relevant characters; separate local choice/menu from remote waiting. |
| ASX-18.03 | Reward cards | Native gains, one separator, icon/title/status header and actual values/retained-bonus hover; retained gem format is not active gameplay. |

### ASX-19 — Run/identity isolation

Mode: **Both**. Disposition: **active**. Details: [Contract](CLIENT-ADAPTER.md).

Candidate profiles: `official`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-19.01 | New run/session/clock rollback | Reset owned state; no old bonuses, menus, rewards, events or cosmetic receipts leak. |
| ASX-19.02 | Slot reused | Bind slot+GUID and generation; new actor cannot inherit claims, pending rewards or effect attribution. |
| ASX-19.03 | Reentrant native calls | Restore actor/host context, preserve ordering and successful recipient acknowledgements; no held mutable guard or retry of uncertain grant. |
| ASX-19.04 | Solo after coop | Use solo identity without inactive coop reads; native simulation/transport/recovery remains authoritative. |

### ASX-20 — Installation and maintenance

Mode: **Distribution**. Disposition: **active**. Details: [Contract](../release/ASX-INSTALLATION.md).

Candidate profiles: `asx-install`, `tooling`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-20.01 | Install on supported closed client | Owned ASX/start/docs only, no native EXE/archives. Automatically deploy ASXUI to benilla-config/AddOns/ASXUI; AddOns/ASX is separate gated bridge. |
| ASX-20.02 | Startup/update | No-argument auto-launch; explicit --menu for maintenance. Every update needs confirmation, Enter defers, noninteractive input cannot approve implicitly. |
| ASX-20.03 | Changed/missing README-ASX.txt or ASX/Client/LICENSE | Noncritical docs do not block otherwise verified launch; preserve modified docs, restore missing ones through repair, back up replacements on update. |
| ASX-20.04 | Critical files/metadata/unsafe paths | Roles are product-owned, never inferred by extension. Verify critical sources/projections and client; unsafe paths always block. Invalid ownership blocks maintenance writes, not separately verified intact launch. |
| ASX-20.05 | --check/report | Offline read-only check collects issues without repair/preparation; diagnostic export works with invalid manifest metadata. |
| ASX-20.06 | Repair | Reauthenticate signed same-version cache/channel/local triple, show plan and obtain consent; preserve unknown files/settings/saves. No implicit adoption of unknown loader or addon. |
| ASX-20.07 | Update/interruption/retry/rollback | Bounded signed inventories/hashes/journals/backups; retain sequence high-water mark and rollback pointer. No reinstall loop; failed attempts need explicit retry. |
| ASX-20.08 | Code/runtime replacement/uninstall | Restart before using replaced launcher code; runtime uses closed-launcher complete archive. Remove only owned matching files, preserve progression/unrelated addons. |
| ASX-20.09 | Release | Normal artifact, approved reproducibility reference and exact signed archive are separate gates. No implicit commit/push/publication/installation. |

### ASX-21 — Native marks and duplicate loot

Mode: **Both**. Disposition: **upstream**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `official`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-21.01 | Already-collected loot | Native personal marks/collection remain authoritative; no ASX duplicate-loot gold or extra grant. |
| ASX-21.02 | Contiguous mark gains | Sum earned values in one popup with latest native total; duplicate item popup silent. Notification itself awards nothing. |

### ASX-22 — Crystal Resonance

Mode: **Both**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `crystals`, `official-party-ui`, `asxui`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-22.01 | Generation | Independent map-wide native models, equal free colors, no minimap pins/kill drops. Prototype 4–6 opportunities in seconds 30–870, max two scenes; terrain/budget/early end can reduce discoveries. Endless intervals 90–240 seconds. |
| ASX-22.02 | Color pickup | Red +12% damage, Blue +7% cooldown recovery, Green +15% max HP, Yellow +12% area; personal additive permanent run gains. Native amulet effects remain, no ASX amulet gem. |
| ASX-22.03 | First/group claims | Alive proximity grants once per appearance-time identity; model stays visible even to collector. First claim starts shared 60 simulation seconds, never extended by later claims. |
| ASX-22.04 | All claims/expiry/solo/departure | Common removal on all remaining claims or timeout, solo immediate; unclaimed has no timer. Pause freezes time, late/reused identities cannot claim. |
| ASX-22.05 | Death/revival/UI restore | Bonuses survive death, reset next run. One permanent effect with total icon count and per-color totals; only new pickups notify, no replay of restored totals. |

### ASX-23 — ASXUI HUD and minimap

Mode: **Both**. Disposition: **active**. Details: [Contract](../features/NATIVE-WORLD-UI.md).

Candidate profiles: `asxui`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-23.01 | Enable/disable | Existing native artwork, reversible owned anchors/levels/scripts/visibility; UI settings never change gameplay. |
| ASX-23.02 | Actions/XP/equipment | One stats extension; native XP art/top border intact. Order amulet, ring, ring, trinket, trinket, native empty silhouettes. |
| ASX-23.03 | Stats input | Hover previews; button/character binding toggle ASX stats and native character sheet together; checked indicator follows state, no animation. |
| ASX-23.04 | Stats layout | 29 supported native stats, nonzero rows, four columns/three vertical separators; short labels/full-term help. Width XP+32 units, no bottom border, above items below XP art/foreground gryphons. |
| ASX-23.05 | Shared stat changes | Refresh visible rows with actual effective coop XP/gold multipliers; missing shared value hides instead of falling back to personal stats. |
| ASX-23.06 | HP/auras/tooltips | Raised independent compact native-textured capsule, current HP and level only; aura rows above HP with durations above icons. Player/target roots hide reversibly; item tooltips use actual owner and verified enchantments. |
| ASX-23.07 | Permanent effects/channel bars | Fortune/Crystal medallions beside HP, total crystal count and Fortune category overlay. Icon-free Sayge/native resurrection style; remaining Sayge seconds inside track. |
| ASX-23.08 | Top strip | Kills/gold/cache icons/values vertically centered in ornamented 300-unit strip inset eight units; full border, clock below, soft center light behind affordable gold rather than a ring. |
| ASX-23.09 | Encounter bars | Boss/awake guardian/goblin bars below top clock, centered names without percentages; multiple supported rows visible. |
| ASX-23.10 | Nameplates | First load enables enemy/Overlapping once, preserving later choices. Half-width six-unit smooth capsules, no levels/skulls. Names only for unique highlighted elite/guardian/boss/final; ambiguous/untargeted hidden. |
| ASX-23.11 | Objectives | Always-visible ornamented inset header below minimap; grouped Guardians and separate Sayge quest, hierarchical spacing. Upcoming guardians appear within 60 seconds; arrival timer on separate indented line only final 30 seconds. |
| ASX-23.12 | Multiple minimap pins | All valid active lairs; priority Sayge > active guardian > Supplies > cache > returning guardian. Distinct native cache/Supplies icons, no crystal pins; retire individual scenes without blanking others. |
| ASX-23.13 | Movement/turn/zoom/pan | Foreground native-art player arrow uses valid AZS_GetPlayerPosition each frame, not snapshots; pins use live native projection/zoom. Invalid data restores native arrow/flag fallback, no guessed center. |
| ASX-23.14 | Minimap decoration | Sun/title/close strip hidden reversibly; map/ring/zoom remain native. |
| ASX-23.15 | Popups | Compact native art, wrapped content/stacking without overlap. Green numeric bonuses/white labels, separate duration, sale amount/native gold coin. New item receipts use real icon; duplicates silent; no chat-parsed substitute for item receipt. |
| ASX-23.16 | Settings/ally panels | Widget position/scale/opacity/visibility persist in native SavedVariables; hiding changes only display. Ally builds use matching theme with buffs outside right; no clipped text/texture gaps. |

### ASX-24 — Combat text

Mode: **Both**. Disposition: **active**. Details: [Contract](../features/NATIVE-WORLD-UI.md).

Candidate profiles: `official-party-ui`, `support`, `asxui`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-24.01 | Normal/critical damage | Native face, world projection and animation; each fresh size divided by three once, never compounded by reuse. |
| ASX-24.02 | Healing/other categories | Damage seam leaves their size unchanged; allied healing retains identity/fractional accumulation. |
| ASX-24.03 | Fonts/icons | Native face default. Critical spell icons/guessed Lua impact projection not implemented. Optional older OFL face also changes healing and compounds scaling; never enable silently. |

### ASX-25 — Participant markers

Mode: **Coop**. Disposition: **active**. Details: [Contract](../features/NATIVE-WORLD-UI.md).

Candidate profiles: `official-party-ui`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-25.01 | Coop start/late join | Host initializes once with distinct seven non-skull icons, reserving existing marks and skipping marked actors. |
| ASX-25.02 | Manual change/reset/solo | Preserve manual choices on live recheck; new run resets initialization, solo no assignment. Skull manual only, no automatic goblin skull. |

### ASX-26 — Treasure Goblins

Mode: **Both**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `rules`, `official`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-26.01 | First opportunity | Never before first boss; pre-final bad-luck protection (25%, 1/3, 50%, 100% on five-boss maps), then 20–40-second placement delay. Not fixed early; short run/placement failure can prevent actual appearance. |
| ASX-26.02 | Active goblin | Max one, native model/transport, guardian-based native health scaling, no attack/ordinary loot, taunt immune. Flee within terrain/arena without distant-recycling teleport; 25 simulation seconds including arrival delay, pause freezes deadline. |
| ASX-26.03 | Kill/escape | Kill reserves personal native amulet choice and native-scaled whole marks per present participant, distinct amulets where possible. Escape/missing instance grants nothing; no gold/fabricated boss or pity progression. |
| ASX-26.04 | Downed/choosing recipient | Marks credit once; amulet request waits without replacing current choice; native pause remains authoritative. |
| ASX-26.05 | Disable/failure/presentation | Host optional setting controls generation; no uncertain spawn retry. Native announcement/HP bar, no automatic skull. |

### ASX-27 — Lost Supplies

Mode: **Solo / Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `rules`, `diagnostics`, `official-party-ui`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-27.01 | Generation | Map-wide native cargo, first attempt 30 seconds then every 30–50 with 50% chance; multiple 90-second reserves. Pause freezes time, no player-near fallback. |
| ASX-27.02 | Host and guest claim same reserve | Fixed personal reward per appearance-time identity, once each. Hide only for collector; other can still collect. All claims/expiry remove globally, late/reused identities excluded. |
| ASX-27.03 | Consumables | Seven 60-second personal families: recovery 8/12/16%, crit chance 4/7/10 points, reduction 3/5/7 points with no HP bonus, max HP 20/30/40%, crit damage 20/35/50%, area 15/25/35%, speed 10/15/20%. Tiers encounters 0–1/2–3/4+. |
| ASX-27.04 | Refresh/rebuild/expiry | Different families coexist; same refreshes without stacking time/power or downgrading stronger tier. No rebuild accumulation/free max-HP heal; expire owned aura rows. |
| ASX-27.05 | Materials | Immediate whole-gold sale once per personal claim to shared wallet, usual strongest contributions/no skip division. Show actual credited amount, no crafting inventory. |
| ASX-27.06 | Quality/options/aura independence | Cosmetic reference item quality independent of tier/sale bucket. lost_supplies controls generation; no choice/pause. Projection works without Fortune/fatigue active. |

### ASX-28 — Sayge's Trial

Mode: **Solo / Coop**. Disposition: **active**. Details: [Contract](../features/FEATURE-RULES.md).

Candidate profiles: `rules`, `diagnostics`, `official-party-ui`. [Scenario mappings and limits](../../compat/feature-validation.json).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-28.01 | Scheduling | Enabled trial replaces boss Fortunes; after first boss/between bosses, not after final normal encounter, endless continues. One map-wide walkable rune/decorative Sayge/candles, no combat actor/reward contribution. |
| ASX-28.02 | Unfinished trial/later boss | Keep identity/location/progress, upgrade reward stage without reset/replacement/banked extras. No availability deadline or premature reward. |
| ASX-28.03 | Presence/death/pause | Twenty cumulative seconds; every active coop actor must be alive/playing/inside. Leaving/death suspends without reset, resume when all return; native pause freezes time. |
| ASX-28.04 | Progress UI | Visible only for local inside actor, collective waiting explicit; never interpolate suspended progress. |
| ASX-28.05 | Success/new trial/disable | One personal replacement Fortune per current actor, reserved before native IO; no prior acquisition replay. Later eligible encounter schedules next; disabling trials restores boss Fortunes. |

## ASX-29 — Cooperative area bonuses

Mode: **Coop**. Disposition: **active**. Details: [Rules](../features/FEATURE-RULES.md).

| Criterion | Trigger / case | Required outcome |
| --- | --- | --- |
| ASX-29.01 | Positive global area bonus | Retain 50%: +40% becomes +20%; +100% becomes +50%. Cover native and added global contributions together. |
| ASX-29.02 | Solo, baseline, reductions | Solo unchanged; 1.0 remains 1.0; values below 1.0 remain unchanged; no changes to individual spell base radii. |
| ASX-29.03 | Actor/rebuild/snapshot | Host and every authoritative remote actor adjusted once per complete rebuild. Nested same-game callbacks do not compound the reduction; guest snapshots are never scaled again. |
| ASX-29.04 | Presentation | Effective global stat reflects the reduced result; item descriptions may still show their nominal bonus. Do not rewrite item history or individual spell definitions. |
| ASX-29.05 | Cooperative information aura | Show Cooperative Rules before helpful buffs; no timer, cancellation or native slot. Tooltip gives the 50% rule, blank separation, then the two gray display notes. Remain present while dead/paused/choosing; hide in solo, outside a run or on stale/missing payload. |
| ASX-29.06 | Independent addon presentation | API 1.9 exposes detached cooperativeRules values/texts without ASXUI. ASXUI borrows the bridge button for first position; disabling the HUD restores native-row fallback without duplicate frames, anchor cycles or altered native IDs/clicks. |

## Cross-cutting invariants and source map

Native engine owns enemy scaling, density, schedules, map/difficulty, transport,
movement, targeting and progression except explicitly listed ASX rules. No retired
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
| API/party/HUD | `native-ui/official-party.lua`, `addons/ASXUI/`, runtime `party_ui.rs` | [API](../features/ADDON-API.md), [Party](../features/OFFICIAL-PARTY-UI.md), [World UI](../features/NATIVE-WORLD-UI.md), [ASXUI guide](../../addons/ASXUI/README.md) |
| Installation/release | `asx/`, `tools/build/asx_*.py` | [Installation](../release/ASX-INSTALLATION.md), [Reproducibility](../release/REPRODUCIBILITY.md) |
| Coverage/native boundaries | `compat/features.json`, `feature-validation.json`, `contract-links.json`, `suites.json`, exact descriptor | [Workflow](COMPATIBILITY-WORKFLOW.md), [Testing](../development/TESTING.md) |

Integrating another branch requires adding its intended behavior, source mapping
and verification cases here. A prototype/model/helper alone is not acceptance.
