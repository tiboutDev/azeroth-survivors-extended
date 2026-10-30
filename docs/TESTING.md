# Targeted ASX validation

The supported source target is official 0.130.0. Review the
[feature inventory](ASX-FEATURES.md) and [adapter boundary](CLIENT-ADAPTER.md)
when changing native integration. Validation must cover the changed risk and
state its limits; test counts alone are not a completion criterion.

## Commands and scope

Checks are run manually. There are no GitHub Actions workflows in this
repository; pushing a branch does not launch tests, builds or reproducibility
checks. Select and run the relevant commands before relying on their results.

```sh
sh tools/dev/check.sh --profile official --profile tooling --dry-run
sh tools/dev/check.sh --profile official --reason "Selected native boundary changed"
sh tools/dev/check.sh --profile full
sh tools/dev/check.sh --list
```

With no explicit profile, changed files select checks conservatively. Unknown
executable changes select `full`; documentation-only changes select none.
Repeated profiles combine and deduplicate commands. The runner never builds a
DLL, deploys files, opens a game or controls the desktop. Reports and step durations
are written under ignored `test-results/targeted/` directories.

`full` supersedes other profiles in the same invocation. When required, run
`official-party-ui` separately so its explicit Lua 5.1 check is not replaced by
generic Python discovery, where that optional dependency can be skipped.

`full` tests the current 0.130 Rust graph, Python suite and Clippy. Retired
reader/tool implementations and their exclusive tests are not part of the gate.

| Profile | Current risk covered |
| --- | --- |
| official | Native 0.130 selection, normalized snapshots, menu ownership, native policy transactions, current source boundaries and evidence assessors |
| official-party-ui | Retained native children, owner origins, menu projection, world-number packet decoding, marker policy, shared card boundaries and required Lua 5.1 behavior through uv/lupa |
| asx-install | Owned native-folder installation, signatures, transactions, interrupted-client recovery, rollback and publication inventory verification |
| gems | Fixed gem identity, selected jewelry transactions and evidence rejection |
| enchantments | Ring identity/ranks, selected jewelry transactions and evidence rejection |
| glyphs | Retained rule catalogs and shared bonus representation; glyph gameplay remains disabled on 0.130 |
| fatigue | Resurrection penalties, recovery policy and selected native integration |
| rules | All independent gameplay policy and contract tests |
| combat | Marks, taunts and immunity policy plus selected native fixtures |
| support | Healing, delivery and periodic ownership policy plus selected native fixtures |
| periodic | Portable periodic-healing and support accounting rules |
| tooling | Check selection, current source graph, bounded trial dispatch, disposable profiles, import guards and build-without-deployment behavior |
| full | All portable Rust and discovered Python tests, plus Clippy for game/gameplay |

The retired custom-engine crates and their exclusive profiles are removed.
Historical recipes and test-plan assumptions do not authorize reactivating that
engine. Generic evidence parsers and their tests remain where useful for existing
reports, even when a fixture name refers to an older client.

## Choosing validation

Start with focused portable tests. Define the expected observable result and
stop condition before any live trial. Use `full` for releases, official-client
compatibility changes or broad executable changes without a mapped scope.
Complete both current relevant checks and the explicit full gate at those
milestones. The full portable gate does not include native Windows execution,
private EXE inspection, live gameplay, release hygiene or reproducibility.

Check Windows compilation deliberately when platform-specific source changes.
Execute ABI tests when wrappers, register/stack contracts or generated code
change. Existing shared tests cover argument rewrites, early return and
nonvolatile-register preservation without a game.

For native/shared behavior requiring a live observation, use one relevant bounded
scenario per coherent change: combat or support. Add recovery only for a lifecycle
or shared-state risk. Reuse one trial's logs for multiple assertions and stop when
the chosen scope passes. A failure or uncovered risk must justify another trial.

Use at most **two simultaneous owned game clients**, normal priority and all
available CPUs. Stop owned clients immediately if severe slowdown is reported.
Do not inject desktop input, change hardware settings or take control of an
interactive session without current authorization. Visual observations require
explicit scope; ordinary gameplay trials do not enable screenshots by default.

## Artifacts and evidence

Documentation and Python changes require no DLL compilation. Do not build normal
and diagnostic DLLs after every small change. Reuse an artifact only when it
contains the relevant source. Build normal artifacts for requested normal play,
sharing or an explicit stabilization/release milestone, not to close integration.

Keep diagnostic fixtures and probes behind `test-hooks`. Restore or retain the
previous normal artifact after diagnostic-only work and state which changes it
lacks. Never call a normal build live-tested on the strength of a diagnostic run.
Existing logs validate their exact binary and scenario, not a later rebuild.

Record selected profiles, reasons, durations, hashes of binaries actually tested,
observations and omissions. Preserve failed attempts and their original verdicts.
An assessor's synthetic-log test validates the assessor, not gameplay. Optional
platform/private-image skips are not passing evidence; required Lua behavior must
run through the explicit uv/lupa check rather than an optional discovery skip.

Keep installed games, shared kits, saves, historical evidence and releases
unchanged unless explicitly requested. Use isolated disposable test profiles.
Machine paths, paired runtime locations and session notes belong in ignored local
configuration. Project reports use portable references and omit those values.

