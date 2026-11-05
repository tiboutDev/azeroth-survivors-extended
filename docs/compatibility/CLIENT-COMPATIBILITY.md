# Official 0.141.0 client boundary

The sole source target is the exact Windows x86-64 executable with SHA-256:

```text
6ca5f4ba4eb6b26716e4b7d9e03cbd25ddafe7582dcd8e55333e070a28058afa
```

`bindings/official1410.rs`, generated from `compat/clients/0.141.0.json`, owns
native contacts, complete instruction guards, field translations and carrier
choices. The launcher registry and private importer use the same identity and
include `Data/patch-C.MPQ`. Similar code or a version label never enables an
unknown executable. Direct native launch remains inert.

## Review boundaries

Exact-image tests execute native producers and consumers independently of the
AZSX policy tests. Compare call shapes, ownership, fields and continuations as
separate contracts. The AST field-use gate also checks closures, looped keys,
selected arrays, free/UFCS calls and host/private paths; it cannot replace native
execution or prove a raw buffer schema. A resolved address without its retained entry guard is
incomplete. The pinned-image suite also checks that retained guards cannot
silently disappear while address tables are migrated.

Portable gates, Windows stand-ins and live trials have separate artifact-scoped
receipts. Source selection is not an approved release artifact or proof of a
rendered frame. Keep the existing approved normal-DLL reference unchanged until
a new artifact is explicitly approved; see [reproducibility](../release/REPRODUCIBILITY.md).
Compatibility updates require diagnostic solo and two-player scenarios followed
by a separate normal-artifact trial, as described in the [checklist](COMPATIBILITY-CHECKLIST.md).

## Native calls and layouts

- The Game/private Actor insertion at +0x2a8 moves the consumed later fields by
  24 bytes; the independent native swap confirms both namespaces. Saved profile
  fields move from +0x50 onward. Specialization appearance reads its selected
  profile fields rather than retaining old literals.

- Unit-field insertion carries opaque value bits in R8D. Preserve those bits;
  the two-word setter is a different operation and must not overwrite a neighbor.
- At Hurt’s absorb-clear seam, R15 holds the consumed shield spell; R12 is the
  incoming damage kind. Those carriers must remain distinct.
- Incoming Hurt now borrows a source enum as argument five. Deferred damage
  copies any custom source text and reconstructs a call-scoped descriptor.
  Healing retains its separate five-argument metadata/source contract.
- Runtime and cast abilities are 96 bytes; the learned book uses 104-byte rows.
  The cast's rank is at byte 48, power at 56 and its 40-byte modifier block at 8.
  The native owner formatter reads that rank field and returns 176 bytes of
  optional owned strings. Do not substitute a learned-row layout or viewer text.
- Native perk tags are u32 and selected payloads are reordered. Normalize only
  bounded observations. Dispatch and cloned rows retain their native bytes.
  New Tier 0.5 effects remain native, including tags beyond the former stat-row
  boundary. Stat-only rows retain a u32 stat kind and never enter effect dispatch; never classify them using the old one-byte enum.
- Game, private Actor and inherited canonical field namespaces remain distinct.
  XP receipts use their own 16-byte rows and preserve earlier recipients. Chest
  price, opened-cache count and pending-item banks have separate host/private
  mappings. Character race and gender are separate bytes in both contexts.
  The native swap exchanges 3,094 bytes of the 0xc20-byte private actor, preserving
  its padding. Review each consumed field independently. The old source-state
  word is unused by the strongest-shield path and deliberately has no new mapping.
- Native strongest same-spell shields retain their stronger pool. Cooperative
  source loans preserve that producer, unrelated absorption and rollback capacity.
  Tranquility counts raw healing, including overheal at full HP or a capped
  Tree of Life shield. The shield itself still respects its native cap. Native critical rolls and attribution run.
- Divine Aegis and Tranquility use different radius-query return sites. Trace the
  real native talent path rather than picking the first similar query in a large
  dispatcher. The six-argument query borrows its exclusion slice on the stack.
- Basic pursuit's distance is spilled at frame +0x3a0. The final goblin movement
  candidate uses XMM10 for XY and XMM14 for Z. Change only selected carriers;
  native movement, terrain, speed and AI remain authoritative.
- The native target picker returns the actor slot separately from its pose.
  The enemy stores that slot at +0x118. Creature classification is at +0x220;
  the encounter variant starts at +0x30. Verify actual ordinary, guardian and
  boss producers before applying those classifications.

