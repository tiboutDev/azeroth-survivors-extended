# Native world and reward presentation

AZSX enriches the [selected native client](../compatibility/CLIENT-COMPATIBILITY.md) without replacing world rendering,
combat logs, nameplate health updates or the native marker transport.

## Standalone presentation: AZSXUI

The optional [AZSXUI addon](../../addons/AZSXUI/README.md) owns enemy plate styling
and minimap pin rendering. Install it in `benilla-config/AddOns/AZSXUI`, outside
`AZSX/`. Plate styling works through the native addon API without the DLL.
The DLL installs no plate-input or plate-style hooks. A narrowly scoped exact-build
damage-size seam preserves the native font and enemy attachment.

AZSXUI retains native plate widgets and scales their width to 50% by default.
Enemy health uses a six-unit capsule with a one-unit outline and the native
status-bar grayscale ramp/colors, applied uniformly across body and caps. Its rounded ends use the native 128px portrait alpha mask. A clipped full-width fill preserves rounded ends at
low HP. Its frames are allocated once per retained plate (bounded to 256), then
reused; color/geometry writes occur only when changed. Native border/glow and
health texture are hidden only while the replacement has valid native values,
and restored on invalid data, friendly reuse or styling disable. Raid markers,
friendly rows, health updates, placement and visibility keys remain native. AZSXUI enables enemy plates and native
`nameplateLayout=overlap` once on first load; subsequent native choices are preserved. Enemy levels/skulls and names are hidden by default. A unique highlighted
target positively identified as elite/guardian/boss/final keeps its original name
font and alpha. Missing/ambiguous ranks and untargeted rows remain hidden; the
addon never infers plate identity from a matching encounter name alone.

Native damage uses its unchanged face, projection, lifetime and normal/critical
animation, at one third of the freshly calculated normal size and one half of the freshly calculated critical size. The selected
`DAMAGE_TEXT_SIZE` seam is guarded by exact executable identity and instruction
bytes. The renderer owns the current row in rbx and the interpolated/clamped
size in xmm8. Only normal/critical categories (0/2), excluding native green
healing, multiply xmm8 by 1/3 and 1/2 respectively before the original conversion and projection.
No stored size is modified, so repeated frames do not compound the reduction.
All other registers and integer flags are preserved; no callbacks or allocations
occur. Windows execution tests cover repeated baselines, categories and heals.
This adds one exact native compatibility contact. Critical spell icons remain
deferred; there is no Lua renderer or guessed target matching.

AZSXUI leaves the world-number font to the client.
No outgoing size CVar or `FontString` option is claimed to control world text.

AZSXUI `Minimap.lua` renders custom supplies/trials and native cache anchors (`AZS_CHESTS` fallback)
with the same circular portrait/tracking-ring style. The retained AZSX Lua code
exposes the versioned `AZSX_GetState(1)` contract (validated current personal
events, exact projection and session identity). It creates no minimap widgets. Accurate panning
and ping inversion still require this bridge; the standalone HUD, nameplates, native boss bars and guardian objectives run directly on the supported native client.
The addon hides pins when projection is absent/stale, outside a run or disabled,
and retires old local-ping references on session change. Native cache purchases,
scenes, availability, raid markers and minimap controls remain untouched.

AZSXUI also offers an explicit camera/inspection controller. World right click
toggles persistent native mouselook; menus, choices, talent and character panels
release it and never reacquire automatically on close. Native WorldFrame handlers
remain chained. The controller yields to SurvivorsCombatMode when installed.
No pointer warp or native camera hook is involved.

## Reference addon: projected combat text

