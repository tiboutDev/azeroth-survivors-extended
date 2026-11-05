# Compatibility tooling reference

The [compatibility checklist](COMPATIBILITY-CHECKLIST.md) owns execution order.
This page explains the tools and metadata used by that procedure. Use
[TESTING.md](../development/TESTING.md) for check selection and
[AZSX-BEHAVIOR.md](AZSX-BEHAVIOR.md) for expected outcomes.

## Sources of truth

| File / tool | Owns |
| --- | --- |
| `azsx/supported-games.json` | Selected executable identity and archive inventory |
| `compat/clients/`, `binding-template.rs` | Reviewed literals and generated native descriptor |
| `compat/field-uses.json` | Reviewed indirect field keys/namespaces and method fingerprints |
| `compat/features.json` | Stable feature IDs, dispositions and domains |
| `compat/contract-links.json`, `domains.json` | Operation-level attribution and conservative fallback |
| `compat/feature-validation.json` | Candidate validation scopes and known coverage gaps |
| `compat/suites.json` | Executable live scenarios, prerequisites and plans |
| `tools/dev/test_plan.py` | Portable check profiles and changed-file selection |

Feature and binding documentation are generated, then checked:

```sh
python3 tools/dev/binding_registry.py --write
python3 tools/dev/feature_catalog.py --write-doc
python3 tools/dev/azsx_dev.py check
```

Neither generation nor catalog attribution approves an unknown executable.

## Native inventory and comparison

```sh
python3 tools/dev/azsx_dev.py inventory > "$EVIDENCE/contracts.json"
python3 tools/dev/compare_official.py --old "$OLD_EXE" --new "$NEW_EXE" \
  --descriptor "$OLD_DESCRIPTOR" --out "$EVIDENCE/comparison.json"
python3 tools/dev/azsx_dev.py impact --comparison "$EVIDENCE/comparison.json" > "$EVIDENCE/impact.json"
```

The Rust `native-inventory` helper parses free functions, impl methods, private
helpers and inline modules, including production code after test modules. It
excludes test-only items and retains unknown platform/feature conditions. It also
records field/memory operations and local/loop bindings. Macro expansion and a
complete call graph are not inferred.

Explicit contract links carry feature IDs, rationale and existing source references.
Other methods retain conservative domain/file attribution; unclassified operations
affect every feature. Missing/ambiguous explicit links fail inventory. A coverage
association is a review aid, not evidence that a scenario executed every method.

The comparer produces unapproved instruction-aligned candidates. It uses bounded
caches, a 2048 MiB address-space limit and four function owners per batch. Run one
comparison process at a time. Continue with the same inputs plus
`--resume "$EVIDENCE/comparison.json"`; use `--owner <RVA>` or `--retry-failures`
for unresolved owners. Checkpoints validate image and required-contact inventories.
A completed comparison still requires ABI, schema and ownership review.

## Build receipts and staging

After code validation, build only the artifact kinds needed by the chosen scope:

```sh
sh tools/dev/build-chest-fixture.sh
sh tools/dev/build-windows.sh
python3 tools/dev/azsx_dev.py stage --receiver "$RECEIVER" --dll "$DLL"
```

The build helpers compile only. Adjacent `.artifact.json` receipts bind compiler
command, toolchain/flags, compilation inputs, artifact kind and DLL SHA-256.
Changed inputs invalidate that build; separately loaded Lua does not require a
new DLL. Rust-embedded content remains a compilation input.

Staging requires a reviewed private import and verifies the source-side compiler
receipt. It copies the selected DLL, retained UI/AZSXUI and executable scenario
tools/fixtures. Rust sources and compiler helpers stay in WSL. Unchanged files are
not recopied; unknown receiver files and prior results are preserved. Do not stage
while clients are running or target a player installation.

## Identity scopes

| Identity | Changes when |
| --- | --- |
| Compilation fingerprint + DLL hash/kind | Rust/resources/toolchain/build inputs or resulting bytes change |
| UI fingerprint | Retained Lua or AZSXUI files change |
| Scenario/tool fingerprint | That scenario, its fixture data or executed tooling changes |
| Native identity | The selected executable changes |

Build/staging receipt schema 2 separates those scopes. `source_fingerprint` remains
an alias for `compilation_fingerprint`; historical schema-1 receipts retain their
original combined semantics. Old evidence is never relabelled as a new build.
Other scenario declarations and catalog metadata do not invalidate an unrelated
proof; shared executable tooling can.

Windows receivers use a trusted local execution inventory, not a source checkout
or a signed publication manifest. `validate` checks that inventory automatically.
`preflight` is available when an inspection without a launch is useful; it need
not be run separately before every `validate` command.

## Bounded scenarios

Inspect the relevant plan in WSL:

```sh
python3 tools/dev/azsx_dev.py plan events
python3 tools/dev/azsx_dev.py plan compatibility
```

Run a selected pair in the staged receiver with Windows Python:

```text
python -I -B tools/dev/azsx_dev.py validate solo-events
python -I -B tools/dev/azsx_dev.py validate coop-events
```

Suites declare kind, player count, maximum time, readiness and solo prerequisite.
Coop uses the exact artifact/UI/tool identity of its required solo proof. Group
all diagnostic work before normal work to avoid repeated DLL changes. A plan is
a selection aid, not a requirement to run all available scenarios for every edit.

Readiness triggers assessment. Early completion requires the same acceptance
function as final completion; every required observation must pass. A retry writes
new evidence and retains the failure. `retry shield_consumed` is a focused scope,
not a reason to restart unrelated campaigns.

At most two owned clients run with normal priority/all CPUs and disposable profiles.
The private desktop runner verifies each child's desktop and owns cleanup.
Native ready/ROAM controls remain authoritative. Interactive desktop access and
screenshots require explicit scope. See [testing](../development/TESTING.md#scenario-selection)
for scenario families, including native revival, progression, UI and exact archives.

## Lua diagnostics and results

`tools/dev/diagnostics/observer.lua` is opt-in tooling, absent from packaged UI.
It samples changed observations at most twice a second, for at most 300 seconds
and 512 records. A test-hooks-only mailbox sends records to the owned log without
persisting settings. Ordinary setters retain their original arguments and return.

Records bind schema, session, slot, sequence and clock. Collectors reject foreign,
missing/duplicate or malformed records. Event suites retain gameplay assessor
results beside `lua-observations-*.jsonl` and `lua-evidence-*.json`. Native model
values, widget visibility/text and actual visual layout are separate observations.

Workflow receipts retain actual DLL/native/source/UI/tool identities, elapsed time,
verdict and result path. Keep them and failed attempts in ignored local evidence.
Normal, diagnostic and signed-archive results have distinct scopes. A later pass
is not a diagnosis of an earlier intermittent failure.
