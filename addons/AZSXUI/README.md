# AZSXUI

Standalone Lua interface for Azeroth Survivors 0.138.0. The native client supplies the action bar, health, level and experience, combat counters, character stats, nameplates, boss bars and guardian objectives. Camera controls work independently of the mod.

A compatible AZSX addon API enables mod events and map pins, personal equipment details, cooperative rules, shared economy and kill rankings as their data becomes available. Standalone play keeps the game's equipment and loot displays. Native world-number scaling and additional healing numbers require the DLL.

## Install

AZSX includes this addon and installs it automatically in
`AzerothSurvivors/benilla-config/AddOns/AZSXUI` when starting AZSX. Its code is
updated with signed AZSX updates; no separate archive is needed. Preferences stay
in the native SavedVariables store. Do not edit the managed source files to
configure the addon; use the commands below.

For standalone use, place the `AZSXUI` folder inside the native game’s `benilla-config/AddOns/` directory and restart the client. The resulting path must be `benilla-config/AddOns/AZSXUI/AZSXUI.toc`. Keep its `.toc`, Lua and
licenses together. AZSX can later adopt matching files or, with consent and a
backup, replace different code. Removing AZSX removes only its unchanged managed
addon files and preserves preferences and unknown files.

## Behavior and commands

- Enemy nameplates use 50% of their original geometry by default. The addon keeps
  the native bar, health updates, world placement and visibility controls.
- Friendly plates keep their original geometry. Recycled plates are re-evaluated.
- Level/skull labels are hidden on styled enemy plates. Native border/glow and
  health art are restored when styling is disabled. Raid markers stay native.
  Compact capsules use the native health texture and colors, a one-pixel outline
  and a fixed six-unit height (four-unit fill); width follows the plate scale.
  Rounded ends are clipped proportionally even at very low HP. Frames are reused
  across native plate recycling; invalid health data restores the native art.
  Names keep their native font size.
- Enemy names are hidden by default, including missing or ambiguous identities.
  Only a uniquely highlighted target positively classified as an elite, guardian,
  boss or final boss keeps its name. `AZS_UnitKind` takes precedence; generic
  native elite classification is a fallback only when that API is absent.
  Untargeted special enemies may therefore also have hidden names: Lua does not
  expose a GUID/unit token for every retained plate. Names in `AZS_UNITS` alone
  are not sufficient to identify a plate. Reused plates are checked each frame.
- AZSXUI uses the native world-number font. With the matching
  AZSX DLL, native outgoing damage renders at one third size for normal hits and one half for critical hits without changing fonts.
- AZSXUI owns the circular custom-event and native-cache minimap pins, their native
  portrait mask, tracking-ring border, hover text and edge projection. Supply and
  trial icons retain their styles; caches use the native `INV_Box_02` icon; supplies retain `INV_Crate_01`.
  Both keep the same circular mask and tracking ring. A Mechanical Chicken
escort uses the native `INV_Misc_EngGizmos_06` icon at its current exposed position and
removes the pin when the scene ends.
  Native cache coordinates come directly from `AZS_CHESTS`. Native cache opening,
  purchases, event progress and per-recipient availability stay authoritative.

### Minimap data boundary

`Minimap.lua` consumes `AZSX_GetState(1).data` from the matching current AZSX
addon contract: native view center, six zoom radii, native ping reference,
run/recipient identity and bounded personal event rows. Live zoom comes only
from `Minimap:GetZoom()`, never the bridge. Missing/invalid zoom hides pins. It does not decode a
second copy of the private payload, copy cache coordinates through the DLL,
register engine POIs or send gameplay commands. The embedded AZSX presentation
no longer creates pins or listens for minimap pings.

Pins require the bridge, valid current-run data and the native Minimap widget.
Without it, nameplate styling remains independent but minimap pins stay hidden;
hero coordinates alone cannot reconstruct a panned minimap reliably. Hidden or
stale data, menus and `/azsxui pins off` hide all addon pins. Available caches and
events remain visible while choosing or paused. Removing AZSXUI removes this
optional pin presentation, not native world objects or gameplay. Only the replaced native guardian flag is hidden, with its prior preference
restored when pins are disabled/unavailable. Up to nine custom, 128 guardian
and 128 native cache rows
are rendered; extra native caches are omitted from this bounded cosmetic view.

Commands:

```text
/azsxui pins on
/azsxui pins off
/azsxui plates on
/azsxui plates off
/azsxui scale 0.6
```

