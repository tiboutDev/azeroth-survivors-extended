# Working in AZSX

Work from the authoritative WSL checkout. Resolve its branch/worktrees with Git;
Windows receivers are execution copies. Keep code, UI and documentation in English.
Preserve existing edits. Use **tiboutDev** for Git identity; commit, push, merge and
publication require the user's request. Keep installed player games, saves and
official assets unchanged. Never send messages or control the interactive desktop
without authorization. Fast mode requires an explicit request.

## Read the relevant reference

Start with [development](docs/development/DEVELOPMENT.md) and
[building](docs/development/BUILDING.md). Then read only what the task needs:

| Task | Reference |
| --- | --- |
| Native integration | [Adapter contract](docs/compatibility/CLIENT-ADAPTER.md), [current client](docs/compatibility/CLIENT-COMPATIBILITY.md) |
| Feature change or client port | [Feature index](docs/compatibility/AZSX-FEATURES.md), affected criteria in [expected behavior](docs/compatibility/AZSX-BEHAVIOR.md) |
| Tests or live trials | [Testing](docs/development/TESTING.md) |
| Client port | [Compatibility checklist](docs/compatibility/COMPATIBILITY-CHECKLIST.md) |
| Party UI | [Native party UI](docs/features/OFFICIAL-PARTY-UI.md) |
| Packaging or release | [Release checklist](docs/release/RELEASE-CHECKLIST.md), [installation contract](docs/release/AZSX-INSTALLATION.md) |

Update the authoritative contract alongside a semantic change. Keep stable
acceptance IDs and disabled/upstream dispositions. Generate the feature index
from `compat/features.json`; do not edit its generated table directly.

## Source boundaries

- `crates/azsx-gameplay/`: portable rules and copied semantic values.
- `crates/azsx-game/`: selected native schemas, addresses, fields, ABIs and hooks.
- `src/runtime/`: feature coordination and lifecycle.
- `src/presentation/`, `native-ui/`, `addons/AZSXUI/`: presentation.
- `azsx/`, `tools/build/`: launcher and native-folder distribution.

Only the exact client selected in `azsx/supported-games.json` is supported.
Generated descriptors and `azsx_game::current` own its native data. Historical
modules are test references. The retired custom simulation, transport and lobby
must not return as a fallback. Activation remains explicit through the AZSX launcher.

## Native invariants

Native callbacks are synchronously reentrant. Release state locks, mutable
borrows and container references before native calls. Copy or reserve work first;
preserve order, partial commits and cleanup. Never retry an uncertain grant.
Do not take the entire runtime to work around borrowing.

Keep actor slot, GUID and run/session identity together. Rebind after callbacks,
restore borrowed actor contexts and retire departed identities. Native row
layouts stay in the adapter; services return bounded copied/owned values.
Foreign effects keep their producer, recipient, native coefficients and meters.

Native AI, ordinary NPC ownership, scaling, schedules, allocation and networking
remain native. Restore temporary pursuit poses before subsequent native logic.
Use selected `NATIVE_PLAYERS` bounds; native capacity is five. Preserve the private
support-route tag layout and reject invalid recipients.

Dead participants receive native shared XP and whole-level receipts immediately;
choices wait for revival. Do not restore an XP bank, duplicate native rewards or
reapply cooperative multipliers. Native marks replace duplicate-loot compensation.

Enrich retained native UI children and preserve owner-formatted tooltip text.
Publish the addon API through the selected virtual script reader with native
string ownership. No MPQ scanning, installed internal addon or guessed owner data.
Native Lua `string.byte` accepts one index; keep stubs consistent.

## Proportionate validation

Use [TESTING.md](docs/development/TESTING.md) to choose checks once for the change.
Run focused checks while fixing a block, then `full` once at a compatibility or
release milestone. Reuse valid results when their inputs are unchanged. Do not
repeat passing checks or broaden scope without a failure or an uncovered risk.
Documentation-only edits need no test campaign, DLL build or game launch.

For a client port, complete all code wiring and expected-output checks before a
candidate DLL or live trial: direct/indirect field calls, closures, arrays,
host/private namespaces, raw buffers and native ABI/ownership. The AST field audit
checks coverage; native execution and transaction tests check semantics. Group
fixes instead of discovering missing translations through repeated game launches.

Use bounded live trials only for observations code cannot establish. At most
**two owned clients**, normal priority and all CPUs, in an isolated receiver.
Stop them on severe slowdown. Screenshots/interactive desktop use require their
own scope. A compiler, creation log or shown flag is not a visual review.

Build only when compilation inputs change or a new artifact is requested.
Diagnostic fixtures stay behind `test-hooks`; retain/restore a normal artifact
for normal play. Record hashes, actual results and omissions; keep failed attempts.
A passed retry does not explain an intermittent failure.

## Local state and release scope

Keep machine paths, handoffs, disassembly, test outputs and work backups in ignored
local storage; see [LOCAL-STORAGE.md](docs/development/LOCAL-STORAGE.md). Preserve
unique work before removing obsolete files. Old names needed for migration or
recorded native-data provenance are not automatically obsolete.

Release signing, ownership, reproducibility and archive validation belong to the
[release checklist](docs/release/RELEASE-CHECKLIST.md), not routine development.
No implicit credential discovery, approved-reference replacement or publication.
Final preparation uses `azsx_release_layout.py`: upload assets and three compact
guides stay in the preparation folder; technical evidence stays in ignored local
release storage. Native saves/settings and existing ownership/migration behavior
remain intact.
