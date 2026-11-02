"""Run bounded reviewed-client trials on a verified private Windows desktop.

No desktop switch or OS keyboard/mouse injection. Opt-in pause trials may notify
only their own game windows of focus. A kill-on-close
job owns the gated child and descendants; trials use normal priority and all CPUs.
"""
import argparse
import ctypes
import json
from ctypes import wintypes as W
import os
from pathlib import Path
import secrets
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))
from client_registry import VERSION


def build_parser():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--seconds', type=int, default=45)
    parser.add_argument('--resource-probe', action='store_true', help='Verify normal priority and inherited CPU affinity; launches no game')
    modes = parser.add_mutually_exclusive_group()
    modes.add_argument('--extensions-trial', type=int, choices=(1, 2), metavar='PLAYERS')
    modes.add_argument('--ritual-trial', action='store_true')
    modes.add_argument('--supplies-trial', action='store_true')
    modes.add_argument('--goblins-trial', type=int, choices=(1, 2), metavar='PLAYERS')
    modes.add_argument('--gems-trial', type=int, choices=(1, 2), metavar='PLAYERS')
    modes.add_argument('--enchantments-trial', type=int, choices=(1, 2), metavar='PLAYERS')
    modes.add_argument('--reward-wallet-trial', action='store_true')
    modes.add_argument('--immunity-aggro-trial', action='store_true')
    modes.add_argument('--azsx-install', action='store_true', help='Run an explicitly selected packaged normal AZSX trial')
    parser.add_argument('--extensions-normal', action='store_true', help='Use the reviewed normal 0.129 binary for --extensions-trial')
    parser.add_argument('--legacy-azsx-package',type=Path,help='Read-only previous package for migration validation')
    parser.add_argument('--azsx-package', type=Path, help='Prepared package archive for --azsx-install')
    parser.add_argument('--child', type=Path, help=argparse.SUPPRESS)
    return parser


def parse_args(argv=None):
    parser = build_parser()
    args = parser.parse_args(argv)
    if args.seconds < 15 or args.seconds > 300:
        parser.error('Use a bounded 15..300 second trial')
    if args.extensions_normal and args.extensions_trial is None:
        parser.error('--extensions-normal requires --extensions-trial')
    if args.azsx_package and not args.azsx_install:
        parser.error('--azsx-package requires --azsx-install')
    if args.legacy_azsx_package and not args.azsx_install:parser.error("--legacy-azsx-package requires --azsx-install")
    if args.azsx_install and not args.azsx_package:
        parser.error('--azsx-install requires --azsx-package')
    game_selected = bool(args.ritual_trial or args.supplies_trial or args.goblins_trial or args.extensions_trial or args.gems_trial or args.enchantments_trial or args.reward_wallet_trial or args.immunity_aggro_trial or args.azsx_install)
    selected = args.resource_probe or game_selected
    if not selected:
        parser.error('Choose a 0.129 trial mode or --resource-probe')
    if args.resource_probe and game_selected:
        parser.error('--resource-probe must run by itself')
    return args


def child_flags(args):
    """Return the validated mode forwarded through the gated child process."""
    flags = []
    for name in ('resource_probe', 'azsx_install', 'extensions_normal', 'reward_wallet_trial', 'immunity_aggro_trial', 'supplies_trial', 'ritual_trial'):
        if getattr(args, name):
            flags.append('--' + name.replace('_', '-'))
    for name in ('extensions_trial', 'gems_trial', 'enchantments_trial', 'goblins_trial'):
        value = getattr(args, name)
        if value:
            flags.extend(('--' + name.replace('_', '-'), str(value)))
    if args.azsx_package:
        flags.extend(('--azsx-package', str(args.azsx_package)))
    if getattr(args,"legacy_azsx_package",None):flags.extend(("--legacy-azsx-package",str(args.legacy_azsx_package)))
    return flags


