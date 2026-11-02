# AZSX addon contract 1.13

The DLL appends the provider to the selected native WorldStateFrame initialization script.
Neither `AddOns/ASX` nor `AddOns/AZSX` is installed or required. The retained
party presentation and native fallback auras initialize through the same path.
Direct native launches do not expose the mod contract. AZSXUI is optional.

The former `ASX_*` globals and `AZS_RUN.asx` transport key are renamed to
`AZSX_*` and `AZS_RUN.azsx`. Third-party consumers must update their names;
`AZS_*` native globals remain unchanged. Existing contract data shapes remain compatible.

AZSX follows Azeroth Survivors' addon conventions: global tables, matching
`*_UPDATED` timestamps, `AZSX_Get...` functions and registered Lua handlers.
This is an AZSX extension, not a replacement for the game's `AZS_*` API. No new
network transport, outgoing addon message or gameplay command is introduced.
The currently supported engine is the reviewed 0.138.0 build.

## Availability and version

The matching AZSX DLL/retained Lua bridge must be active in this process. API
functions may be absent when AZSX is disabled or before its Lua has loaded; an
addon must tolerate that and retry after world entry or with bounded polling.
AZSXUI is a consumer, not the provider: other addons do not need AZSXUI installed.

`AZSX_GetAPIVersion()` returns `major, minor` (currently `1, 12`). Major 1 is a
stable contract: incompatible field meanings/types/removals require a new major;
minor versions may add optional fields. Ignore unknown fields. Tests cover the
contract; this source change does not update an older installed DLL.

`AZSX_GetState([major])` returns an independently owned snapshot. Omitted major
selects v1; passing `1` explicitly is recommended. Unsupported majors return
`nil, "unsupported_version"`. No native pointers or 64-bit numeric player GUIDs
are exposed. Mutating the returned tables, including nested arrays/colors, never
modifies AZSX or snapshots obtained by another consumer.

## Snapshot envelope

| Field | Meaning |
|---|---|
| `api.major`, `api.minor` | Contract version, currently 1.13 |
| `available` | Boolean; check before accessing run fields |
| `reason` | When unavailable: `outside_run`, `stale`, or `missing_or_invalid_payload` |
| `revision` | Monotonic content revision within this Lua environment; not persisted across client reloads |
| `phase` | Native phase (`playing`, `choosing`, `paused`, `dead`, etc.; `unknown` if unavailable) |
| `session.epoch` | Opaque string; changes with the AZSX session/membership generation |
| `session.recipientGUID` | Exact decimal string; never convert it to a Lua number |
| `updatedAt` | Last accepted AZSX payload update, in `GetTime()` seconds; omitted when unavailable |
| `data` | Structured local-recipient state below |
| `texts` | AZSX-owned presentation strings/assets/colors, separate from values |

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
the local recipient's AZSX features are public in v1; remote builds are not part
of this contract. Cache coordinates remain in native `AZS_CHESTS`.

| Record | Fields and units |
|---|---|
| `data.fortune` | `boss`: encounter progression stage, not a unique reward ID; optional v1.7 `event`: actual award receipt, stable through heartbeat, party-generation and ritual-arrival changes; `category`: active rolls 0..2 (Luck/XP/Gold); legacy 3/4 retain pickup/speed meanings but are no longer produced; `rarity` 0..4; `amount`, `best`: hundredths of a percentage point (1200 means 12%) |
| `data.economy` | Optional v1.7 cooperative `xp` and `gold` effective multipliers (1.35 means +35%), projected from the actual AZSX award policies for the local recipient. Gold retains collector-specific native effects; values need not match across different builds. Missing means unavailable, not zero. These replace the personal `AZS_STATS` values for display; never add them to those values. |
| `data.sharedEconomy` | v1.8 cooperative projection with independently optional `xp` and `gold` multipliers. An empty/partial table explicitly marks unavailable shared values; hide those cells instead of falling back to personal `AZS_STATS`. Solo omits this record. The legacy `data.economy` complete-pair guarantee remains unchanged. Values use the same award policies and replace, never add to, native personal values. |
| `data.fatigue` | `stacks` 0..3; `seconds`: remaining alive-game-time seconds, rounded upward; not a wall-clock deadline |
| `data.consumables[family]` | Dictionary keyed 0..6, **not** a 1-based list: `family`, `tier` 1..3, `remaining` whole game-time seconds, `index` native aura slot 0..31 |
| `data.discovery` | `event`: recipient/session-scoped receipt ID; `rarity` 0..3: legacy gameplay reward bucket, not name color; optional v1.2 `quality` 1..2: Classic Common/Uncommon cosmetic quality; `gold`: actual whole-gold sale receipt (0 for a consumable discovery). No native item ID is promised |
| `data.sayge` | `event`: trial identity; `held`: authoritative cumulative centiseconds; `maximum`: target centiseconds (currently 2000); `inside`: local native presence boolean; optional v1.1 `allInside`: collective readiness (false suspends progress without losing held time) |
| `data.minimap` | `identity`: opaque session/recipient key; `x`, `y`: world view center in yards; `radii`: 1-based six world radii for zoom 0..5; `pingX`, `pingY`: world ping reference. No zoom fallback field |
| `data.markers` | 1-based list, at most 9: `event`, `kind` (0 supplies, 1 trial), world-yard `x`, `y`, and `left` remaining game-time seconds (0 for a persistent trial) |

