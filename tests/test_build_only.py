import os
import sys
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = ("build-windows.sh", "build-chest-fixture.sh")


class BuildScriptsAreBuildOnly(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name) / "repo"
        dev = self.root / "tools/dev"
        dev.mkdir(parents=True)
        for name in SCRIPTS:
            shutil.copy2(ROOT / "tools/dev" / name, dev / name)
        for name in ("artifact_record.py", "client_registry.py"):
            shutil.copy2(ROOT / "tools/dev" / name, dev / name)
        (self.root / "asx").mkdir()
        shutil.copy2(ROOT / "asx/supported-games.json", self.root / "asx/supported-games.json")
        (dev / "env.sh").write_text(":\n")
        self.bin = Path(self.temp.name) / "bin"
        self.bin.mkdir()
        self.calls = Path(self.temp.name) / "calls.log"
        cargo = self.bin / "cargo"
        cargo.write_text("#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$BUILD_TEST_LOG\"\n" +
                         "out=target; marker=normal\nwhile [ \"$#\" -gt 0 ]; do case \"$1\" in --target-dir) out=$2; shift;; --features) marker=AZCOOP_TEST_EXTENSIONS;; esac; shift; done\n" +
                         "mkdir -p \"$out/x86_64-pc-windows-msvc/release\"\nprintf '%s' \"$marker\" > \"$out/x86_64-pc-windows-msvc/release/avrt.dll\"\n")
        cargo.chmod(0o755)
        (self.bin / "python3").symlink_to(sys.executable)
        rustc = self.bin / "rustc"
        rustc.write_text("#!/bin/sh\nprintf 'synthetic-test-compiler\\n'\n")
        rustc.chmod(0o755)
        self.env = os.environ.copy()
        self.env["PATH"] = str(self.bin) + os.pathsep + self.env["PATH"]
        self.env["BUILD_TEST_LOG"] = str(self.calls)

    def tearDown(self):
        self.temp.cleanup()

    def test_scripts_only_build_and_legacy_build_only_flag_is_safe(self):
        for name in SCRIPTS:
            for args in ([], ["--build-only"]):
                with self.subTest(script=name, args=args):
                    self.calls.write_text("") if self.calls.exists() else None
                    result = subprocess.run(["sh", str(self.root / "tools/dev" / name), *args], env=self.env, capture_output=True, text=True)
                    self.assertEqual(result.returncode, 0, result.stderr)
                    lines = self.calls.read_text().splitlines()
                    self.assertEqual(len(lines), 1)
                    self.assertIn("xwin build", lines[0])
                    receipt = list(self.root.glob("target/**/avrt.artifact.json"))
                    self.assertTrue(receipt)
                    self.assertTrue(all(json.loads(p.read_text())["schema"] == 2 for p in receipt))
                    self.assertFalse(list(self.root.glob("runtime-*")))
                    self.assertFalse(any("deployment-called" in line for line in lines))

    def test_unknown_arguments_fail_before_build_or_deployment(self):
        for name in SCRIPTS:
            with self.subTest(script=name):
                self.calls.unlink(missing_ok=True)
                result = subprocess.run(["sh", str(self.root / "tools/dev" / name), "--surprise"], env=self.env, capture_output=True, text=True)
                self.assertEqual(result.returncode, 2)
                self.assertIn("Unknown argument", result.stderr)
                self.assertFalse(self.calls.exists())


if __name__ == "__main__":
    unittest.main()
