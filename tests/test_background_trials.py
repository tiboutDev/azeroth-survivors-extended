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
    def test_weapon_catalog_requires_every_reviewed_new_display(self):
        displays=[24756,26494,25036,23825,6798,25133,23836,32782,29171]
        def log(rows,count=None):
            return f"Weapon display catalog ({len(rows) if count is None else count}) {rows}"
        self.assertTrue(extensions.weapon_catalog_available(log(displays+[29698])))
        for display in displays:
            self.assertFalse(extensions.weapon_catalog_available(log([v for v in displays if v!=display])))
        self.assertFalse(extensions.weapon_catalog_available(log(displays,41)))
        self.assertFalse(extensions.weapon_catalog_available("Weapon display catalog (50)"))

    def test_private_pause_movement_pairs_each_key_with_its_release(self):
        import private_trial_focus as focus
        with mock.patch.object(focus, "_post_owned_windows", return_value=2) as post:
            for tick in range(8):
                self.assertEqual(focus.step_owned_windows([101, 202], tick), 2)
        keys = []
        for args, _ in post.call_args_list:
            self.assertEqual(args[0], [101, 202])
            keys.extend(row[:2] for row in args[1] if row[0] in (0x100, 0x101))
        self.assertEqual(keys, [(0x100, 0x57), (0x101, 0x57), (0x100, 0x53), (0x101, 0x53)])

    def test_ritual_evidence_only_satisfies_the_ritual(self):
        host = "scenario step: ritual action=0\nscenario step: ritual action=1"
        assess = extensions.observed_event_steps
        self.assertEqual(assess(host, 1, False, False, True, False), {"observed_ritual_steps": True})
        self.assertEqual(assess(host, 1, True, True, True, False), {
            "observed_goblin_steps": False, "observed_supply_steps": False, "observed_ritual_steps": True})
        self.assertEqual(assess(host, 2, True, False, False, True), {
            "observed_goblin_steps": False, "observed_coop_steps": False})

    def test_disposable_profiles_accept_the_current_exact_supported_image(self):
        # A launcher-only version bump used to pass all gates while the live
        # fixture refused to create any disposable character for that image.
        games = json.loads((ROOT / 'azsx/supported-games.json').read_text())['games']
        fixture = json.loads((ROOT / 'tools/dev/fixtures/party-talents115.json').read_text())
        accepted = {fixture['sha256'], *fixture.get('compatible_sha256', [])}
        for version, game in games.items():
            with self.subTest(version=version):
                self.assertIn(game['sha256'], accepted)

    def test_empty_retired_and_incomplete_routes_fail_during_portable_parse(self):
        for argv in ([], ['--official115'], ['--tooltip-party1133'], ['--official120'], ['--azsx-install'], ['--azsx-install122'], ['--azsx-mixed'], ['--version1133'], ['--players', '3']):
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
            ('--azsx-install', '--azsx-package', 'prepared.zip'): ['--azsx-install', '--azsx-package', 'prepared.zip'],
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

    def test_only_exact_info_goblin_policy_refusal_is_expected(self):
        line="[ INFO ]  142.156s treasure goblin taunt rejected: enemy=17379661029583094025 spell=23922"
        self.assertTrue(extensions.adapter_log_clean(line))
        for changed in (line.replace("[ INFO ]", "[ ERROR ]"),
                        line.replace("[ INFO ]", "[ WARN ]"),
                        line + " unexpected adapter failure",
                        "[ INFO ]  1.000s Native heal rejected: bad frame",
                        line + "\n[ ERROR ] Native read failed",
                        "Mod HUD update failed"):
            self.assertFalse(extensions.adapter_log_clean(changed))

    def test_focused_wallet_assessor_failure_cannot_be_replaced_by_compat_scope(self):
        self.assertFalse(extensions.compatibility_scope(reward_wallet=True))
        self.assertFalse(extensions.compatibility_scope(diagnostic_probe=True))
        checks = {'started': True}
        extensions.merge_assessor_checks(checks, {'wallet_native_readback': False})
        self.assertFalse(all(checks.values()))
        extensions.merge_assessor_checks(checks, {'wallet_native_readback': True})
        self.assertFalse(checks['wallet_native_readback'])
        for focused in ('gems', 'enchantments', 'goblins', 'supplies', 'ritual'):
            with self.subTest(mode=focused):
                self.assertFalse(extensions.compatibility_scope(**{focused: True}))
    def test_child_dispatch_is_mocked_and_preserves_current_selection(self):
        args = background.parse_args(['--azsx-install', '--azsx-package', 'prepared.zip'])
        called = []
        mocked = mock.Mock(run=lambda *a, **kw: called.append((a, kw)))
        with mock.patch.dict('sys.modules', {'azsx_install_trial': mocked}):
            background.run_child(args, Path('/tmp/test-results/trial/start'))
        self.assertEqual(called, [((45,), {'game_version': '0.141.0', 'package_path': Path('prepared.zip'), 'legacy_package': None})])

    def test_archive_preflight_distinguishes_pending_owned_dll_replacement(self):
        from azsx_install_trial import initial_check_passed
        unchanged=("profile", "settings")
        for old,new,expected in ((None,"new",1),("same","same",0),("old","new",1)):
            with self.subTest(old=old,new=new):
                self.assertTrue(initial_check_passed(expected,unchanged,unchanged,old,old,new))
                self.assertFalse(initial_check_passed(1-expected,unchanged,unchanged,old,old,new))
                self.assertFalse(initial_check_passed(expected,unchanged,("changed","settings"),old,old,new))
                self.assertFalse(initial_check_passed(expected,unchanged,unchanged,"unexpected",old,new))
                self.assertFalse(initial_check_passed(2,unchanged,unchanged,old,old,new))

    def test_packaged_coop_uses_native_ready_input_without_restarting_autostart(self):
        import azsx_install_trial
        host = azsx_install_trial.native_coop_controls(1)
        guest = azsx_install_trial.native_coop_controls(2)
        self.assertEqual(host["WOW_ROGUE_TAPS"].split(";"), ["410,485"] * 32)
        self.assertEqual(guest["WOW_ROGUE_TAPS"].split(";"), ["306,525"] * 32)
        for controls in (host, guest):
            self.assertEqual(controls["WOW_ROGUE_AUTOSTART"], "1")
            self.assertEqual(controls["WOW_ROGUE_ROAM"], "1")
            self.assertEqual(controls["AZSX_LOG_NATIVE_UI"], "1")
        for slot in (0, 3, True, 1.0):
            with self.subTest(slot=slot), self.assertRaises(ValueError):
                azsx_install_trial.native_coop_controls(slot)

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

class PackageVersionTests(unittest.TestCase):
    def test_revision_and_legacy_names_require_the_exact_native_version(self):
        from azsx_install_trial import package_version_matches
        for version in ("0.141.0-r1","0.141.0-r2","0.141.0-azsx.1"):
            self.assertTrue(package_version_matches(version,"0.141.0"))
        for version in ("0.140.0-r1","0.141.0-r0","0.141.0-r1-extra","0.141.0-azsx.",None):
            self.assertFalse(package_version_matches(version,"0.141.0"))