def run_child(args, gate):
    if args.resource_probe:
        k = ctypes.WinDLL('kernel32', use_last_error=True)
        k.GetCurrentProcess.restype = W.HANDLE
        k.GetProcessAffinityMask.argtypes = [W.HANDLE, ctypes.POINTER(ctypes.c_size_t), ctypes.POINTER(ctypes.c_size_t)]
        k.GetPriorityClass.argtypes = [W.HANDLE]
        k.GetPriorityClass.restype = W.DWORD
        actual, system = ctypes.c_size_t(), ctypes.c_size_t()
        if not k.GetProcessAffinityMask(k.GetCurrentProcess(), ctypes.byref(actual), ctypes.byref(system)):
            raise ctypes.WinError(ctypes.get_last_error())
        expected = json.loads((gate.parent / 'limits.json').read_text())
        result = dict(actual_mask=actual.value, expected_mask=expected['mask'], priority=k.GetPriorityClass(k.GetCurrentProcess()), game_launched=False)
        result['passed'] = result['actual_mask'] == result['expected_mask'] and result['priority'] == 0x20
        (gate.parent / 'resource-result.json').write_text(json.dumps(result, indent=2))
        print(json.dumps(result), flush=True)
        if not result['passed']:
            raise SystemExit(1)
        return
    sys.path.insert(0, str(ROOT / 'tools/dev'))
    if args.azsx_install:
        from azsx_install_trial import run
        run(args.seconds, game_version=VERSION, package_path=args.azsx_package,legacy_package=args.legacy_azsx_package)
        return
    from extensions_trial import run
    if args.goblins_trial:
        run(args.seconds, players=args.goblins_trial, goblins=True)
    elif args.gems_trial:
        run(args.seconds, players=args.gems_trial, gems=True)
    elif args.enchantments_trial:
        run(args.seconds, players=args.enchantments_trial, enchantments=True)
    elif args.reward_wallet_trial:
        run(args.seconds, players=2, reward_wallet=True)
    elif args.immunity_aggro_trial:
        run(args.seconds, players=2, immunity=True)
    elif args.ritual_trial:
        run(args.seconds, players=1, ritual=True)
    elif args.supplies_trial:
        run(args.seconds, players=1, supplies=True)
    elif args.extensions_trial:
        run(args.seconds, players=args.extensions_trial, normal=args.extensions_normal)


