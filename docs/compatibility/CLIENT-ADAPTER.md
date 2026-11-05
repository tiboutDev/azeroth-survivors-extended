# Selected native client contract

The [client registry](../../azsx/supported-games.json) and
[current client notes](CLIENT-COMPATIBILITY.md) identify the supported image.
`SelectedAdapter::select` accepts its exact SHA-256. Version labels and approximate
instruction matches do not select another client.

## Ownership

| Layer | Responsibility |
| --- | --- |
| `azsx-gameplay` | Portable rules, identities and copied semantic values |
| `azsx-game/src/official.rs` | Facade used by the DLL |
| `azsx-game/src/official_native/` | Native schemas, memory IO, calls and hook contracts |
| `compat/clients/`, binding template | Reviewed native literals and generated selected descriptor |
| `src/runtime/` | Coordination, lifecycle and bounded production service bindings |

Production consumers use `azsx_game::current`. Its generated translations are
self-contained; historical descriptors are test references, not selectable clients
or chained production fallbacks. Offsets and native container layouts stay inside
the adapter. Services expose operations and copied/owned results rather than
Runtime, memory accessors or mutable backing containers.

## Complete wiring review

For a port, review all consumed contacts and schemas before candidate DLLs or live
trials. Include entry guards, callers, register/stack carriers, return values,
vector strides, enums, field extents, source/recipient ownership and cleanup.
Instruction similarity is a candidate match, not ABI proof.

`compat/field-uses.json` pins indirect arguments and host/private namespaces to
Rust method fingerprints. `field_audit.py` covers closures, arrays and UFCS/free
calls as well as direct calls. It rejects missing or stale field reviews. Raw
native buffers and call semantics still need independent expected-output checks.

Generate with `binding_registry.py --write`; verify generated bindings and the
feature inventory with `azsx_dev.py check`. Generation never discovers an ABI.
Current layout details belong in [CLIENT-COMPATIBILITY.md](CLIENT-COMPATIBILITY.md),
not in copied historical instructions throughout the docs.

## Calls, identities and reentry

Native callbacks can reenter synchronously. Copy observations or reserve/remove
pending work before a native call. Release state locks, RefCell borrows and map
entries; preserve FIFO/key order, mutation-before-call order and partial commits.
Never retry an uncertain grant or replace the whole runtime to avoid borrowing.

Bind actors by slot plus GUID and run/session lifetime. Reject inactive or reused
identities, rebind after callbacks and restore borrowed contexts on every exit.
Native five-player capacity remains unchanged. Private support route tags use
three low bits for the slot and bit 3 for hybrid group delivery; reject invalid
sixth-slot targets.

Game, private Actor and inherited canonical field namespaces are distinct. Verify
both ends and the complete byte extent of a translated span. A common shift,
matching start key or valid pointer alone does not establish a layout.

Temporary native inputs are loans: preserve unrelated state, reconstruct through
the native path and restore against current validated storage. A rejected cast
may shorten a vector; rollback must use its capacity and re-resolved owner, not
assume the previous length or pointer remains valid.

## Native call shapes

Unit fields carry opaque 32-bit payloads. Select the reviewed integer-register or
raw XMM-bit ABI without arithmetic conversion. A two-word setter would also change
the neighboring field. Windows stand-ins verify payload bits and preservation.

Incoming Hurt and native healing have independently selected source descriptors.
Deferred hits copy borrowed metadata and custom text before native storage is
reused, then build a call-scoped descriptor. Refused observations retain original
native delivery; fallback keeps the same source. Preserve every native argument,
including damage kind, critical/proc scope and attribution.

Runtime abilities, learned rows, cast arguments and owner formatter inputs have
separate extents and rank/modifier fields. Validate native construction, lookup
and destruction; do not reuse a layout merely because two rows look alike.

Perk observations normalize only reviewed variants into owned copies. Those copies
never go back into the native perk vector or dispatcher. Unknown/new variants
stay native-owned. Stat-only rows must not enter effect dispatch.

Basic pursuit and final movement use independent selected carriers. Modify only
the intended scalar/candidate, preserve the other registers/lanes/flags and restore
borrowed origin poses before later native logic. Ordinary NPC ownership, AI,
scaling, schedules and allocation remain native.

## Effects and progression

- Foreign HoTs, shields, retaliation and death bursts keep their producer,
  beneficiary, native coefficients, counters and meter ownership. Do not run a
  foreign heal through the recipient's outgoing talents.
- Strongest same-spell shields keep the retained pool's producer and native
  refreshed expiry, including auxiliary pools. A foreign refresh does not acquire
  ownership of an untracked native personal shield. Loans preserve unrelated absorb.
- Tranquility uses the selected native raw-healing budget, including overheal.
  Preserve its native throttling and critical/proc behavior. Separate healing-meter credit from damage
  fuel; retain RNG changes across nested callbacks while restoring borrowed scope.
- Polymorph borrows its living producer only around the native burst. Keep the
  triggering hit with the attacker, restore context before later on-hit effects
  and retain native eligibility checks. Do not duplicate the burst or Faerie Fire.
- Set/necklace clone hooks select their Game, event, identity and count carriers
  independently. DoT owner writes retain native sentinels and already assigned
  owners. Vulnerability reads/writes use one coefficient/expiry pair.
- Dead participants gain native shared XP and whole-level receipts immediately;
  native choice eligibility still waits for revival. Review producers, queued
  consumers and phase continuations independently. Preserve raw XP carriers and
  rounding; whole-level deficit awards do not enter ordinary XP normalization.
- Personal reward acknowledgements commit before another actor's callback-capable
  grant. Preserve card order and native currency/stat reconstruction. No XP bank,
  duplicate loot compensation or replay of an already granted receipt.

## Retained presentation and hooks

Use native owner formatting, rank and identity for tooltips. Keep its owned text
intact; missing/ambiguous data is hidden rather than replaced by viewer values.
Publish the addon provider through the selected virtual script reader, preserving
native initialization and string allocation. AZSXUI is an optional consumer;
no internal installed addon or MPQ scanning is required.

Native text categories, direct/periodic event tags, GUID lookup predicates and
message writers retain separate contracts. A similar shield/heal emitter prologue
can produce the wrong effect. Check real native construction and callers. Holy
Nova and Holy Shock retain native behavior; retired hybrid group routes stay off.

`src/install.rs` owns one hook per native target, including optional batches.
Rollback affects only the failed batch. Guards accept a recorded installed prefix
only at its owned address with the reviewed original and unchanged tail. Shared
presenters forward through that registered entry; they do not install a second hook.
Unknown jumps remain invalid, and teardown clears ownership.

## Validation scope

Use [TESTING.md](../development/TESTING.md) and the
[compatibility checklist](COMPATIBILITY-CHECKLIST.md). Review every feature's
active, disabled or upstream disposition. Code checks establish wiring/contracts;
exact-artifact live observations establish the remaining integration behavior;
visual claims need inspected captures. Keep these scopes and any failures explicit
without repeating already valid checks.
