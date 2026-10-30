# ASXUI

Independent Lua presentation addon for Azeroth Survivors. It works without the
ASX DLL for its nameplate styling. Minimap rendering uses the optional ASX
projection/event bridge described below. No addon memory reads or hooks.

## Install

Copy this whole folder to `AzerothSurvivors\benilla-config\AddOns\ASXUI`, then
restart the client. Keep its `.toc`, Lua, font and licenses together. It does not
belong inside the game's `ASX` folder. Removing the addon restores the native
presentation at the next client start; its saved preferences may remain.

## Behavior and commands

- Enemy nameplates use 70% of their original geometry by default. The addon keeps
  the native bar, health updates, world placement and visibility controls.
- Friendly plates keep their original geometry. Recycled plates are re-evaluated.
- Level/skull labels are hidden on styled enemy plates. Border and raid marker
  textures are retained. Names keep their native font size.
- Enemy names are hidden by default, including missing or ambiguous identities.
  Only a uniquely highlighted target positively classified as an elite, guardian,
  boss or final boss keeps its name. `AZS_UnitKind` takes precedence; generic
  native elite classification is a fallback only when that API is absent.
  Untargeted special enemies may therefore also have hidden names: Lua does not
  expose a GUID/unit token for every retained plate. Names in `AZS_UNITS` alone
  are not sufficient to identify a plate. Reused plates are checked each frame.
- The native damage font is preserved by default. The earlier renamed OFL number
  face remains an explicit opt-in (`numbers on`), rather than a default. Existing
  saved `numbers=true` preferences are respected; use `numbers off` and restart
  to restore the startup native face. No confirmed same-face outgoing world-number
  size setting has been found for the reviewed engine.
- ASXUI owns the circular custom-event and native-cache minimap pins, their native
  portrait mask, tracking-ring border, hover text and edge projection. Supply and
  trial icons retain their styles; caches use the native `INV_Box_02` icon; supplies retain `INV_Crate_01`.
  Both keep the same circular mask and tracking ring.
  Native cache coordinates come directly from `AZS_CHESTS`. Native cache opening,
  purchases, event progress and per-recipient availability stay authoritative.

### Minimap data boundary

`Minimap.lua` consumes `ASX_GetState(1).data` from the matching current ASX
addon contract: native view center, six zoom radii, native ping reference,
run/recipient identity and bounded personal event rows. Live zoom comes only
from `Minimap:GetZoom()`, never the bridge. Missing/invalid zoom hides pins. It does not decode a
second copy of the private payload, copy cache coordinates through the DLL,
register engine POIs or send gameplay commands. The embedded ASX presentation
no longer creates pins or listens for minimap pings.

Pins require the bridge, valid current-run data and the native Minimap widget.
Without it, nameplate styling remains independent but minimap pins stay hidden;
hero coordinates alone cannot reconstruct a panned minimap reliably. Hidden or
stale data, menus and `/asxui pins off` hide all addon pins. Available caches and
events remain visible while choosing or paused. Removing ASXUI removes this
optional pin presentation, not native world objects or gameplay. Only the replaced native guardian flag is hidden, with its prior preference
restored when pins are disabled/unavailable. Up to nine custom, one guardian
and 128 native cache rows
are rendered; extra native caches are omitted from this bounded cosmetic view.

Commands:

```text
/asxui pins on
/asxui pins off
/asxui plates on
/asxui plates off
/asxui scale 0.7
/asxui numbers on
/asxui numbers off
```

Plate styling and marker toggles change immediately and are reversible. The number-face preference
applies after restarting the client because the engine caches it at load time.
Use the native visibility keys or options to show nameplates; ASXUI does not
force them on or change bindings. Avoid enabling another addon that styles the
same plates/pins or selects a conflicting damage font at the same time.

## Development and limits

Source is in this directory. Run `sh tools/dev/check.sh --profile asxui` from the
repository root for required Lua 5.1 tests. API probes are explicitly opt-in
helpers under `tools/dev`, not shipped addon files. Existing ASX DLLs still
contain their old styling until rebuilt; do not combine their scaling with this
addon when judging size. Standalone tests use an exact native executable without
any ASX DLL. Visual size, layout and special-rank coverage require separately
scoped live observations; portable tests alone are not rendering evidence.

## Licenses and references

ASXUI code: GNU GPL v3, see `LICENSE`. The number font is a renamed derivative of
Barlow Condensed SemiBold by Jeremy Tribby and contributors, under the SIL Open
Font License; see `Fonts/OFL.txt`. No native game fonts or textures are bundled.

