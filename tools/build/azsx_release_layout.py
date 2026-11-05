"""Finalize the minimal publication folder and preserve every other file locally."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import sys
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(Path(__file__).resolve().parent))
from azsx_verify_publication import verify
from azsx_release_identity import release_title
KEEP={'UPLOAD','README-PREPARATION.md','RELEASE-NOTES.md','PUBLISH-ON-GITHUB.md'}

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def finalize(folder,evidence):
    folder,evidence=Path(folder).resolve(),Path(evidence).resolve()
    if folder==evidence or folder in evidence.parents or evidence in folder.parents:
        raise ValueError('Evidence and publication folders must be separate')
    if not folder.is_dir()or any(p.is_symlink()for p in folder.iterdir()):raise ValueError('Release links are not supported')
    pending=[p for p in folder.iterdir()if p.name not in KEEP]
    if any(not p.is_file()for p in pending):raise ValueError('Unexpected release directory; review before finalizing')
    for name in KEEP:
        if not (folder/name).exists():raise ValueError('Missing publication content: '+name)
    if evidence.exists():
        for p in pending:
            target=evidence/p.name
            if target.exists()and (target.is_symlink()or not target.is_file()or sha(target)!=sha(p)):
                raise ValueError('Evidence conflict: '+p.name)
    metadata=folder/'PUBLICATION.json' if (folder/'PUBLICATION.json').exists()else evidence/'PUBLICATION.json'
    meta=json.loads(metadata.read_text())
    title=release_title(meta['version'])
    expected={'AZSX.zip','azsx-'+meta['version']+'.zip','azsx-latest.json','azsx-latest.json.sig'}
    upload=folder/'UPLOAD'
    if not upload.is_dir()or set(meta['assets'])!=expected or {p.name for p in upload.iterdir()}!=expected or any(p.is_symlink()or not p.is_file()for p in upload.iterdir()):
        raise ValueError('Expected exactly four regular publication assets')
    # Verify signatures and payload hashes before any file is relocated.
    verify(folder,True,evidence if not (folder/'PUBLICATION.json').exists()else None)
    pending=[p for p in folder.iterdir()if p.name not in KEEP]
    evidence.mkdir(parents=True,exist_ok=True)
    inventory=json.loads((evidence/'layout-inventory.json').read_text())if(evidence/'layout-inventory.json').exists()else []
    for p in sorted(pending):
        target=evidence/p.name
        if not target.exists():shutil.copy2(p,target)
        if sha(target)!=sha(p):raise ValueError('Evidence copy verification failed')
        entry={'name':p.name,'sha256':sha(p),'size':p.stat().st_size}
        if entry not in inventory:inventory.append(entry)
    # All copies are verified before any originals are removed. Preserve the
    # complete original README/guide as evidence before updating local guidance.
    for name in ['README-PREPARATION.md','PUBLISH-ON-GITHUB.md']:
        target=evidence/('original-'+name)
        if not target.exists():shutil.copy2(folder/name,target)
    (evidence/'layout-inventory.json').write_text(json.dumps(inventory,indent=2)+'\n')
    for p in pending:p.unlink()
    meta=json.loads((evidence/'PUBLICATION.json').read_text())
    source=json.loads((evidence/'SOURCE-COMMIT.json').read_text())if(evidence/'SOURCE-COMMIT.json').exists()else{}
    validation=json.loads((evidence/'VALIDATION.json').read_text())if(evidence/'VALIDATION.json').exists()else{}
    commit=source.get('release_source_commit',source.get('commit','Not recorded'))
    (folder/'README-PREPARATION.md').write_text(f'''# {title}

AZSX version: `{meta['version']}`.
Sequence **{meta['sequence']}**. Release source: `{commit}`.
Validation: {'passed' if validation.get('technical_validation_passed') else 'see project evidence'}.
Reproducibility approval: {'approved' if validation.get('reference_approved') else 'pending'}.

- `UPLOAD/`: the four files to attach to GitHub.
- `RELEASE-NOTES.md`: player-facing release notes.
- `PUBLISH-ON-GITHUB.md`: manual publication steps.

Technical evidence is preserved in the project at `{evidence}`.
Folder organization does not push, tag or publish anything.
''',encoding='utf-8')
    (folder/'PUBLISH-ON-GITHUB.md').write_text(f'''# Manual publication

GitHub release title: **{title}**.
Prepared tag **{meta['tag']}**, sequence **{meta['sequence']}**.
Use release source `{commit}`, copy `RELEASE-NOTES.md`, and attach exactly the
four files in `UPLOAD/`. Use **tiboutDev** for authenticated Git/GitHub operations.
Publication requires separate authorization.

From the project, verify the prepared files before upload:

```sh
uv run --with cryptography python3 tools/build/azsx_verify_publication.py --folder "{folder}" --evidence-dir "{evidence}" --local
```

After publication, run the same command without `--local` to check public assets.
Technical evidence stays in the project and is not uploaded.
''',encoding='utf-8')
    return {'folder':str(folder),'evidence':str(evidence),'preserved_files':len(inventory)}

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--folder',type=Path,required=True);p.add_argument('--evidence-dir',type=Path,required=True);a=p.parse_args()
    evidence=a.evidence_dir.resolve()
    if not evidence.is_relative_to((ROOT/'.local/releases').resolve()):raise ValueError('Evidence must stay under project .local/releases')
    print(json.dumps(finalize(a.folder,evidence)))
if __name__=='__main__':main()
