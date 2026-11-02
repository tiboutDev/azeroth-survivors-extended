# Banished Demon

This source feature targets the reviewed 0.138.0 native client. It can run in
solo and in cooperative games. It is not evidence of an installed or published
release. It uses only native client models, effects and combat definitions.

## Encounter

The host schedules a first opportunity after a random 90–150 simulation
seconds. Later opportunities are spaced by 180–300 seconds. Placement draws
ground-checked sites and can fail or retry; no demon is guaranteed per run.
The encounter never overlaps an active Mechanical Chicken escort. A binding
scene contains a rune, a captive demon and a Council Warlock. Each demon draws
one of the native Felguard Elite, Infernal or Satyr Shadowcaster combat kits.

All active participants must be alive, playing and within four yards of the
binding. Two seconds of cumulative preparation release one native hostile
demon. Leaving the circle suspends preparation and returning resumes it. The
host alone owns scene creation, combat completion and reward. The demon uses
native targeting and damage. The scene expires after four simulation minutes;
the terminal receipt stays available briefly and owned scenery is removed.
Run teardown and identity changes clear the scene without reward.

A confirmed native kill awards one native Banish charge to the shared run
counter. This is the same charge consumed by the native card selection, not an
invented item, mark or talent. The reward is reserved before the native write
and cannot be repeated by a later callback. The cooperative counter is
game-local; it is not an independent per-participant inventory.

The opening announcement presents the binding and challenge without revealing the reward.

The optional AZSXUI addon shows the encounter under Objectives, places a native
`Spell_Shadow_Cripple` minimap marker at the binding while active and displays
a one-time reward toast. API `data.soloEvent` provides the same structured
state without requiring AZSXUI. The captive and warlock currently form a static
binding scene; the old 0.131.0 channel packet has not been adopted without a
reviewed 0.138.0 ABI.

## Compatibility and validation

The 0.138.0 adapter guards Banish reads, consumption, serialization, action,
demon definitions, native spawn, placement and scene packet contacts. A future
client requires an exact-image review. Portable policy/API tests and Windows
ABI checks do not replace isolated solo and two-client trials. Visual quality
of the native models and the guest's Banish count must be checked in game.
