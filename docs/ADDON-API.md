# ASX addon contract 1.2

ASX follows Azeroth Survivors' addon conventions: global tables, matching
`*_UPDATED` timestamps, `ASX_Get...` functions and registered Lua handlers.
This is an ASX extension, not a replacement for the game's `AZS_*` API. No new
network transport, outgoing addon message or gameplay command is introduced.
The currently supported engine is the reviewed 0.130.1 build.

## Availability and version

The matching ASX DLL/retained Lua bridge must be active in this process. API
functions may be absent when ASX is disabled or before its Lua has loaded; an
addon must tolerate that and retry after world entry or with bounded polling.
ASXUI is a consumer, not the provider: other addons do not need ASXUI installed.

`ASX_GetAPIVersion()` returns `major, minor` (currently `1, 2`). Major 1 is a
stable contract: incompatible field meanings/types/removals require a new major;
minor versions may add optional fields. Ignore unknown fields. Tests cover the
contract; this source change does not update an older installed DLL.

`ASX_GetState([major])` returns an independently owned snapshot. Omitted major
selects v1; passing `1` explicitly is recommended. Unsupported majors return
`nil, "unsupported_version"`. No native pointers or 64-bit numeric player GUIDs
are exposed. Mutating the returned tables, including nested arrays/colors, never
modifies ASX or snapshots obtained by another consumer.

## Snapshot envelope

| Field | Meaning |
|---|---|
| `api.major`, `api.minor` | Contract version, currently 1.2 |
| `available` | Boolean; check before accessing run fields |
| `reason` | When unavailable: `outside_run`, `stale`, or `missing_or_invalid_payload` |
| `revision` | Monotonic content revision within this Lua environment; not persisted across client reloads |
| `phase` | Native phase (`playing`, `choosing`, `paused`, `dead`, etc.; `unknown` if unavailable) |
| `session.epoch` | Opaque string; changes with the ASX session/membership generation |
| `session.recipientGUID` | Exact decimal string; never convert it to a Lua number |
| `updatedAt` | Last accepted ASX payload update, in `GetTime()` seconds; omitted when unavailable |
| `data` | Structured local-recipient state below |
| `texts` | ASX-owned presentation strings/assets/colors, separate from values |

Available phases are playing, choosing, paused and dead, with a valid private
payload no more than four wall-clock seconds old. Outside these conditions the
snapshot is explicitly unavailable, has empty `data`/`texts` and no session.
Do not keep rendering old data after an unavailable snapshot. An available run
may omit optional features: absent data does not mean a zero-valued reward.

Revisions change for structured/text changes, phase, session or availability.
A heartbeat or receipt timestamp refresh alone does not advance the revision.
The heartbeat still prevents expiration during common pauses. API snapshots are
observations for presentation; they are not authority for purchases, rewards,
progress, damage or timers.

## Structured data

All fields below are optional except where their enclosing record exists. Only
the local recipient's ASX features are public in v1; remote builds are not part
of this contract. Cache coordinates remain in native `AZS_CHESTS`.

| Record | Fields and units |
|---|---|
| `data.fortune` | `boss`: ASX grant/event counter, not an enemy spawn ID; `category`: active rolls 0..2 (Luck/XP/Gold); legacy 3/4 retain pickup/speed meanings but are no longer produced; `rarity` 0..4; `amount`, `best`: hundredths of a percentage point (1200 means 12%) |
| `data.fatigue` | `stacks` 0..3; `seconds`: remaining alive-game-time seconds, rounded upward; not a wall-clock deadline |
| `data.consumables[family]` | Dictionary keyed 0..6, **not** a 1-based list: `family`, `tier` 1..3, `remaining` whole game-time seconds, `index` native aura slot 0..31 |
| `data.discovery` | `event`: recipient/session-scoped receipt ID; `rarity` 0..3: legacy gameplay reward bucket, not name color; optional v1.2 `quality` 1..2: Classic Common/Uncommon cosmetic quality; `gold`: actual whole-gold sale receipt (0 for a consumable discovery). No native item ID is promised |
| `data.sayge` | `event`: trial identity; `held`: authoritative cumulative centiseconds; `maximum`: target centiseconds (currently 2000); `inside`: local native presence boolean; optional v1.1 `allInside`: collective readiness (false suspends progress without losing held time) |
| `data.minimap` | `identity`: opaque session/recipient key; `x`, `y`: world view center in yards; `radii`: 1-based six world radii for zoom 0..5; `pingX`, `pingY`: world ping reference. No zoom fallback field |
| `data.markers` | 1-based list, at most 9: `event`, `kind` (0 supplies, 1 trial), world-yard `x`, `y`, and `left` remaining game-time seconds (0 for a persistent trial) |

