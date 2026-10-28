"""Keep current gameplay policy portable and native translation version-bound."""
import tomllib
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class GameplayBoundary(unittest.TestCase):
    def test_gameplay_policy_has_no_native_or_transport_dependencies(self):
        manifest = tomllib.loads((ROOT / "crates/asx-gameplay/Cargo.toml").read_text())
        self.assertFalse(manifest.get("dependencies", {}))

        for source in (ROOT / "crates/asx-gameplay/src").rglob("*.rs"):
            text = source.read_text()
            for forbidden in (
                "asx_game::", "azcoop_world::", "windows_sys::", "minhook::",
                "crate::Regs", "dyn Client", 'extern "system"', "unsafe {",
                "layout(", "registers(", "SHA1133", "SHA112",
            ):
                self.assertNotIn(forbidden, text, str(source))

    def test_native_layout_selection_stays_behind_the_official_facade(self):
        native = (ROOT / "crates/asx-game/src/official_native.rs").read_text()
        selection = native[native.index("pub fn select("):native.index("pub fn game_address(")]
        self.assertIn("crate::official125::SHA256=>Revision::V125", selection)
        self.assertIn("_=>return Err", selection)
        self.assertNotIn("Revision::V115=>", selection)

        facade = (ROOT / "crates/asx-game/src/official.rs").read_text()
        self.assertIn("crate::official_native::Adapter::select(hash)?", facade)
        self.assertIn("pub use crate::official_native::pause::PauseLayout", facade)
        self.assertIn("self.native.pause_addresses()", facade)

    def test_pause_renderer_receives_layout_from_selected_official_runtime(self):
        renderer = (ROOT / "src/presentation/pause.rs").read_text().split("#[cfg(test)]", 1)[0]
        self.assertIn("use asx_game::official::PauseLayout", renderer)
        self.assertIn("pub fn install_selected(base:usize,layout:PauseLayout)", renderer)
        self.assertNotIn("azcoop_bridge", renderer)
        self.assertNotRegex(renderer, r"SHA(?:112|1133)|native(?:112|1133)::")

        runtime = (ROOT / "src/runtime/bootstrap.rs").read_text()
        self.assertIn("adapter.pause_layout()", runtime)

    def test_jewelry_callbacks_use_bounded_services(self):
        for name in ("gems", "enchantments"):
            source = (ROOT / f"src/runtime/{name}.rs").read_text()
            # Diagnostic fixtures remain native integration clients; production
            # choice/acceptance/rebuild coordination cannot reach their runtime.
            callbacks = source[source.index("pub fn card("):source.index("pub fn annotation(")]
            for forbidden in ("Runtime", "RUNTIME", "io::", ".adapter", ".local", ".log"):
                self.assertNotIn(forbidden, callbacks, name)
            self.assertIn("&dyn Services", callbacks)

        services = (ROOT / "src/runtime/reward_services.rs").read_text()
        contract = services.split("trait Services {", 1)[1].split("\n}", 1)[0]
        for forbidden in ("Runtime", "Local", "SelectedAdapter", "Access", "Mutex", "Deref", "Any"):
            self.assertNotIn(forbidden, contract)

    def test_xp_callbacks_do_not_access_runtime_or_native_memory(self):
        source = (ROOT / "src/runtime/xp_hooks.rs").read_text()
        for forbidden in ("RUNTIME", "Runtime", "io::Access", ".local", ".adapter", ".party"):
            self.assertNotIn(forbidden, source)
        services = (ROOT / "src/runtime/xp_hooks/services.rs").read_text()
        contract = services.split("trait Services {", 1)[1].split("\n}", 1)[0]
        for forbidden in ("Runtime", "Local", "SelectedAdapter", "Access", "Mutex", "Deref", "Any"):
            self.assertNotIn(forbidden, contract)

    def test_deferred_xp_and_pause_have_bounded_runtime_bindings(self):
        for name in ("experience", "pause", "elites", "blessings"):
            source = (ROOT / f"src/runtime/{name}.rs").read_text()
            self.assertNotRegex(source, r"\b(?:RUNTIME|Runtime)\b|io::Access|\.(?:local|adapter|party)\b(?!\s*\()", name)
            services = (ROOT / f"src/runtime/{name}/services.rs").read_text()
            contract = services.split("trait Services {", 1)[1].split("\n}", 1)[0]
            for forbidden in ("Runtime", "Local", "SelectedAdapter", "Access", "Mutex", "Deref", "Any"):
                self.assertNotIn(forbidden, contract, name)

    def test_pause_flags_and_ui_trampoline_stay_with_their_owner(self):
        for name in ("session", "menu_pause", "fixture"):
            source = (ROOT / f"src/runtime/{name}.rs").read_text()
            for forbidden in ("PAUSED.", "HOST_PAUSE_AUTHORITY", "UI_EVENT_ORIGINAL"):
                self.assertNotIn(forbidden, source, name)

    def test_session_menu_and_lifecycle_do_not_expose_the_shared_runtime(self):
        for name in ("session", "menu_pause", "lifecycle"):
            source = (ROOT / f"src/runtime/{name}.rs").read_text()
            if name == "menu_pause":
                # The controlled native fixture remains an integration client.
                source = source[source.index("static ACTIVE:"):source.index("static MENU_SEEN:")]
            self.assertNotRegex(source, r"\b(?:RUNTIME|Runtime)\b|io::Access|\.(?:local|adapter|party)\b(?!\s*\()", name)
            services = (ROOT / f"src/runtime/{name}/services.rs").read_text()
            contracts = re.findall(r"trait \w+[^\{]*\{(.*?)\n\}", services, re.S)
            self.assertTrue(contracts, name)
            for contract in contracts:
                self.assertNotRegex(contract, r"\b(?:Runtime|Local|SelectedAdapter|Access|Mutex|Deref|Any)\b", name)


if __name__ == "__main__":
    unittest.main()
