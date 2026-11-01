# Native compatibility checklist

Ordered maintainer procedure. Use it with the [behavior acceptance catalog](ASX-BEHAVIOR.md),
[workflow](COMPATIBILITY-WORKFLOW.md), [adapter contract](CLIENT-ADAPTER.md) and
[test guide](../development/TESTING.md). This checklist does not authorize commits, deployment or
publication. A completed patch does not automatically complete a release.

Commands below run from the authoritative WSL checkout unless marked **Windows**.
Prepare dependencies using [BUILDING.md](../development/BUILDING.md). Shell variables are explicit
placeholders: set `PATCH`, `NATIVE_SOURCE`, `OLD_EXE`, `NEW_EXE`, `OLD_DESCRIPTOR`
and `RECEIVER` to reviewed values in ignored local workspace notes. Never point
`RECEIVER` at a player installation. Use a fresh `EVIDENCE=.local/compat-$PATCH`.
Store actual verdicts in evidence, not checked boxes in this reusable document.

## 1. Establish scope and preserve the starting state

```sh
git status --short --branch
git worktree list
git log -1 --format='%H %an <%ae> / %cn <%ce>'
mkdir -p "$EVIDENCE"
git rev-parse HEAD > "$EVIDENCE/source-head.txt"
git diff --binary > "$EVIDENCE/source-before.patch"
```

- **Expected:** intended branch/worktree identified; existing edits reviewed and preserved.
  Read AGENTS.md, the linked contracts and current compatibility notes first.
- **Keep:** source revision, dirty-file inventory/fingerprints, starting patch,
  authorized scope, isolated receiver paths and hashes of protected release assets.
- **Block:** ambiguous source ownership, unrelated edits that would be overwritten,
  or an official/player installation selected as the receiver. Do not reset changes.

## 2. Identify and review the new native image

```sh
sha256sum "$OLD_EXE" "$NEW_EXE" > "$EVIDENCE/native-images.sha256"
python3 tools/dev/asx_dev.py inventory > "$EVIDENCE/contracts-before.json"
python3 tools/dev/compare_official.py --old "$OLD_EXE" --new "$NEW_EXE" \
  --descriptor "$OLD_DESCRIPTOR" --out "$EVIDENCE/comparison.json"
```

The comparer processes bounded batches. Continue with the same arguments plus
`--resume "$EVIDENCE/comparison.json"`; investigate ambiguous/unresolved contacts
individually. Exit success for one batch is not completion of the inventory.
Review producers and consumers, row extents, registers, stack arguments, return
sites and hook ownership. Similar instructions are only candidate matches.

- **Expected:** exact executable and archive inventory identified; every retained
  contact has reviewed semantics, not merely a matching prologue.
- **Keep:** image hashes, inventory, comparison checkpoints, unresolved-contact
  decisions and private ABI/layout evidence. Never commit native binaries/disassembly.
- **Block:** guessed offsets, unexplained ABI/register changes, unresolved required
  contacts or broader activation rules used to accept an unknown executable.

## 3. Map all expected behaviors and update the selected adapter

```sh
python3 tools/dev/asx_dev.py impact --comparison "$EVIDENCE/comparison.json" > "$EVIDENCE/impact.json"
python3 tools/dev/asx_dev.py plan compatibility > "$EVIDENCE/plan.json"
```

Review **every** ASX-01 through ASX-29 section in [ASX-BEHAVIOR.md](ASX-BEHAVIOR.md).
Create its per-patch evidence table: criterion IDs, trigger/expected outcome,
PASS/FAIL/NOT RUN/BLOCKED/NOT APPLICABLE, proof and remaining limits. Review both
roles, spending/granting, alive/dead actors and session resets. Revisit its recurring
regression priorities; those priorities do not replace the rest of the catalog.

Update the exact descriptor, selected adapter, native registry/import inventory
and disposable-profile acceptance only after review. Preserve native-owned and
disabled behavior. Do not refresh native balance references without a request.

```sh
python3 tools/dev/binding_registry.py --write
python3 tools/dev/feature_catalog.py --write-doc
python3 tools/dev/asx_dev.py check
```

- **Expected:** generated bindings/catalog agree with reviewed source; all behavior
  criteria have a disposition and a relevant verification plan.
- **Keep:** impact report, acceptance matrix, descriptor review and source diff.
- **Block:** missing feature disposition, unexplained behavior change, weakened
  checks, or changed registry labels without reviewed native contracts.

## 4. Run portable, Lua and platform checks

```sh
sh tools/dev/check.sh --profile full
sh tools/dev/check.sh --profile official-party-ui
cargo xwin check --locked --target x86_64-pc-windows-msvc -p azeroth-coop-native --tests --features test-hooks
```

