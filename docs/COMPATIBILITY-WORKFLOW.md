# Compatibility workflow

Keep the authoritative checkout on Linux. Generated Windows receivers are
execution targets, never alternate source checkouts. Planning and checks do not
build, copy files or launch clients.

## Review and plan

```sh
python3 tools/dev/asx_dev.py check
python3 tools/dev/asx_dev.py inventory > .local/contracts.json
python3 tools/dev/asx_dev.py impact --comparison .local/comparison.json > .local/impact.json
python3 tools/dev/asx_dev.py plan events
sh tools/dev/check.sh --profile full
sh tools/dev/check.sh --profile diagnostics
```

`asx/supported-games.json` owns executable identity and archive inventory.
`compat/clients/` owns reviewed native literals; the template contains stable
translation code. `compat/features.json` owns stable feature IDs, dispositions
and affected domains. Feature documentation is generated with
`feature_catalog.py --write-doc`; descriptors with `binding_registry.py --write`.
These commands do not approve unknown images. Save-fixture acceptance is still
an independent explicit gate.

The impact report connects named native methods to features. Unclassified
contracts conservatively affect every feature. Instruction matches are only
candidates: layouts, caller ABI, registers, shared entry ownership and native data
need review. Disabled/upstream features remain in the inventory.

## Build and explicitly stage

Use the appropriate build-only helper when a new DLL is necessary. Each helper
records the compiler command, conservative Rust/resource compilation inputs and resulting DLL
hash in an adjacent `.artifact.json`. Source changes during compilation refuse
the receipt. Normal and diagnostic artifacts are distinct.

```sh
sh tools/dev/build-chest-fixture.sh
python3 tools/dev/asx_dev.py stage --receiver "$RECEIVER" --dll target/test-hooks/x86_64-pc-windows-msvc/release/avrt.dll
```

Staging requires an existing reviewed private import and an owned unchanged DLL.
It verifies the artifact against current source before copying source/tools and
DLL. It does not import native assets, delete unknown files or launch a game.
Do not stage while clients are running. Staging is an explicit test-receiver
operation, never a deployment command for player installations.

## Bounded validation

In the staged receiver, with Windows Python:

```text
python -I -B tools/dev/asx_dev.py validate solo-events
python -I -B tools/dev/asx_dev.py validate coop-events
```

Suites in `compat/suites.json` declare player count, artifact kind, maximum time,
readiness and solo prerequisite. Readiness shortens execution; each existing
feature assessor still decides acceptance. A missing or failed observation
cannot pass because another event completed. Coop requires a passing solo receipt
for the exact DLL, compilation fingerprint, UI fingerprint, shared diagnostic tools and prerequisite-scenario fingerprint and native version. Normal proof cannot
substitute for diagnostic proof or vice versa. A retry creates new evidence and
preserves the failed run; use `retry shield_consumed` only for that focused scope.

No suite builds or deploys by default. Normal/diagnostic staging remains explicit;
plans describe ordering, not permission to run every available campaign.
At most two owned clients run, at normal priority with all CPUs. Screenshots
remain opt-in. Test result directories and immutable workflow receipts preserve
actual binary identity, verdicts and limitations.

## Lua diagnostics

`tools/dev/diagnostics/observer.lua` is a tools-only, opt-in observer injected into
owned diagnostic profiles. It is absent from packaged presentation. It samples
changed widget/buff/tooltip observations at most twice a second, for at most
300 seconds and 512 records. Native/Rust state and Lua model observations remain
separate from actual widget visibility, dimensions and text.

A guarded test-hooks-only native setter consumes a reserved diagnostic mailbox
before native configuration storage. Ordinary calls preserve arguments/returns.
The registered name is only a log transport; it does not activate ASX or persist
a user setting. Each record carries schema, session, slot, sequence and clock;
Rust adds copied actor identity when available. The collector rejects wrong
sessions, missing/repeated sequence numbers, nonfinite clocks and malformed data.
No network, input injection, tooltip activation or screenshot is used.

Event suites export `lua-observations-*.jsonl` and `lua-evidence-*.json`, checking
actual toast/progress/buff observations alongside independent gameplay assessors.
A model message alone does not prove rendering. Passive tooltip snapshots do not
prove unobserved hover content; pixel aesthetics still need a scoped visual check.

## Shared native hook ownership

`src/install.rs` enforces one owner per native target, including optional batches.
An optional installation failure rolls back its own batch; earlier batches remain
registered for complete teardown. Native-entry guard acceptance requires the
recorded original and installed five-byte prefixes plus an unchanged guard tail.
Unknown jumps are never accepted.

The shared item presenter retains native forwarding through the registered
entry. Goblin reward scopes wrap that route rather than installing a second hook;
ring/gem card presentation and kept-bonus tooltips retain distinct construction
and native formatting entries. Review shared entry composition when adding a
feature; do not install independent hooks on the same address or introduce an
unbounded per-frame event bus. Windows stand-in tests cover multi-batch install,
duplicate rejection and complete native restoration.

