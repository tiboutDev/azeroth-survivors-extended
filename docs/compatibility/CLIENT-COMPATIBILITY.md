# Official 0.138.0 client boundary

The sole source target is the exact Windows x86-64 executable with SHA-256:

```text
eb5fd1c1b96e6180fcf5f3d685d90a61471af28d22e5f060c02b7b7663fbf79e
```

`bindings/official138.rs` owns current native contacts, complete instruction
guards, field translations and register/frame choices. Version labels and
similar instruction sequences cannot select another executable. The launcher
registry and private importer use this same exact identity. Direct native launch
remains inert; the retired custom engine is not a fallback.

## Selected layouts

Host Game and private actor fields have independent translations. Private actor
stride is `0xb58`; native peers use stride `0x858`. Native capacity remains five.
Snapshots copy only the identity, resources, phase and bounded headers consumed
by party/support observations. Unused owned native context data is not cloned.
Gold moves independently from adjacent vitals and native earned-gold counters;
the wallet continues to project its selected gold fields only.

Enemy records use stride `0x208`. The selected schema separates GUID/death/target
from the moved aura, health, position and vulnerability fields. Every reader,
writer and retained fixture uses that schema, without changing native ownership,
crowd-control budgets, spawn rules or target policy for special abilities.

The NPC continuation uses XMM15 for distance. Temporary actor XY is at frame
`+0x70`; frame `+0x80` belongs to the native iterator and must not be overwritten.
Encounter handling reads the unique boss definition from R15 and cross-checks
Game in R13 against the selected `+0x928` frame slot. The old definition slot
now holds an unrelated log-queue reference and must not be read as a boss.
The basic-action Game pointer is at `+0x3b0`, with origin Z at `+0x26c`.
Shield-break callbacks read Game from RBX and the spell from R15. The aura-row
pointer is at RBP+0x38 at the projection seam; periodic-heal UI payloads start at
RBP+0x2d8. These are callback-local lifetimes, not general stack translations.

The native compositor uses a 64-byte tagged armor key. The selected adapter
preserves its four-byte prefix and race byte. Metadata, appearance, entity and
armor-key frame locations are selected independently.

Fresh damage-number size now uses XMM8; RBX owns the message. Direct and
periodic heal wrappers have separate reviewed frames and event discriminators.
The native entity resolver is now a query predicate plus a map lookup borrowing
a pointer to the map reference. Healing/meter calls additionally require a fifth
borrowed source descriptor; synthetic ASX healing uses the native unspecified
source tag, while Hurt forwarding preserves both native source stack words.

Native pause input retains tag 7 and its boolean payload. ASX menu ownership
uses the existing native reliable transport, with selected role/stream/peer
layouts and unchanged Win64 argument positions. Native reliability and recovery
remain upstream responsibilities; ASX does not restore its retired transport.

Support deliveries retain 56-byte rows, actor auras retain 32-byte rows including
the native producer GUID, and perks retain 64-byte rows. Foreign periodic effects
keep producer/recipient ownership and original native accounting. ASX extends
only the retained AZSRUN row in the native addon output, preserving other rows.

The 0.130 Death Recap remains native, including its modal close/Escape route.
ASX retains unknown native HUD fields and only enriches its owned presentation.
Amulet target milestones also remain native; gem stat inputs do not reconstruct
target schedules or restore the removed rounding-only target.

Event clock, terrain, endless and mark fields are selected descriptor constants.
Unconsumed context gaps without a reviewed translation fail closed rather than
being covered by a guessed global layout shift.

## Feature dispositions

These are source dispositions for [the stable inventory](ASX-FEATURES.md), not
claims that every feature has been exercised in a normal running selected client.

