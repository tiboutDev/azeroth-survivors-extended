"""Current ASX scope selection must cover real targets and never launch a game."""
import importlib.util
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('test_plan', ROOT / 'tools/dev/test_plan.py')
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)


class Plans(unittest.TestCase):
    def test_unknown_executable_and_registry_changes_require_full(self):
        for name in ('src/new_native.rs', 'Cargo.toml', 'Cargo.lock',
                     'asx/supported-games.json', 'assets/lobby.png',
                     'validation/runtime-profile.json',
                     'crates/asx-game/src/bindings/official125.rs',
                     'dllshim/removed_adapter.py'):
            with self.subTest(path=name):
                self.assertEqual(m.choose([name]), ['full'])

    def test_docs_only_do_not_start_checks(self):
        self.assertEqual(m.plan(None, ['AGENTS.md', 'docs/TESTING.md'])['commands'], [])

    def test_current_native_and_retained_presentation_paths_keep_official_checks(self):
        for name in ('crates/asx-game/src/official_native/support.rs',
                     'crates/asx-game/src/official_native/pause.rs',
                     'crates/asx-game/src/official.rs',
                     'src/runtime.rs', 'src/presentation/pause.rs'):
            with self.subTest(path=name):
                plan = m.plan(None, [name])
                self.assertEqual(plan['profiles'], ['official'])
                self.assertIn(m.SOURCE, plan['commands'])
                self.assertIn(m.NATIVE, plan['commands'])
                self.assertTrue(any('official_native::' in c for c in plan['commands']))

    def test_rules_and_feature_scopes_cover_current_contracts(self):
        for name, scope in (
            ('crates/asx-gameplay/src/support.rs', 'rules'),
            ('src/runtime/gems.rs', 'gems'),
            ('src/runtime/enchantments.rs', 'enchantments'),
            ('src/runtime/party_ui.rs', 'official-party-ui'),
            ('asx/core.py', 'asx-install'),
            ('tools/build/asx_verify_publication.py', 'asx-install'),
            ('tools/dev/native_profile_codec.py', 'tooling'),
            ('tools/dev/background_desktop.py', 'tooling'),
        ):
            with self.subTest(path=name):
                self.assertEqual(m.choose([name]), [scope])

    def test_scopes_merge_deduplicate_and_cannot_weaken_full(self):
        plan = m.plan(['official', 'rules'], [])
        commands = plan['commands']
        self.assertEqual(commands.count(m.ARCH), 1)
        self.assertEqual(len(commands), len({tuple(c) for c in commands}))
        self.assertEqual(m.plan(['official', 'full'], [])['profiles'], ['full'])
        self.assertFalse(any('--features' in command and 'legacy-inspection' in command
                            for command in m.PROFILES['full']))

    def test_all_portable_cargo_targets_exist_and_exclude_retired_crates(self):
        portable = {'asx-game': 'asx-game', 'asx-gameplay': 'asx-gameplay'}
        for profile, commands in m.PROFILES.items():
            for command in commands:
                if command[0] != 'cargo':
                    continue
                self.assertIn('--locked', command)
                packages = [command[i + 1] for i, token in enumerate(command[:-1]) if token == '-p']
                self.assertTrue(packages)
                self.assertTrue(set(packages) <= portable.keys(), (profile, packages))
                for i, token in enumerate(command[:-1]):
                    if token == '--test':
                        target = command[i + 1]
                        declared = set()
                        for package in packages:
                            manifest_path = ROOT / 'crates' / portable[package] / 'Cargo.toml'
                            manifest = tomllib.loads(manifest_path.read_text())
                            if manifest.get('package', {}).get('autotests', True):
                                for inferred in (manifest_path.parent / 'tests' / (target + '.rs'),
                                                 manifest_path.parent / 'tests' / target / 'main.rs'):
                                    if inferred.is_file():
                                        declared.add(target)
                            for entry in manifest.get('test', []):
                                if (manifest_path.parent / entry['path']).is_file():
                                    declared.add(entry['name'])
                        self.assertIn(target, declared, (profile, target))

    def test_all_declared_python_targets_exist(self):
        for profile, commands in m.PROFILES.items():
            for command in commands:
                if 'unittest' in command:
                    pattern = command[command.index('-p') + 1]
                    self.assertTrue(list((ROOT / 'tests').glob(pattern)), (profile, pattern))
        self.assertIn(m.SOURCE, m.PROFILES['tooling'])
        self.assertIn(m.NATIVE, m.PROFILES['official-party-ui'])

    def test_checks_never_build_deploy_or_launch_games(self):
        for name in m.PROFILES:
            plan = m.plan([name], [])
            self.assertFalse(plan['dll_built'])
            self.assertFalse(plan['live_game_launched'])
            for command in plan['commands']:
                self.assertNotIn('build', command)
                self.assertNotIn('background_desktop.py', ' '.join(command))
                self.assertNotIn('live_smoke.py', ' '.join(command))


if __name__ == '__main__':
    unittest.main()
