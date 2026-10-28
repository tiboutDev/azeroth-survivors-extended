# Official 0.122.0 descriptor

The sole supported executable SHA-256 is:

```text
ec3a5b15fa39fed63ce4516c4fc159bdac92691ca44f88befc849010338d14f7
```

The native selector, source registry and ASX manifest validator reject other
builds. `official122.rs` owns current-image entries, byte guards and actor copy
spans. Reviewed field/RVA translators inherited from official descriptor files
remain implementation data; their original clients are not selectable.

## Current native distinctions

The host Game contains a 24-byte vector at offset `0xa08`; host snapshots omit
that vector rather than cloning its owned allocation. Private actor rows retain
stride `0x9e8`. Host and private layouts are separate contracts. Role, peer count,
stream, lobby/UI fields, menu frame and native pursuit continuation use selected
current-image choices. Do not reintroduce retired revision branches or guess a
layout from a version string.

Enemy HP, damage, density, placement, schedules, native sets, Heroic behavior and
native rewards remain native. ASX does not restore custom difficulty or spawn
coordinators. Native capacity remains five.

All stable [ASX feature IDs](ASX-FEATURES.md) require a disposition when this
boundary changes. Glyph gameplay is disabled; duplicate-loot gold compensation
is disabled in favor of native personal marks. Inherited glyph/bonus helpers do
not establish installed glyph gameplay.

## Verification

Current memory fixtures and exact-byte checks live in `crates/asx-game/tests/`
and selected native modules. Root Windows tests exercise reviewed wrapper/ABI
contracts without a game. Follow [testing](TESTING.md) for the changed boundary.
Keep source checks, exact artifacts and evidence limits separate. Controlled
build references live under [validation](../validation/README.md); generated run
reports belong to ignored local evidence. Neither establishes universal gameplay
parity or publication readiness.
