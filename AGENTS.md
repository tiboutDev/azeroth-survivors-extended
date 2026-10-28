# Working scope

Read [building](docs/BUILDING.md) and [development](docs/DEVELOPMENT.md).
Resolve the active checkout and worktrees from Git. Machine paths, pairing,
editor handoffs and session notes belong in ignored local configuration, not
project documentation. Never implement source changes in a generated receiver.

Keep UI and current documentation in English. Preserve unrelated changes,
history, reports, results and backups. Do not commit, push, merge, publish,
modify installed player games/shared kits/official assets, broaden permissions
or enable Fast without an explicit request. Never send messages to other people
or control the interactive desktop without current authorization.

## Current architecture

The only supported source target is the reviewed official **0.125.0** engine.
Exact executable hashes and native entry bytes gate activation; version labels
and approximate signatures do not enable an unknown client. The DLL activates
only through the explicit official ASX launcher path. Direct native launch stays
inert. The custom simulation, transport, lobby and recovery engine is retired;
never restore it as a fallback or run it beside official simulation.

- `crates/asx-gameplay/` owns portable rules and contracts, with no native
  layouts, registers, Windows APIs or transport dependencies.
- `src/runtime/` coordinates features and lifecycle. Keep imports explicit
  and access bounded rather than exposing the whole runtime.
- `crates/asx-game/src/official.rs` forwards selected native operations;
  `official_native/` and the exact descriptor own schemas, offsets, native calls,
  hook bytes and field/register translations. Shared inherited tables are not
  permission to select retired builds or a claim of completed domain migration.
- `src/presentation/` and `native-ui/` enrich retained native presentation.
- `asx/` and `tools/build/asx_*.py` own the current native-folder distribution.
  No legacy launcher, helper executable, browser UI or isolated-AppData product
  is an ASX runtime dependency. `avrt-probe` remains an independent export probe.

Read [CLIENT-ADAPTER.md](docs/CLIENT-ADAPTER.md) before native-integration changes
and [ASX-FEATURES.md](docs/ASX-FEATURES.md) before feature or compatibility changes.
Update the stable feature inventory and affected references for semantic changes.
Do not infer current source, normal-runtime or published-kit parity from each other.

## Native correctness

Native callbacks are synchronously reentrant. Never hold a state mutex, RefCell
borrow, map-entry reference or whole-runtime guard across a native call. Copy or
remove pending work first; allow nested callbacks and preserve FIFO/key order,
partial commits, mutation-before-call ordering and cleanup after errors. Do not
use `mem::take` of an entire runtime to avoid proper borrowing boundaries.

Keep identities scoped to slot plus GUID, session/generation and run lifetime.
Reject invalid/inactive source contexts before native calls, rebind after
callbacks, restore host/actor context and retire departed identities. Reset
through defined lifecycle boundaries, not by guessing from reused addresses.
Keep native row/layout choices inside the selected adapter; policy consumes
bounded copied/owned values and domain operations, not native backing containers.

Preserve successful per-actor acknowledgements before attempting the next actor.
Do not retry uncertain native commits, double-grant shared/personal rewards,
reapply group multipliers or feed replayed credits into new shared receipts.
Foreign periodic heals/shields/retaliation retain caster/recipient attribution,
native talent/proc/meter ownership and native budget/cooldown semantics.
Never process a foreign HoT through the recipient's outgoing talents or meter.

Marks and taunts apply their agreed policies without changing native NPC ownership
or special-cast targeting. Restore temporary actor poses before subsequent native
logic. Enemy/boss HP, damage, density, spawn placement/selection/allocation,
schedules and host options remain native. Do not restore retired difficulty
interceptions or custom DPS ranking, resurrection UI or replacement party roots.

Native capacity remains five players. Use adapter-owned `NATIVE_PLAYERS` for
bounds; do not alter lobby admission or native vector capacities. Private support
route tags reserve the low three bits for target slots and bit3 for hybrid group
delivery; validate producers/consumers and reject invalid sixth-slot targets.

Current glyph gameplay is disabled and native marks replace duplicate-loot gold
compensation. Retained shared bonus representations/helpers are not proof of
installed glyph gameplay. Preserve ring/gem identity, reward-card ordering and
native stat reconstruction. The known post-death XP desynchronization remains
open; this cleanup does not authorize resuming its paused investigation.

## Native presentation and activation

