# Official 0.135.1 client boundary

The sole source target is the exact Windows x86-64 executable with SHA-256:

```text
71e5f1767279db4bd428d1cd9c5d50d37759b72295386a69f9e3f6e70bc59249
```

`bindings/official1351.rs` owns current native contacts, complete instruction
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

Enemy records use stride `0x200`. The selected schema separates GUID/death/target
from the moved aura, health, position and vulnerability fields. Every reader,
writer and retained fixture uses that schema, without changing native ownership,
crowd-control budgets, spawn rules or target policy for special abilities.

The NPC continuation uses XMM15 for distance. Temporary actor XY is at frame
`+0x70`; frame `+0x80` belongs to the native iterator and must not be overwritten.
Encounter handling reads the unique boss definition from `+0x908` and Game from
the selected `+0x918` frame slot. R13 holds the killed enemy GUID. Other frame origins stay descriptor-owned.
The basic-action Game pointer is at `+0x3b0`, with origin Z at `+0x284`.
Shield-break callbacks read Game from RBX and the spell from R15. The aura-row
pointer is at RBP+0x38 at the projection seam; periodic-heal UI payloads start at
RBP+8. These are callback-local lifetimes, not general stack translations.

The native compositor uses a 64-byte tagged armor key. The selected adapter
preserves its four-byte prefix and race byte. Metadata, appearance, entity and
armor-key frame locations are selected independently.

Fresh damage-number size now uses XMM8; R14 still owns the message. Direct and
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

Native achievements, sets, marks,
crowd-control rules, caster collision and ability caps remain native. A client
update does not authorize replacing those upstream mechanics with ASX rules.

## Validation scope

Exact-image checks, portable rules/memory tests, Lua behavior and Windows ABI
execution validate different boundaries. Use the full portable gate and the
relevant Windows checks in [testing](TESTING.md). Generated instruction evidence,
logs and candidate reports stay in ignored local storage.

A new normal artifact and a separate bounded normal-client trial are required
before claiming runtime compatibility or release readiness. Source checks do
not validate an installed DLL, a downloaded kit or an unchanged historical build
reference. No build, deployment, publication or normal-client trial follows
implicitly from this document.

## 0.135.1 review

The adapter is pinned to the exact executable above. Historical 0.133 descriptors
remain comparison references and cannot activate a client. Static review and
generated guards do not substitute for portable, Windows ABI and isolated live
qualification of the actual resulting DLL.

Game and Actor translations are composed independently over reviewed native
spans. Their common actor header grew, mutable scalars moved before the derived
stats block, and one native stat was inserted. The two complete native swap
leaves were executed sequentially with independent byte sentinels: all 3,060
translated Game bytes and 2,341 translated Actor bytes agree. Four bytes of retired
native padding remain unmapped. Direct native XP,
gold and health consumers independently identify the corresponding scalar fields.
The peer-row stride stays 0x858; server role, stream, sink and mark fields move by
0x88. Private Actor contexts occupy 0xb58 bytes.

Enemy rows occupy 0x200 bytes. Their constructor preserves the selected HP,
position, identity, aura and vulnerability offsets; the elite flag moves to
0x1f8. Movement uses separately reviewed stack locations. Aura-row output, native
encounter Game storage and the goblin amulet pool have distinct frame changes.
The pool's live count is R15; its callback preserves the other native registers.
Goblin creation and handle rebinding use the same selected 0x200 enemy stride,
including nonzero array indices and compaction; no legacy 504-byte stride remains.

Native item-modifier discriminants 37/38 become 43/44; their coefficients remain
native. ASX does not overwrite these upstream data or reinstate the removed spell.

Both executable identity and every selected entry guard remain mandatory. All
eleven native archives match the previously reviewed data set. Native balance,
encounter schedules, native marks and simulation ownership remain upstream-owned.
The historical 0.131.0-asx.3 reproducibility approval is unchanged. This port does
not publish a release or alter an installed player client.
