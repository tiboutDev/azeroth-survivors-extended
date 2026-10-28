# Targeted ASX validation

The supported source target is official 0.125.0. Review the
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

`full` tests the current 0.125 Rust graph, Python suite and Clippy. Retired
reader/tool implementations and their exclusive tests are not part of the gate.

| Profile | Current risk covered |
| --- | --- |
| official | Native 0.125 selection, normalized snapshots, menu ownership, native policy transactions, current source boundaries and evidence assessors |
| official-party-ui | Retained native children, native owner origins, menu projection and required Lua 5.1 behavior through uv/lupa |
| asx-install | Owned native-folder installation, signatures, transactions, interrupted-client recovery, rollback and publication inventory verification |
| gems | Fixed gem identity, selected jewelry transactions and evidence rejection |
| enchantments | Ring identity/ranks, selected jewelry transactions and evidence rejection |
| glyphs | Retained rule catalogs and shared bonus representation; glyph gameplay remains disabled on 0.125 |
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
the selected 0.125 support path, retaining bounds/life/health/motion regressions
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
