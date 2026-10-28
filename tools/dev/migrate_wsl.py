"""Copy this clean development project to Linux; retain the Windows workspace.

Run under WSL. Copies tracked sources, independent Git metadata and tool caches.
Game runtimes stay on Windows. No deletion, hardlinked Git objects or global
shell/toolchain configuration changes are performed.
"""
import argparse
import json
import os
from pathlib import Path
import secrets
import shutil
import subprocess
import time

ROOT=Path(__file__).resolve().parents[2]


def run(*args,**kwargs):
    return subprocess.run(args,check=True,**kwargs)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--destination',type=Path,default=Path.home()/'dev/azeroth-survivors-coop')
    args=parser.parse_args()
    destination=args.destination.expanduser().absolute()
    if not os.environ.get('WSL_DISTRO_NAME') or not ROOT.as_posix().startswith('/mnt/'):
        raise SystemExit('Run this migration under WSL from the existing Windows development workspace.')
    if destination.exists():raise SystemExit('Destination already exists; refusing to overwrite it.')
    if not destination.parent.is_dir() or destination.as_posix().startswith('/mnt/'):
        raise SystemExit('Choose a new directory on the Linux filesystem with an existing parent.')
    if subprocess.check_output(['git','status','--porcelain'],cwd=ROOT):raise SystemExit('Commit or preserve source changes before migrating.')
    if not (ROOT/'.git').is_dir() or (ROOT/'.git/objects/info/alternates').exists():raise SystemExit('Independent Git checkout required.')
    tools=ROOT.parent.parent/'work/rust-tools'
    if not all((tools/name).is_dir() for name in ('cargo','rustup','xwin')):raise SystemExit('Build tools are missing.')
    if not (ROOT/'runtime-01100/IMPORT-01100.json').is_file():raise SystemExit('Private Windows runtime is missing.')
    if (ROOT/'.wsl-source.json').exists():raise SystemExit('This workspace is already paired with a WSL source checkout.')
    started=time.monotonic();destination.mkdir()
    print('Copying tracked sources and independent Git history…',flush=True)
    files=subprocess.check_output(['git','ls-files','-z'],cwd=ROOT)
    run('rsync','-a','--from0','--files-from=-',str(ROOT)+'/',str(destination)+'/',input=files)
    run('rsync','-a',str(ROOT/'.git')+'/',str(destination/'.git')+'/')
    # NTFS may report every source as executable. Restore Git's file modes.
    records=subprocess.check_output(['git','ls-files','-s','-z'],cwd=ROOT)
    for record in records.split(b'\0'):
        if not record:continue
        metadata,name=record.split(b'\t',1);mode=metadata.split()[0]
        if mode in (b'100644',b'100755'):os.chmod(destination/os.fsdecode(name),0o755 if mode==b'100755' else 0o644)
    run('git','config','--local','core.filemode','true',cwd=destination)
    run('git','config','--local','core.ignorecase','false',cwd=destination)
    print('Copying Rust compiler, dependency cache and Windows SDK to Linux…',flush=True)
    (destination/'.dev-tools').mkdir()
    run('rsync','-a','--stats',str(tools)+'/',str(destination/'.dev-tools')+'/')
    run('python3',str(destination/'tools/dev/prepare_wsl_sdk.py'),cwd=destination)
    # Historical analysis still reads the private Windows game images. These
    # links do not copy MPQs or saves and are excluded from Git/deployment.
    for runtime in sorted(ROOT.glob('runtime*')):
        if runtime.is_dir() and (runtime.name=='runtime' or runtime.name.startswith('runtime-')):
            (destination/runtime.name).symlink_to(runtime,target_is_directory=True)
    evidence=destination/'test-results';evidence.mkdir()
    for path in (ROOT/'test-results').glob('*.json'):
        if path.is_file():shutil.copy2(path,evidence/path.name)
    identity=secrets.token_hex(16)
    config=dict(version=1,id=identity,source=str(destination),windows_workspace=str(ROOT),distribution=os.environ['WSL_DISTRO_NAME'])
    (destination/'.dev-local.json').write_text(json.dumps(config,indent=2)+'\n')
    # Windows entry points use the pairing only after the copy has completed.
    before=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT)
    after=subprocess.check_output(['git','rev-parse','HEAD'],cwd=destination)
    if before!=after or subprocess.check_output(['git','status','--porcelain'],cwd=destination):raise SystemExit('Post-copy Git verification failed; original retained.')
    (ROOT/'.wsl-source.json').write_text(json.dumps(config,indent=2)+'\n')
    proof=dict(source_commit=before.decode().strip(),source_files=len(files.split(b'\0'))-1,elapsed_seconds=round(time.monotonic()-started,2),windows_game_copied=False,original_retained=True)
    (evidence/'wsl-migration.json').write_text(json.dumps(proof,indent=2)+'\n')
    print('Migration copy verified: '+str(destination),flush=True)


if __name__=='__main__':main()
