# Native world and reward presentation

ASX enriches the reviewed 0.130 client without replacing world rendering,
combat logs, nameplate health updates or the native marker transport.

## Standalone presentation: ASXUI

The optional [ASXUI addon](../addons/ASXUI/README.md) owns enemy plate styling
and minimap pin rendering. Install it in `benilla-config/AddOns/ASXUI`, outside
`ASX/`. Plate styling works through the native addon API without the DLL.
The DLL installs no plate-input, plate-style or damage-size hooks.

ASXUI retains native plate widgets and changes their geometry to 70% by default.
Friendly rows, health updates, placement, border art and visibility keys remain
native. Enemy levels/skulls and names are hidden by default. A unique highlighted
target positively identified as elite/guardian/boss/final keeps its original name
font and alpha. Missing/ambiguous ranks and untargeted rows remain hidden; the
addon never infers plate identity from a matching encounter name alone.

The native world-number face is now the default. The earlier renamed OFL face
with one-third glyph geometry remains opt-in for existing users, with a client
restart required. It affects the shared world-healing face too. There is no
confirmed outgoing native same-face size control in the reviewed 0.130 engine:
`DAMAGE_TEXT_FONT` selects a cached face, while UI `FontString:SetFont` and
`CombatText` size controls concern separate UI text. Unknown retail CVars are
not treated as working controls merely because Lua accepts a value. A Lua
replacement renderer with the native face remains an investigation, not shipped
world-number parity or a reason to restore native damage-size hooks.

ASXUI `Minimap.lua` renders custom supplies/trials and native `AZS_CHESTS` caches
with the same circular portrait/tracking-ring style. The retained ASX Lua code
exposes the versioned `ASX_GetState(1)` contract (validated current personal
events, exact projection and session identity). It creates no minimap widgets. Accurate panning
and ping inversion still require this bridge; standalone plate styling does not.
The addon hides pins when projection is absent/stale, outside a run or disabled,
and retires old local-ping references on session change. Native cache purchases,
scenes, availability, raid markers and minimap controls remain untouched.

## Native healing messages retained in ASX

Native direct and periodic healing logs do not produce world numbers in this
client. ASX still connects their actual positive amounts to the native world-text
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

ASX observes the final clamped HP gain at the native meter boundary. Native
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
all new callback/helper entry bytes. The retired damage-size stub and its exclusive ABI tests are removed.
Retained healing callbacks keep their argument/register coverage.

## Reward annotations

The shared bonus UI captures the existing card separator's native Node and
rarity color only at its reviewed callsite. Its four grid vectors must be empty;
owned allocations cannot be aliased. A fresh native gradient bundle is constructed
and attached once before each ASX annotation, including compact retained bonuses.
Cards without ASX annotations retain their original construction.

The header remains centered with a 32px icon beside a left-aligned vertical title/status
column with no additional row gap. Its native 40px row has no extra top margin; new bonus values follow
below the header. Retained bonuses keep `Active` in that text column and the
existing native hover tooltip for their values. The adapter constructs the text
column through the existing native Node/ChildOf builder and borrows the native
commands only within the synchronous card build. When native formatting delivers
the status as a later line, only the column entity is retained until the end of
that card build; the caption reborrows the current callback's command queue.
Original `Sets...`, `Applies...` and `Improves...` wording is preserved. Glyph gameplay stays disabled;
these shared helpers also serve enchantments and amulet gems.

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

## Further ASXUI extraction candidates

The remaining presentation-only candidates are ASX aura icons/descriptions/
countdowns and retained party buff/build widgets. These already use Lua in the
embedded presentation; extracting
them primarily separates addon ownership and settings, rather than removing native
memory hooks. A single validated read-only bridge should retain recipient/session
identity, freshness, reward deduplication and native owner-formatted tooltips.
Gameplay calculations, authoritative timers, grants and scene creation stay in ASX.

Native reward-card/menu decorations and world-heal publication are not equivalent
Lua-only candidates: their current native widget/data seams have no confirmed
complete addon replacement. Prefer an actual reduction in native capture through
proven APIs over replacing existing native HUD components solely to move code.

The projection adapter no longer reads either native minimap zoom field. ASXUI
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
ASXUI with per-widget saved position, scale, opacity and visibility. ASX remains
the text/data producer through `ASX_GetState(1)` (`data` and `texts`). These candidates
are implemented; aura and party/build presentation remain possible future work.