Current Fortune categories: 0 Luck, 1 Experience, 2 Gold. Legacy IDs 3 Pickup
Radius and 4 Movement Speed retain their historical meanings for compatibility,
but current ASX no longer generates them. Rarity: 0 Common, 1 Uncommon, 2 Rare, 3 Epic, 4 Legendary (discoveries
use only 0..3). Consumable families: 0 Recovery, 1 Critical, 2 Defense,
3 Vitality, 4 Ferocity, 5 Reach, 6 Swiftness (v1.2). Do not use `table.getn`/`ipairs` to enumerate
consumables; use `pairs` or the explicit family range. Event IDs must be combined
with `session.epoch` and `session.recipientGUID` for deduplication. Zero `left`
has different meanings for a persistent trial and an expired supply pin.

Projection still requires native mode/center/ping reads in ASX. Use native
`Minimap:GetZoom()` for zoom and `GetPingPosition()` for live projection. Hero
position is not a substitute for the center of a panned view. Unavailable
projection is represented by a missing `data.minimap`.

## ASX text and presentation

`texts.supplies`, `texts.fortune` and `texts.sayge` contain the applicable
`title`, `reward`, `note`, `icon` and RGB `color` fields; omit fields that do not
apply (Sayge currently provides title/note only). Supplies additionally provides
`name` and `description` as uncomposed reward strings. `texts.consumables[family]`
contains `name`, `description` and full texture `icon` path.

Titles/notes/reward strings are authored/formatted by ASX. No localization
language is promised; do not parse these strings to recover numbers or identity.
Use `data` to draw your own representation and `texts` to retain ASX wording.
Colors and assets are presentation metadata, not gameplay authority. Clients may
format supplied values differently without changing their meaning.

## Native-style globals and notifications

`ASX_STATE` is the last snapshot published by the retained observer.
`ASX_STATE_UPDATED` is its publication time in `GetTime()` seconds. They are
populated on the observer's first update. They can lag a fresh getter until the
next UI frame. Publication includes changed receipt timestamps, while handlers
are called only for meaningful revision changes. Treat the globals as read-only,
like native `AZS_*` tables: a Lua global cannot prevent another addon from
mutating that shared view. Use the getter for isolation. Global mutations do not
reach ASX's internal state and are replaced at the next publication.

`ASX_RegisterStateHandler(handler)` returns true, or `nil, "invalid_callback"`
/ `nil, "subscriber_limit"` (maximum 64 distinct functions). Registering the
same function twice is idempotent. `ASX_UnregisterStateHandler(handler)` returns
whether it removed a registration. Keep the original function to unregister it.

Handlers receive **one independently copied snapshot**, including unavailable
states. Registration itself does not invoke a callback: read initial state with
the getter. Delivery is on subsequent observer updates, coalesces intermediate
changes and is not an event history or exactly-once reward feed. A failing
handler cannot block others. Unregistering a pending handler suppresses its
call; new handlers added during delivery wait for a later change. Getters inside
a handler are supported and do not recursively emit notifications.

This follows the native registered-handler pattern (such as banner handlers).
It does not invent a client event or send `CHAT_MSG_ADDON` messages. Native
`AZS_*` events and native goblin CHAT_MSG announcements remain unchanged.

```lua
local frame = CreateFrame("Frame")
local bound, last = false, 0
local function changed(state)
    if not state.available then
        -- Hide your ASX widgets.
        return
    end
    local trial = state.data.sayge
    if trial then
        -- Draw held/maximum; no parsing of state.texts.sayge.note required.
        local fraction = trial.held / trial.maximum
    end
end
frame:SetScript("OnUpdate", function()
    if bound or GetTime() - last < 0.25 then return end
    last = GetTime()
    if type(ASX_GetAPIVersion) ~= "function" then return end
    local major = ASX_GetAPIVersion()
    if major ~= 1 then frame:SetScript("OnUpdate", nil); return end
    local ok = ASX_RegisterStateHandler(changed)
    if ok then
        bound = true
        changed(ASX_GetState(1))
        frame:SetScript("OnUpdate", nil)
    end
end)
-- Before retiring your consumer: ASX_UnregisterStateHandler(changed).
```

## Private interfaces

`AZS_RUN.asx` remains private compressed transport, not this public schema.
`ASX_GetEventPresentation()` and `ASX_GetMinimapData()` are pre-v1 compatibility
adapters, now detached too; new addons should use `ASX_GetState(1)`. Diagnostic
readouts are not public contracts. ASXUI now consumes v1 and safely hides its
views if the getter is missing or the major is unsupported. No UI addon is
required to decode the private envelope or read native memory.

Validate with `sh tools/dev/check.sh --profile asxui --profile official-party-ui`.
API tests cover deep-copy isolation, exact GUID strings, schema/version errors,
stale/session/phase changes, heartbeat suppression and handler lifecycle/errors.

In v1.1, Sayge `allInside` reflects host-authoritative collective presence; the
ASX note explains waiting while another participant is outside. Older private
five-field ritual packets leave this property absent (unknown), rather than
pretending local presence proves collective readiness. Existing v1 consumers
remain compatible and must ignore the added field.
