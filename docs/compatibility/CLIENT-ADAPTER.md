# Selected native client boundary

AZSX source supports only official **0.138.0**. `SelectedAdapter::select` accepts
only the reviewed executable SHA-256. A version label or a similar instruction
sequence cannot enable another client. Selection does not bypass entry-byte,
image-bound or native-layout verification.

## Ownership

| Layer | Responsibility |
| --- | --- |
| `crates/azsx-gameplay/src/` | Portable rules, identities and copied semantic values |
| `crates/azsx-game/src/official.rs` | Selected native facade used by the DLL |
| `crates/azsx-game/src/official_native/` | Native translation, memory validation, call shapes and hook contracts |
| `crates/azsx-game/src/bindings/official138.rs` | Exact current-image identity and descriptor choices |
| `crates/azsx-game/src/memory.rs`, `enemies.rs` | Neutral memory interface and exact enemy parsing used by current support |
| `src/runtime/` | Feature coordination and bounded production service bindings |

Active native consumers resolve descriptor data through the `azsx_game::current`
re-export. The alias currently points to `official138`; it does not select or
authorize an executable. Exact hash selection remains explicit in the adapter.
Event appearance stack slots and native marks tables also belong to that
descriptor, rather than to feature coordination.

The selected descriptor is generated from `compat/clients/0.138.0.json` and
`compat/binding-template.rs`. Its field translations are resolved, including
explicit invalid gaps; they do not invoke older descriptors. Historical modules
are test-only comparison references. Reviewed canonical keys are retained where
call sites require them, with translation owned by the current adapter.

Run `python3 tools/dev/azsx_dev.py check` to verify generated source and the feature
inventory. Generation never discovers or approves a new ABI. Exact selection,
entry guards, call-shape review and private-image checks remain required.

## Native calls and state

Native game callbacks can reenter synchronously. Copy observations or remove
pending deliveries before calling native code; never retain a mutable runtime
borrow, gameplay-state mutex or map entry across a callback-capable call. Preserve
partial commits, mutation-before-call order, slot plus GUID identity and cleanup
when a callback rejects or fails. An uncertain native commit must not be retried.

Keep offsets, native row/register layouts and native helper addresses in selected
game access. Feature services expose bounded operations and copied/owned results;
they do not expose Runtime, Local, mutable containers, Deref or Any escapes.
Session has separate command, world, update and HUD ports. Other feature modules
still have direct Runtime dependencies: local service extraction is not a claim
that every domain has a complete native-free API.

The reviewed Hurt call has six arguments: Game, amount, enemy, melee flag,
native source kind and native spell. The final two words occupy Windows x64
stack arguments five and six; the native consumers read both. The selected
adapter owns `NativeHurtCall`. Original forwarding and queued taunt redirection,
including fallback to the original actor, preserve both words. Generated support
hits use an explicit source kind and spell zero. The support-call interface
requires the tagged implementation so an adapter cannot silently discard them.
A Windows stand-in verifies independent nonzero values for both stack words.
Dead-actor progression uses seven separately guarded native phase checks: two
shared-XP consumers, two whole-level receipt producers and three queued
whole-level reward consumers. The selected adapter owns their
registers and continuations. Only an authoritative coop host with a bounded
current slot and nonzero GUID may rejoin native scaling/award after the dead
check. No actor-vector traversal occurs while it is lent out. Native award
updates XP, levels and pending choices without changing the dead phase; native
Playing eligibility still controls opening choices. No deferred bank is replayed.

The reviewed 0.133 healing/meter consumer additionally reads a fifth argument:
a borrowed native source descriptor. `call_native_heal` supplies the native
unspecified-source tag used by lifesteal, allowing native attribution from the
active actor and spell. It preserves the four existing values and bounds the
source lifetime to the call. The earlier 0.131 callee ignores the extra argument.
The Windows stand-in checks the source pointer contents as well as the original
arguments. This contract is independent of executable activation and live proof.

The retired lobby menu-cache extension has no runtime consumer. Its two unused
facade/adapter seams are removed rather than becoming 0.133 compatibility
dependencies. Retained reward-card construction and owner tooltips keep their
separate active entries.

