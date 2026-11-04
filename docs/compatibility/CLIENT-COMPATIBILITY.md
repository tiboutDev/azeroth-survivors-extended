# Official 0.139.2 client boundary

The sole source target is the exact Windows x86-64 executable with SHA-256:

```text
0bb6c557c5f6cfeea212761a33554f9a5647ce7cb7e03b8a163b95489496e908
```

`bindings/official1392.rs` owns current native contacts, complete instruction
guards, field translations and register/frame choices. Version labels and
similar instruction sequences cannot select another executable. The launcher
registry and private importer use this same exact identity. Direct native launch
remains inert; the retired custom engine is not a fallback.

## 0.139.2 qualification

The normal DLL passed solo play and a two-player host/join run on this executable.
The diagnostic DLL passed solo and co-op core, support, pause, revival marker,
and both event suites. Private trials used normal priority and all available CPUs.
Both players claimed supplies, completed the ritual and received their goblin
rewards. The guest observed the escaped and killed goblins leave the scene.
The revival marker showed the downed ally's name and distance and disappeared
after native revival. These results are scoped to the tested source and binaries.

The candidate executable embeds build stamp `v0.2.0-1039-g5cb6deb36`
and SHA `5cb6deb3643ec2c7f3922bd4fd68eb26020a20ff`. Benilla's public
`v0.3.0` sources help identify engine UI and presentation code. The stamp
identifies the game build. Its upstream merge base remains unconfirmed.
[Benilla v0.3.0 source](https://github.com/samwhosung/benilla/tree/68734926a981301ead72b204f13e2be083845067)

- The run-info missing-drops tooltip uses the native Armory boss-loot and item
  lookups. Candidate checks cover the collection's owned-ID vector, the returned
  names, row bounds and native deallocation contract for three classes and zones.
- Retained pause callbacks still reach the native pause checks, label and button
  builders. The visibility byte remains at frame +0x2fe and the title font
  value at +0x470. Both recovery-button call sites resolve to their selected
  constructors.
- Shared award classification uses the return address of the native award
  call. The 0.139.2 local skip and automatic gold conversion return at
  0x5ec3549 and 0x5f76e19. Anchoring on a nearby instruction misclassifies
  their gold as a world drop. The XP callers preserve the raw-amount registers
  consumed by AZSX.
- The specialization look maps learned talent pointers through nine
  build-specific tables. Compare the ordered keys of every tree when
  updating the descriptor. Both projections use the visible-item resolver's
  returned pointer. The candidate preview borrows the collection from stack
  +0x48. The run and preview buffers moved independently.
- Party enrichments still append to the second addon line, AZSRUN, and read
  native spell names, ranks and descriptions from the actual ability formatter.

- Peer rows expanded from 0x858 to 0x870. Review each field against its owning
  structure. Some Game fields moved by 0x18 while adjacent groups kept their offsets.
- The host's peer pointer and count are at Server +0xfb0 and +0xfb8. Outgoing
  packets look up the assigned slot at peer +8 and require an active row and
  the stream flag at +0x86d. Incoming commands identify a row by vector index.
  In an isolated native run, closing the guest reduced the host's count from
  one to zero. Its private actor still had the same GUID and entered phase 4.
  Keep network delivery and downed actor state separate.
- Check all 13 co-op talent notes against native talent key headers. Retained
  reward widgets must keep their native row allocations and constructor arguments.
- Check Polymorph eligibility at the actor-mode load and the ownership scan.
  The actor mode moved from DIL to R10B. The set DoT builder has five supported
  return sites with three caller layouts. Verify all five before attributing damage.
- Trace the fatigue row pointer from the 48-row allocation to the harmful aura
  slots. On this image the pointer is at RBP+0x10. Trace the fractional healing
  packet from construction through insertion into the 32-byte-row queue at
  Game+0x700. Verify that a grenade appends exactly one projectile carrying the
  AZSX token, native spell ID and additional flight delay.
- The 0.139.2 binding manifest records the reviewed addresses and field
  ranges. The complete native swap exchanges 2,901 bytes of the 0xb58-byte
  private context in both versions. Its last three bytes stay untouched.
  Cross-image tests check the mapped host fields against the bytes actually
  moved. Both native Game constructors now cover the previous host field table.
  The object grows from 0xf50 to 0xf68 with a 24-byte insertion at +0xcf0.
  Three conditional float payloads require checking their active load/store
  instructions because the constructors leave those bytes unwritten.
  The host table covers all 3,060 previous bytes. The two historical lookup tables also cover
  their previous 3,128 and 3,152 bytes. Distinct settings bytes passed into
  both constructors and a native append to the relocated 40-byte-row vector
  distinguish the last fields from adjacent storage. The descriptor generator refuses a partial manifest.
- Native heal-text lookup returns the complete entity handle. Its eligibility
  query rejects an absent GUID, a recycled entity generation and ineligible
  components. The native Hunter's Mark visual helper updates one living enemy
  and queues a change only when its aura is added or removed.
- The goblin compositor carries the entity in R12 at both the metadata and
  armor-key callbacks. Its atlas, appearance and armor key begin at frame
  +0xc40, +0xc58 and +0x680. Check the individual operands before installing
  these hooks. A two-player trial confirmed the native armor compositor and
  the visible equipped model beside its projected Treasurer anchor.
- The UI event sink copies its borrowed text and length into an owned string
  in the native ring. The native reward callbacks have now been exercised through a paid reroll,
  three goblin amulet offers, acceptance by index and cache conversion. The
  amulet pool hook receives the active Game at frame +0x198, the weighted vector
  at +0x1a0 and its length in R15. The exclusion callback borrows 24-byte
  string headers. Once a card is accepted, its key becomes the active amulet.
- The goblin spawn hook substitutes the creature definition in R15 only for the
  requested Game in RSI. The following ordinary spawn retains its own entry.
  The final movement callback still observes XY in XMM13 and Z in XMM14.
  The source altitude moved from frame +0x26c to +0x24c. The native recycling
  iterator returns a list of distant enemy GUIDs for AZSX to filter.
- Both nameplate style callers use the vector at +0x4778 (previously +0x4b18).
  The main-hand rendering loop keeps the slot in R12B and the entity handle,
  including its generation, in RBX. Both entity-loading paths reach the item
  resolver with the same handle.
- The direct and periodic UI handlers use tags 0x97 and 0x96. Their callees
  emit HEAL_CRIT and PERIODIC_HEAL respectively. The periodic handler reads
  20-byte rows. Trace UI events to their logger before choosing a hook because
  unrelated wrappers share the same epilogue and wire tags belong to another layer.
- Native coinage still uses field 0x498. The generic single-field setter remains
  callable. The native gold-award path now updates existing keys inline, so a
  detour on the generic setter alone cannot observe every balance change.
  Chest purchase and reroll already reconcile the balance after the native call.
  The two-player wallet scenario verified single debits for cache purchases and
  paid rerolls, unchanged balances on free rerolls, single skip credits, and
  matching native balances on both clients.
- The retained UI payload begins at Rc +0x18. Its unit aliases now begin at
  payload +0x3290 (previously +0x2f10). Self is the u64 at aliases +0x60.
  Four party GUIDs start at +0x88 and four pet GUIDs at +0xa8. The native
  formatter distinguishes all eight slots. In a two-client native run on the
  map, both clients reported `UnitIsUnit("player","party1")` as false even when
  the displayed names matched. Empty party slots did not resolve.
- Event creation takes Game, an entry/display/scale descriptor, XYZ, object
  type and flags. The native constructor increments Game +0xed4, returns its
  0xf110 object GUID, and appends one 32-byte 0xa9 packet to the Game vector at
  +0x700 (capacity), +0x708 (pointer), +0x710 (length). The 0x5f metadata must
  precede creation. An offline execution test inspects both the serialized
  local packet and the connected peer's queued message. A second emulated
  server executes the guest dispatcher through its local transport boundary. The destroy packet carries the full created GUID to the guest alone
  for a personal hide, or to both transports for a scene removal. Two-client trials
  cover independent crate claims, ritual completion and scene cleanup.
- Execution of the scene-unit builder, field-map destructor and update packer
  confirms the borrowed field map, consumed update block and owned packet on
  the candidate. The notice, creature metadata, escort spline and Sayge fixture
  produce the same bytes as 0.138.0. Terrain sampling distinguishes half-yard
  cells and rejects out-of-bounds coordinates. The navigation clip stops at a
  blocked cell and preserves an unobstructed goal.
- The allocation thunks use the Windows process heap. The native free path
  recovers the original allocation for alignments above 16 bytes. Offline tests
  execute both sides with the imported heap functions isolated.
- Both bootstrap detours have standalone address and guard constants. Include
  these in offline inventories. The initialized virtual reader returns an owned
  copy of the cached script. The real lobby writer queues a hero message and a
  separate host-room message with the supplied name. These tests use a private
  cache and local queue. The isolated clients initialized the addon and joined a real AZSX party.
- The twelve map names, five bosses per map, event bounds and marks factors match
  the reviewed image. Seven downed-XP branch and resume pairs are recorded.
  Native direct and periodic heal consumers preserve the event, object map,
  query and writer slots used by the adapter. Text buffer growth preserves
  56-byte rows, and packet queue growth preserves 32-byte rows.
- Executing the native keyboard enqueue and apply paths on a constructed Game
  changes the phase from playing to paused and back. The guest filter consumes
  the pause action only for its active stream. Sending the AZSX pause opcode
  queues the epoch and state in the native reliable envelope.
- The damage renderer reads the category and color from R14, with fresh size in
  XMM8. The 0.138.0 hook used RBX. Select the row register independently of the
  size register. Windows stand-ins exercise both row registers with a decoy row.
- Codex insertion retains an existing higher rarity when adding the three
  excluded economy trinkets. The native save entry and collection argument
  were identified offline. Normal solo launches on new and existing disposable
  profiles verified persistence, preserved higher rarities and left the second
  save byte-identical.
- Native world-anchor projection returned current on-screen coordinates in the
  isolated client. A real two-player knockdown showed the ally name, distance and revive
  icon. The icon disappeared after the native revival.

Point `AZSX_REVIEW_IMAGE_1392` at the private executable to include candidate
image checks during portable testing. Those checks inspect the binary offline.

## Selected layouts

Host Game and private actor fields have independent translations. Private actor
stride is `0xb58`; native peers use stride `0x870`. Native capacity remains five.
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
Game in R13 against the selected `+0x958` frame slot. The old definition slot
now holds an unrelated log-queue reference and must not be read as a boss.
The basic-action Game pointer is at `+0x3b0`, with origin Z at `+0x24c`.
Shield-break callbacks read Game from RBX and the spell from R15. The aura-row
pointer is at RBP+0x10 at the projection seam; periodic-heal UI payloads start at
RBP+0x2d8. These are callback-local lifetimes, not general stack translations.

The native compositor uses a 64-byte tagged armor key. The selected adapter
preserves its four-byte prefix and race byte. Metadata, appearance, entity and
armor-key frame locations are selected independently.

Fresh damage-number size uses XMM8 and R14 owns the message. Direct and
periodic heal wrappers have separate reviewed frames and event discriminators.
The native entity resolver is now a query predicate plus a map lookup borrowing
a pointer to the map reference. Healing/meter calls additionally require a fifth
borrowed source descriptor; synthetic AZSX healing uses the native unspecified
source tag, while Hurt forwarding preserves both native source stack words.

Native pause input retains tag 7 and its boolean payload. AZSX menu ownership
uses the existing native reliable transport, with selected role/stream/peer
layouts and unchanged Win64 argument positions. Native reliability and recovery
remain upstream responsibilities; AZSX does not restore its retired transport.

Support deliveries retain 56-byte rows, actor auras retain 32-byte rows including
the native producer GUID, and perks retain 64-byte rows. Foreign periodic effects
keep producer/recipient ownership and original native accounting. AZSX extends
only the retained AZSRUN row in the native addon output, preserving other rows.

The 0.130 Death Recap remains native, including its modal close/Escape route.
AZSX retains unknown native HUD fields and only enriches its owned presentation.
Amulet target milestones also remain native; gem stat inputs do not reconstruct
target schedules or restore the removed rounding-only target.

Event clock, terrain, endless and mark fields are selected descriptor constants.
Unconsumed context gaps without a reviewed translation fail closed rather than
being covered by a guessed global layout shift.

## Feature dispositions

These are source dispositions for [the stable inventory](AZSX-FEATURES.md), not
claims that every feature has been exercised in a normal running selected client.

| ID | Disposition for the selected client |
| --- | --- |
| AZSX-01 | Exact hash, native guards and explicit launcher activation; reject other clients. |
| AZSX-02 | Preserve receipt scopes, raw-register sources, caller classification and rounding. |
| AZSX-03 | Preserve host wallet policy with independently translated balances and earned-gold fields. |
| AZSX-04 | Preserve one debit, participant identities and once-only personal native choices. |
| AZSX-05 | Keep guardian rewards native; no supplemental item grant. |
| AZSX-06 | Preserve choice/menu barriers over the native pause and reliable command paths. |
| AZSX-07 | Preserve allied support eligibility, native coefficients and producer attribution. |
| AZSX-08 | Preserve shield/proc accounting, foreign retaliation and periodic ownership. |
| AZSX-09 | Preserve strongest Hunter's Mark and native visuals through selected enemy fields. |
| AZSX-10 | Preserve eligible taunts and actor pose restoration without replacing native special targeting. |
| AZSX-11 | Preserve immunity target-pool loans and identity/lifetime cleanup. |
| AZSX-12 | Preserve fatigue stacks and simulation-clock recovery with native aura presentation. |
| AZSX-13 | Preserve Fortune rules and native aura/stat presentation; Sayge trials provide rewards when enabled, with encounter grants retained only as the disabled-trial fallback. |
| AZSX-14 | Preserve the three economy-item exclusions; the pickup-radius trinket remains available. |
| AZSX-15 | Reserved removed ID; no native dependencies. |
| AZSX-16 | Preserve per-ring enchantment identity/ranks and scoped native stat loans. |
| AZSX-17 | Enrich retained native party children and native owner-correct tooltips. |
| AZSX-18 | Preserve native menu/card roots, assets, flags and participant ownership. |
| AZSX-19 | Preserve run/session resets, slot plus GUID bindings and callback ordering. |
| AZSX-20 | Align native-folder registry, importer and build metadata with the exact selected executable. |
| AZSX-21 | Keep marks upstream-native; duplicate-loot gold compensation remains disabled. |
| AZSX-22 | Preserve personal crystal claims, permanent Crystal Resonance totals and native stat loans; the former amulet gem is retired. |
| AZSX-23 | Preserve AZSXUI Lua HUD and nameplate presentation through native addon APIs; no DLL nameplate hooks. |
| AZSX-24 | Preserve fresh-size damage text and native direct/periodic healing text through reviewed frame lifetimes. |
| AZSX-25 | Preserve random native markers and manual reservations through the moved server marker table. |
| AZSX-26 | Preserve optional goblin spawning, fleeing, bounds, equipment, amulet choices and whole native mark rewards. |
| AZSX-27 | Preserve personal supplies, consumable/stat loans, materials, masked minimap pins and discovery UI. |
| AZSX-28 | Preserve persistent Sayge trials, native scenes, group presence, Fortune replacement and progress UI. |
| AZSX-29 | Keep 50% of positive AoE bonuses in coop, preserve base spell sizes and solo, and expose effective values and the co-op help button. |

Native achievements, sets, marks,
crowd-control rules, caster collision and ability caps remain native. A client
update does not authorize replacing those upstream mechanics with AZSX rules.

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
AZSX observes their owning systems, before event destruction, with independent
copied-event, query, object-map and writer-parameter slots. The shared query and
GUID lookup ABIs remain unchanged. Damage text retains XMM8 sizing but requires
RBX for its message category/color. Windows stand-ins exercise that register
contract and preserve flags and unrelated SIMD lanes.

Menu class-grid and NPC appearance frames move independently. Goblin reward
Game storage moves to +0x198 and its pool to +0x1a0; the native count remains R15.
The same Game slot governs the economy-trinket filter. Downed-XP gates include
both the host consumers and the separately reviewed guest reward producers.
Native pause tag 7, input ownership and the reliable queue remain unchanged.

Native common-item modifier tags 43/44 become 47/48. AZSX continues to pass their
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

The Crystal Resonance assessor checks personal damage, cooldown recovery and maximum health. A yellow pickup must add 8% Luck per crystal to the collecting actor, with critical chance and area unchanged.

The ranked-enchantment probe observes the completed outer rebuild: Reach must
therefore retain its full bonus in solo and exactly half in coop. The runner
accepts only the exact informational goblin taunt-refusal record as expected
policy; error-level, malformed and unrelated rejection records remain failures.
Missing live witnesses remain failed attempts even when a later retry passes.