- Goblin snapshots and retirement use selected native HP, death and motion fields.
  A fresh live spawn must not resolve as a kill because a retired field is zero.
- Diagnostic encounter selection reads the selected optional encounter ID;
  guardian selection requires a live non-boss row with the native optional guardian
  context. Retired field positions must never select ordinary enemies or AZSX event
  creatures for diagnostic kills. Fixtures include conflicting retired-field values.

## Rewards, progression and events

- Effect XP retains its raw amount in R15 rather than R12. Classify awards at
  the actual return address and preserve native rounding, shared-receipt inputs
  and the original behavior of unrecognized callers.
- Dead participants receive shared XP and whole-level receipts immediately.
  Seven independently guarded phase checks retain their own continuations;
  native Playing-only gates still control opening reward choices. Do not replay
  receipts after revival or traverse an actor vector while it is lent out.
- Native choices keep their capacity/pointer/length ordering. Exercise offers,
  acceptance, paid/free rerolls, insufficient funds, skip/conversion and the
  existing ring/amulet identities separately. Commit personal acknowledgements
  before another actor's callback-capable grant.
- The native jewelry pool still places `insignia`, `eldre_thalas` and `chosen`
  at indices 16, 17 and 18. AZSX excludes those entries and retains the pickup
  trinket. The loop's Game spill is frame +0x198; its current total length must
  not be mistaken for the last excluded index.
- All 13 native map rows are 240 bytes and each normal boss definition is 104
  bytes. Scholomance is inserted before Ahn'Qiraj. Late maps must remain within
  native placement, guardian, drop and mark-table bounds. Schedules, new boss
  mechanics and native balance changes remain native.
- Encounter completion carries Game in R15 and the definition at frame +0x310,
  with an independent Game spill at +0x338. Observe the native pre-transition
  callback; do not widen it to another new boss-controller branch by proximity.
- Scene creation, unit fields, per-participant delivery, terrain options and
  spline sequencing retain their distinct ABIs. Creation logs do not prove
  visible models or hand equipment. Native liquid samples still reject event
  placement even when navigation would otherwise allow the location.
- The unfinished Banished Demon event remains disabled in normal builds. Its
  retained preview uses independently guarded Banish contacts and native kits;
  reviewing those contacts does not enable the event.

## Retained presentation and profiles

- Minimap projection reads the selected borrowed UI context, independently of
  its self/party aliases. Native center, ping, visibility and indoor/outdoor fields
  have their own layout; projection radii match the native tables.
- Owner identity uses native self/party aliases and current slot plus GUID.
  Native downed-member/world-anchor APIs provide run/map-scoped coordinates;
  they do not replace native resurrection, target frames or transport. The
  exact-image Lua check executes the embedded native API with AZSXUI to verify
  downed-row copies, map/space scoping, projection request arguments and frame
  handler cleanup. Its low-level camera primitive is a stand-in, so it does not
  establish rendered placement.
- Aura export uses the actual 48-row allocation via frame +0x38. Aura expiry
  is an absolute simulation timestamp. Fractional healing observes the final
  clamped gain, with separate native source carriers for direct and periodic paths.
- Run-info missing drops come from native Armory loot and item constructors for
  every boss and class in all 13 zones. Their vectors are caller-owned. Returned
  item names and bonus vectors are owned; icon data belongs to the native catalog.
  Initialize that shared catalog before assessing per-call allocation balance.
- Specialization appearance uses native Look precedence, collection indicators
  and current tree keys. Existing model mappings cover the previously reviewed
  sets. New unmapped sets, including Tier 0.5, retain native appearance; do not
  invent a model or reuse a neighboring set's mapping during a client port.
- Native profile parsing retains rank, prerequisite and collection authority.
  Disposable fixtures require a separately reviewed executable allowlist and
  native parser checks; a codec roundtrip alone is insufficient.
- Codex discovery uses native sorted insertion and preserves a higher existing
  rarity. Reopening the same profile must not create repeated grants or saves.
  Native load migrations, serialization and disk writes retain their owners.
- The retired talent-selection injector is rejected before launching a trial.
  Production Co-op notes still enrich the retained native talent window. Visual
  qualification uses that window and requires explicit visual-trial scope.

Private images, disassembly, machine paths, mutable results and failed attempts
belong in ignored evidence. They are not release payload or reusable instructions.
