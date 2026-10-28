# Native party presentation

ASX enriches the official 0.125 interface. The native `PlayerFrame` and
`PartyMemberFrame1..4` retain health, names, death/resurrection and connection
behavior. Five-player capacity means the local player plus four remote frames;
ASX does not replace lobby admission or create a second roster authority.

## Retained children

`native-ui/official-party.lua` adds positive buffs and compact spell/item build
rows as retained children. Native tooltips and buff/debuff APIs remain distinct.
Do not replace party roots, native health bars, resurrection UI or lobby menus,
and do not introduce a DPS ranking. Player units and pet frames are different
owners; no player build should be attached to a pet frame.

`src/runtime/party_ui.rs` owns cosmetic GUID/generation freshness and
presentation policy. Selected native access owns owner HUD translation, native
formatter output and native string/buffer bounds. The native menu class atlas is
selected separately; decorations borrow the live native builder and remain
cosmetic.

## Owner identity and tooltip contents

Remote values must come from the owner, not the viewer's `SetAction` or a static
formula. Match slot/order and GUID/generation; ambiguous names, missing owners,
invalid builds or stale delivery hide the cosmetic row rather than guess.
Preserve unchanged identities across heartbeats and common pauses so hovering
does not flicker or discard still-valid data.

Native spell and item formatter output carries the owner's text, talents,
bonuses and colors. The native Lua `string.byte` accepts one index only; portable
stubs must retain that behavior. Keep decoded presentation bounded and separate
from gameplay state and native mutation authority.

## Pause and diagnostic boundaries

Keep native menu/choice ownership separate from group waiting presentation.
See [cooperative pause](COOP-PAUSE.md). A local choice notification is not
permission to release the host's barrier.

Diagnostic writer channels and controlled fixtures are `test-hooks` only and
explicitly opted in. Production addon activation uses the owned file/read-buffer
gate described in [installation](ASX-INSTALLATION.md). Do not add persisted CVars,
a parallel network or modifications to native game assets as a presentation path.

Run `official-party-ui` for required Lua 5.1 behavior and selected-memory checks.
A script load, retained-child allocation or sent packet does not prove a visible
frame. Rendered owner-tooltip claims require an explicit bounded visual trial
against the actual artifact; portable checks alone do not establish that result.