Technical references: [Survivor Plates and Survivor UI](https://github.com/morderous/survivor-ui)
and [SurvivorUI Classic](https://github.com/morderous/survivorui-classic).
These projects demonstrated the native Lua plate widgets and the `DAMAGE_TEXT_FONT`
route. ASXUI keeps its own small implementation and does not depend on them.

The source font was read from survivor-ui commit
`eb1f4eee5aee80e2a2d957f0a15fb09a7343c625`, file
`SurvivorUI/Fonts/BarlowCondensed-SemiBold.ttf`.
Source SHA-256: `7b619d14bc2327509a9ef32b0890f709626f7ecc9ff61191c2a4314c5499d2d9`.
Rebuild the renamed derivative with:

```sh
uv run --with fonttools python tools/build/asxui_font.py --source <source-font.ttf> --output addons/ASXUI/Fonts/ASXUI-Numbers.ttf
```

## ASX event widgets

`Events.lua` owns Supply/Fortune discovery panels and Sayge's progress bar.
`ASX_GetState(1)` supplies structured values in `data` and every semantic title,
reward, note and color in `texts`, all from ASX. ASXUI does not calculate
rewards, timers or gameplay. Goblin appearance/defeat/escape announcements keep
their existing native CHAT_MSG channel; they are not these custom panels.

Each widget has independent saved settings in `ASXUIDB.widgets`, applied without
a client restart. A shared anchor is centered below the native top HUD (UIParent TOP, 0,-85).
Visible widgets stack in Sayge, Supplies, Fortune order, with a 12-unit gap;
scaled height is included and hidden/expired panels leave no empty slot.
Widget position settings are offsets from their automatic stack slot, not world
coordinates. Scale changes the full widget/text without shifting the shared
anchor. Defaults are zero widget offsets, scale/opacity 1 and visible.
Old absolute Y preferences migrate to equivalent offsets from their old defaults.

```text
/asxui widget anchor position 0 -85
/asxui widget supplies position 0 0
/asxui widget fortune scale 1.2
/asxui widget sayge opacity 0.75
/asxui widget supplies show off
/asxui widget supplies show on
/asxui widget fortune reset
```

Widget names: `supplies`, `fortune`, `sayge`; `anchor` accepts position/reset. Position bounds are -4096..4096,
scale 0.5..2 and opacity 0..1. Invalid commands are rejected; malformed saved
preferences return to safe defaults. Hidden notifications still consume their
normal presentation lifetime and do not replay when enabled. Choice/pause
suppression and identity-scoped Supply queuing remain in effect. Sayge hiding
only hides the display, never affects presence, progress or completion. In coop,
the bar stops and ASX supplies a waiting message while any required participant
is outside; earned progress resumes when everyone returns. Missing
or stale ASX data hides all widgets. The matching rebuilt ASX bridge is required.

## Public ASX data contract

ASXUI consumes [ASX addon contract 1.0](../../docs/ADDON-API.md) through
`ASX_GetState(1)` and `ASX_GetAPIVersion()`. Structured values live in `data`;
ASX-owned wording/assets/colors live in `texts`. It no longer consumes the old
private presentation/minimap getters. Other addons can use the same native-style
globals, getters and registered handlers without installing ASXUI. Missing or
unsupported API versions hide ASXUI's dependent widgets; standalone plates remain.

The guardian minimap guide uses native `AZS_GetFlaggedLair()` data and the native
`Ability_Warrior_Challange` taunt icon, with the same round border as other pins.
Only the replaced native flag is hidden; its prior HUD preference is restored
when pins/projection are unavailable. Sayge progress uses WoW's native casting
bar texture/border, with the trial title inside and status below. Fortunes use
green +X% values rather than visible rarity labels. Supplies buffs last 60 game
seconds; unclaimed piles last 90 game seconds (claimed piles still retire early).

ASX contract 1.2 adds the seventh consumable family (`6`, Swiftness) and a
separate `discovery.quality` for Classic name colors. ASXUI uses supplied colors
and text; it does not infer color from buff tier or gold value.

## Classic WoW notifications and buff order

Supplies/Fortune popups use native UI-Tooltip-Background/Border, UI-Quickslot2
and Friz Quadrata, preserving ASX wording, quality colors, anchors, scaling,
opacity and notification lifetime. Cache pins use native INV_Box_02, distinct
from Supplies INV_Crate_01. No texture is generated or bundled.

The native Sayge Fortune button moves to the first visual buff slot, at the
right end of the first stock buff row. Crystal Resonance follows Fortune, or
leads the row when Fortune is absent. It requires its current API aura slot and
matching native icon. Buff IDs/clicks/durations remain native.
Native anchor changes are preserved, and ASXUI restores its original anchors
when neither priority buff is present or ASX data is unavailable. The native tooltip distinguishes
owned Fortune strength from a stronger effective party bonus.
