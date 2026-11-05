# Testing AZSX

Choose validation for the changed behavior. The [compatibility checklist](../compatibility/COMPATIBILITY-CHECKLIST.md)
owns client-port order; the [release checklist](../release/RELEASE-CHECKLIST.md) owns
release qualification. Expected outcomes are in [AZSX-BEHAVIOR.md](../compatibility/AZSX-BEHAVIOR.md).

## Choose the smallest useful scope

| Change | Verification |
| --- | --- |
| Documentation only | Review content, links and generated-document consistency where affected |
| Python/developer tools | Tests for those tools; no DLL or game |
| Portable rule | Relevant rule tests and Clippy |
| Lua/UI | Relevant Lua 5.1 profile; visual trial only for a rendered-layout claim |
| Native schema/call/hook | Selected-memory and exact-image expected outputs; Windows compilation/ABI execution where changed |
| Client port or release milestone | One complete `full` code gate after the batch is ready, then remaining artifact/live checks |

Do not rerun passing checks because another task starts. Reuse results while the
relevant source, tests and configuration remain unchanged. After a failure, fix
the cause and rerun the affected checks; expand scope only for a new risk.

## Commands

```sh
sh tools/dev/check.sh --dry-run
sh tools/dev/check.sh --profile official --profile tooling --dry-run
sh tools/dev/check.sh --profile support --reason "Allied effect ownership changed"
sh tools/dev/check.sh --profile full
sh tools/dev/check.sh --list
```

Without a profile, changed files select checks; unknown executable changes choose
`full`, while documentation-only changes choose none. Profiles combine/deduplicate.
`full` supersedes narrower profiles and already runs Rust, Python **including Lua
5.1 through pinned lupa**, the native-inventory helper and Clippy. Do not follow it
with `official-party-ui` or `azsx-install` just to repeat included tests.

Checks are manual; there is no GitHub Actions campaign on push. The runner never
builds a DLL, stages files or launches a game. Results, commands and durations go
to ignored `test-results/targeted/` directories.

| Profile | Scope |
| --- | --- |
| `official` | Selected adapter, native transactions, ownership and evidence assessors |
| `official-party-ui` | Native party presentation, markers, text and Lua behavior |
| `azsxui` | Standalone addon HUD, plates, minimap, settings and restoration |
| `azsx-install` | Launcher, package ownership, updates and removal |
| `rules` | All portable gameplay rules |
| `combat`, `support`, `periodic`, `fatigue` | Their respective combat/support/lifecycle contracts |
| `gems`, `crystals`, `enchantments` | Their respective reward/stat/presentation contracts |
| `tooling`, `diagnostics` | Developer tools and diagnostic infrastructure |
| `full` | Complete portable code gate, including discovered Lua behavior |

## Native code checks

Finish the code review before a new candidate DLL or live port trial. Include
field calls through closures, arrays, indirect/UFCS helpers and host/private
branches, plus raw buffers, enums, ownership and register/stack contracts.

```sh
python3 tools/dev/azsx_dev.py check
python3 tools/dev/field_audit.py > "$EVIDENCE/field-uses.json"
sh tools/dev/check.sh --profile full
```

The AST field audit rejects missing keys and stale indirect reviews. It checks
coverage; native producer/consumer execution checks semantics. Do not validate an
offset by seeding a fixture from that same offset without an independent expected
result. See the [adapter contract](../compatibility/CLIENT-ADAPTER.md).

Private-image tests execute native code under emulation without starting the game.
Set only the image inputs needed by the reviewed tests:

| Environment variable | Exact-image suite |
| --- | --- |
| `AZSX_REVIEW_IMAGE_1410` | Current native contracts in `test_1410_native_review.py` |
| `AZSX_REVIEW_IMAGE_1400` | Historical comparison contracts in `test_1400_native_review.py` |
| `AZSX_REVIEW_IMAGE_1392` | Historical comparison contracts in `test_1392_native_review.py` |
| `AZSX_REVIEWED_IMAGE_138` | Earlier reviewed catalog/comparison input |
| `AZSX_REVIEWED_IMAGE_1351` | Recorded native balance/arithmetic reference |

Image hashes are checked by the suites. Missing private/platform dependencies may
skip tests; record those as unexecuted, not passes. Historical inputs are provenance,
not additional supported clients. No MPQ scanning or native asset modification.