Source review of [SurvivorsDragonUi](https://github.com/migwynkriid13/SurvivorsDragonUi/tree/0bef905c1dc75691dc4951d1bf8a06b5df781bf2)
at `0bef905c1dc75691dc4951d1bf8a06b5df781bf2` identified useful Lua-only techniques:

- `modules/nameplates/parser.lua` derives patterns from native localized combat
  format globals instead of assuming English chat sentences.
- `modules/nameplates/sct.lua` obtains already-projected plate centers with
  `GetCenter()` and effective-scale conversion, snapshots the origin for each
  animated number, and positions a texture beside critical text.
- The renderer indexes target names and recent plate HP losses. Its
  `ResolveOrigin` first prefers a unique matching loss, then a unique loss or
  visible candidate. Those are correlations, not destination GUID evidence.
- Ambiguous events use a centroid or ultimately the center of `WorldFrame`.
  This preserves message visibility but cannot guarantee enemy attachment.
  Percentage-only health bars cannot be matched to absolute damage amounts.

These findings establish a practical Lua nameplate-text technique, not a new
native world-to-screen or exact combat-destination API. Multiple same-name
enemies, interest-set omissions, lethal hits before plate discovery and plate
visibility remain important limits. AZSXUI does not disable native damage or
ship the reference fallback against the requested enemy-impact behavior.
Native same-face damage sizing now uses the exact seam described above;
exact critical-icon association remains deferred. The reference review is not
live validation of that external addon.
No external code, custom textures or fonts have been copied into AZSXUI.

## Native healing messages retained in AZSX

Native direct and periodic healing logs do not produce world numbers in this
client. AZSX still connects their actual positive amounts to the native world-text
writer. Native periodic variant 1 is healing; variant 0 already emits damage.
The direct critical flag retains native critical animation. Green messages are
prefixed with `+`; native entity lookup supplies attachment and position.

Both caster and recipient must match current native player/party aliases.
Periodic packets are observed before the native recipient-only log filter.
Missing or ambiguous identities drop only the cosmetic message. The writer is
borrowed solely inside the existing native log callback; owned native strings
transfer once to its native message queue. No healing, talents, meters or stat reconstruction are triggered by presentation.
The original native-log bridge emits no extra packets.

### Fractional periodic healing

Native periodic healing applies HP continuously and accounts for it in the native
meter without issuing a world-text packet. The shared heal helper also omits
packets for spell zero and effective gains below one HP. Merely observing the
native periodic combat-log callback cannot prove this gameplay path's display.

AZSX observes the final clamped HP gain at the native meter boundary. Native
periodic calls carry a scoped context; replayed foreign ticks carry their copied
caster identity and spell. Gains already reported by the native helper are
excluded. A bounded portable accumulator keeps source/recipient/spell identities
separate, retains fractional remainders and publishes whole amounts at most once
per simulation second. Native frame clocks and lifecycle teardown retire replay
state; invalid/nonpositive gains and overhealing produce no credit.

Publication uses the native cosmetic SPELLHEALLOG constructor and UI outbox.
It never calls a heal, changes HP, triggers a proc or adds a meter receipt. Native
UI delivery, including its existing cooperative transport, then feeds the retained
world-text renderer. Every helper/callback has exact-build entry-byte guards;
packet buffers transfer once, and uncertain commits are never retried.


Exact callback/register/layout contacts belong to the selected version descriptor and
`official_native/combat_text.rs`, not the portable rules crate. Guards cover
all new callback/helper entry bytes. The retained damage-size seam and its ABI test are scoped to normal/critical damage.
Retained healing callbacks keep their argument/register coverage.

## Reward annotations

The shared bonus UI captures the existing card separator's native Node and
rarity color only at its reviewed callsite. Its four grid vectors must be empty;
owned allocations cannot be aliased. A fresh native gradient bundle is constructed
and attached once before each AZSX annotation, including compact retained bonuses.
Cards without AZSX annotations retain their original construction.

The header remains centered with a 32px icon beside a left-aligned vertical title/status
column with no additional row gap. Its native 40px row has no extra top margin; new bonus values follow
below the header. Retained bonuses keep `Active` in that text column and the
existing native hover tooltip for their values. The adapter constructs the text
column through the existing native Node/ChildOf builder and borrows the native
commands only within the synchronous card build. When native formatting delivers
the status as a later line, only the column entity is retained until the end of
that card build; the caption reborrows the current callback's command queue.
Original `Sets...`, `Applies...` and `Improves...` wording is preserved; these shared helpers serve enchantments and retained gem representations.

## Cooperative markers

The host initializes active participants once per run, keyed by native slot and
GUID. Each receives a random available icon from all eight native markers,
without duplicates. Late arrivals receive an available icon once. Every occupied icon stays reserved, including marks already on participants
at initialization or on late arrivals. An already marked participant is considered
initialized without assigning another icon, including across a new run.

Initialization is committed before invoking the native marker command. The
adapter checks the current native table again immediately before each command;
an occupied icon or an already marked recipient cancels that automatic assignment. An
uncertain presentation operation is never retried over a manual choice. Refresh,
pause or resurrection does not enforce markers, and players may change or remove
them freely. Run teardown/identity change/clock rollback retires the initialization.
Solo mode never enters this host-only cooperative path. No new transport or
independent membership authority is introduced.

## Validation

Use the `official-party-ui` profile, including explicit Lua 5.1 checks, marker
policy tests, packet decoding and selected presentation memory tests. Execute
Windows ABI tests when wrappers/register contracts change. Visual trials must
inspect actual solo and two-player output; script creation, send acknowledgements
and native log entries alone are not visual evidence. Record artifact identities
and uncovered situations separately in ignored local evidence.

The fractional-heal observation seams require the full saved-register context: RSI
owns Game and RDI carries the shared helper spell. A volatile-only snapshot cannot
read those fields. The handlers observe the captured values without changing native
registers, flags or healing; Windows ABI probes exercise both installed stubs.

## Further AZSXUI extraction candidates

The remaining presentation-only candidates are AZSX aura icons/descriptions/
countdowns and retained party buff/build widgets. These already use Lua in the
embedded presentation; extracting
them primarily separates addon ownership and settings, rather than removing native
memory hooks. A single validated read-only bridge should retain recipient/session
identity, freshness, reward deduplication and native owner-formatted tooltips.
Gameplay calculations, authoritative timers, grants and scene creation stay in AZSX.

Native reward-card/menu decorations and world-heal publication are not equivalent
Lua-only candidates: their current native widget/data seams have no confirmed
complete addon replacement. Prefer an actual reduction in native capture through
proven APIs over replacing existing native HUD components solely to move code.

The projection adapter no longer reads either native minimap zoom field. AZSXUI
uses `Minimap:GetZoom()` exclusively, validates integer zoom 0..5 and hides pins
when it is missing/invalid, including during ping callbacks. `EventMinimap`
contains only center, six radii and world ping; its unused single radius and zoom
fields are removed. This removes one selected native zoom read and two native
zoom offsets, not a hook. The memory test leaves both old fields unmapped.

No complete native Lua replacement for the current center/mode/world-ping reads
has been confirmed. Hero world coordinates are not a panned view center;
`GetPingPosition()` is normalized and needs the world reference. Interior/exterior
zoom CVars can be equal, so comparing them does not reliably identify the mode.
The UI frame's screen center is not a world coordinate. Keep those reads until
an equivalent API is demonstrated across pan, indoor/outdoor changes and pings.

Supply/Fortune notifications and the Sayge progress panel are now extracted into
AZSXUI with per-widget saved position, scale, opacity and visibility. AZSX remains
the text/data producer through `AZSX_GetState(1)` (`data` and `texts`). These candidates
are implemented; aura and party/build presentation remain possible future work.

## Encounter stack and bottom HUD layering

AZSXUI reads boss, awake-guardian and AZSX goblin health from the native addon
tables. Goblins are native `AZS_UNITS` elite rows named `Venture Co. Treasurer`,
the fixed AZSX definition/query name, carrying actual spawn ID/HP/max HP. The
40-row native list bounds discovery; missing data hides the cosmetic row. Encounter health bars match the top counter ribbon in width. Up to two bars share each centered line, with room beneath for the largest debuff list on that line. The native `debuffs` arrays for bosses and guardians, plus `AZS_GetDebuffs(id)` for the goblin, supply compact icons beneath the matching bars. Their icon and debuff border come from the native `PartyBuffButtonTemplate`, with the original native data and entity IDs; no target-name matching or new hook is needed.
This does not infer a nameplate GUID or introduce a new native memory contact.

Bottom HUD order is equipment, statistics, native XP, then native gryphons.
XP's complete casting-border artwork remains intact and draws over the stats
panel joint. The owned XP level is restored on disable, preserving external
changes. The player health capsule sits 216 UI units above the screen bottom, above the equipment row. Native comma-separated item effects occupy separate tooltip lines while retaining owner text and enchantment lines. The stats panel has three vertical column rules, no horizontal group
separator, a thin native tooltip border and a clipped lower edge. Objectives use a compact closed ornamented native header, inset from the minimap
anchor instead of touching the screen edge. Crystal Resonance has one total count on its icon; its full tooltip is
unchanged.

## Current tracker presentation

The counter ribbon is detached eight UI units from the top with a complete ornamented
native border. The 40-unit Objectives header is inset eight units from its
minimap anchor; one Guardians quest contains live names/counts and upcoming
arrivals, alongside spawned bosses, Sayge and the chicken escort. Rows size to their text. All active
lairs receive pins, with explicit retained-frame levels: Sayge, live guardians,
Supplies, caches, then soon-returning guardians. No coordinate jitter or
replacement native ping logic is added. AZSXUI reads the live player position to fade nearby pins. The native minimap draws the player arrow.

Encounter names are centered, without percentages. Player HP displays only its
current amount. The character-sheet command and stats-button click open/close the native character
sheet and AZSX stats together while the HUD is active. Hover remains a compact
stats-only preview. Configured bindings, other tabs and native fallback remain
unchanged. API 1.8 projects XP and gold independently; unavailable shared values
are hidden instead of being replaced with personal native values.
Native goblin creation no longer reads the raid table or invokes its command to
assign skull; no automatic unmarking is performed. The diagnostic raid-table observer is host-only; solo trials explicitly skip
that raw-table observation rather than counting it as a pass. Existing participant marker policy remains
unchanged, with skull available for manual use.

Spawned bosses appear first, one objective per living boss. Guardian arrival timers follow the name and appear only
at 30 native seconds or less. Earlier incoming entries have no numeric timer.
Counter cells anchor by their vertical center, including after HUD reactivation.
AZSXUI suppresses item-drop toasts for native `isNew=false` receipts; earned marks
keep their own notifications. With a current structured provider, the native loot
canvas is hidden before drops arrive and restored if the provider becomes
unavailable or the HUD is disabled. The API journal still exposes every receipt.

Local equipment retains the native HUD `slot` category in the bounded AZSX bridge.
AZSXUI reserves neck, two finger and two trinket positions, preserving owner text
and native order within each category. Legacy category-less providers retain their
sequential display. The Supplies sale amount remains AZSX-owned and uses the native
small gold coin artwork, without converting whole AZSX gold into another currency.
Quest titles and individual entries have larger gaps than an entry and its timer.

Pins fade to 15% opacity at the player and recover full relative opacity at 20 UI units. Their positions and mouse targets stay available at every opacity. Pins of the same type, priority and rim status share one icon with a count only when their projected centers are less than 10 UI units apart (half a portrait width).

### Encounter debuff parity

Encounter icons use the actual native `TargetDebuffButtonTemplate`, native
count children, target-button sizes and `DebuffTypeColor` borders. They retain
per-entity native debuff data and show at most sixteen icons. Hover forwards the
native target tooltip only when the selected entity ID and one unique native
aura icon agree. Otherwise the tooltip shows the encounter's own effect name, stack count and remaining time when available.

At run entry, the action bar, player vitals and aura positions are applied together when the native bar is ready. Empty equipment slots reserve the layout while the first owner payload is pending.