Readiness conditions only trigger an assessment. Early completion requires the same
acceptance function used for the final verdict to pass; pending observations keep
the trial running within its declared budget. Other scenarios do not invalidate a
solo proof when their metadata changes. Shared executable tooling still does.

Scenario proofs hash their own declaration and shared executable test tools.
The coop gate compares the required solo scenario's hash, not the coop scenario's
hash. Historical globally scoped proofs are not relabelled as new evidence.
Native-domain catalog metadata changes alone do not invalidate gameplay evidence;
DLL source identity and executable tooling remain independently checked.

## Native inventory coverage and precise links

The inventory parses Rust items with the development-only `native-inventory`
helper (syn, with its own locked dependency manifest). It visits free functions,
impl methods and inline modules throughout each selected native source file,
including production items after test modules. It excludes items whose cfg
requires `test`; unknown platform/feature conditions are conservatively retained.
Private helpers are included. Macro expansion and a complete Rust call graph are
not inferred; source inventory is not proof of ABI compatibility.

`compat/contract-links.json` contains reviewed operation-level feature links.
These take precedence over the conservative domain fallback. Missing explicit
contract IDs, invalid feature IDs and ambiguous contract IDs fail inventory.
Every contract records its feature attribution, link basis and relevant scenario
IDs from `compat/suites.json`. Only explicitly reviewed links narrow review;
unclassified contracts still affect every feature. A zero unclassified count
alone does not establish complete behavioral coverage.

These are read-only developer checks; no DLL compilation or live campaign is
needed for an inventory-only change. The parser tests run in the full/tooling
portable profiles. Helper build outputs are excluded from Git and receiver staging.

### Extended feature attribution

`contract-links.json` records operation-level owners together with the rationale,
source references and shared/diagnostic/retained context. Jewelry card hooks,
spell/support modifiers, native identity snapshots and reward presenters keep
multiple owners where appropriate. Disabled glyph and upstream-native mark or
guardian dispositions are retained; attribution does not enable their hooks.

Contracts not explicitly reviewed retain the union of their domain owners and
all explicitly reviewed owners elsewhere in the same source file. This prevents
an ambiguous byte reader or context helper from losing cross-feature consumers.
New contracts remain conservative; renamed/removed explicit contracts fail the
metadata check. Every explicit link requires valid feature IDs and existing
project-relative source references. Source references document review rationale,
not an automatically inferred call graph.

`feature-validation.json` declares candidate portable profiles and scope notes
for all feature IDs. Reports separate those profiles from scenarios that declare
the feature. Missing dedicated live suites remain visible as `live_suite_gap`;
shared scenarios are not proof that every linked contract executed. Installer
coverage remains separate from native selector/registry identity agreement.
No candidate profile, live-suite association or disposition is an execution
receipt or an ABI approval. Full compatibility milestone gates remain required.

### Separate fingerprint domains

Build receipt schema 2 excludes separately loaded `native-ui/` Lua from DLL
compilation identity. Rust sources, workspace manifests/lockfile, toolchain and
Windows resources remain conservatively included, including Rust test source
within those trees. This is not a claim of minimal compiler dependency discovery.
The `source_fingerprint` field remains a compatibility alias of
`compilation_fingerprint`; both must agree. Exact DLL hash and artifact kind
checks remain mandatory.

Retained native UI and the independent ASXUI addon are hashed independently at staging and validation. Validation identity
continues to cover scenario-specific executable tooling and fixture data. A UI
or fixture change requires the relevant new validation, but does not invalidate
a schema-2 DLL whose compilation inputs are unchanged. Rust-embedded content
remains a compilation dependency and must not be excluded on that assumption.
Legacy schema-1 receipts retain their original combined-input semantics; they
are never converted or re-labelled without an actual compiler invocation.
Old proofs without a UI identity cannot satisfy the new coop prerequisite.

### Execution-only Windows staging

`asx_dev.py stage --receiver <isolated-folder> --dll <built-dll>` verifies the
compiler receipt against the authoritative checkout before deployment. It copies
only the selected DLL, native UI, ASXUI, client/scenario registries and the union of
Python dependencies and fixtures required by the declared live scenarios.
Rust sources, Cargo inputs, portable tests and compiler helpers stay in WSL.
Unchanged execution files are compared by SHA-256 and are not copied again.
Unknown receiver files and prior results are preserved, including old source
copies; they are not consulted as proof of the current artifact.

