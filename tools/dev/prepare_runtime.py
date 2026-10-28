"""Prepare a private runtime from an existing installation, without copying player data.

Run with Linux Python under WSL, then use Install-Build.py after cross-compilation.
The source installation is read-only. All destinations are beneath this checkout.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil

PROJECT = Path(__file__).resolve().parents[2]
RUNTIME = PROJECT / 'runtime'


def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for data in iter(lambda: f.read(8 << 20), b''):
            h.update(data)
    return h.hexdigest()


def prepare(source):
    source = Path(source).resolve()
    if RUNTIME.exists():
        raise SystemExit('runtime already exists; it is never overwritten automatically')
    if source == RUNTIME or source in RUNTIME.parents:
        raise SystemExit('source and private runtime must be separate')
    RUNTIME.mkdir()
    copied = {}

    def copy(rel):
        src, dst = source / rel, RUNTIME / rel
        if not src.is_file():
            raise FileNotFoundError(src)
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dst)
        digest = sha(src)
        if digest != sha(dst):
            raise RuntimeError(f'copy verification failed: {rel}')
        copied[str(rel).replace('\\', '/')] = digest

    # Only code, runtime dependencies, game assets and public compatibility data.
    # No .coop-rooms, logs, configuration, certificates or personal saves.
    for rel in ['coop_launcher.py', 'COOP-VERSION.txt', 'AzerothSurvivors.exe', 'coop-update/channels.json']:
        copy(Path(rel))
    for folder in ['reconstruction', 'dllshim', 'relay']:
        for path in (source / folder).rglob('*'):
            if path.is_file() and '__pycache__' not in path.parts and path.suffix in {'.py', '.json', '.html', '.txt'}:
                copy(path.relative_to(source))
    for folder in ['.coop-runtime', 'native/Data', 'native/reconstruction']:
        for path in (source / folder).rglob('*'):
            if path.is_file() and '__pycache__' not in path.parts and path.suffix != '.pyc':
                copy(path.relative_to(source))
    for rel in ['native/AzerothSurvivors.exe', 'native/avrt.dll', 'native/azcoop-net.exe',
                'native/channel-manifest.json', 'native/channel-game.json']:
        copy(Path(rel))
    (RUNTIME / 'analysis').mkdir(exist_ok=True)
    # Source modifications are installed separately; do not claim upstream binaries contain our fixes.
    shutil.copy2(PROJECT / 'dllshim/dll_shim.py', RUNTIME / 'dllshim/dll_shim.py')
    (RUNTIME / 'coop-update/no-auto-update').touch()
    (RUNTIME / 'coop-lab').mkdir(exist_ok=True)
    (RUNTIME / 'coop-lab/save-mode.txt').write_text('separate\n')
    (RUNTIME / 'coop-region.txt').write_text('lan\n')
    (PROJECT / 'original-files.json').write_text(json.dumps(copied, indent=2))
    # The launcher has a distinct private profile and a fresh UI port. Automatic updates stay off.
    launcher = '@echo off\r\nsetlocal\r\ncd /d "%~dp0"\r\n' \
        'if not exist "DEV-BUILD-READY" (echo Build the development DLL first. & pause & exit /b 1)\r\n' \
        'set "COOP_AUTO_UPDATE=0"\r\nset "COOP_UPDATE_CHECKED=1"\r\n' \
        'set "COOP_PROFILE_FOLDER=coop-lab/personal-native"\r\n' \
        '"%~dp0.coop-runtime\\python.exe" -I -B "%~dp0coop_launcher.py" --region lan %*\r\n' \
        'if errorlevel 1 pause\r\n'
    (RUNTIME / 'Start-Coop-Dev.cmd').write_bytes(launcher.encode('ascii'))
    print(f'Private runtime prepared: {RUNTIME}; {len(copied)} copied files verified')


def verify(source):
    original = json.loads((PROJECT / 'original-files.json').read_text())
    changed = [rel for rel, digest in original.items() if sha(Path(source) / rel) != digest]
    if changed:
        raise SystemExit('Source files changed since copy: ' + ', '.join(changed))
    print(f'Original installation unchanged: {len(original)} SHA-256 checks passed')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('source', type=Path)
    parser.add_argument('--verify-original', action='store_true')
    args = parser.parse_args()
    (verify if args.verify_original else prepare)(args.source)
