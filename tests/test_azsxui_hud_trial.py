"""Real renderer tolerances and text containment in the HUD trial assessor."""
import sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools/dev'))
from azsxui_hud_trial import compact_popup,native_goblin_unmarked

class PopupAssessorTests(unittest.TestCase):
 def test_native_coordinate_rounding_is_accepted(self):
  self.assertTrue(compact_popup(dict(width=320.0000134039,height=80.000008206469,icon=32,inside=True)))
 def test_goblin_mark_assessment_requires_actual_azsx_native_observations(self):
  log='INFO goblin native marker observation: guid=123 skull=false\n'
  self.assertTrue(native_goblin_unmarked(log))
  for bad in ('','skull=false',log+'INFO goblin native marker observation: guid=456 skull=true',log.replace('123','0')):
   self.assertFalse(native_goblin_unmarked(bad))
 def test_wrong_dimensions_or_clipped_text_are_rejected(self):
  for change in ({'width':388},{'height':70},{'icon':46},{'inside':False},{'width':float('nan')},{'height':None}):
   value=dict(width=320,height=80,icon=32,inside=True);value.update(change)
   self.assertFalse(compact_popup(value))


# Persistence logic is Lua-owned; no real client/profile is started here.
from test_official_party_lua import STUB,LuaRuntime
@unittest.skipUnless(LuaRuntime,'Requires Lua 5.1')
class NativeVisibilityTests(unittest.TestCase):
 def setUp(self):
  self.lua=LuaRuntime();self.lua.execute(STUB)
  self.lua.execute('AZSXUIDB={};prefs={clock=true,loot=true,revive=false};function AZS_IsHudShown(k)return prefs[k]end;function AZS_SetHudShown(k,v)prefs[k]=v end')
  self.lua.execute((Path(__file__).resolve().parents[1]/'addons/AZSXUI/Theme.lua').read_text())
 def test_claim_restore_preserves_originally_hidden_parts(self):
  self.lua.execute('assert(AZSXUI_ClaimNativeHud("clock") and not prefs.clock and AZSXUIDB.nativeHudVisibility.clock);assert(AZSXUI_ClaimNativeHud("revive")==false and not prefs.revive);AZSXUI_RestoreNativeHud();assert(prefs.clock and not prefs.revive and AZSXUIDB.nativeHudVisibility.clock==nil)')
 def test_saved_original_values_recover_a_previous_session(self):
  self.lua.execute('AZSXUIDB.nativeHudVisibility={clock=true,loot=true,revive=false};prefs.clock=false;prefs.loot=false;event="ADDON_LOADED";arg1="AZSXUI";objects.AZSXUINativeVisibilityController.scripts.OnEvent();assert(prefs.clock and prefs.loot and not prefs.revive);assert(AZSXUIDB.nativeHudVisibility.clock==nil)')
 def test_logout_restores_preferences_and_cannot_reclaim_until_world_entry(self):
  self.lua.execute('AZSXUI_ClaimNativeHud("clock");event="PLAYER_LOGOUT";objects.AZSXUINativeVisibilityController.scripts.OnEvent();assert(prefs.clock and AZSXUIDB.nativeHudVisibility.clock);AZSXUI_ClaimNativeHud("clock");assert(prefs.clock);event="PLAYER_ENTERING_WORLD";objects.AZSXUINativeVisibilityController.scripts.OnEvent();AZSXUI_ClaimNativeHud("clock");assert(not prefs.clock)')

if __name__=='__main__':unittest.main()
