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

## World and reward extensions

[Native world presentation](NATIVE-WORLD-UI.md) documents enemy plate styling,
world damage/healing text, shared card separators and one-time random cooperative
markers. These use native owners and the same owned presentation activation gate.

## World-event presentation

Consumable metadata and event coordinates are copied from the authoritative
host or solo model. Six owned cosmetic aura slots may coexist; tier, effect and native
simulation countdown come from native data, while ordinary buff APIs still
forward unchanged. Discovery notifications retain their widget and queue unique
receipts. The minimap retains bounded child pins using selected native view
center/radius; edge-clamped targets explicitly indicate that they are outside
the current view. Stale/removed events hide their pins. No Lua write controls
placement, collection, timers, rewards, stats or pause authority.

The shared native UI-context lookup forwards its argument and owned Rc result
unchanged and only observes the selected borrowed data address. It does not
acquire an additional native borrow, retain an Rc, allocate or scan objects.
Minimap context therefore does not depend on enemy nameplate visibility.
Expired consumables retire only their six owned recipe-placeholder rows;
unrelated native class buffs keep their original rows and icons.

Native buff buttons cache textures until an aura event. Cosmetic consumable
identity changes explicitly call the original retained-button update, restoring
Lua's implicit `this` afterward. Countdown-only metadata does not rebuild the
buttons. This prevents cached `Temp` faces and expired zero-second icons.

Consumable tooltips use the native GameTooltip with plain white name, actual
effect and remaining duration, using the native duration formatter when present.
No tier colors, tier label or refresh-rule explanation are added to the tooltip.

Sayge's Trial has no availability countdown. Its progress UI shows only the
20-second cumulative presence objective, and its minimap pin remains until the
trial is completed or the run ends. The ritual carries native inside/outside presence; supplies retain their finite
countdown and normal expiry.

## Event visual integration

Event panels follow the inspected Azeroth Survivors native panel constructor,
using its dark brown (0.031, 0.024, 0.016) fill, with a thin
(0.427, 0.333, 0.145) border and a 10-pixel corner radius. Heading and muted
colors are (0.973, 0.824, 0.416) and (0.725, 0.663, 0.510). The game loads
Morpheus and FrizQT; event headings use Morpheus and details use FrizQT.
Retained native regions reproduce that geometry using native portrait masks;
no stock dialog border, quickslot rim, or redistributed native image is added.

Discovery notifications stay fully opaque and hide instantly at their deadline.
Queue/model/buff work remains at 4 Hz. Progress visually follows confirmed
presence samples without predicting or granting additional progress. The Sayge
panel shows only when native participant presence reports the player inside the circle;
leaving hides it without clearing the cumulative objective. Ritual metadata's
last field is now an inside flag (0/1), not an availability countdown.

Minimap pins are circular crate and Fortune-orb images masked by the client's
SetPortraitToTexture, surrounded by the native tracking ring. They are retained
children of Minimap, not entries in its engine POI registry. Interior projection
uses the full native map scale; only the circle's outer edge is inset by the icon
radius. Between metadata samples, GetZoom and GetPingPosition supply the current native
projection. The adapter copies all six native radii and the retained ping world
point, allowing Lua to reconstruct the live center and project at the current
zoom immediately. One MINIMAP_PING UI listener preserves the last center until
the changed reference arrives; it never changes the native ping. Pins stay
visible across zoom changes. No exposed Lua API registers custom engine POIs.
Only active pin geometry and confirmed progress smoothing run per frame; no model, buff,
icon asset, entity scan or network rebuild is driven by that path.

In coop, the host appends each recipient's own consumables, receipt, ritual
presence and remaining event pins to the retained addon payload. Shared progress
is identical; panel visibility and claimed reserve pins are personal. The native
scene packets are independent of cosmetic Lua metadata. Guests reconstruct their
own minimap center from the live native projection, with local ping-event
coordinates preserving the reference across subsequent host metadata samples.
