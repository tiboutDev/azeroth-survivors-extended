# Local generated data

These locations are ignored and are not source-distribution content.

| Location | Purpose | Retention |
| --- | --- | --- |
| `target/` | Cargo caches, intermediate objects and compiled artifacts | Caches are regenerable; keep exact binaries/symbols needed by evidence before purging |
| `test-results/` | Generated check reports, trials, artifact receipts and private retirement backups | Ordinary runs can be pruned after preserving relevant evidence; backups are not disposable cache |
| `.local/evidence/` | Verified compressed local evidence and binary archives, with hash indexes | Preserve until an explicit evidence-retention decision; never commit or publish implicitly |
| `.local/workspace/` | Optional editor/environment notes and workspace-specific handoffs | Local developer configuration, not project instructions |

`target/` and `test-results/` will be recreated by future builds/checks. Neither
needs to stay populated for the source to work. Removing Cargo caches costs a
rebuild; it does not remove source or dependency lock information. Shared tool
caches configured outside the checkout are separate and must not be purged as
part of project cleanup.

## Verified archive and cleanup

Stop builds, checks and trials before cleanup. Inspect first:

```sh
python3 tools/dev/local_artifacts.py
```

To explicitly archive and then remove the two generated trees:

```sh
python3 tools/dev/local_artifacts.py --apply
```

The helper accepts no arbitrary deletion directory. It refuses tracked files,
redirected root/archive directories and special files. It archives every local
result/backup plus exact DLL/EXE/PDB/MAP files and build receipts from `target/`,
verifies content hashes, link targets and timestamps, then rechecks unchanged
source inventory before removal. It does not follow links into another workspace
or delete shared caches. A failed verification leaves the original trees intact.

Each unique archive has a JSON index with relative paths, original timestamps,
file hashes and the archive checksum. Preserve both. Restore needed evidence into
a separate ignored directory and verify it against that index; avoid overwriting
an active source checkout or blindly restoring symlinks over current files.

Private archives can contain old source, logs, paths and release material. They
preserve evidence without making it current project documentation. They must not
be attached to a source release or uploaded without a separate review.