Historical results remain in Git history and private evidence backups. They
validate only their original artifacts and do not authorize retired workflows.

## Test layout

The Python suite covers feature behavior, evidence assessors, packaging, developer
tools and repository boundaries; it is not a count of live game scenarios. Judge
each test by the regression it detects. Small checks can be valuable for wrong
owner attribution, unsafe update paths, stale evidence and ignored source files.
Remove redundant assertions or tests exclusive to retired implementations when
their dependency review establishes that they no longer protect retained code.

Current Rust integrations are under `crates/asx-game/tests/current/`, with
Cargo targets `client_binding`, `jewelry` and `enemies`. Enemy fixtures exercise
the selected 0.130 support path, retaining bounds/life/health/motion regressions
without a historical reader dependency. Python evidence assessors used by current
trials live with the current test suite; their event formats can retain native
version tags without requiring an obsolete implementation.

Generated reports and exact-artifact evidence can be archived using
[local storage](LOCAL-STORAGE.md). Retention and source-test coverage are separate:
archiving old run outputs does not remove the test assertions.

## Publication checks

`scripts/hygiene_gate.py` is the strict publication-content gate.
`scripts/audit_local_information.py` checks current publishable files and, with
`--history`, reachable Git objects without printing matched values. Current-file
privacy and historical Git review are separate; neither authorizes publication.

Reviewed text references to game filenames are permitted only at their exact
source paths and line hashes with checked occurrence counts. They do not exempt
actual archives, textures or other game payloads. New or changed references must
be reviewed; the sole embedded Windows resource is checked by its exact path and
content hash. Keep the reference inventory narrow instead of excluding whole
source files or removing asset checks.

`tools/build/asx_verify_publication.py --folder <prepared-folder> --local` checks
prepared asset hashes, signatures and the stable download alias. It does not
launch a game or publish. A release also requires strict source hygiene, an
explicit historical-Git decision and exact signed-archive/runtime validation.

The [reproducibility guide](REPRODUCIBILITY.md) describes local and Windows-native
builder contracts. Controlled references belong under
[validation](../validation/README.md); generated results and cleanup snapshots
belong to ignored local evidence. Exact hashes/bytes, reentry, participant
identities and transactional ownership are independent boundaries: passing one
does not establish the others.

### Combined event compatibility trial

For event compatibility, prefer one diagnostic run with `ASX_TEST_ALL_EVENTS=1`
and the default extension trial (`--extensions-trial 1` for solo, `2` for coop).
The solo suite combines the native goblin lifecycle, personal supplies/stat
expiry and a complete Sayge ritual. The coop suite combines goblin replication with
the group ritual and personal supply claims. Preserve each feature's assessor
verdict; a passing event must not erase another event's failed checks.

The suite stops when all event cycles have completed instead of waiting out its
maximum duration. Separate repeat/persistence fixtures remain available for
changes to those rules. Keep the solo acceptance gate before the cooperative run. This fixture flag is
not a gameplay option and does not change production spawn frequency. Sparse
visual captures remain opt-in. Actual goblin movement uses per-update diagnostic
aggregates: two-second position chords alone cannot establish flee direction on
a curved route.

For a support-only adapter check, `ASX_TEST_COMPLETION121=support` with
`ASX_TEST_SUPPORT_ONLY=1` stops after the twelve native casts and observed
results. The default support profile still includes resurrection recovery and
immunity scenarios. `ASX_TEST_SUPPORT_SHIELD_ONLY=1` narrows a failed support
check to the actual foreign shield cast and break; it retains the native
assertions and waits for real target immunity to expire. It never removes a
native protection to force a passing result. A focused shield retry can share the pause trial by combining
`ASX_TEST_COMPLETION121=pause` and `ASX_TEST_SUPPORT_SHIELD_ONLY=1`; both
assessors must pass. Pause trials stop six seconds after
both controlled native menus have closed, retaining movement-resumption evidence.

See the [compatibility workflow](COMPATIBILITY-WORKFLOW.md) for resolved manifests,
artifact receipts, declarative suites and bounded Lua evidence.

### Observed event fixture progression

The combined event suites opt into `ASX_TEST_OBSERVED_STEPS=1`, forwarded only to
owned test-hook clients. Supply steps wait for native scene creation and personal
claim acknowledgements; encounter damage waits for an actual native target;
cooperative material sales follow completed Fortune delivery. The first goblin waits for observed encounter completion, cleared native encounter
targets and resumed player phases, so encounter damage cannot kill it. The second goblin
starts after the first resolves; its controlled kill waits for sufficient actual
movement observations. The first goblin still expires after 25 simulation seconds,
consumables keep their native test duration, and Sayge still requires 20 seconds
of accumulated presence. No Fast mode, simulation clock rewrite or immunity removal
is used. Production event schedules and independent persistence fixtures retain
their previous rules. Step logs identify initiated actions; existing gameplay/UI
assessors separately require actual results before early completion.

The observed-step fixture moves owned participants to a walkable witness position
after cooperative event rewards are acknowledged, before measuring the second
goblin. This isolates motion sampling from ordinary attacks without changing
health or immunity. Its first taunt control is not repeated on the kill subject.
Workflow receipts record actual harness elapsed seconds. Compare those separately
from host-log timestamps; neither is a general performance guarantee.
