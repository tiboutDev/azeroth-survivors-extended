"""Interactive launcher choices without a game, network or installed files."""
import base64
import importlib.util
import json
import io
import sys
import tempfile
import zipfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"azsx"))
spec=importlib.util.spec_from_file_location("azsx_menu_launch",ROOT/"azsx/launch.py")
launch=importlib.util.module_from_spec(spec);spec.loader.exec_module(launch)

class MenuTests(unittest.TestCase):
    def setUp(self):
        output=redirect_stdout(io.StringIO());output.__enter__();self.addCleanup(output.__exit__,None,None,None)
    def menu(self,answers):
        with patch("builtins.input",side_effect=answers),patch.object(launch.core,"manifest",return_value={"version":"0.130.0-azsx.1","game":{"version":"0.130.0"}}):
            return launch.menu(ROOT)
    def test_maintenance_menu_defaults_to_exit_and_invalid_input_reprompts(self):
        self.assertIsNone(self.menu(["wrong",""]))
        self.assertEqual(self.menu(["1"]),"check")
    def test_destructive_choices_require_explicit_confirmation(self):
        for option,action in (("3","rollback"),("4","uninstall")):
            self.assertEqual(self.menu([option,""]),"menu")
            self.assertEqual(self.menu([option,"1"]),action)
    def test_end_of_input_exits_without_action(self):
        self.assertIsNone(self.menu([EOFError()]))
    def test_menu_check_is_offline_and_cannot_launch_game(self):
        with patch.object(launch,"run",return_value=0) as run:
            launch.execute(ROOT,"check",interactive=True)
            self.assertEqual(run.call_args.args[1:4],(False,True,False))
    def test_direct_and_menu_launch_use_automatic_updates(self):
        with patch.object(launch,"run",return_value=0) as run:
            launch.execute(ROOT,"play",interactive=True)
            self.assertIsNone(run.call_args.kwargs["confirm_update"])
            launch.execute(ROOT,"play",interactive=False)
            self.assertIsNone(run.call_args.kwargs["confirm_update"])

    def test_packaged_readme_uses_selected_game_version(self):
        spec=importlib.util.spec_from_file_location("azsx_menu_package",ROOT/"tools/build/azsx_package.py")
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
                self.assertEqual(manifest["version"],manifest["game"]["version"]+"-r1")
                self.assertEqual(archive.name,"azsx-"+manifest["version"]+".zip")
                preparation=json.loads((archive.parent/"PREPARATION.json").read_text())
                self.assertEqual(preparation["title"],manifest["game"]["version"]+" — AZSX Update 1")
                readme=z.read("README-AZSX.txt").decode()
                self.assertIn(manifest["game"]["version"]+" build",readme)
                self.assertNotIn("{GAME_VERSION}",readme)
                for name in launch.core.AZSXUI_FILES:
                    source=launch.core.AZSXUI_SOURCE+name
                    self.assertIn(source,manifest["files"])
                    self.assertEqual(z.read(source),(ROOT/"addons/AZSXUI"/name).read_bytes())
                self.assertIn("AZSXUI is included",readme)
                self.assertNotIn("eight fixed amulet gems",readme)
                import re
                self.assertEqual(set(re.findall(r"0\.\d+\.\d+",readme)),{manifest["game"]["version"]})

    def test_revision_package_signatures_and_sequence_remain_launcher_compatible(self):
        spec=importlib.util.spec_from_file_location("azsx_revision_package",ROOT/"tools/build/azsx_package.py")
        package=importlib.util.module_from_spec(spec);spec.loader.exec_module(package)
        with tempfile.TemporaryDirectory() as folder:
            base=Path(folder);runtime=base/"runtime"
            (runtime/"Lib/cryptography").mkdir(parents=True)
            (runtime/"Lib/cryptography/__init__.py").write_bytes(b"")
            (runtime/"python.exe").write_bytes(b"fixture interpreter")
            dll=base/"avrt.dll";dll.write_bytes(b"MZ fixture DLL")
            key=Ed25519PrivateKey.generate();keyfile=base/"disposable-key.pem"
            keyfile.write_bytes(key.private_bytes(serialization.Encoding.PEM,serialization.PrivateFormat.PKCS8,serialization.NoEncryption()))
            public=base64.b64encode(key.public_key().public_bytes(serialization.Encoding.Raw,serialization.PublicFormat.Raw)).decode()
            read_text=Path.read_text
            def fixture_trust(path,*args,**kwargs):
                return json.dumps({"keys":[public]}) if path==ROOT/"azsx/channel.json" else read_text(path,*args,**kwargs)
            # Trust only the disposable test key, without changing repository configuration.
            for revision,sequence in ((1,41),(2,42)):
                version="0.141.0-r"+str(revision);output=base/version
                with patch.object(Path,"read_text",fixture_trust):
                    archive=package.build(output,dll,runtime,version=version,key=keyfile,sequence=sequence)
                raw=(output/"azsx-latest.json").read_bytes();sig=(output/"azsx-latest.json.sig").read_bytes()
                game=json.loads((ROOT/"azsx/supported-games.json").read_text())["games"]["0.141.0"]
                feed=launch.updates.verify(raw,sig,[public],game["sha256"],sequence-1)
                self.assertEqual(feed["manifest"]["version"],version)
                self.assertEqual(feed["sequence"],sequence)
                self.assertEqual(feed["package"]["asset"],"azsx-"+version+".zip")
                self.assertEqual(launch.updates.verified_package(archive.read_bytes(),feed)["AZSX/App/Bin/avrt.dll"],dll.read_bytes())
                with self.assertRaisesRegex(ValueError,"Older AZSX"):
                    launch.updates.verify(raw,sig,[public],game["sha256"],sequence+1)

    def test_default_launch_does_not_open_menu(self):
        with patch.object(launch.sys,"argv",["launch.py"]),patch.object(launch,"menu",side_effect=AssertionError("menu opened")),patch.object(launch,"execute",return_value=0) as execute:
            self.assertEqual(launch.main(),0)
            self.assertEqual(execute.call_args.args[1],"play")
