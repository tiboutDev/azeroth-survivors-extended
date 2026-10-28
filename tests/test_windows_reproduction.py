"""Keep the native Windows rebuild gate tied to its own reviewed manifest."""
import json
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
REPRODUCE = ROOT / "scripts/reproduce.ps1"
RELEASE = ROOT / "tools/build/release.ps1"
POWERSHELL = shutil.which("pwsh") or shutil.which("powershell.exe")


class WindowsReproduction(unittest.TestCase):
    def test_gate_checks_native_manifest_before_starting_builder(self):
        source = REPRODUCE.read_text(encoding="utf-8-sig")
        missing_check = source.index("if (-not (Test-Path -LiteralPath $ExpectedManifest")
        parse = source.index("ConvertFrom-Json")
        validation = source.index("Invalid reviewed Windows-native ASX reference.")
        build = source.index("reproducible native Windows ASX build")
        launch = source.index("$builder = Join-Path")
        self.assertLess(missing_check, parse)
        self.assertLess(parse, validation)
        self.assertLess(validation, build)
        self.assertLess(build, launch)
        self.assertIn("$expected.builder -ne 'windows-msvc'", source)
        self.assertNotIn("asx-reproducible-build-xwin.json", source)

    def test_native_builder_uses_explicit_target_and_encoded_path_flags(self):
        source = RELEASE.read_text(encoding="utf-8-sig")
        self.assertIn("--target x86_64-pc-windows-msvc --target-dir $targetRoot", source)
        self.assertIn("$env:CARGO_ENCODED_RUSTFLAGS", source)
        self.assertIn("[char]31", source)
        self.assertIn("$targetRoot 'x86_64-pc-windows-msvc\\release\\avrt.dll'", source)
        self.assertIn("[Text.Encoding]::Unicode.GetString($bytes)", source)

    @unittest.skipUnless(POWERSHELL, "Native PowerShell is available only on Windows")
    def test_missing_and_invalid_references_fail_before_cargo(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            cargo_log = directory / "cargo-called.txt"
            bin_dir = directory / "bin"
            bin_dir.mkdir()
            fake_cargo = bin_dir / "cargo.cmd"
            fake_cargo.write_text(
                '@echo called>"%CARGO_REPRO_TEST_LOG%"\r\nexit /b 91\r\n',
                encoding="ascii",
            )
            env = os.environ.copy()
            env["PATH"] = str(bin_dir) + os.pathsep + env.get("PATH", "")
            env["CARGO_REPRO_TEST_LOG"] = str(cargo_log)

            missing = directory / "missing.json"
            invalid = directory / "invalid.json"
            invalid.write_text(json.dumps({
                "schema": 1,
                "game_version": "0.122.0",
                "artifact": "avrt.dll",
                "builder": "xwin-msvc",
                "sha256": "0" * 64,
            }), encoding="utf-8")
            for reference, expected_message in (
                (missing, "No reviewed Windows-native ASX reference."),
                (invalid, "Invalid reviewed Windows-native ASX reference."),
            ):
                with self.subTest(reference=reference.name):
                    result = subprocess.run(
                        [POWERSHELL, "-NoProfile", "-File", str(REPRODUCE),
                         "-ExpectedManifest", str(reference)],
                        cwd=ROOT, env=env, capture_output=True, text=True,
                        errors="replace", timeout=30,
                    )
                    if os.name != "nt" and ("UnauthorizedAccess" in result.stdout + result.stderr or
                            "UtilBindVsockAnyPort" in result.stdout + result.stderr):
                        self.skipTest("Windows PowerShell is blocked by WSL path interop/policy")
                    self.assertNotEqual(result.returncode, 0, result.stdout)
                    self.assertIn(expected_message, result.stdout + result.stderr)
                    self.assertFalse(cargo_log.exists(), result.stdout)

    @unittest.skipUnless(POWERSHELL, "Native PowerShell is available only on Windows")
    def test_reproducer_checks_artifact_in_target_dir_even_when_its_path_has_spaces(self):
        fixture_dll = b"deterministic test fixture\r\n"
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            bin_dir = directory / "fake toolchain"
            bin_dir.mkdir()
            target_dir = directory / "separate target cache"
            cargo_log = directory / "cargo flags.txt"
            fake_cargo = bin_dir / "cargo.cmd"
            fake_cargo.write_text(
                '@echo off\r\n'
                'set "target=%~7"\r\n'
                'if not exist "%target%\\x86_64-pc-windows-msvc\\release" mkdir "%target%\\x86_64-pc-windows-msvc\\release"\r\n'
                'echo deterministic test fixture>"%target%\\x86_64-pc-windows-msvc\\release\\avrt.dll"\r\n'
                'echo %CARGO_ENCODED_RUSTFLAGS%>"%ASX_REPRO_TEST_FLAGS%"\r\n'
                'exit /b 0\r\n', encoding="ascii",
            )
            fake_rustc = bin_dir / "rustc.cmd"
            fake_rustc.write_text('@echo C:\\Fake Rust Toolchain\\sysroot\r\n', encoding="ascii")
            expected = directory / "native-reference.json"
            expected.write_text(json.dumps({
                "schema": 1,
                "game_version": "0.122.0",
                "artifact": "avrt.dll",
                "builder": "windows-msvc",
                "sha256": hashlib.sha256(fixture_dll).hexdigest(),
            }), encoding="utf-8")
            env = os.environ.copy()
            env["PATH"] = str(bin_dir) + os.pathsep + env.get("PATH", "")
            env["CARGO_TARGET_DIR"] = str(target_dir)
            env["CARGO_ENCODED_RUSTFLAGS"] = "unreviewed inherited override"
            env["ASX_REPRO_TEST_FLAGS"] = str(cargo_log)
            result = subprocess.run(
                [POWERSHELL, "-NoProfile", "-File", str(REPRODUCE),
                 "-ExpectedManifest", str(expected)],
                cwd=ROOT, env=env, capture_output=True, text=True,
                errors="replace", timeout=30,
            )
            if os.name != "nt" and ("UnauthorizedAccess" in result.stdout + result.stderr or
                    "UtilBindVsockAnyPort" in result.stdout + result.stderr):
                self.skipTest("Windows PowerShell is blocked by WSL path interop/policy")
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertEqual(
                (target_dir / "x86_64-pc-windows-msvc" / "release" / "avrt.dll").read_bytes(),
                fixture_dll,
            )
            self.assertTrue(cargo_log.is_file())
            flags = cargo_log.read_text(encoding="ascii").rstrip("\r\n").split(chr(31))
            self.assertIn("-Cdebuginfo=0", flags)
            self.assertIn("-Clink-arg=/TIMESTAMP:1", flags)
            self.assertTrue(any(value.startswith("--remap-path-prefix=C:\\Fake Rust Toolchain\\sysroot=") for value in flags))
            self.assertNotIn("unreviewed inherited override", flags)


if __name__ == "__main__":
    unittest.main()
