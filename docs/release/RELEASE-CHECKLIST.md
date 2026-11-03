# Release preparation checklist

Ordered maintainer procedure, separate from publication. Read [installation and
distribution](AZSX-INSTALLATION.md), [reproducibility](REPRODUCIBILITY.md) and
[publication checks](../development/TESTING.md#publication-checks). For a new native patch,
complete [COMPATIBILITY-CHECKLIST.md](../compatibility/COMPATIBILITY-CHECKLIST.md) first.

Use WSL from the authoritative checkout, with [build dependencies](../development/BUILDING.md).
Set reviewed local variables: `VERSION`, `GAME_VERSION`, `SEQUENCE`, `OUTPUT`,
`EVIDENCE`, `CANDIDATE`, `DLL`, `RUNTIME`, `RECEIVER` and `SIGNING_KEY`.
`OUTPUT` must be a new preparation folder; `EVIDENCE` must be under
`.local/releases/<version>`; `CANDIDATE` must be a fresh ignored `test-results/`
path. Keep concrete machine paths in local notes. `SIGNING_KEY` is only a path
explicitly supplied through authorized local configuration; never print key bytes
or the complete signing configuration. Do not discover credentials automatically.

## 1. Freeze scope and release identity

```sh
git status --short --branch
git log -1 --format='%H %an <%ae> / %cn <%ce>'
git worktree list
mkdir -p "$EVIDENCE"
git rev-parse HEAD > "$EVIDENCE/source-head.txt"
git diff --binary > "$EVIDENCE/source-before.patch"
```

Choose the next version and strictly increasing feed sequence from actual prior
release records, including prepared/published releases. Record which earlier
folders/assets must remain unchanged and their hashes. Do not infer publication
from a prepared folder. Review dirty source before packaging; never silently
commit it or represent it as part of HEAD. If commits were requested, use tiboutDev
for both author and committer and verify their identities afterwards.

- **Expected:** native target, version, sequence, source state, signing authority
  and preservation boundaries are explicit.
- **Keep:** source revision/patch/fingerprints, prior asset hashes, chosen identity.
- **Block:** ambiguous source state, conflicting sequence/version, reused output
  directory, unauthorized signing/commit or an unqualified native client.

## 2. Pass source gates and review documentation

```sh
python3 tools/dev/azsx_dev.py check
sh tools/dev/check.sh --profile full --profile azsx-install
sh tools/dev/check.sh --profile official-party-ui
python3 scripts/hygiene_gate.py
python3 scripts/audit_local_information.py --history --report "$EVIDENCE/PRIVACY-AUDIT.json"
git diff --check
```

Review README, packaged README, player installation/features, compatibility target,
release notes and migration instructions. Keep balance references at their recorded
provenance unless an explicit refresh was requested. Carry existing exact-source
Lua/ABI evidence where valid, identifying it as carried evidence; run missing or
invalidated checks. Full portable checks do not execute game/archive trials.

- **Expected:** relevant gates pass; player instructions match actual installation,
  automatic compatible updates and addon inclusion.
- **Keep:** profile results/logs, Lua/ABI scope, hygiene report, current and reachable
  history privacy review; platform skips stay visible.
- **Block:** failed required checks, secret/private/native payload leakage, stale
  critical player instructions or unresolved required compatibility outcomes.

## 3. Produce the normal release candidate

```sh
python3 tools/build/azsx_reproducibility.py --candidate "$CANDIDATE"
```

The producer makes two independent fresh normal builds. Select the emitted DLL
whose hash matches the candidate, retain both receipts, and verify source/recipe
fingerprints stayed unchanged. Review absence of diagnostic hooks/developer paths,
PE timestamp and debug directory. Do not overwrite the approved reference yet.

For a launcher/docs-only release, an existing approved DLL may be reused only with
verified unchanged compilation inputs and explicit artifact-scoped evidence.
Record the reuse rationale; do not claim fresh builds or fresh runtime tests.
A changed native target or compiled source requires a new qualified artifact.

- **Expected:** identical independent bytes, complete provenance, normal build.
- **Keep:** candidate JSON, both build/compilation receipts, build logs and DLL hash.
- **Block:** differing builds, missing receipts, changed inputs, test hooks or
  substituting an old/unapproved reference by assumption.

## 4. Qualify that exact normal DLL in isolation

```sh
python3 tools/dev/azsx_dev.py stage --receiver "$RECEIVER" --dll "$DLL"
```

In the verified receiver, with **Windows Python**:

```text
python -I -B tools/dev/azsx_dev.py preflight
python -I -B tools/dev/azsx_dev.py validate solo-normal
python -I -B tools/dev/azsx_dev.py validate coop-normal
```

Use prior results only when their DLL/source/UI/tool/scenario identities still
qualify. Run solo first, then coop with at most two owned clients, private desktop,
normal priority and all CPUs. Never deploy to a player installation for this gate.

- **Expected:** real startup, progression and native coop; clean required assessors.
- **Keep:** actual binary hash, staging and normal-suite receipts, logs and limits.
- **Block:** wrong artifact kind/hash, stale proof, missing required outcome or
  treating diagnostic-only results as evidence for normal conditionals.

## 5. Build and authenticate the signed archive

Verify bundled Windows Python provenance and cryptography availability. Verify the
explicit signing key's public part against the configured trust list without
exposing the private key. Then:

```sh
python3 tools/build/azsx_package.py --output "$OUTPUT" --dll "$DLL" \
  --runtime "$RUNTIME" --version "$VERSION" --game-version "$GAME_VERSION" \
  --sequence "$SEQUENCE" --private-key "$SIGNING_KEY"
mkdir "$OUTPUT/UPLOAD"
cp "$OUTPUT/azsx-$VERSION.zip" "$OUTPUT/UPLOAD/azsx-$VERSION.zip"
cp "$OUTPUT/azsx-$VERSION.zip" "$OUTPUT/UPLOAD/AZSX.zip"
cp "$OUTPUT/azsx-latest.json" "$OUTPUT/azsx-latest.json.sig" "$OUTPUT/UPLOAD/"
```

Create `PUBLICATION.json` in the intermediate output with `repository`, `tag`,
`version`, `sequence`, public `public_key`, `published: false` and `assets` mapping
**exactly four filenames** to their byte `size` and `sha256`. Use hashes of files,
not copied assumptions. Write `SOURCE-COMMIT.json` with `payload_source_commit`,
`release_source_commit` and any explicit dirty-source scope. The package builder
creates PREPARATION.json; it does **not** create publication metadata or UPLOAD.

```sh
python3 tools/build/azsx_verify_publication.py --folder "$OUTPUT" --local
```

Also authenticate/unpack through `azsx/updates.py` (`verify` then `unpack`) using
the intended native hash and prior sequence, and compare extracted DLL, AZSXUI,
presentation and launcher bytes with the recorded source. This payload check is
separate from the publication verifier's signatures/archive-alias check.

- **Expected:** correct signature/sequence/native identity, exact normal DLL and
  complete AZSXUI; no native executable/MPQs, private data or signing key in payload.
- **Keep:** PREPARATION, PUBLICATION, SOURCE-COMMIT, signature/payload verification
  and runtime provenance. Keep these private, outside upload attachments.
- **Block:** unsigned/invalid feed, wrong key or identity, payload mismatch,
  unexpected files, or manual edits to signed archive/feed after validation.

## 6. Validate the exact signed archive on Windows

Before finalization, the versioned archive and its PREPARATION.json must be
siblings. With **Windows Python from the isolated receiver**, substitute actual
Windows paths for the placeholders:

```text
python -I -B tools/dev/background_desktop.py --azsx-install --azsx-package "<prepared-folder>/azsx-<version>.zip" --seconds 50
```

For a supported legacy-layout migration, add
`--legacy-azsx-package "<legacy-fixture>/AZSX.zip"`. Copy the authentic old archive
and its matching PREPARATION.json into a disposable fixture folder; do not write
metadata into the old publication folder. Verify the original hashes first.
A .2-to-current-layout test is distinct from a same-layout automatic update.
For changed update/rollback/runtime-worker behavior, add its relevant focused
checks from [AZSX-INSTALLATION.md](AZSX-INSTALLATION.md) and [TESTING.md](../development/TESTING.md);
the archive smoke alone does not prove those transactions.

Inspect result.json, not just process exit. Currently the base trial has 26
checks, or 30 with legacy migration; counts are informational, every required
check must pass. The pre-migration read-only diagnostic may correctly report an
old owned DLL pending replacement. Startup must subsequently install the exact
new DLL, preserve preferences and complete actual native coop gameplay.

- **Keep:** exact archive/DLL hashes, per-check results, launcher/native logs,
  desktop/resource receipts and every failed attempt.
- **Block:** any required failed/missing check, modified archive, corrupted native
  data or migration conflict. Fix source/runner according to the actual cause;
  rebuild/re-sign and revalidate if payload changes. Never waive a product failure.

## 7. Obtain explicit reproducibility approval when required

Only after the candidate, normal trials and signed-archive results are reviewable,
request approval of the exact DLL SHA-256, explaining the previous reference,
two-build result, runtime scope and preserved publication boundaries.

If approval is pending or declined, keep the candidate separate and mark preparation
incomplete. The producer never approves its own candidate. After explicit approval,
preserve the superseded reference in private evidence, update
`validation/azsx-reproducible-build-xwin.json` and [REPRODUCIBILITY.md](REPRODUCIBILITY.md)
with the decision/date/scope, then run:

```sh
python3 scripts/check_xwin_reference.py --candidate "$CANDIDATE"
git diff --check
```

A candidate differing from the old reference is expected to fail this comparison
before approval; that does not authorize changing it. Commit reference/docs only
if authorized, with tiboutDev. Reference approval alone does not authorize Git
push, tagging or publication. An unchanged already-approved reference needs no
invented replacement approval.

- **Keep:** approval text/date/hash, old and new references, comparison result,
  commit identity or an explicit uncommitted-source record.
- **Block:** missing approval for a new reference, mismatched hash/recipe/source,
  or historical approval reused for a different binary.

## 8. Finalize the minimal preparation folder

Write RELEASE-NOTES.md, README-PREPARATION.md and PUBLISH-ON-GITHUB.md in English.
Include required native version, AZSXUI inclusion and applicable migration/repair
instructions. Write VALIDATION.json with honest `technical_validation_passed`,
`reference_approved`, actual hashes, scenario scope and limitations. Update
SOURCE-COMMIT/PREPARATION metadata to the actual final source/approval state.

```sh
python3 tools/build/azsx_release_layout.py --folder "$OUTPUT" --evidence-dir "$EVIDENCE"
python3 tools/build/azsx_verify_publication.py --folder "$OUTPUT" --evidence-dir "$EVIDENCE" --local
git status --short --branch
```

- **Expected:** root contains exactly `UPLOAD/`, `README-PREPARATION.md`,
  `RELEASE-NOTES.md`, `PUBLISH-ON-GITHUB.md`; UPLOAD contains exactly four assets.
  Technical evidence is retained under `.local/releases/<version>/`.
- **Keep:** final layout inventory, verification, source identity, approval and
  complete scoped test evidence, including failures. Verify protected old assets
  and native source hashes remain unchanged. Any final metadata edits require
  another local verification; documentation-only edits need no game relaunch.
- **Block:** conflicting evidence, extra upload files, invalid signatures, missing
  required approval, changed protected releases or misleading validation claims.
- **Handoff:** report commits/uncommitted changes, folder, four assets, tests and
  limits. State explicitly whether any push/tag/publication occurred. Preparation
  stops here unless publication is separately authorized; after actual publication,
  verification without `--local` checks the remote assets and latest links.