Current Fortune categories: 0 Luck, 1 Experience, 2 Gold. Legacy IDs 3 Pickup
Radius and 4 Movement Speed retain their historical meanings for compatibility,
but current AZSX no longer generates them. Rarity: 0 Common, 1 Uncommon, 2 Rare, 3 Epic, 4 Legendary (discoveries
use only 0..3). Consumable families: 0 Recovery, 1 Critical, 2 Defense,
3 Vitality, 4 Ferocity, 5 Reach, 6 Swiftness (v1.2). Do not use `table.getn`/`ipairs` to enumerate
consumables; use `pairs` or the explicit family range. Discovery/trial IDs must be combined
with `session.epoch` and `session.recipientGUID` for deduplication. Fortune v1.7
`event` is a process-lifetime award receipt: deduplicate by recipient and event,
not by membership epoch or encounter stage. A new award may have identical
category and amount; an unchanged award survives party-generation changes. Zero `left`
has different meanings for a persistent trial and an expired supply pin.

Projection still requires native mode/center/ping reads in AZSX. Use native
`Minimap:GetZoom()` for zoom and `GetPingPosition()` for live projection. Hero
position is not a substitute for the center of a panned view. Unavailable
projection is represented by a missing `data.minimap`.

## AZSX text and presentation

`texts.supplies`, `texts.fortune` and `texts.sayge` contain the applicable
`title`, `reward`, `note`, `icon` and RGB `color` fields; omit fields that do not
apply (Sayge currently provides title/note only). Supplies additionally provides
`name` and `description` as uncomposed reward strings. `texts.consumables[family]`
contains `name`, `description` and full texture `icon` path.

Titles/notes/reward strings are authored/formatted by AZSX. No localization
language is promised; do not parse these strings to recover numbers or identity.
Use `data` to draw your own representation and `texts` to retain AZSX wording.
Colors and assets are presentation metadata, not gameplay authority. Clients may
format supplied values differently without changing their meaning.

## Native-style globals and notifications

`AZSX_STATE` is the last snapshot published by the retained observer.
`AZSX_STATE_UPDATED` is its publication time in `GetTime()` seconds. They are
populated on the observer's first update. They can lag a fresh getter until the
next UI frame. Publication includes changed receipt timestamps, while handlers
are called only for meaningful revision changes. Treat the globals as read-only,
like native `AZS_*` tables: a Lua global cannot prevent another addon from
mutating that shared view. Use the getter for isolation. Global mutations do not
reach AZSX's internal state and are replaced at the next publication.

`AZSX_RegisterStateHandler(handler)` returns true, or `nil, "invalid_callback"`
/ `nil, "subscriber_limit"` (maximum 64 distinct functions). Registering the
same function twice is idempotent. `AZSX_UnregisterStateHandler(handler)` returns
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
        -- Hide your AZSX widgets.
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
    if type(AZSX_GetAPIVersion) ~= "function" then return end
    local major = AZSX_GetAPIVersion()
    if major ~= 1 then frame:SetScript("OnUpdate", nil); return end
    local ok = AZSX_RegisterStateHandler(changed)
    if ok then
        bound = true
        changed(AZSX_GetState(1))
        frame:SetScript("OnUpdate", nil)
    end
