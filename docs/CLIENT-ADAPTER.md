# Selected native client boundary

ASX source supports only official **0.130.1**. `SelectedAdapter::select` accepts
only the reviewed executable SHA-256. A version label or a similar instruction
sequence cannot enable another client. Selection does not bypass entry-byte,
image-bound or native-layout verification.

## Ownership

| Layer | Responsibility |
| --- | --- |
| `crates/asx-gameplay/src/` | Portable rules, identities and copied semantic values |
| `crates/asx-game/src/official.rs` | Selected native facade used by the DLL |
| `crates/asx-game/src/official_native/` | Native translation, memory validation, call shapes and hook contracts |
| `crates/asx-game/src/bindings/official1301.rs` | Exact current-image identity and descriptor choices |
| `crates/asx-game/src/memory.rs`, `enemies.rs` | Neutral memory interface and exact enemy parsing used by current support |
| `src/runtime/` | Feature coordination and bounded production service bindings |

Active native consumers resolve descriptor data through the `asx_game::current`
re-export. The alias currently points to `official1301`; it does not select or
authorize an executable. Exact hash selection remains explicit in the adapter.
Event appearance stack slots and native marks tables also belong to that
descriptor, rather than to feature coordination.

The selected descriptor is generated from `compat/clients/0.130.1.json` and
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
or generated code change. Current tests cover the selected 0.130 path;
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