Use the dependencies and exact-image inputs documented in [TESTING.md](../development/TESTING.md).
Execute relevant Windows ABI stand-ins when wrappers, registers, stack contracts
or generated code change; cross-compilation is not execution. Record the actual
Windows test command/binary and serial execution in evidence. Inspect the selected
profiles rather than assuming `full` includes Lua execution or live checks.

- **Expected:** relevant tests and Clippy pass; required Lua 5.1 behavior executes;
  changed native ABI contracts have executed Windows evidence.
- **Keep:** profile result JSON/logs, command lines, durations, exact-image identity,
  ABI test hashes/results and explicit platform skips.
- **Block:** failed required check, stale/wrong native fixture, missing ABI execution,
  or reporting a skipped test as passed.

## 5. Import and stage the isolated diagnostic receiver

```sh
python3 tools/dev/import_official.py --source "$NATIVE_SOURCE" --receiver "$RECEIVER"
sh tools/dev/build-chest-fixture.sh
python3 tools/dev/asx_dev.py stage --receiver "$RECEIVER" \
  --dll target/test-hooks/x86_64-pc-windows-msvc/release/avrt.dll
```

Reuse an already verified import when appropriate; do not overwrite an existing
receiver blindly. Import requires the newly reviewed selected registry. With
**Windows Python, from the staged receiver**:

```text
python -I -B tools/dev/asx_dev.py preflight
```

- **Expected:** exact private import, diagnostic artifact receipt and execution
  inventory match. No clients run while staging.
- **Keep:** import, compiler, staging and preflight receipts, DLL/UI/tool fingerprints.
- **Block:** ownership conflict, stale artifact, wrong executable, partial staging
  or a receiver modified independently of WSL source.

## 6. Validate behavior: solo before two-player coop

State the risk, expected observation and stop condition before live work. Select
bounded suites from the plan according to the acceptance matrix; do not run every
suite automatically. Example event sequence, **Windows receiver**:

```text
python -I -B tools/dev/asx_dev.py validate solo-events
python -I -B tools/dev/asx_dev.py validate coop-events
```

Use core/support/pause/progression or focused UI suites where the review requires
them. The runner enforces declared solo prerequisites for the exact artifact.
Use at most two owned clients, private desktop, normal priority and all CPUs.
Screenshots and visual claims require separately authorized visual observation.

- **Expected:** each relevant assessor observes the intended outcome; actual
  progression/gameplay occurs, not merely lobby connection or activation.
- **Keep:** workflow receipts, per-suite results, logs, limits and acceptance IDs.
  Preserve failures and the narrower scope of any retry.
- **Block:** missing witness, regression, stale prerequisite or compromised isolation.
  Fix the cause; never reinterpret an unobserved case as a pass.

## 7. Validate the normal artifact separately

```sh
python3 tools/build/asx_release_binary.py --target-dir "$NORMAL_TARGET" --receipt "$NORMAL_RECEIPT"
python3 tools/dev/asx_dev.py stage --receiver "$RECEIVER" \
  --dll "$NORMAL_TARGET/x86_64-pc-windows-msvc/release/avrt.dll"
```

Choose fresh local target/receipt paths. For a release immediately following the
port, [release checklist step 3](../release/RELEASE-CHECKLIST.md#3-produce-the-normal-release-candidate)
can supply this artifact instead of building it twice. **Windows receiver**:

```text
python -I -B tools/dev/asx_dev.py preflight
python -I -B tools/dev/asx_dev.py validate solo-normal
python -I -B tools/dev/asx_dev.py validate coop-normal
```

- **Expected:** normal artifact has no diagnostic hooks; solo and coop pass on its
  exact hash. Retain a normal artifact in the receiver after diagnostic work.
- **Keep:** normal compiler/staging receipts, both normal trial results and limits.
- **Block:** diagnostic-only evidence used to certify production, stale DLL/UI,
  native activation failure or required normal-runtime failures.

## 8. Close the compatibility review

```sh
python3 tools/dev/asx_dev.py check
git diff --check
git status --short --branch
```

Update the [current-client notes](CLIENT-COMPATIBILITY.md), affected behavior/API
and player documentation. Recheck protected native/release hashes. Summarize source,
diagnostic and normal-runtime evidence separately; name untested cases and their
impact. Required unresolved outcomes block full behavioral-parity claims. Optional
limits remain explicit, never silently waived. Do not rerun unchanged scenarios
without a failure, new source change or uncovered risk.

- **Keep:** final criterion matrix, source revision/dirty fingerprint, hashes,
  preservation check and validation summary.
- **Block:** undocumented required gap, altered protected assets or source/artifact
  mismatch. Commit only if requested, with tiboutDev as author and committer.
- **Next:** use [RELEASE-CHECKLIST.md](../release/RELEASE-CHECKLIST.md) if release preparation
  is authorized. Compatibility acceptance alone does not authorize publication.