At the reviewed Hurt absorb-clear seam, RBX owns Game and R15 owns the
consumed shield spell. R12 still holds the incoming damage kind and must never
be used as that spell. A register fixture distinguishes the two values explicitly
and checks the clamped absorbed amount. The 0.131 descriptor is corrected as well;
the same register ownership was independently checked in the 0.133 consumer.

World-number rendering selects its fresh-size register explicitly: XMM7 in
0.131 and XMM8 in the reviewed 0.133 renderer, with R14 still owning the entry.
The Windows probe preserves both registers' upper lanes, the other size register,
flags, non-damage categories and healing. Native healing event tags are checked
before borrowing payloads. The 0.133 GUID resolver is split into a three-argument
native query predicate and a lookup borrowing a pointer to the map reference;
stand-ins verify both argument lists and reject lookup after a failed query.
The selected 0.133 descriptor binds these contracts; runtime validation remains a separate gate.

The hybrid heal queue emitter is bound by its actual descriptor kind and native
cast callers. In 0.133 it additionally snapshots native healing-source metadata
before appending kind 14. The nearby kind-15 shield emitter retains a similar ABI
and shorter body; matching that prolog would silently produce the wrong effect.
An exact-image semantic test checks kind 14, fraction/duration stores and a real
native cast call edge, independently of the selected entry guard. Cooperative
qualification must not require synthetic Holy Nova or Holy Shock ally receipts: those
spells now retain native cast/impact behavior. The retired group-delivery tag is
rejected. The native healing emitter remains a reviewed native cast dependency,
not an AZSX hybrid-emission API.

## Compatibility review

Read [the stable feature inventory](AZSX-FEATURES.md) and
[the current descriptor notes](CLIENT-COMPATIBILITY.md). Account for every feature
ID, including disabled or upstream-native behavior. Compare exact build data,
source policy and artifact validation separately. Matched call sites provide
candidate references only: they do not establish the callee ABI or the target
data layout. The offline comparer keeps those candidates separate from aligned
instructions and refuses checkpoints created with the previous reference-inference
policy; retain old evidence and start a fresh comparison instead. Use the profiles in
[testing](../development/TESTING.md); run Windows stand-ins when native register/stack wrappers
or generated code change. Current tests cover the selected 0.133 path;
the full gate additionally enables that graph to preserve its tests.

See [Rust architecture](../development/RUST-ARCHITECTURE.md) for workspace boundaries and
[party presentation](../features/OFFICIAL-PARTY-UI.md) for owner-data and retained-widget rules.

Native callable entry guards must account for AZSX-owned hooks without accepting
arbitrary jumps. The hook installer retains the exact original and installed
five-byte prefix for each successful hook. The shared item presenter accepts
that recorded patch only at its registered address, with the reviewed original
prefix and an unchanged remaining guard. It still calls the hooked entry so
optional goblin reward scopes retain their native forwarding behavior. Unknown
patches or changes to the guard tail fail closed; uninstall clears ownership.

Native queued whole-level rewards are also consumed while dead. Their native
threshold-deficit calculation runs at receipt time; only the resulting choices
wait for revival. The three native consumers retain separate Playing-only choice
gates. These level-fill awards never enter the shared-XP multiplier/receipt path.

The two whole-level producers originally discarded receipts when the current
native actor was dead. Their guarded continuations preserve just the native
level counter addition; item, gold and other reward eligibility is not widened.
The adapter validates the bounded host stack or remote receipt payload before
rejoining, without writing phase/HP, inventing XP or retrying uncertain awards.


The exact 0.138.0 Polymorph rupture scope begins after the triggering damage
has committed and ends at the common join before subsequent on-hit procs.
Both interior seams are guarded. A separately guarded entry bypasses the native
attacker ability-kind scan only for a unique active foreign sheep; the native
hit-eligibility checks still run before borrowing the producer. This allows an
ally without a special ability to rupture the sheep. The native ownership GUID list is read from
Game+ad8/ae0 or Actor+9d0/9d8, independently verified against the native swap.
The adapter normalizes guest-to-guest transitions through the host, rebinds
slot/GUID, and restores the attacker after native burst completion. The runtime
keys outstanding loans by native frame to preserve nested callback ownership;
no state borrow spans a native call. The native explosion itself is not replaced.
Selected-memory and exact-image checks do not establish live gameplay parity.
