# Native party presentation

ASX enriches the supported native interface. The native `PlayerFrame` and
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
formatter output and native string/buffer bounds. Lobby class-icon decorations
are retired: ASX installs no atlas-capture, lobby-button, status-refresh or
roster-cache hooks. Only the shared reward-header sizing hook remains; native
lobby rendering and in-game party identity remain unchanged.

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
markers. ASXUI styles nameplates and selects the world-number font independently;
healing messages and cooperative markers retain their native ASX owners.

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

ASXUI now owns minimap pins, including native caches read from `AZS_CHESTS`.
The embedded presentation exposes the read-only versioned `ASX_GetState(1)` bridge
and creates no pins. Without ASXUI, this optional custom pin layer is absent.
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
ASXUI updates pin membership at 10 Hz and retained geometry each frame; ASX
keeps model/buff refresh at 4 Hz and confirmed progress smoothing per frame.
The read-only bridge checks the current payload freshness without rescanning
entities or rebuilding unchanged payloads. Missing projection hides pins.

In coop, the host appends each recipient's own consumables, receipt, ritual
presence and remaining event pins to the retained addon payload. Shared progress
is identical; panel visibility and claimed reserve pins are personal. The native
scene packets are independent of cosmetic Lua metadata. Guests reconstruct their
own minimap center from the live native projection, with local ping-event
coordinates preserving the reference across subsequent host metadata samples.

## ASX presentation envelope

`AZS_RUN.asx` is an ASX-owned extension transported on the native `AZSRUN`
line; it is not part of the native addon API. The producer, retained Lua view,
widget names (`ASXOfficialParty...`) and diagnostic readout
(`ASXOfficialPartyReadout`) use ASX naming. The former field is not an alias.
Historical reports and unrelated legacy diagnostic interfaces are unchanged.

Each recipient receives `M` membership and ASX metadata (including local
Fortune, fatigue and personal events), but `S` spell and `I` item build rows
only for other players. Local builds are still captured for those other
recipients. Native local build presentation remains authoritative.

The `A` minimap row is now
`A|owner|centerX|centerY|r0|r1|r2|r3|r4|r5|pingX|pingY`.
Coordinates use hundredths and radii use millionths. The unused single radius
is omitted; six zoom radii, native center and ping coordinates remain available
for zoom, pan and local-ping projection. Older thirteen-field (zoom) and fourteen-field (radius plus zoom) rows are rejected.
The current row has twelve fields. ASXUI requires a valid live
`Minimap:GetZoom()`; there is no payload zoom fallback.
The compressed envelope and content header versions are unchanged: the renamed
field and bundled Lua consumer define this private interface together.

The wall-clock heartbeat remains in `1|epoch|recipientGUID|heartbeat`, including
while native gameplay time is paused. It refreshes native transport without
changing stable cosmetic identities or invalidating a hovered owner tooltip.

## Extracted ASXUI event presentation

Supply/Fortune notifications and Sayge progress widgets are now owned by
ASXUI `Events.lua`, not the embedded addon. ASX still formats all semantic text
through `ASX_GetState(1).texts` from validated current recipient data.
The addon keeps the native-styled panel recipe, notification lifetimes,
identity-scoped deduplication/queueing and cosmetic progress smoothing. Saved
per-widget position, scale, opacity and visibility do not change gameplay.
Aura tooltips and buff-slot enrichment remain embedded. Native goblin CHAT_MSG
announcements remain on their existing native channel. This extraction changes
UI ownership; it does not remove a native gameplay hook.

The supported addon-facing interface is now [ASX contract 1.0](ADDON-API.md):
`ASX_STATE`, `ASX_STATE_UPDATED`, `ASX_GetAPIVersion()`, `ASX_GetState(1)` and
registered state handlers. Values and ASX text are separate and getters/handlers
return detached copies. ASXUI uses this public interface. Older named getters
remain private compatibility adapters; the compressed wire format stays private.

Event widgets share an ASXUI root anchored below the native top HUD and stack
visible scaled panels without overlapping by default. Saved old absolute widget
positions migrate to offsets from stack slots. Sayge's v1.1 API adds collective
readiness separately from local presence, so an inside player sees an ASX waiting
message and frozen cosmetic progress while another required player is outside.

## Priority buff presentation

ASXUI/Buffs.lua promotes the existing native Fortune button into the first
visual buff position (rightmost in the stock layout). Crystal Resonance follows
immediately, or takes the first position without Fortune. Crystal identification
requires both the current API aura index and its native gem icon. It remaps only frame
anchors, including wrapped rows. Native IDs, buffIndex, tooltips, duration
children and click handlers are unchanged. Missing/stale ASX state, an absent
priority buff or ambiguous icons leave the native layout in control. Original anchors
are restored only while ASXUI still owns them; fresh native layout changes win.
Restore clears the owned graph before rebuilding anchors to avoid cycles.

The retained ASX tooltip keeps native GameTooltip styling: white title, green
effect and secondary scope/lifetime details. When a party fortune is stronger,
separate Your fortune and Active party bonus rows distinguish the owned value
from the effective shared value; they are not added together.
