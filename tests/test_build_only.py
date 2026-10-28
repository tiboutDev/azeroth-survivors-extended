import os
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
        (dev / "env.sh").write_text(":\n")
        self.bin = Path(self.temp.name) / "bin"
        self.bin.mkdir()
        self.calls = Path(self.temp.name) / "calls.log"
        cargo = self.bin / "cargo"
        cargo.write_text("#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$BUILD_TEST_LOG\"\n")
        cargo.chmod(0o755)
        python = self.bin / "python3"
        python.write_text("#!/bin/sh\nprintf 'deployment-called\\n' >> \"$BUILD_TEST_LOG\"\nexit 99\n")
        python.chmod(0o755)
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
