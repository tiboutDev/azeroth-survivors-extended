# Selected native client boundary

ASX source supports only official **0.135.1**. `SelectedAdapter::select` accepts
only the reviewed executable SHA-256. A version label or a similar instruction
sequence cannot enable another client. Selection does not bypass entry-byte,
image-bound or native-layout verification.

## Ownership

| Layer | Responsibility |
| --- | --- |
| `crates/asx-gameplay/src/` | Portable rules, identities and copied semantic values |
| `crates/asx-game/src/official.rs` | Selected native facade used by the DLL |
| `crates/asx-game/src/official_native/` | Native translation, memory validation, call shapes and hook contracts |
| `crates/asx-game/src/bindings/official1351.rs` | Exact current-image identity and descriptor choices |
| `crates/asx-game/src/memory.rs`, `enemies.rs` | Neutral memory interface and exact enemy parsing used by current support |
| `src/runtime/` | Feature coordination and bounded production service bindings |

Active native consumers resolve descriptor data through the `asx_game::current`
re-export. The alias currently points to `official1351`; it does not select or
authorize an executable. Exact hash selection remains explicit in the adapter.
Event appearance stack slots and native marks tables also belong to that
descriptor, rather than to feature coordination.

The selected descriptor is generated from `compat/clients/0.135.1.json` and
`compat/binding-template.rs`. Its field translations are resolved, including
explicit invalid gaps; they do not invoke older descriptors. Historical modules
are test-only comparison references. Reviewed canonical keys are retained where
call sites require them, with translation owned by the current adapter.

Run `python3 tools/dev/asx_dev.py check` to verify generated source and the feature
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
This contract correction does not approve a new executable or resume the separate
post-death XP investigation.

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
qualification still requires effective Holy Nova and Holy Shock heal receipts.

## Compatibility review

Read [the stable feature inventory](ASX-FEATURES.md) and
[the current descriptor notes](CLIENT-COMPATIBILITY.md). Account for every feature
ID, including disabled or upstream-native behavior. Compare exact build data,
source policy and artifact validation separately. Matched call sites provide
candidate references only: they do not establish the callee ABI or the target
data layout. The offline comparer keeps those candidates separate from aligned
instructions and refuses checkpoints created with the previous reference-inference
policy; retain old evidence and start a fresh comparison instead. Use the profiles in
[testing](TESTING.md); run Windows stand-ins when native register/stack wrappers
or generated code change. Current tests cover the selected 0.133 path;
the full gate additionally enables that graph to preserve its tests.

See [Rust architecture](RUST-ARCHITECTURE.md) for workspace boundaries and
[party presentation](OFFICIAL-PARTY-UI.md) for owner-data and retained-widget rules.

Native callable entry guards must account for ASX-owned hooks without accepting
arbitrary jumps. The hook installer retains the exact original and installed
five-byte prefix for each successful hook. The shared item presenter accepts
that recorded patch only at its registered address, with the reviewed original
prefix and an unchanged remaining guard. It still calls the hooked entry so
optional goblin reward scopes retain their native forwarding behavior. Unknown
patches or changes to the guard tail fail closed; uninstall clears ownership.
