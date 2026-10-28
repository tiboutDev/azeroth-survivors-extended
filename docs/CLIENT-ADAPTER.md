# Selected native client boundary

ASX source supports only official **0.125.0**. `SelectedAdapter::select` accepts
only the reviewed executable SHA-256. A version label or a similar instruction
sequence cannot enable another client. Selection does not bypass entry-byte,
image-bound or native-layout verification.

## Ownership

| Layer | Responsibility |
| --- | --- |
| `crates/asx-gameplay/src/` | Portable rules, identities and copied semantic values |
| `crates/asx-game/src/official.rs` | Selected native facade used by the DLL |
| `crates/asx-game/src/official_native/` | Native translation, memory validation, call shapes and hook contracts |
| `crates/asx-game/src/bindings/official125.rs` | Exact current-image identity and descriptor choices |
| `crates/asx-game/src/memory.rs`, `enemies.rs` | Neutral memory interface and exact enemy parsing used by current support |
| `src/runtime/` | Feature coordination and bounded production service bindings |

Older official descriptor files remain where their field/RVA translations or
constants are still used by 0.125. This is data reuse, not selectable old-client
support. Retired profile/readers/projectile modules and their exclusive tests
are removed; neutral memory and enemy parsing remain current dependencies.

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
source policy and artifact validation separately. Use the profiles in
[testing](TESTING.md); run Windows stand-ins when native register/stack wrappers
or generated code change. Current tests cover the selected 0.125 path;
the full gate additionally enables that graph to preserve its tests.

See [Rust architecture](RUST-ARCHITECTURE.md) for workspace boundaries and
[party presentation](OFFICIAL-PARTY-UI.md) for owner-data and retained-widget rules.