| ID | Disposition for the selected client |
| --- | --- |
| ASX-01 | Exact hash, native guards and explicit launcher activation; reject other clients. |
| ASX-02 | Preserve receipt scopes, raw-register sources, caller classification and rounding. |
| ASX-03 | Preserve host wallet policy with independently translated balances and earned-gold fields. |
| ASX-04 | Preserve one debit, participant identities and once-only personal native choices. |
| ASX-05 | Keep guardian rewards native; no supplemental item grant. |
| ASX-06 | Preserve choice/menu barriers over the native pause and reliable command paths. |
| ASX-07 | Preserve allied support eligibility, native coefficients and producer attribution. |
| ASX-08 | Preserve shield/proc accounting, foreign retaliation and periodic ownership. |
| ASX-09 | Preserve strongest Hunter's Mark and native visuals through selected enemy fields. |
| ASX-10 | Preserve eligible taunts and actor pose restoration without replacing native special targeting. |
| ASX-11 | Preserve immunity target-pool loans and identity/lifetime cleanup. |
| ASX-12 | Preserve fatigue stacks and simulation-clock recovery with native aura presentation. |
| ASX-13 | Preserve Fortune rules and native aura/stat presentation; Sayge trials provide rewards when enabled, with encounter grants retained only as the disabled-trial fallback. |
| ASX-14 | Preserve the three economy-item exclusions; the pickup-radius trinket remains available. |
| ASX-15 | Reserved removed ID; no native dependencies. |
| ASX-16 | Preserve per-ring enchantment identity/ranks and scoped native stat loans. |
| ASX-17 | Enrich retained native party children and native owner-correct tooltips. |
| ASX-18 | Preserve native menu/card roots, assets, flags and participant ownership. |
| ASX-19 | Preserve run/session resets, slot plus GUID bindings and callback ordering. |
| ASX-20 | Align native-folder registry, importer and build metadata with the exact selected executable. |
| ASX-21 | Keep marks upstream-native; duplicate-loot gold compensation remains disabled. |
| ASX-22 | Preserve personal crystal claims, permanent Crystal Resonance totals and native stat loans; the former amulet gem is retired. |
| ASX-23 | Preserve ASXUI Lua HUD and nameplate presentation through native addon APIs; no DLL nameplate hooks. |
| ASX-24 | Preserve fresh-size damage text and native direct/periodic healing text through reviewed frame lifetimes. |
| ASX-25 | Preserve random native markers and manual reservations through the moved server marker table. |
| ASX-26 | Preserve optional goblin spawning, fleeing, bounds, equipment, amulet choices and whole native mark rewards. |
| ASX-27 | Preserve personal supplies, consumable/stat loans, materials, masked minimap pins and discovery UI. |
| ASX-28 | Preserve persistent Sayge trials, native scenes, group presence, Fortune replacement and progress UI. |
| ASX-29 | Keep 50% of positive AoE bonuses in coop, preserve base spell sizes and solo, and expose effective values plus native informational aura. |

Native achievements, sets, marks,
crowd-control rules, caster collision and ability caps remain native. A client
update does not authorize replacing those upstream mechanics with ASX rules.

## Validation scope

Exact-image checks, portable rules/memory tests, Lua behavior and Windows ABI
execution validate different boundaries. Use the full portable gate and the
relevant Windows checks in [testing](../development/TESTING.md). Generated instruction evidence,
logs and candidate reports stay in ignored local storage.

A new normal artifact and a separate bounded normal-client trial are required
before claiming runtime compatibility or release readiness. Source checks do
not validate an installed DLL, a downloaded kit or an unchanged historical build
reference. No build, deployment, publication or normal-client trial follows
implicitly from this document.

## 0.138.0 review

The exact descriptor retains Game/Actor scalar translations, the 0xb58 actor
context and 0x858 peer extent. Native enemy rows grow to 0x208; their elite flag
moves to +0x200. Identity, HP, aura and position offsets remain separate reviewed
fields. The native constructor and encounter iterator independently confirm the
row extent; multi-row and compaction fixtures exercise nonzero indices.

Direct and periodic healing now pass through separate extracted native helpers.
ASX observes their owning systems, before event destruction, with independent
copied-event, query, object-map and writer-parameter slots. The shared query and
GUID lookup ABIs remain unchanged. Damage text retains XMM8 sizing but requires
RBX for its message category/color. Windows stand-ins exercise that register
contract and preserve flags and unrelated SIMD lanes.

Menu class-grid and NPC appearance frames move independently. Goblin reward
Game storage moves to +0x198 and its pool to +0x1a0; the native count remains R15.
The same Game slot governs the economy-trinket filter. Downed-XP gates include
both the host consumers and the separately reviewed guest reward producers.
Native pause tag 7, input ownership and the reliable queue remain unchanged.

Native common-item modifier tags 43/44 become 47/48. ASX continues to pass their
native definitions to native formatting/stat routines. This port does not change
their coefficients. The native balance documents remain the explicitly recorded
0.135.1 reference; their values were not refreshed by this compatibility task.

All feature dispositions above and stable acceptance IDs remain applicable.
Artifact-scoped tests and live results are retained in ignored compatibility
evidence. Earlier 0.135.1 validation and the approved reproducibility reference
are historical evidence, not approval of a 0.138.0 artifact. Prepared .2 and .3
releases are preserved; this work does not publish or commit a release.

The nine retained pause-UI guards are now included in the selected descriptor
and verified before installation; presentation no longer duplicates their bytes.
Offline contact inventory includes both ends of the downed-XP detours and native
return sites, and resolves historical role keys before comparing images.

The Crystal Resonance core assessor checks the approved 12/7/15/12 nominal
inputs. Its diagnostic probe observes the inner stat rebuild, before the outer
coop AoE reduction. Effective native/HUD area values require their separate
observation; nominal crystal evidence does not establish that display claim.

The ranked-enchantment probe observes the completed outer rebuild: Reach must
therefore retain its full bonus in solo and exactly half in coop. The runner
accepts only the exact informational goblin taunt-refusal record as expected
policy; error-level, malformed and unrelated rejection records remain failures.
Missing live witnesses remain failed attempts even when a later retry passes.