Read [OFFICIAL-PARTY-UI.md](docs/OFFICIAL-PARTY-UI.md) before UI changes. Enrich
native `PlayerFrame`/`PartyMemberFrame1..4` children only. Native health, death,
resurrection, lobby, pause and replay/menu authority stay native.

Owner tooltips must use selected native owner formatting/origins, never viewer
`SetAction`, guessed names or static formulas. Hide ambiguous/stale/missing owner
data. Preserve unchanged cosmetic identities through wall-clock heartbeats and
long common pauses. The native Lua `string.byte` supports one index only; keep
that restriction in portable stubs. Inspect actual rendered frames for visual
claims; creation/send logs alone do not prove visibility or tooltip parity.

Use the owned addon read gate: exact file identity and fixed prefix, one changed
byte in the approved read buffer, on-disk addon inactive. Keep NtReadFile import
hooking scoped to the game image; preserve all arguments/status/last error.
Do not scan MPQs, hook OS code globally, alter arbitrary addon reads or restore
custom-CVar activation. Diagnostic probes remain opt-in and `test-hooks` only.

## Validation and artifact policy

Read [docs/TESTING.md](docs/TESTING.md). Use the smallest current relevant profiles
through `sh tools/dev/check.sh --profile <name>`; repeat profiles to combine.
Inspect `--dry-run` when needed. Unknown executable changes select the full
portable gate; documentation-only changes launch no tests. Retired custom-engine
profile names and historical recipes are not current instructions.

State the risk, expected observable result and stop condition before live work.
Start with portable checks. Check platform-specific source deliberately; execute
Windows ABI probes when wrappers, registers, stack contracts or generated code
change. Preserve production forwarding/identity coverage for any retained wrapper.
Do not write implementation-mirroring tests for reversible low-impact edits.

Use at most **two simultaneous owned game clients**, normal priority and all
available CPUs. Never mask slowdown with game-client affinity, quotas, memory
limits or reduced priority. Stop owned clients immediately if severe slowdown is
reported. Private jobs may retain kill-on-close cleanup; do not change global or
player settings. Portable build/check budgets are separate.

Use at most one relevant bounded live scenario per coherent change unless a
failure or uncovered risk justifies another. Combat and support are the defaults;
add recovery only for shared-state/transport/lifecycle risks. No routine unrelated
installer, boss, UI or repeated normal/diagnostic campaigns. Ordinary observations
do not enable screenshots; explicit visual trials use sparse captures/probes.

Documentation/Python changes require no DLL build. Reuse an artifact only when it
contains the relevant source. Diagnostic builds serve controlled fixtures; retain
or restore a normal artifact after them and state any source it lacks. Build a
new normal DLL for requested normal play, sharing or an explicit stabilization/
release milestone, never as an integration-closing ritual. A separate normal live
trial is required for releases, compatibility updates or materially different
production conditionals. No test or build runner deploys by default.

Record profiles, reasons, durations, actual binary hashes, evidence and limits.
Existing logs validate only their exact artifact/scenario. Preserve failed
attempts and distinguish assessor tests, compilation and live evidence. Skipped
checks are not passes. Required Lua 5.1 behavior runs through uv/lupa explicitly.
Use the full portable gate plus relevant checks at release/compatibility milestones;
full portable checks do not implicitly perform Windows, game or package trials.

## Distribution and publication

Read [ASX-INSTALLATION.md](docs/ASX-INSTALLATION.md). Current packaging overlays
`start-asx.cmd`, `README-ASX.txt` and owned `ASX/` files on a supported native
folder. Use native saves/settings; never redirect AppData or restore legacy
Coop/UserData installation logic. Never ship native EXEs/MPQs or touch official
assets. Detect loader collisions before deployment.

Updates, rollback and uninstall use bounded inventories and ownership hashes,
preserve unknown/modified files and verify signatures before applying content.
Recovery that replaces imported client code restarts the launcher before using
restored files. Keep anti-loop sequence behavior and native profile backups.
No automatic upload or implicit access to real signing credentials. Runtime
replacement remains an explicit complete-archive operation with clients closed.

Validate the exact signed archive at a release milestone. Source-publication
hygiene and reproducibility remain required safeguards. Reproducibility requires
an approved current ASX artifact reference, not old Coop hashes or a fabricated
baseline. A passing Markdown audit does not sanitize source, reports or Git
history. No source push/publication follows implicitly from cleanup or preparation.

Project documentation describes current source and supported workflows.
Historical checkpoints remain in Git history and private evidence backups;
they do not supersede these instructions or authorize retired workflows.
