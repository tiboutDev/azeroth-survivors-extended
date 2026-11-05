# Native compatibility checklist

Port the AZSX integration used by the new native client. Keep native-owned AI,
networking, balance and new game content native. Use the
[adapter contract](CLIENT-ADAPTER.md), [expected behavior](AZSX-BEHAVIOR.md) and
[tooling reference](COMPATIBILITY-WORKFLOW.md) for their respective details.

The sequence is **complete wiring → code validation → candidate DLL → isolated
live observations**. Do not use game launches to discover missing translations.
This procedure does not authorize commits, publication or release-reference changes.

Commands run in the authoritative WSL checkout unless marked Windows. Configure
`OLD_EXE`, `NEW_EXE`, `NATIVE_SOURCE`, `OLD_DESCRIPTOR`, `EVIDENCE`, `RECEIVER` and `DLL` in ignored
local notes. The receiver must be isolated from installed player games.

## 1. Establish the starting state

```sh
git status --short --branch
git worktree list
git log -1 --format='%H %an <%ae> / %cn <%ce>'
mkdir -p "$EVIDENCE"
git rev-parse HEAD > "$EVIDENCE/source-head.txt"
git diff --binary > "$EVIDENCE/source-before.patch"
sha256sum "$OLD_EXE" "$NEW_EXE" > "$EVIDENCE/native-images.sha256"
```

Preserve unique untracked work too; `git diff` does not include it. Identify the
new executable/archive inventory and protected installations/release assets.
Read the relevant instructions and keep paths, images and disassembly private.

## 2. Finish the entire code wiring block

Before changing the selected registry, inventory the old integration and compare:

```sh
python3 tools/dev/azsx_dev.py inventory > "$EVIDENCE/contracts-before.json"
python3 tools/dev/compare_official.py --old "$OLD_EXE" --new "$NEW_EXE" \
  --descriptor "$OLD_DESCRIPTOR" --out "$EVIDENCE/comparison.json"
python3 tools/dev/azsx_dev.py impact --comparison "$EVIDENCE/comparison.json" > "$EVIDENCE/impact.json"
```

Resume bounded comparison batches as described in the tooling reference. Then
review all consumed native boundaries and group their fixes:

- Contacts, complete instruction guards, callers and continuations.
- Register/stack arguments, returns, allocation and ownership.
- Game/private Actor fields, indirect/UFCS calls, closures and indexed constants.
- Raw buffer extents, vector strides, enums and native producer/consumer outputs.
- Identities, reentry, partial rewards, restoration and lifecycle cleanup.

Map all feature sections and stable acceptance IDs to code checks, remaining live
observations, native-owned behavior or disabled behavior. One proof may cover many
criteria; do not schedule one game scene per row. In particular, review spending
and granting, both roles, dead actors, recipient/source attribution and resets.

Update the exact descriptor, selected adapter, registry/import inventory and
native profile-fixture acceptance. Keep unreviewed fields explicit rather than
using guessed shifts. Regenerate the selected metadata:

```sh
python3 tools/dev/binding_registry.py --write
python3 tools/dev/feature_catalog.py --write-doc
python3 tools/dev/azsx_dev.py check
python3 tools/dev/field_audit.py > "$EVIDENCE/field-uses.json"
```

The field audit is a coverage check. Native ABI, raw schemas and expected behavior
need their independent tests. No candidate DLL or game trial while required
wiring remains unresolved.

## 3. Validate the code once the block is ready

```sh
sh tools/dev/check.sh --profile full
cargo xwin check --locked --target x86_64-pc-windows-msvc -p azeroth-coop-native --tests --features test-hooks
```

`full` already includes Lua 5.1. Supply the reviewed private-image inputs from
[TESTING.md](../development/TESTING.md#native-code-checks). Execute Windows ABI
stand-ins for changed wrappers/registers/stack contracts; a cross-build alone is
not execution. Retain results, exact image/test-EXE identities and skipped scopes.

Fix failed checks and rerun the affected checks. Once the block is green, reuse
that evidence unless its inputs change; do not repeat the full gate at each later
staging, scenario or release step.

## 4. Prepare the candidate artifacts

Import native files into the isolated receiver if needed:

```sh
python3 tools/dev/import_official.py --source "$NATIVE_SOURCE" --receiver "$RECEIVER"
```

Build the diagnostic candidate for fixtures and the normal candidate needed for
compatibility qualification. Build helpers do not deploy:

```sh
sh tools/dev/build-chest-fixture.sh
sh tools/dev/build-windows.sh
python3 tools/dev/azsx_dev.py stage --receiver "$RECEIVER" --dll "$DLL"
```

Record their receipts/hashes. Reuse an existing artifact only if it contains the
current compilation inputs. Separately loaded UI/tool changes need restaging or
new relevant observations, not a DLL rebuild. Never stage over a running client.

## 5. Observe the remaining integration behavior

Inspect the available plan, then select the scenarios covering unresolved risks:

```sh
python3 tools/dev/azsx_dev.py plan compatibility
```

Use Windows Python in the receiver. For each selected family, run its required
solo prerequisite before coop on the same artifact. Complete diagnostic families
first, then stage the normal DLL and run `solo-normal` before `coop-normal`.
`validate` performs preflight itself.

Events, allied effects, revival and pause are shared scopes; select additional
installer/UI/long-run trials only for a changed or uncovered behavior. Reuse valid
existing observations. At most two owned clients, normal priority/all CPUs.
Screenshots are separate scope, not a default qualification campaign.

Stop the affected scenario on error. Diagnose from code/logs before retrying;
record the retry beside the original failure. Do not weaken an expected outcome,
change native gameplay or rebuild unrelated binaries to obtain a green report.

## 6. Consolidate and stop

Record code and exact-artifact observations against the selected criteria. State
PASS, FAIL, NOT RUN or NOT APPLICABLE with scope/reason. Keep normal/diagnostic,
visual and package evidence separate. A passing retry does not resolve an
unexplained intermittent failure; retain it as open.

Leave the intended normal artifact in the isolated receiver, close owned clients
and report changes, verification and remaining limits. Update maintained source
references; keep session-specific status, paths and logs in ignored local evidence.
Stop here unless commit/release/publication work was requested. The
[release checklist](../release/RELEASE-CHECKLIST.md) reuses valid compatibility
results and adds only the remaining release-specific work.
