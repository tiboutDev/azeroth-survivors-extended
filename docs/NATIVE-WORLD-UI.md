# Native world and reward presentation

ASX enriches the reviewed 0.125 client without replacing world rendering,
combat logs, nameplate health updates or the native marker transport.

## Enemy nameplates

The adapter transforms native `vplates` inputs before construction, rather than
restyling discovered Lua frames later. Enemy bars use 70% of the freshly supplied
native dimensions and retain the native border and health updates. Their narrower
root retains the original native top-midpoint anchor and projected coordinates. Friendly blue/green rows
remain unchanged. Native cached settings retain their original comparison baseline
to avoid repeated resizing of unchanged frames.

Normal enemies lose their name; native elite, rare, rare-elite and world-boss ranks
retain it at the original font size. Classification uses the native GUID-keyed
UnitInfo table, as native `UnitClassification` does, rather than display names.
Level and skull labels are suppressed before the first native draw. The original
border texture still includes a level ornament; removing that ornament requires
separate visual validation. No Lua pool discovery, periodic restyling or replacement
health bars are used. Native visibility preferences are unchanged.

## World combat numbers

The selected adapter hooks the native damage-size calculation after its
interpolation and clamp, before projection. Native outgoing damage categories
0 and 2 use one third of the freshly calculated native size. Stored sizes,
values, native critical animation, XP/power notifications and incoming combat
text are unchanged. Green healing messages keep their own native size.

Native direct and periodic healing logs do not produce world numbers in this
client. The adapter connects their actual positive packet amounts to the same
native world-text message writer used by damage. Native periodic variant 1 is
healing; variant 0 already emits damage and must not be duplicated. The direct
critical flag selects the native critical animation. Messages are green and
prefixed with `+`; native entity lookup supplies attachment and position.

Both caster and recipient must match current native player/party aliases.
Periodic packets are observed before the native recipient-only log filter.
Missing or ambiguous identities drop only the cosmetic message. The writer is
borrowed solely inside the existing native log callback; owned native strings
transfer once to its native message queue. No healing, talents, meters, stat
reconstruction or extra network messages are triggered by presentation.

Exact callback/register/layout contacts belong to `official125.rs` and
`official_native/combat_text.rs`, not the portable rules crate. Guards cover
all new callback/helper entry bytes. Windows execution checks the size stub's
category exclusions, fresh-baseline behavior and flag preservation.

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
