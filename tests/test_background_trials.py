import importlib.util
import json
import sys
import unittest
from contextlib import redirect_stderr
from io import StringIO
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools/dev'))
spec = importlib.util.spec_from_file_location('background_desktop', ROOT / 'tools/dev/background_desktop.py')
background = importlib.util.module_from_spec(spec)
spec.loader.exec_module(background)

extension_spec = importlib.util.spec_from_file_location('extensions_trial', ROOT / 'tools/dev/extensions_trial.py')
extensions = importlib.util.module_from_spec(extension_spec)
extension_spec.loader.exec_module(extensions)


class BackgroundTrials(unittest.TestCase):
    def test_disposable_profiles_accept_the_current_exact_supported_image(self):
        # A launcher-only version bump used to pass all gates while the live
        # fixture refused to create any disposable character for that image.
        games = json.loads((ROOT / 'asx/supported-games.json').read_text())['games']
        fixture = json.loads((ROOT / 'tools/dev/fixtures/party-talents115.json').read_text())
        accepted = {fixture['sha256'], *fixture.get('compatible_sha256', [])}
        for version, game in games.items():
            with self.subTest(version=version):
                self.assertIn(game['sha256'], accepted)

    def test_empty_retired_and_incomplete_routes_fail_during_portable_parse(self):
        for argv in ([], ['--official115'], ['--tooltip-party1133'], ['--official120'], ['--asx-install'], ['--asx-install122'], ['--asx-mixed'], ['--version1133'], ['--players', '3']):
            with self.subTest(argv=argv), self.assertRaises(SystemExit), redirect_stderr(StringIO()):
                background.parse_args(argv)

    def test_supported_modes_forward_only_the_selected_125_route(self):
        cases = {
            ('--extensions-trial', '2'): ['--extensions-trial', '2'],
            ('--extensions-trial', '1', '--extensions-normal'): ['--extensions-normal', '--extensions-trial', '1'],
            ('--goblins-trial', '1'): ['--goblins-trial', '1'],
            ('--ritual-trial',): ['--ritual-trial'],
            ('--supplies-trial',): ['--supplies-trial'],
            ('--gems-trial', '2'): ['--gems-trial', '2'],
            ('--enchantments-trial', '1'): ['--enchantments-trial', '1'],
            ('--reward-wallet-trial',): ['--reward-wallet-trial'],
            ('--immunity-aggro-trial',): ['--immunity-aggro-trial'],
            ('--asx-install', '--asx-package', 'prepared.zip'): ['--asx-install', '--asx-package', 'prepared.zip'],
            ('--resource-probe',): ['--resource-probe'],
        }
        cases[('--resource-probe', '--extensions-trial', '1')] = None
        for argv, expected in cases.items():
            if expected is None:
                with self.subTest(argv=argv), self.assertRaises(SystemExit), redirect_stderr(StringIO()):
                    background.parse_args(argv)
                continue
            with self.subTest(argv=argv):
                args = background.parse_args(argv)
                self.assertEqual(background.child_flags(args), expected)

    def test_focused_wallet_assessor_failure_cannot_be_replaced_by_compat_scope(self):
        self.assertFalse(extensions.compatibility_scope(reward_wallet=True))
        checks = {'started': True}
        extensions.merge_assessor_checks(checks, {'wallet_native_readback': False})
        self.assertFalse(all(checks.values()))
        extensions.merge_assessor_checks(checks, {'wallet_native_readback': True})
        self.assertFalse(checks['wallet_native_readback'])
        for focused in ('gems', 'enchantments', 'glyph_retention', 'goblins', 'supplies', 'ritual'):
            with self.subTest(mode=focused):
                self.assertFalse(extensions.compatibility_scope(**{focused: True}))
    def test_child_dispatch_is_mocked_and_preserves_current_selection(self):
        args = background.parse_args(['--asx-install', '--asx-package', 'prepared.zip'])
        called = []
        mocked = mock.Mock(run=lambda *a, **kw: called.append((a, kw)))
        with mock.patch.dict('sys.modules', {'asx_install_trial': mocked}):
            background.run_child(args, Path('/tmp/test-results/trial/start'))
        self.assertEqual(called, [((45,), {'game_version': '0.130.0', 'package_path': Path('prepared.zip')})])

    def test_shared_wallet_dispatch_requires_two_clients_without_starting_them(self):
        args = background.parse_args(['--reward-wallet-trial'])
        runner = mock.Mock()
        with mock.patch.dict('sys.modules', {'extensions_trial': mock.Mock(run=runner)}):
            background.run_child(args, Path('unused-gate'))
        runner.assert_called_once_with(45, players=2, reward_wallet=True)
        with mock.patch.object(Path, 'read_bytes', side_effect=AssertionError('Read game assets')):
            with self.assertRaisesRegex(ValueError, 'two participants'):
                extensions.run(45, players=1, reward_wallet=True)


if __name__ == '__main__':
    unittest.main()
