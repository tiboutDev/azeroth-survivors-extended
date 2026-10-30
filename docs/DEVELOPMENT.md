# ASX development

ASX extends the official native 0.130.1 engine. Player documentation starts at the
[project home](../README.md). For engineering work, read the [testing policy](TESTING.md),
[feature inventory](ASX-FEATURES.md) and [adapter boundary](CLIENT-ADAPTER.md).

## Source organization

| Area | Location | Responsibility |
| --- | --- | --- |
| Rules | `crates/asx-gameplay/src/` | Native-independent decisions and state transitions |
| Official orchestration | `src/runtime.rs`, `src/runtime/` | Activation, lifecycle and feature coordination |
| Selected native access | `crates/asx-game/src/official.rs`, `official_native/`, version descriptors | Exact-build selection, bounded memory IO and native call contracts |
| Presentation | `native-ui/`, official party/menu/reward modules | Retained native children, owner text and reward presentation |
| Distribution | `asx/`, `tools/build/asx_*.py` | Native-folder launcher, packaging and transactional updates |
| Windows resources | `resources/windows/`, `build.rs` | DLL version resource and the system avrt export-forwarding list |
| Build references | `validation/` | Controlled artifact identities and reproducibility inputs |

Trace dependencies before moving or removing code. Directory age and versioned
filenames do not prove unused code. Preserve dirty/untracked bytes before
retirement, update consumers and verify the remaining graph. Retire an exclusive
test with its obsolete implementation; retain useful format and boundary tests.
For platform-independent build instructions, see [building](BUILDING.md); local
workspace notes and generated caches are covered by [local storage](LOCAL-STORAGE.md).
The [Rust architecture decisions](RUST-ARCHITECTURE.md) describe workspace
conventions and current-build data.

The internal crates use the project's `asx-` prefix. `asx-gameplay`
contains portable rules; `asx-game` contains selected native integration. Those
active packages are not the retired custom Coop engine. The separate `avrt-probe` checks Windows DLL exports. Cargo package
names use hyphens and Rust imports use underscores.

`native-ui/official-party.lua` is the retained in-game Lua presentation shipped
by the ASX packager. `asx/` contains the Python launcher, removal/update handling
and distribution metadata, not a second Rust engine. The old `launcher/` and
unused port-review profiles are retired. `resources/windows/version.rc` is the readable Windows
resource source; `version.res` is its committed compiled form consumed by
`build.rs` so builds do not depend on a local resource compiler. The export list
is also an intentional build input, not a generated cache to ignore.

Gem and enchantment production coordinators consume the bounded operations in
`src/runtime/reward_services.rs`, rather than the shared Runtime or native
memory accessors. The production binding retains selected-adapter translation;
diagnostic fixtures remain separate native integration clients. This internal
boundary is not a public plugin API or a complete progression-domain migration.

Official orchestration has these internal owners:

| Module | Ownership |
| --- | --- |
| `bootstrap.rs` | Exact-build verification and ordered installation; XP, UI and session entries remain in their original single batch |
| `xp_hooks.rs` | Shared-XP wrapper, thread-local input and award-entry stub; bounded services bind multiplier/native translation |
| `experience.rs` | Deferred credits, identity retirement, take-before-replay and safe rejection; bounded services bind native observations/replay |
| `pause.rs` | Pause authority flags, native UI notices and event/HUD callbacks; bounded services bind native reads and lifecycle/diagnostic dispatch |
| `session.rs` | Commands, world/HUD passes and server updates through callback-specific ports; native ordering and observation-only behavior remain intact |
| `menu_pause.rs` | Participant menu ownership and epoch checks through bounded native/cache services; controlled fixtures retain separate integration access |
| `lifecycle.rs` | Ordered feature teardown and solo/Coop transitions through owned snapshot/cache operations |

Pause flags and the native event trampoline are private to their owner. Other
coordinators query or change authority through explicit operations; diagnostic
event replay uses a scoped helper. Session, menu and lifecycle production
coordinators no longer access Runtime, adapter fields or memory directly.
Bindings retain their exact selected operations and short cache mutation scopes;
they do not return runtime/container references or accept arbitrary callbacks.
Session diagnostics preserve the sampling guard across non-reentrant memory
observations, and role reads remain separate before/after the native update.

This completes the orchestration split. Other feature implementations still
depend on Runtime; these local services are not a global native-free contract or
a complete domain migration.

## Active build boundary

The root DLL activates only for `AZCOOP_ENGINE=official` and the exact reviewed
0.130.1 executable. Retired measurement/custom-engine launcher modes remain inert.
The workspace contains the root DLL, portable game/gameplay crates and the
independent Windows `avrt-probe`. Historical bridge, simulation, transport and
patch-generator crates are removed.

Retained pause presentation lives in `src/presentation/pause.rs`; its immutable
native contacts come from the selected adapter. Crash-record decoding retains a
small historical name catalog without linking the old engine. Root tests use a
shared test lock independently of the retired breakpoint implementation.