Plate styling and marker toggles change immediately and are reversible.
AZSXUI initially enables enemy plates and Overlapping once. Native visibility
keys/options remain available and later choices are respected; bindings are unchanged. Avoid enabling another addon that styles the
same plates or pins.

## Camera and inspection

In the world, right click toggles persistent native camera look. A second right
click releases the cursor for tooltips. Menus, reward choices, character and
talent panels also release it; closing a panel leaves inspection mode active
until the next right click. Left click keeps its native action. A small CAMERA
label is shown while mouselook is active. The Key Bindings menu can assign
AZSXUI camera toggle and release keys. `/azsxui camera toggle`,
`/azsxui camera inspect` and `/azsxui camera status` are also available.

This controller does not move the pointer. The native Lua build has no confirmed
cursor-position setter; the explicit switch prevents unexpected recapture after
consulting UI. Disable/remove SurvivorsCombatMode before using this controller:
its own WorldFrame hooks and automatic reacquisition conflict, so AZSXUI leaves
camera control to that addon whenever its functions are present.

## Development and limits

Source is in this directory. Run `sh tools/dev/check.sh --profile azsxui` from the
repository root for required Lua 5.1 tests. API probes are explicitly opt-in
helpers under `tools/dev`, not shipped addon files. Use the matching AZSX release for dependent widgets; do not combine this addon
with an older build that also applies native nameplate scaling. Standalone tests use an exact native executable without
any AZSX DLL. Visual size, layout and special-rank coverage require separately
scoped live observations; portable tests alone are not rendering evidence.

## Licenses and references

AZSXUI code: GNU GPL v3, see `LICENSE`.

