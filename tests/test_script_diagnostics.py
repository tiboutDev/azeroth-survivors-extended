"""A diagnostic error dialog must never be accepted as probe evidence."""
import sys
import unittest
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1]/"tools/dev"))
from script_diagnostics import assess

class ScriptDiagnosticsTests(unittest.TestCase):
    def test_probe_errors_and_blocked_actions_fail(self):
        for text in ["ui_script: runtime error: AZCOOP1125 class=Druid members=0", "AzCoopNativeProbe has been blocked from an action only available to the Blizzard UI", "ui_script: runtime error: unexpected nil"]:
            self.assertFalse(assess(text)["passed"])

    def test_only_specific_base_warning_is_classified_separately(self):
        warning="ui_script(FrameXML/Interface\\FrameXML\\FloatingChatFrame.xml): runtime error: SetScript: unsupported script 'OnInputLanguageChanged'"
        result=assess(warning)
        self.assertTrue(result["passed"])
        self.assertEqual(result["known_base_warnings"], [warning])
        self.assertFalse(assess(warning.replace("FloatingChatFrame.xml", "Coop.lua"))["passed"])

    def test_current_party_probe_has_no_error_or_protected_input(self):
        text=(Path(__file__).resolve().parents[1]/"tools/dev/fixtures/party1125-probe.lua").read_text()
        for forbidden in ["error(", "seterrorhandler(", "MoveForwardStart", "MoveForwardStop"]:
            self.assertNotIn(forbidden, text)
        self.assertIn('SetCVar("azcoop_probe_report"', text)