end)
-- Before retiring your consumer: AZSX_UnregisterStateHandler(changed).
```

## Private interfaces

`AZS_RUN.azsx` remains private compressed transport, not this public schema.
`AZSX_GetEventPresentation()` and `AZSX_GetMinimapData()` are pre-v1 compatibility
adapters, now detached too; new addons should use `AZSX_GetState(1)`. Diagnostic
readouts are not public contracts. AZSXUI now consumes v1 and safely hides its
views if the getter is missing or the major is unsupported. No UI addon is
required to decode the private envelope or read native memory.

Validate with `sh tools/dev/check.sh --profile azsxui --profile official-party-ui`.
API tests cover deep-copy isolation, exact GUID strings, schema/version errors,
stale/session/phase changes, heartbeat suppression and handler lifecycle/errors.

In v1.1, Sayge `allInside` reflects host-authoritative collective presence; the
AZSX note explains waiting while another participant is outside. Older private
five-field ritual packets leave this property absent (unknown), rather than
pretending local presence proves collective readiness. Existing v1 consumers
remain compatible and must ignore the added field.

## Crystal Resonance (API minor 3)

From minor 10, `state.data.crystals` is present from run initialization, including before the first pickup. It contains integer
`red`, `blue`, `green`, `yellow`, `total`, and the native presentation `index`
(0–31; 32 means no free native slot). `bonuses` contains accumulated percentage
values named `damage`, `cooldownRecovery`, `maximumHealth`, `criticalStrikeChance`. The legacy `areaOfEffect` field remains zero for compatibility; yellow crystals now grant critical strike chance.
`state.texts.crystals` contains the AZSX-authored title, icon, collection note, per-color tooltip `details` and `footer`. Zero counts carry discovery guidance.
Every getter/handler snapshot remains detached. No crystal position is exposed
to the minimap bridge. Aura indices are presentation metadata, not identities.

## Crystal pickup presentation (API minor 4)

`texts.crystals.pickups[red|blue|green|yellow]` supplies AZSX-authored `title`,
`reward`, `note`, `icon`, RGB `color` (effect) and `titleColor` (crystal color).
These are per-crystal values; structured accumulated totals remain in `data`.
Consumers can compare consecutive personal counts to announce new pickups. This
is a snapshot interface, not receipt history: seed a baseline on first available
state, recipient/session change or reconnect, and do not replay restored totals.
Counts decreasing or native run time resetting retire the old baseline/queue.
AZSXUI queues each observed increment for three seconds, capped at 16 notices,
and defers display through choices/pauses. Coalesced different colors use a stable
color order; no exact cross-color pickup ordering is promised by snapshots.

## Local equipment (optional, v1.5)

`data.equipment` is an optional 1-based array of at most five records, each with
`slot` (presentation order, 1..5: amulet, rings, trinkets) and `rank` (1..255). The matching
`texts.equipment[slot]` contains the native `name`, texture `icon` and formatted
tooltip `text`. These are already captured owner strings, not formulas for an
addon to recompute. An optional `category` contains native `amulet`, `ring` or `trinket` metadata.
AZSXUI uses it to reserve all five positions even when a slot is empty. Older
providers without categories retain their sequential display. No persistent
inventory ID is promised.

An empty array explicitly means a captured empty local build. An absent array
means unavailable; do not replace it with fake empty slots or retain old items.
Copies, session ownership, expiration and revision rules match the rest of v1.
Malformed, duplicate, foreign or incomplete equipment sequences invalidate the
private snapshot. Native remote build rendering is separate. Local spell rows
are still omitted, and this extension adds no native hook or memory reader.

## API 1.6: local native loot receipts

`AZSX_GetAPIVersion()` returns `1, 6`. `state.data.loot` is an array of up to
16 recent local native drop receipts, each `{event, rarity, isNew}`.
`state.texts.loot[event]` contains `{name, icon}`; `icon` is the native
`INV_...` basename, resolved under `Interface/Icons/`. Rarity is the original
native 0..5 quality. Treat `event` as an opaque positive receipt sequence scoped
by `state.session.epoch` and `recipientGUID`; deduplicate it across heartbeats.
The journal is bounded, not a durable history or gameplay reward endpoint.
It is cleared at run/recipient identity boundaries. Values come from the
already-local native `drop` UI event; no new native hooks, grants, collection
reads, name lookup or extra network messages are involved. Only the separately
requested damage-size adjustment adds a new guarded native contact.

AZSXUI shows item toasts only for `isNew=true`. Already-collected receipts remain
in this API for other consumers; AZSXUI consumes their sequence without a toast.
Mark notifications are independent. AZSXUI owns native loot-canvas visibility
while this current structured provider is available and restores it otherwise.

### HUD receipt presentation (v1.7)

AZSXUI consumes one Fortune receipt once. A pending award waits for a readable
native phase; pausing a started toast does not restart its six-second wall-time
lifetime. Missing metadata never grants or re-announces a reward.
Local and allied item text retains native owner formatting, with an appended
AZSX enchantment name and current effect only when the native ring icon uniquely
matches a reviewed definition and its current owner GUID/rank is verified.
Uncertain matches omit the annotation. No previews, viewer stats or guessed
localized names enter these tooltips.
Marks still use actual native English receipts. Contiguous receipts within a
three-second quiet window, renewed on each receipt, add their gains into one popup and retain the latest native
collection total. Repeated/stale totals are ignored within that burst. Item
ordering and gaps in observed totals prevent merging; gameplay grants are untouched.

## Cooperative information aura (v1.9)

`data.cooperativeRules` is present only with a fresh cooperative payload in an
available run phase, including dead, paused and choosing. It is independent of
AZSXUI, party size changes, gem ownership and Fortune ownership. An unavailable
snapshot or a solo payload omits it. The cooperative economy envelope identifies
the native coop projection even when its numeric fields are unavailable.

| Field | Meaning |
| --- | --- |
| `id` | Stable semantic identifier `cooperative_rules` |
| `permanent` | `true`: no timed expiration during this cooperative run; not persistence across runs |
| `priority` | `1`: display before other helpful auras |
| `areaBonusShare` | `0.5`: fraction of positive global area/range bonuses retained; informational only |

`texts.cooperativeRules` contains `title`, native `icon` texture path, `rule`,
and a `notes` array. These are detached copies, like the rest of the API. Render
the rule in normal text, followed by a blank line and gray notes:

```text
Cooperative Rules
Area and range bonuses are 50% effective.

