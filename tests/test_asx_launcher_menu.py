"""Interactive launcher choices without a game, network or installed files."""
import importlib.util
import io
import sys
import tempfile
import zipfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"asx"))
spec=importlib.util.spec_from_file_location("asx_menu_launch",ROOT/"asx/launch.py")
launch=importlib.util.module_from_spec(spec);spec.loader.exec_module(launch)

class MenuTests(unittest.TestCase):
    def setUp(self):
        output=redirect_stdout(io.StringIO());output.__enter__();self.addCleanup(output.__exit__,None,None,None)
    def menu(self,answers):
        with patch("builtins.input",side_effect=answers),patch.object(launch.core,"manifest",return_value={"version":"0.130.0-asx.1","game":{"version":"0.130.0"}}):
            return launch.menu(ROOT)
    def test_enter_plays_and_invalid_input_reprompts(self):
        self.assertEqual(self.menu(["wrong",""]),"play")
    def test_destructive_choices_require_explicit_confirmation(self):
        for option,action in (("6","rollback"),("7","uninstall")):
            self.assertEqual(self.menu([option,""]),"menu")
            self.assertEqual(self.menu([option,"1"]),action)
    def test_end_of_input_exits_without_action(self):
        self.assertIsNone(self.menu([EOFError()]))
    def test_update_prompt_offers_accept_defer_and_cancel(self):
        feed={"manifest":{"version":"0.130.0-asx.2"},"package":{"size":1024}}
        for answer,expected in (("1",True),("",False),("2",False)):
            with patch.object(launch.sys.stdin,"isatty",return_value=True),patch("builtins.input",return_value=answer):self.assertEqual(launch.confirm_update(feed),expected)
        with patch.object(launch.sys.stdin,"isatty",return_value=True),patch("builtins.input",return_value="0"),self.assertRaises(launch.UpdateCancelled):launch.confirm_update(feed)
    def test_menu_check_is_offline_and_cannot_launch_game(self):
        with patch.object(launch,"run",return_value=0) as run:
            launch.execute(ROOT,"check",interactive=True)
            self.assertEqual(run.call_args.args[1:4],(False,True,False))
    def test_direct_and_menu_launch_both_require_update_confirmation(self):
        with patch.object(launch,"run",return_value=0) as run:
            launch.execute(ROOT,"play",interactive=True)
            self.assertIs(run.call_args.kwargs["confirm_update"],launch.confirm_update)
            launch.execute(ROOT,"play",interactive=False)
            self.assertIs(run.call_args.kwargs["confirm_update"],launch.confirm_update)

    def test_packaged_readme_uses_selected_game_version(self):
        spec=importlib.util.spec_from_file_location("asx_menu_package",ROOT/"tools/build/asx_package.py")
        package=importlib.util.module_from_spec(spec);spec.loader.exec_module(package)
        with tempfile.TemporaryDirectory() as folder:
            base=Path(folder);runtime=base/"runtime"
            (runtime/"Lib/cryptography").mkdir(parents=True)
            (runtime/"Lib/cryptography/__init__.py").write_bytes(b"")
            (runtime/"python.exe").write_bytes(b"fixture interpreter")
            dll=base/"avrt.dll";dll.write_bytes(b"MZ fixture DLL")
            archive=package.build(base/"release",dll,runtime)
            with zipfile.ZipFile(archive) as z:
                manifest=launch.core.read_json(z.read(launch.core.MANIFEST))
                readme=z.read("README-ASX.txt").decode()
                self.assertIn(manifest["game"]["version"]+" build",readme)
                self.assertNotIn("{GAME_VERSION}",readme)
                for name in launch.core.ASXUI_FILES:
                    source=launch.core.ASXUI_SOURCE+name
                    self.assertIn(source,manifest["files"])
                    self.assertEqual(z.read(source),(ROOT/"addons/ASXUI"/name).read_bytes())
                self.assertIn("ASXUI is included",readme)
                self.assertNotIn("eight fixed amulet gems",readme)
                import re
                self.assertEqual(set(re.findall(r"0\.\d+\.\d+",readme)),{manifest["game"]["version"]})

    def test_default_launch_does_not_open_menu(self):
        with patch.object(launch.sys,"argv",["launch.py"]),patch.object(launch,"menu",side_effect=AssertionError("menu opened")),patch.object(launch,"execute",return_value=0) as execute:
            self.assertEqual(launch.main(),0)
            self.assertEqual(execute.call_args.args[1],"play")
    def test_noninteractive_update_is_deferred(self):
        with patch.object(launch.sys.stdin,"isatty",return_value=False),patch("builtins.input",side_effect=AssertionError("input without console")):
            self.assertFalse(launch.confirm_update({}))
