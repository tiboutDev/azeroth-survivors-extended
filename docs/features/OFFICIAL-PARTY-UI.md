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
gate described in [installation](../release/ASX-INSTALLATION.md). Do not add persisted CVars,
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
only for other players. Contract 1.5 additionally carries at most five local
items in a separate bounded equipment sequence for ASXUI; local spells remain
omitted. Local builds are still captured for those other
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

## ASXUI local HUD

ASXUI `Hud.lua` extends the native action-bar artwork using existing textures,
retains native action/micro-button scripts and preserves native XP rendering/height/scripts while widening its geometry. Native gryphons remain at the front. A top-centered icon counter strip flush with the screen edge
reads native kills/gold/cache cost independently of the bottom layout; a three-column fixed-size drawer slides
through a clipping scroll frame and omits displayed-zero character stats.
Native time and the next-boss countdown are displayed below the counters using
public native run APIs; the native clock has no movable Lua frame. Its prior
visibility is retained, including an already hidden clock. The native build/tally/clock visibility preferences and owned anchors are restored on
disable or unavailable data. Local equipment is provided by contract 1.5 from
the existing captured owner build, with explicit empty snapshots and native
tooltip text; no additional native capture, ABI contact or gameplay hook is added.

The user-selected local layout now visually hides the old PlayerFrame/TargetFrame
roots (without unregistering their native events) and shows native health values
in a health-only HUD bar. Original alpha/mouse states are restored on disable.
The retained aura buttons dock above this bar, separating harmful filters and
preserving Fortune/Crystal priority, IDs, duration/click handlers and restoration.
Health and aura anchors stay fixed while the title-free drawer slides above
health, intentionally covering the aura rows and their duration labels. Closing
the drawer exposes the original aura buttons for inspection/cancellation.
Ally panels style existing retained
spell/item children without replacing party roots or owner formatting. Allied
buff buttons sit outside the panel on its right, with up to eight icons per
row and two rows. Native IDs, owner/recipient data and tooltip scripts stay
unchanged, and their original anchors restore when the HUD is disabled.
Read-only objectives in the upper-left use held native lairs (including distant guardians), falling back to living-guardian rows when the lair API is absent, and ASX Sayge state; retained party frames are laid out beneath them.

The health bar is centered at two thirds of the bottom HUD width. Native buff
and debuff rows sit above it, each centered independently (eight icons per row,
including centered partial rows), without shrinking icons or duration text. Native duration labels move above
the icons; their values and visibility remain native. The statistics drawer
opens immediately above health and intentionally covers auras while open.
The top counter ribbon uses the native dialog-header body/sides and lower
trim, without the upper rail or top corners. Its content area is 36 units high,
leaving eight units above and below the 20-unit icons before the lower trim.
The clock remains below it with a clear gap. Player health has a capsule outline and a 22-unit fill, leaving room around
its 12-unit text. Native circular masks and the native white texture shape
the ends; a clipping viewport preserves the silhouette at low HP.

Event pickup panels, objectives and allied build panels use an unmarked area of the native quest-log
parchment, shaded dark brown to soften the background without introducing new
assets. Other HUD panels keep their existing darker theme. Pickup panels use a
340-by-88 default footprint and 32-unit icon, growing for wrapped native text;
the automatic notification stack uses the resulting height and saved scale.

The warm paper extends beneath the popup/objective/allied-build border rather than leaving
a black inner gutter. No native texture is redistributed.


### Native-textured health, permanent badges and channels

The capsule's foreground uses the original native status-bar texture. Retained
ally build panels align center-to-center with their party roots, with aura
children outside the panel. Crystal/Fortune medallions use only current public
ASX state and prefer the selected native buff tooltip. Index reuse is checked
again on hover; ambiguous/missing indices never target another aura. Native
buttons are visually suppressed only while a valid badge replaces them, then
restore their original alpha/mouse state. Other aura rows keep native handlers.

Sayge and resurrection share a native casting-bar skin without an icon. The
resurrection view consumes `AZS_GetRevive()` directly and retains/restores the
native visibility preference. No revival command, timing or participant policy
is added. Loot/mark notices consume local native English chat receipts and keep
their text verbatim; no wallet delta, new memory read or ABI contact is needed.
Native loot is hidden only after a compatible receipt establishes the replacement;
unsupported locales retain native presentation. This does not suppress chat.

### Current HUD follow-ups

The optional economy projection reuses the exact XP/gold award calculations and
does not mutate native stat tables, credit gold, or apply multipliers again.
Native stats remain the fallback when the projection is unavailable. The stat
panel rebuilds its nonzero rows at the existing four-Hz cadence. Item annotations
match native definition icons and verify owner GUID and current rank before
reading the locked ASX enchantment. Native text is retained independently; missing
or ambiguous annotation data is omitted. No new hook or native call is installed.
Fortune award receipts are separate from encounter scaling and ritual identity.
See the v1.7 [addon contract](ADDON-API.md) for units and notification lifetimes.

The exact local recipient envelope does not wait for cosmetic roster/build data.
Its internal generic label is not a remote name or a unit-binding fallback.
Missing local equipment stays absent, and unverified remote builds stay hidden;
independent local event/economy values can still be decoded. Changed or missing
equipment data dismisses an already open tooltip instead of showing stale values.