Technical references: [Survivor Plates and Survivor UI](https://github.com/morderous/survivor-ui)
and [SurvivorUI Classic](https://github.com/morderous/survivorui-classic).
These projects demonstrated the native Lua plate widgets and the `DAMAGE_TEXT_FONT`
route. AZSXUI keeps its own small implementation and does not depend on them.

## AZSX event widgets

`Events.lua` owns Supply/Fortune/crystal pickup panels and Sayge's progress bar. It also observes the solo or cooperative Mechanical Chicken escort and shows the equipped engineering gadget in the status row below health, using the native-style AZSX API 1.13. The addon does not control movement or rewards.
`AZSX_GetState(1)` supplies structured values in `data` and every semantic title,
reward, note and color in `texts`, all from AZSX. AZSXUI does not calculate
rewards, timers or gameplay. Goblin appearance/defeat/escape announcements keep
their existing native CHAT_MSG channel; they are not these custom panels.

Each widget has independent saved settings in `AZSXUIDB.widgets`, applied without
a client restart. A shared anchor is centered below the native top HUD (UIParent TOP, 0,-125).
Visible widgets stack in Resurrection, Sayge, Supplies, Fortune, Crystal, native rewards order, with a 12-unit gap;
scaled height is included and hidden/expired panels leave no empty slot.
Widget position settings are offsets from their automatic stack slot, not world
coordinates. Scale changes the full widget/text without shifting the shared
anchor. Defaults are zero widget offsets, scale/opacity 1 and visible.
Old absolute Y preferences migrate to equivalent offsets from their old defaults.
The former centered anchor default (0,-85) migrates once to (0,-125), clearing
the expanded top HUD. Other saved positions remain unchanged.

```text
/azsxui widget anchor position 0 -125
/azsxui widget supplies position 0 0
/azsxui widget fortune scale 1.2
/azsxui widget crystals scale 1.1
/azsxui widget sayge opacity 0.75
/azsxui widget supplies show off
/azsxui widget supplies show on
/azsxui widget fortune reset
```

Widget names: `supplies`, `fortune`, `crystals`, `sayge`, `revive`, `loot`, `marks`; `anchor` accepts position/reset. Position bounds are -4096..4096,
scale 0.5..2 and opacity 0..1. Invalid commands are rejected; malformed saved
preferences return to safe defaults. Hidden notifications still consume their
normal presentation lifetime and do not replay when enabled. Choice/pause
suppression and identity-scoped Supply queuing remain in effect. Sayge hiding
only hides the display, never affects presence, progress or completion. In coop,
the bar stops and AZSX supplies a waiting message while any required participant
is outside; earned progress resumes when everyone returns. Missing
or stale AZSX data hides all widgets. The matching rebuilt AZSX bridge is required.

## Public AZSX data contract

AZSXUI consumes [AZSX addon contract](../../docs/features/ADDON-API.md) through
`AZSX_GetState(1)` and `AZSX_GetAPIVersion()`. Structured values live in `data`;
AZSX-owned wording/assets/colors live in `texts`. It no longer consumes the old
private presentation/minimap getters. Other addons can use the same native-style
globals, getters and registered handlers without installing AZSXUI. Missing or
unsupported API versions hide AZSXUI's dependent widgets; standalone plates remain.

The guardian minimap guide uses all native `AZS_GetLairs()` entries (with `AZS_GetFlaggedLair()` only as an older-API fallback) and the native
`Ability_Warrior_Challange` taunt icon, with the same round border as other pins.
Only the replaced native flag is hidden; its prior HUD preference is restored
when pins/projection are unavailable. Sayge progress uses WoW's native casting
bar texture/border, with the trial title inside and status below. Fortunes use
green +X% values rather than visible rarity labels. Timed Supplies buffs last 15–120 game seconds; unclaimed piles last 90 game seconds (claimed piles still retire early).

Consumable family 6 is Swiftness. Family 7 is an armed healing potion with zero remaining seconds until it is consumed. The separate
`discovery.quality` field describe the effect family and Classic name color. AZSXUI uses supplied colors
and text; it does not infer color from buff tier or gold value.

## Classic WoW notifications and buff order

Supplies/Fortune/crystal popups use native UI-Tooltip-Background/Border, UI-Quickslot2
and Friz Quadrata, preserving AZSX wording, quality colors, anchors, scaling,
opacity and notification lifetime. Cache pins use native INV_Box_02, distinct
from Supplies INV_Crate_01. No texture is generated or bundled.

The native Sayge Fortune button moves to the first visual buff slot, at the
right end of the first stock buff row. Crystal Resonance follows Fortune, or
leads the row when Fortune is absent. It requires its current API aura slot and
matching native icon. Buff IDs/clicks/durations remain native.
Native anchor changes are preserved, and AZSXUI restores its original anchors
when neither priority buff is present or AZSX data is unavailable. The native tooltip distinguishes
owned Fortune strength from a stronger effective party bonus.

## Crystal pickup notices

New personal crystal pickups show an AZSX-authored native-style notification with
color and permanent bonus, stacked under the shared HUD anchor. Each notice lasts
three readable seconds; choices and pauses defer display. Rapid pickups queue separately (maximum 16). Restored totals after
addon reload, stale data or session changes do not replay old rewards. Notices
consume consecutive personal snapshots, not a historical event feed.

`/azsxui widget crystals position X Y`, `scale 0.5..2`, `opacity 0..1`,
`show on|off` and `reset` use the same controls as other event widgets.

## Native action bar

The game owns the action bar artwork, buttons, gryphons and XP geometry.

Opening the in-run character sheet also opens Stats. Its 29 rows are grouped under Offense, Specialization, Survival and Utility. Click Stats to collapse or reopen it. Utility shows shared XP and Gold in AZSX co-op, plus personal Luck and Pickup. Within each section, nonzero values appear first, followed by zeros and then unavailable values (—).



Native kills, gold and the next cache cost come from `AZS_RUN.kills`, `gold` and
`chest`. Native crossed-blade, coin and `INV_Box_02` icons replace the three
labels; hover explains each. The counter strip is centered eight units below the screen top, independently of
the bottom HUD. It replaces the native tally while valid. Below it, AZSXUI
presents native run time and the next-boss countdown through `AZS_GetRunTime`,
`AZS_GetUpcomingBosses` and `AZS_RUN.nb`. The native clock has no Lua position
API; its visibility preference is restored when this presentation is disabled
or time is unavailable. An already hidden clock stays hidden.

Five circular equipment positions consume optional AZSX contract 1.5
`data.equipment` / `texts.equipment`. They preserve native display order, ranks,
icons and owner-formatted tooltip bodies; empty positions use a native generic
equipment silhouette. No item category or native tooltip formula is guessed. Native comma-separated item effects appear on separate tooltip lines; native wording, blank lines and enchantment details remain intact.
This reuses the already captured local jewelry metadata; local spell builds
remain excluded from the payload. Missing/stale equipment data hides the row
and restores the previous native build preference. Stats and counters remain
usable without AZSX. Menus, disabled HUD and new sessions retire the drawer.

The installer owns `Hud.lua` alongside the other bundled AZSXUI modules. No
native image or font is redistributed. Another addon replacing the same bar is
not a supported simultaneous layout; use `/azsxui hud off` before enabling it.

### Persistent health, auras and cooperative context

The HUD places a health-only bar and separate native helpful/harmful aura rows
at a screen-relative bottom offset of 216 UI units, independent of the action/XP
and equipment stack. Helpful/harmful auras stay above health with their
native durations above their icons. The statistics panel opens immediately above native XP, in front of equipment. Close it to inspect or cancel a buff. Encounter health bars match the top counter ribbon in width. Up to two bars share each centered line, with room beneath for the largest debuff list on that line. The experience bar centers the run level and native XP progress together, for example `Level 5 · 21 / 27`. Health reads native
`UnitHealth`/`UnitHealthMax`. The portrait/resources are omitted and the original
player and target roots are visually hidden with their updates retained.
Disabling the HUD restores their alpha/mouse state. Native aura buttons retain
their IDs, cancellation, tooltip and duration handlers and full readable size.
With the HUD enabled, Crystal Resonance, Fortune, the equipped gadget and the
charged healing potion share one centered row below health. Only occupied slots appear. With the HUD disabled, Fortune leads the native helpful row,
followed by Crystal Resonance.

Cooperative spell/item children receive a shared native metal/stone panel
beside each retained party frame. Allied buffs sit outside its right edge,
wrapping after eight icons (up to two rows); panel width depends only on
spells/items. Their native identities/tooltips and original anchors are retained. Owner data and tooltip handlers stay owned by
the existing native presentation, and party admission/health/identity are not
replaced. Living guardians and an available Sayge trial appear as read-only
objectives beneath the minimap. Completed/absent objectives retire automatically.
Guardians come from active native `AZS_GetLairs` entries, including distant
guardians, with `AZS_GetGuardians` as a fallback if the lair API is absent.
Sayge comes from the current AZSX
snapshot, without adding a new transport. Rewards use the same native panel
theme while preserving AZSX text, quality colors, timing and widget preferences.

The counter strip has extra vertical
padding for its native icons. Panels use the verified native steel
`UI-DialogBox-Border`; health uses a capsule silhouette with a 22-unit fill and a centered
12-unit font, inset from all four edges. Objectives
use a native golden header beneath the minimap and floating gold quest titles / white tasks; retained cooperative party frames stay at the upper left.

The health bar is centered at two thirds of the bottom HUD width. Native buff
and debuff rows sit above it, each centered independently (eight icons per row,
including centered partial rows), without shrinking icons or duration text. Native duration labels move above
the icons; their values and visibility remain native. The statistics panel
opens immediately above native XP, in front of the equipment row.
The top counter ribbon uses a complete ornamented native frame, detached eight units
from the screen edge. Icons retain their padded 36-unit cells.
The clock remains below it with a clear gap. Player health has a capsule outline and a 22-unit fill, leaving room around
its 12-unit text. The native high-resolution portrait alpha mask shapes
the ends; a clipping viewport preserves the silhouette at low HP.

Event pickup panels and allied build panels use an unmarked area of the native quest-log
parchment, shaded dark brown to soften the background without introducing new
assets. Other HUD panels keep their existing darker theme. Pickup panels use a
320-by-80 default footprint and 32-unit icon, growing for wrapped native text;
the automatic notification stack uses the resulting height and saved scale.

The warm paper extends beneath the popup/allied-build border rather than leaving
a black inner gutter. No native texture is redistributed.

## Permanent-effect medallions and native notices

The health capsule uses the reviewed native `UI-StatusBar` grayscale ramp,
applied uniformly to the body and native 128px portrait-alpha caps. Applying
the same modulation avoids a visible join; clipping preserves low-HP proportions. Ally spell/item panels are centered vertically on their retained
party roots; buffs remain outside their right edge.

Crystal Resonance shows the total collected. Sayge's Fortune keeps its category and bonus in the tooltip; a small circular native icon overlaps the orb at its lower-right corner:
Darkmoon cards for Luck, a book for XP, and a coin for Gold. The gadget and healing potion use the same circular portrait and rim. XP/Gold use the supplied strongest-party
value when applicable. Both
consume the current AZSX snapshot, hide on unavailable data, and use the original
native player-buff tooltip when its current aura can be identified. Missing or
reused native slots fall back to AZSX-provided wording. Native aura IDs, values
and gameplay are unchanged. A right-click forwards the original native aura
handler only while its current identity still matches the badge. Converted aura buttons are only visually hidden;
their alpha/mouse state and native layout restore when the HUD is disabled.

Sayge's Trial uses a compact native casting-bar border/fill without an icon.
The same style presents `AZS_GetRevive()` progress, either reviving an ally or
being revived. Resurrection takes display priority over the ritual. Native
simulation owns progress, interruption and completion; AZSXUI does not advance
the native state. The prior native `revive` visibility preference restores on
HUD disable or unavailable/invalid API data. `/azsxui widget revive ...` uses the
same scale, opacity, position and visibility controls as other event widgets.

Current boss-item popups use API 1.6 structured local receipts with their real
native icon, rarity and new-collection flag. Already-collected items are silent.
Marks use recognized local English chat receipts and the native rune icon,
preserving decimal amounts. Contiguous gains refresh a three-second quiet window
and remain one popup, including duplicate-item mark awards. Older providers retain their bounded chat fallback;
unknown locales keep native loot presentation. Ordinary chat is untouched.
The prior native loot preference restores when the HUD/provider is unavailable.
These messages queue through
choices/pauses without replaying past receipts after HUD disable. A new
ritual arrival reserves six seconds for its native announcement before native
reward popups continue; queued receipts keep their full display lifetime. Widget names
`loot` and `marks` accept the same presentation settings. They do not add AZSX
public API fields, a transport, or native hooks.


Native HUD visibility originals are recorded in `AZSXUIDB.nativeHudVisibility`
before AZSXUI hides a native part. Logout restores native visibility, and addon
load recovers any recorded originals from a previous session. This prevents
persistent native CVars from leaving the clock/loot/revive widgets hidden after
a restart. Originally hidden native parts are not forced on.

With the custom HUD enabled, the native minimap sun/time-of-day ornament and
upper zone-title/close-button strip are visually hidden and their mouse input
disabled. The minimap, border ring, zoom controls and markers stay native.
Disabling the custom HUD restores the original alpha and mouse state.

### Compact HUD defaults and feedback

The first AZSXUI load enables enemy nameplates and sets the native
`nameplateLayout` CVar to `overlap` (Overlapping). This is a one-time default;
later native visibility/layout changes are respected. The previous factory
scales of 0.7/0.6 migrate once to 0.5; other saved scales remain unchanged. Enemy
capsules are six UI units high, with a one-unit outline. Player/enemy caps use
the native 128px `TempPortraitAlphaMask`, avoiding enlargement of WHITE8X8;
the fill body keeps the same native-derived gradient as its caps and clips at actual HP.

The gold value receives an additive center-lit glow from native `Cooldown/star4`, behind the number rather than around it, when held gold covers the
current positive cache cost. Missing/invalid values and unaffordable caches
hide it. AZSX bonus numbers appear green and their labels white in event
notifications; supply item names retain their native reference rarity colors.
Native receipt values become structured presentation: item name/rarity and
quantity, or earned marks and total. Chat itself is unchanged.
Sayge's title and remaining seconds sit inside the casting track; party
waiting feedback stays below it. Progress remains AZSX-authoritative, including
the cooperative suspension rule.

### Bottom-bar refinement and encounter layout



The counter ribbon remains 300 units wide. The permanent in-run Objectives
header is 40 units high, uses an ornamented native dialog cartouche and sits eight
units inside the minimap anchor's right edge. The stats frame uses the native thin tooltip
border, with its bottom edge clipped; oversized dialog-header strips are gone.
Floating quest rows show living guardians, Sayge and empty lairs that will fill
within 60 native seconds. Native `left` controls the countdown; zero means
"Arriving soon". With no current objective, the header remains with a muted
empty-state line. Allied builds stay at the upper left.

Notifications keep a 320-by-80 minimum footprint. Their text block is centered
vertically beside the centered 32-unit icon, with consistent side padding and
height growth for wrapped text. Empty detail lines do not consume layout space.

The native boss/guardian canvas cannot be reanchored through Lua. AZSXUI reads
`AZS_GetBosses()` and awake entries from `AZS_GetGuardians()` into a bounded
health stack below the top ribbon/clock. IDs deduplicate rows, native HP/max
control fill; names are centered and no percentage is displayed, and enraged bosses use a warmer color. Native debuff icons appear beneath each matching bar in at most two compact rows of 12. Stack counts overlay the icons; hovering shows the native name and remaining time. An overflow count marks additional effects. The native
`bosses` visibility preference is retained and restored on disable or missing,
invalid or excessive data (more than eight health rows). Default event popups
move below this stack; explicitly customized popup anchors remain unchanged.

Outgoing damage stays attached to native enemy impacts, with the unchanged
native face at one third of native size for normal hits and one half for critical hits.
Healing size and other world-text categories are unchanged. Critical spell
icons remain deferred; no screen-fixed feed or heuristic Lua target match is
introduced.

Native item pickups use API 1.6 structured local `drop` receipts: the original
native name, rarity, new-collection flag and icon are copied from the existing
UI notification boundary. No name lookup or guessed icon is needed. The
bounded journal is recipient/session scoped and heartbeat deliveries do not
replay a receipt. Marks retain their separate earned/total presentation.

### Goblin health and crystal total

The encounter-health stack also includes live elite `Venture Co. Treasurer`
rows from native `AZS_UNITS`. This is the exact fixed name/rank of the AZSX
native creature definition and query. Native spawn IDs deduplicate health rows;
current/max HP and disappearance stay native. `AZS_GetDebuffs(id)` supplies their debuffs by the same spawn ID. Each icon and border uses the native `PartyBuffButtonTemplate` with its debuff border retained. No new native hook, creature scan,
health estimate or target/nameplate matching is introduced. Other elites and
normal creatures with that name are excluded. The native special-unit list is
bounded to 40 entries; if the goblin is absent from that API, no bar is invented.

Crystal Resonance now shows just the total collected count over the lower-right
of its icon. The four per-color counters beside it are removed. Color totals and accumulated bonuses remain in the original native/AZSX
tooltip and API. Each tooltip row starts with the accumulated bonus, then shows
the number of crystals of that color.

### Detached trackers and character shortcut

The Stats button belongs to the native in-run character sheet. The character binding and other tabs keep their native behavior.

The player health bar shows current HP on the left and percentage on the right. Encounter health bars show centered
native names with no percentage text; their fill still uses native current/max
health. The encounter stack moves down with the detached top ribbon/clock.
All guardian names/counts and arrivals share one Guardians quest; Sayge remains
a separate quest. Wrapped text determines row height instead of fixed rows.

Minimap pins have explicit layers: Sayge > active guardians > Supplies > caches
> returning guardians. Every valid active lair is rendered, not only the flagged
one. Returning lairs appear within 60 native seconds. Native pings/controls are
not modified. Exact same-position pins can still overlap; native world
coordinates and edge projection are preserved. The pool is bounded to 265 pins
(9 custom, 128 caches, 128 lairs). Disabled/invalid projection restores the flag.

Popup duration suffixes move to a dedicated `Duration:` line, retaining the AZSX
text and numeric coloring. This does not change authoritative durations.
Goblins no longer receive an automatic skull; manual raid markers remain native.

Spawned bosses appear first, one objective per living boss. Guardian arrival timers follow the name and appear only
at 30 native seconds or less. Earlier incoming entries have no numeric timer.
Counter cells anchor by their vertical center, including after HUD reactivation.
AZSXUI suppresses item-drop toasts for native `isNew=false` receipts; earned marks
keep their own notifications. With a current structured provider, the native loot
canvas is hidden before drops arrive and restored if the provider becomes
unavailable or the HUD is disabled. The API journal still exposes every receipt.

Equipment uses the native category order: amulet, two rings, then two trinkets,
preserving native order within each category. Empty positions retain their native
neck, finger or trinket silhouette; items never move into a different category.
Supplies sold for gold show the exact AZSX amount with the native WoW tooltip
gold coin. Quest titles, objective entries and arrival timers use distinct
spacing; arrival timers remain limited to the final 30 seconds.

The native map draws the player arrow. AZSX pins fade near the hero and remain hoverable. Pins of the same kind and status with centers less than 10 UI units apart share one count badge until they separate.

### Live stats, level and reward receipts

API 1.7 supplies optional cooperative effective XP/gold multipliers from AZSX's
actual award policy. The HUD substitutes those values for personal AZS_STATS;
other stats stay native, and zero/nonzero row membership refreshes while open.
Character level uses native AZS_RUN.lvl in the experience bar. Item tooltips retain
native owner lines and rank, plus verified current ring enchantments. A stale or
unmatched ring never inherits another player's enchantment.
Fortune notifications consume stable award receipts; choices defer pending
notifications but cannot re-arm an already started toast. Contiguous native mark
receipts within three seconds share one popup with summed gains and the newest
native total, preserving decimals and unrelated item order.

## Cooperative Rules information

Hover the kill counter to see current party members ranked by their personal run kills. Tied players share a rank. The circular Run information button shows N for Normal or H1–H5 for Heroic. Hover or click to see your class’s missing set pieces from all five bosses in this zone. In co-op, the same panel also explains the shared rules.