def main(argv=None):
    args = parse_args(argv)
    if os.name != 'nt':
        raise SystemExit('Windows only; never switches the interactive desktop')
    if args.child:
        gate = args.child.resolve()
        if gate.parent.parent != (ROOT / 'test-results').resolve() or gate.name != 'start':
            raise SystemExit('Invalid child gate')
        until = time.monotonic() + 15
        while not gate.exists():
            if time.monotonic() > until:
                raise SystemExit('Parent did not assign the isolation job')
            time.sleep(.05)
        from private_trial_process import current_private_desktop
        expected_desktop = json.loads((gate.parent/'limits.json').read_text())['desktop_name']
        verified_desktop = current_private_desktop(expected_desktop)
        (gate.parent/'desktop.json').write_text(json.dumps({'verified_private_desktop':True,'name':verified_desktop}),encoding='utf-8')
        run_child(args, gate)
        return

    # Local desktop only. Never request DESKTOP_SWITCHDESKTOP (0x100).
    user = ctypes.WinDLL('user32', use_last_error=True)
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    user.CreateDesktopW.argtypes = [W.LPCWSTR, W.LPCWSTR, ctypes.c_void_p, W.DWORD, W.DWORD, ctypes.c_void_p]
    user.CreateDesktopW.restype = W.HANDLE
    user.CloseDesktop.argtypes = [W.HANDLE]
    user.CloseDesktop.restype = W.BOOL
    kernel.CreateJobObjectW.argtypes = [ctypes.c_void_p, W.LPCWSTR]
    kernel.CreateJobObjectW.restype = W.HANDLE
    kernel.SetInformationJobObject.argtypes = [W.HANDLE, ctypes.c_int, ctypes.c_void_p, W.DWORD]
    kernel.SetInformationJobObject.restype = W.BOOL
    kernel.AssignProcessToJobObject.argtypes = [W.HANDLE, W.HANDLE]
    kernel.AssignProcessToJobObject.restype = W.BOOL
    kernel.GetCurrentProcess.restype=W.HANDLE
    kernel.GetProcessAffinityMask.argtypes=[W.HANDLE,ctypes.POINTER(ctypes.c_size_t),ctypes.POINTER(ctypes.c_size_t)]
    kernel.CloseHandle.argtypes = [W.HANDLE]
    kernel.CloseHandle.restype = W.BOOL

    class BASIC(ctypes.Structure):
        _fields_ = [('process_time',ctypes.c_longlong),('job_time',ctypes.c_longlong),
                    ('flags',W.DWORD),('min_ws',ctypes.c_size_t),('max_ws',ctypes.c_size_t),
                    ('process_limit',W.DWORD),('affinity',ctypes.c_size_t),
                    ('priority',W.DWORD),('scheduling',W.DWORD)]
    class IO(ctypes.Structure):
        _fields_ = [(n,ctypes.c_ulonglong) for n in ('reads','writes','others','read_bytes','write_bytes','other_bytes')]
    class EXTENDED(ctypes.Structure):
        _fields_ = [('basic',BASIC),('io',IO),('process_memory',ctypes.c_size_t),
                    ('job_memory',ctypes.c_size_t),('peak_process',ctypes.c_size_t),('peak_job',ctypes.c_size_t)]

    trial = ROOT/'test-results'/('background-'+secrets.token_hex(4))
    trial.mkdir(parents=True)
    desktop_name = 'AzCoopDev_'+secrets.token_hex(8)
    # CREATEWINDOW, CREATEMENU, READOBJECTS, WRITEOBJECTS; no input journal/hooks.
    desktop = user.CreateDesktopW(desktop_name, None, None, 0, 0x87, None)
    if not desktop:
        raise ctypes.WinError(ctypes.get_last_error())
    job = None
    child = None
    try:
        job = kernel.CreateJobObjectW(None, None)
        if not job:
            raise ctypes.WinError(ctypes.get_last_error())
        limits = EXTENDED()
        # Retain owned-process cleanup, without CPU-affinity or priority limits.
        available,system=ctypes.c_size_t(),ctypes.c_size_t()
        if not kernel.GetProcessAffinityMask(kernel.GetCurrentProcess(),ctypes.byref(available),ctypes.byref(system)):raise ctypes.WinError(ctypes.get_last_error())
        limits.basic.flags = 0x2000  # JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE only
        (trial/'limits.json').write_text(json.dumps({'mask':available.value,'restricted':False,'priority':'normal','desktop_name':desktop_name}))
        print(f'Owned test job: all {available.value.bit_count()} inherited logical CPUs, normal priority',flush=True)
        if not kernel.SetInformationJobObject(job, 9, ctypes.byref(limits), ctypes.sizeof(limits)):
            raise ctypes.WinError(ctypes.get_last_error())
        command = [sys.executable, '-I', '-B', str(Path(__file__).resolve()),
                   '--child', str(trial/'start'), '--seconds', str(args.seconds)]
        command.extend(child_flags(args))
        with (trial/'harness.log').open('wb') as stream:
            from private_trial_process import spawn
            child = spawn(command, ROOT, desktop_name, stream)
            if not kernel.AssignProcessToJobObject(job, W.HANDLE(int(child._handle))):
                raise ctypes.WinError(ctypes.get_last_error())
            (trial/'start').write_text('owned\n', encoding='ascii')
            code = child.wait(timeout=args.seconds+90)
        print(f'Private desktop trial: {trial}\nHarness exit: {code}', flush=True)
        raise SystemExit(code)
    finally:
        if job:
            kernel.CloseHandle(job)
        if child and child.poll() is None:
            child.terminate()
            child.wait(timeout=10)
        if child and hasattr(child,"close"):child.close()
        if desktop:user.CloseDesktop(desktop)


if __name__ == '__main__':
    main()