Check Windows-specific source deliberately. Cross-compiling is not ABI execution:
run relevant Windows test EXEs serially when wrappers, register/stack carriers or
code generation change. See [BUILDING.md](BUILDING.md#windows-target-checks).

Rust integration targets are `client_binding`, `jewelry`, `enemies`, `presentation`
and `wiring` under `crates/azsx-game/tests/current/`. They cover selected fixtures
and transactions. Python covers Lua behavior, tools, assessors and native oracles;
its test count is not a count of live scenarios.

## Builds and live validation

A DLL is needed only when its compilation inputs change or a new artifact is
requested. Documentation, Python tooling and separately loaded Lua do not require
a DLL rebuild. Normal and `test-hooks` artifacts have separate receipts. A
compatibility/release milestone needs normal-artifact evidence as well.

Use the [workflow commands](../compatibility/COMPATIBILITY-WORKFLOW.md) to stage
explicitly into an isolated receiver. Run the required solo prerequisite on the
same artifact before coop. Group diagnostic scenarios together, then normal ones,
so the receiver changes artifact kind only once.

Choose a bounded scenario for the observation that code cannot establish. Plans
are menus of related scopes, not instructions to run every campaign. Reuse one
run for multiple assertions. A failed retry remains a failed attempt in the
record; another passing run does not diagnose intermittence.

Use at most two owned clients, normal priority and all CPUs, with disposable
profiles. Stop on severe slowdown. Screenshots and interactive desktop use require
their own scope. Creation logs, API model data and widget visibility establish
different facts; only inspected captures support visual-layout claims.

## Scenario selection

`compat/suites.json` is the executable catalog: kinds, environment, time budgets,
readiness and prerequisites live there. Inspect a plan with:

```sh
python3 tools/dev/azsx_dev.py plan events
python3 tools/dev/azsx_dev.py plan compatibility
```

| Need | Relevant scenarios / tools |
| --- | --- |
| Core rewards/stats/identity | `solo-core`, `coop-core` |
| Supplies, Sayge and goblins | `solo-events`, `coop-events` |
| Allied casts | `coop-support`; `coop-shield` for a failed shield-specific scope |
| Death and native revival marker | `coop-revive-marker` |
| Dead-player XP and whole-level receipts | `solo-progression-probe`, `coop-progression-probe` |
| Pause and native Escape | `coop-pause`, `coop-native-escape` |
| Normal production conditionals | `solo-normal`, `coop-normal` |
| Addon API/HUD/visual follow-ups | Named addon/UI suites or the dedicated `azsx_api_trial.py`, `azsxui_hud_trial.py`, `azsxui_probe_trial.py` |
| Changed installer/update/migration | Exact signed-archive trial from the release checklist |

Combined event fixtures advance from observed claims, native encounters and reward
acknowledgements. They keep production lifetimes and require actual flee movement,
escape, kill rewards, stat restoration and recipient-specific claims. Readiness
only starts assessment; every required check must pass before early completion.

Support/progression fixtures use actual native casts, damage, receipts and revival.
Positioning belongs only to diagnostic profiles; it must not bypass range,
cooldowns, immunity or native phase eligibility. Same-profile XP comparisons do
not justify forcing equal progression for arbitrary real parties.

The private Windows dispatcher verifies the child desktop and owns process cleanup.
Startup uses the game's ready/ROAM driver in its disposable windows; it does not
write a gameplay phase or take over the user's desktop. Recheck driver details
only if the native dialog, input contract or trial resolution changes.

For an unusual fixture option, read that tool's help/source and its assessor.
Do not add another permanent guide paragraph for each past debugging attempt.

## Results and stopping

Keep commands, input/artifact identities, outcomes and limits with the run. Source,
normal DLL, diagnostic DLL and signed package are distinct evidence scopes.
A synthetic-log test validates an assessor; it does not validate gameplay.
Do not convert missing observations, skips or an unexplained failure into passes.

When a block passes, continue to the next task or report its result. Do not add
unrelated installer, visual, long-run or repeated normal/diagnostic trials as a
closing ritual. Store session-specific findings and unresolved issues under
ignored evidence, as described in [LOCAL-STORAGE.md](LOCAL-STORAGE.md).

## Publication checks

Publication work uses `scripts/hygiene_gate.py` and
`scripts/audit_local_information.py --history`. Current-tree and historical-Git
review have different scopes. Exact source references to native filenames do not
permit distributing native payloads. See the [release checklist](../release/RELEASE-CHECKLIST.md)
for signing, archive validation, reproducibility and per-release artifact records.
Publication still requires an explicit request. These checks are
not part of routine documentation or gameplay maintenance.