The schema-2 local staging receipt binds the compiler receipt, DLL, native
executable identity and complete execution inventory. Run `asx_dev.py preflight`
with Windows Python in the receiver to check these without launching a game.
`validate` runs that same preflight before any scenario. Missing/changed tools,
UI, ownership, executable or receipt fail closed. Source changes still require
source-side artifact verification at staging; Windows does not pretend to
reverify source files it no longer needs. This is trusted local development
provenance, not a signed release manifest or protection from someone rewriting
both the tools and their receipts. Old schema-1 staging receipts require explicit
restaging; build receipts keep their existing schema-1/2 verification rules.

A staging failure never authorizes launching a partially updated receiver.
Staging is incremental rather than a transactional directory swap; a complete
preflight must pass before use. This workflow neither prepares nor modifies kits.

### Targeted coverage gaps

Run `python3 tools/dev/feature_coverage.py --run --output <private-report.json>`
to exercise the portable profiles for the six previously unassigned areas.
`compat/feature-validation.json` records each trigger, expected observation and
remaining live limitation. No game or archive is launched by this command.
Passing these checks is portable evidence, not a native visual or live pass.
Guardian reward counts, native immunity routing, resurrection aura presentation
and world presentation still need focused live evidence after relevant native
changes; reuse an existing relevant run when it actually observes the case.

### Combined cooperative coverage evidence

`coop-coverage` reuses the native support, two-revival/recovery and four-immunity
fixtures, then kills one existing guardian through native damage before the
immunity cases. The guardian step records native choice offers on both actors;
it does not infer a fixed spell grant from ordinary XP level-up offers.
`coverage_evidence.guardian` rejects absent/duplicate/asymmetric offers and
requires an actual three-option offer after the controlled kill.

`coop-world-ui` is a focused visual fallback if the first run cannot observe
nameplates. It enables enemy nameplates only in disposable profiles and samples
existing native plate/healing probes. Never treat a successful callback alone as
rendered evidence. The world assessor decodes original style bytes and checks the
70% transform while preserving native name font size. Direct and periodic
message-queue paths have separate checks; a missing periodic observation remains
unproven even if periodic gameplay healing succeeds. Preserve original failed
results and attach any later narrower conclusions separately.

Both scenarios are bounded and require artifact-scoped solo acceptance through
the workflow runner. An explicitly requested diagnostic fixture-only two-player
trial can be recorded separately; it must not fabricate a missing solo receipt
or claim the normal DLL was tested. Restore the retained normal artifact after
such work. Sparse visual review and remaining omissions belong in private evidence.

### Remaining periodic-text and special-name witnesses

`coop-ui-remaining` is a bounded two-player diagnostic scenario. It enables enemy
nameplates in disposable profiles, creates native NPC witnesses and matches
rendered rank/name rows by GUID on both views. Constructor success and the boss
HUD alone do not prove world-name visibility. NPC durability is raised only for
the disposable elite/guardian witnesses; no production definitions are changed.
The adapter fixture is available only through the propagated `test-hooks` feature.

Self-targeted Rejuvenation probes the native periodic path separately from ASX
foreign-HoT accounting. A refused cast, an ordinary direct-heal queue write or a
positive gameplay heal cannot satisfy periodic-queue acceptance. Preserve those
failures rather than weakening the checks. Native rank/name mapping and the
actual origin of the periodic presentation must be investigated when absent;
do not infer a compatibility offset solely from a missing witness.

`tests/test_remaining_ui_evidence.py` tests rejection of unrelated GUIDs, ordinary
ranks and direct-only healing. This parser coverage is separate from actual live
success. A fixture-only trial must retain its exact build and staging receipts,
then restore the previously owned normal artifact.

The remaining-UI scenario wounds each caster immediately before its native HoT,
uses Renew and Rejuvenation separately and advances only on an accepted cast.
Refused casts are retried at most four times without removing native eligibility
or cooldown rules. Name witnesses flow through the owned ASX log, not DLL stderr;
they retain the original native label on missing classification. Constructor
role and GUID must match the actual retained label on both clients.

Pure HoTs may also emit the native SPELL_HEAL message path. For that path,
acceptance binds a captured spell ID (Renew/Rejuvenation), caster, recipient and
positive amount to the corresponding successful native text-queue write within
50 ms. Instant heals, stale headers and mismatched IDs/amounts are rejected.
The callback's internal `periodic` boolean alone does not identify the originating
spell. Keep pixel review separate from either queue observation.

Fractional-HoT display has separate portable accumulation and native packet-transfer
checks. Native self-HoTs use an inlined HP/meter path, whereas replayed foreign
HoTs use the shared heal helper. Both paths must be observed without counting a
native integer packet twice. A successful test executable is not live proof of
the newly installed callbacks; retain that distinction when a bounded game trial
has already been consumed before the new integration is built.

Extension trials copy ASXUI into each disposable native addon profile. Its files
are part of the staging/UI fingerprint, so a solo prerequisite cannot validate
a different addon build. This is diagnostic staging, not ASX installer ownership.
