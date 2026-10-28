"""Guard the ASX root build against reactivating the retired custom engine."""
import re
import os
import subprocess
import tempfile
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class AsxSourceBoundary(unittest.TestCase):
    def test_gitignore_keeps_runtime_source_visible_and_local_games_private(self):
        with tempfile.TemporaryDirectory() as directory:
            checkout = Path(directory)
            subprocess.run(["git", "init", "--quiet", directory], check=True)
            (checkout / ".gitignore").write_text((ROOT / ".gitignore").read_text())
            paths = ["src/runtime/session.rs", "src/runtime/bootstrap.rs",
                     "src/runtime-extra/rule.rs", "resources/windows/version.res",
                     "runtime/game.exe", "runtime-private/game.exe",
                     "test-results/report.json", ".local/evidence/archive.tar.gz",
                     "target/debug/avrt.dll", "asx/__pycache__/core.pyc"]
            result = subprocess.run(
                ["git", "-c", "core.excludesFile=" + os.devnull, "check-ignore", "--no-index", "--stdin"],
                cwd=checkout, input="\n".join(paths) + "\n", text=True,
                stdout=subprocess.PIPE, check=True,
            )
            self.assertEqual(set(result.stdout.splitlines()), set(paths[4:]))

    def test_root_compiles_the_official_runtime_without_legacy_engine_routes(self):
        source = (ROOT / "src/lib.rs").read_text()
        modules = set(re.findall(r"(?m)^\s*(?:pub\s+)?mod\s+(\w+)\s*;", source))

        required = {"proc", "stub", "install", "log", "runtime", "presentation"}
        self.assertTrue(required <= modules, f"missing ASX modules: {required - modules}")

        retired = {
            "engine", "engine_tests", "bootstrap", "peers", "ipc_client", "net",
            "native_menu", "lobby_ui", "native_coop110", "native_coop112",
            "native_coop1133", "native_coop_state", "native_class_icons110",
            "native_class_icons112", "native_class_icons1133",
        }
        self.assertFalse(retired & modules, f"legacy modules remain active: {retired & modules}")

    def test_root_dependencies_exclude_retired_custom_engine_crates(self):
        manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
        dependencies = manifest["dependencies"]
        retired = {"azcoop-bridge", "azcoop-logic", "azcoop-net", "azcoop-proto", "azcoop-world", "azcoop-patch"}
        self.assertFalse(retired & dependencies.keys(), f"legacy crates remain root dependencies: {retired & dependencies.keys()}")

    def test_workspace_contains_only_active_runtime_and_portable_gameplay(self):
        manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
        members = set(manifest["workspace"]["members"])
        self.assertEqual(
            members,
            {".", "crates/asx-game", "crates/asx-gameplay", "crates/avrt-probe"},
        )
        self.assertIn("crates/avrt-probe", members)

    def test_retired_inspection_is_absent_and_current_parsing_is_neutral(self):
        game = tomllib.loads((ROOT / "crates/asx-game/Cargo.toml").read_text())
        self.assertNotIn("legacy-inspection", game.get("features", {}))
        self.assertFalse((ROOT / "crates/asx-game/src/inspection").exists())
        for target in game["test"]:
            self.assertFalse(target["name"].startswith("inspection_enemy_"))
        for source in (ROOT / "src").rglob("*.rs"):
            text = source.read_text()
            self.assertNotIn("asx_game::readers", text, str(source))
            self.assertNotIn("asx_game::builds", text, str(source))
        selected = (ROOT / "crates/asx-game/src/official.rs").read_text()
        self.assertNotIn("crate::readers", selected)
        self.assertNotIn("crate::builds", selected)

    def test_activation_has_only_the_explicit_official_entry(self):
        source = (ROOT / "src/lib.rs").read_text()
        self.assertIn('std::env::var("AZCOOP_ENGINE").as_deref() == Ok("official")', source)
        self.assertIn("runtime::install", source)
        for legacy_entry in (
            'std::env::var_os("AZCOOP_ENGINE").is_some_and(|v| v == "1")',
            'std::env::var_os("AZCOOP_MEASURE")',
            "engine::install(",
            "bootstrap::wait(",
            "start_peer_network(",
        ):
            self.assertNotIn(legacy_entry, source)

    def test_retained_pause_renderer_has_no_legacy_bridge_dependency(self):
        renderer = (ROOT / "src/presentation/pause.rs").read_text()
        self.assertIn("asx_game::official::PauseLayout", renderer)
        self.assertNotIn("azcoop_bridge", renderer)


if __name__ == "__main__":
    unittest.main()