Shared types are re-exported directly from `official_native`. The exact current
descriptor is `crates/asx-game/src/bindings/official1301.rs`; inherited constants
and translators are resolved in its generated manifest. Historical descriptor
modules remain test-only references; production does not chain their translations.
Historical readers and their exclusive tools/tests are retired. The current
memory interface and enemy parser remain active and have current-client tests.

## Development practices

Keep rules independent of native layouts and Windows APIs. Route native reads,
writes and calls through the exact-build adapter. Preserve callback ordering and
avoid holding mutable state guards across native callbacks.

Use the smallest relevant checks described in the testing policy. Documentation
changes do not require DLL builds or game launches. A build, a controlled live
trial and a distributed release provide different validation scopes.

## Local configuration

Machine-specific paths, cache locations, credentials, deployment targets and
working-session notes belong in ignored local configuration or external notes.
Do not include their values in project documentation, examples or archives.
Document configuration names and portable setup steps instead.

## Current development helpers

`native_profile_codec.py` and `flight_recorder.py` retain the original standalone
profile codec and crash decoder without depending on the retired product. The
0.130 importer uses `native_runtime_import.py` directly. Build entry points compile
only; deployment is a separate explicitly selected action.

The isolated trial dispatcher supports only current 0.130 extension/package modes
and at most two clients. It rejects retired/no-mode inputs before Windows setup.
Tests of parser/dispatch logic do not prove gameplay fixtures succeeded live.

Run publication hygiene, portable/native checks and reproducibility checks
manually as appropriate. Reproducibility requires its own reviewed ASX artifact
reference. No checks run automatically on push, and source cleanup does not
establish publication readiness.

Do not keep session status, cleanup counts or generated run reports as current
project instructions. Store them in ignored local evidence; keep lasting build
references under [validation](../validation/README.md). Source, installed normal
binaries and published packages have separate identities and evidence scopes.

## Offline client comparison

`tools/dev/compare_official.py` produces unapproved address/ABI candidates, never
an activation descriptor. Its CLI requires a host with `resource.RLIMIT_AS` and
limits each analysis process to 2048 MiB of address space and four unprocessed
function owners by default. Instruction caches are bounded. Run one comparison
process at a time; a larger WSL allocation does not justify unbounded analysis.

Use explicit `--old`, `--new`, `--descriptor` and `--out` paths, with evidence
under ignored local storage. Continue with `--resume <checkpoint>`; executable
hashes and the complete descriptor inventory must match. `--retry-failures`
revisits ambiguous owners within the same batch limit; repeat `--owner <RVA>`
to restrict a batch to specific owners. Existing evidence is not
overwritten unless it is explicitly resumed in place. Checkpoints replace their
previous version only after serialization succeeds. A memory-ceiling exit or
external timeout leaves the last completed checkpoint available for review.
Unresolved contacts and heuristic matches require native review even after all
function owners have been visited.

## Preparing a public source import

Preserve the existing public repository history. Prepare a reviewed snapshot of
current source files and apply it in a separate checkout based on the public
branch. Do not merge private development ancestry, push all local branches/tags,
or rewrite existing public history as an implicit source import. Inspect both
the candidate file inventory and its intended base commit before committing.

Exclude ignored workspace state, build outputs, private evidence and Git
metadata from the snapshot. Reject symbolic links and verify copied file hashes.
Compare existing public player documentation before applying updates. Run the
source publication checks from [testing](TESTING.md#publication-checks); a clean
current snapshot does not sanitize unrelated private history or satisfy a missing
artifact reference. Commit and push require explicit authorization.

### Client registry consumers

`asx/supported-games.json` is the executable identity and native archive inventory
source for private imports. `tools/dev/client_registry.py` validates that inventory
and requires an unambiguous selected build. Developer tools must consume these
values rather than copy version strings, hashes or archive lists. This registry
does not authorize a new Rust adapter or establish save-fixture/ABI compatibility;
those remain separately reviewed gates.

See the [compatibility workflow](COMPATIBILITY-WORKFLOW.md) for resolved manifests,
artifact receipts, declarative suites and bounded Lua evidence.

### Event service boundary

`runtime/event_services.rs` owns native binding for supply, ritual and goblin
lifecycle operations. Solo/cooperative supplies, Sayge coordination and shared
placement consume the object-safe service interface, not Runtime or memory
accessors. Native gold invocation is selected-adapter-owned; the service preserves
identity checks and observes the actual post-callback receipt. Goblin machine
context hooks, appearance, movement interception and reward-entry hooks remain
explicit native integration modules. This extraction does not claim that every
ASX coordinator has migrated away from Runtime.

State reservations, per-recipient acknowledgements and stat-input restoration
keep their original order. Services do not expose backing containers or arbitrary
callbacks, and callers must release gameplay-state guards before native calls.