The stats panel shows effective values.
Item and reward descriptions may show base bonuses.
```

The retained AZSX presentation bridge creates `AZSXCooperativeRulesBuff`, a 32-pixel
informational button using the native Prayer of Fortitude icon. It consumes no
native aura slot, has no timer or cancellation action, and never modifies stats.
It is deliberately absent from native `UnitBuff` / `GetPlayerBuff` enumeration;
other addons consume the public record instead. The bridge supplies a leading
native-buff-row fallback without AZSXUI. A stale payload hides the button and
releases any hovered tooltip; heartbeat refreshes do not create duplicate icons.

A renderer may optionally reuse that button via
`AZSX_SetCooperativeRulesDocked(owner, true)`, where `owner` is its nonempty addon
name. This returns the button when currently available and reserves its placement
for that single owner (even if temporarily unavailable). A competing owner gets
`nil`. Do not change its scripts or native aura APIs. Restore any borrowed anchors
and call `AZSX_SetCooperativeRulesDocked(owner, false)` when disabling the layout;
this releases the reservation and restores the bridge's native-row fallback.
Only the owning name can release it. The fallback remains visible while AZSXUI's
HUD is off. Addons that render a separate icon should coordinate ownership to
avoid duplicate informational displays. Availability checks remain mandatory.

## API 1.11: Mechanical Chicken escort and engineering

`data.soloEvent` appears for the local recipient during a solo or cooperative Mechanical Chicken
escort. The record name is retained for API compatibility. It has `event` (positive run-local identity), `kind` (`chicken_escort`),
`stage` (0 waiting, 1 preparing, 2 engaged, 3 succeeded, 4 ended), `held` and
`maximum` (centiseconds), `inside` (boolean), and `progress` / `total` (route
segments, 8–120 according to native map radius). Optional finite `x` / `y` world coordinates
track the live chicken while its native scene object exists; they are absent
after cleanup or on older 1.11 payloads. `texts.soloEvent` supplies a title,
note, native icon, reward and duration. This is read-only presentation data;
the native scene remains authoritative.

`data.engineering` appears after extraction. Its `event` is a recipient-scoped
reward revision, `kind` is one of `net`, `rocket_boots`, `grenade`,
`shrink_ray` (former damaging/shield kinds are retired), and `nextTrigger` / `remaining` are bounded
centiseconds. `texts.engineering` supplies the native icon, title, description,
reward and duration. The gadget lasts only for this run and has no rank.

These records are local, detached copies, omitted in unavailable phases,
and do not require AZSXUI. Combine an event/revision with the session identity
when deduplicating notifications. API minor 12 adds optional escort coordinates only; existing major-1 consumers
may ignore them. A consumer must hide its marker when either coordinate is absent,
invalid, or the escort stage is terminal.


## API 1.12: Banished Demon

`data.soloEvent` also carries the current recipient's Banished Demon
encounter in solo or coop. `kind` is `banished_demon`; `event` is the positive
run-local identity; `stage` uses the same 0–4 lifecycle as the escort.
`held` and `maximum` are centiseconds of cumulative group preparation
(maximum 200), `inside` reports whether all active participants are nearby,
and optional finite `x`/`y` locate the binding while it exists.
`banishes` is the shared native Banish count on the terminal receipt;
`progress` and `total` are zero for this kind. `texts.soloEvent` supplies
the title, current objective, native icon, reward and duration. The record is
a detached read-only copy; consuming Banish remains native. A consumer should
deduplicate a reward by session identity and event, and hide its marker at a
terminal stage.

## API 1.13: utility gadgets and yellow critical chance

Yellow crystals contribute `criticalStrikeChance` in percentage points; the
legacy `areaOfEffect` crystal total remains zero. The current engineering kinds
are `net`, `rocket_boots`, `grenade`, `shrink_ray`. Retired Sapper/Belt records
are rejected; the native projectile's former Mortar wire slot now identifies
the zero-damage Stun Grenade. The major-version getters and detached-copy
contract remain unchanged.
