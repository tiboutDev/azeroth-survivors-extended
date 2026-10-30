"""Native buff identity survives cosmetic order and restoration under Lua 5.1."""
from pathlib import Path
import unittest
from test_official_party_lua import STUB,LuaRuntime
ROOT=Path(__file__).resolve().parents[1]
@unittest.skipUnless(LuaRuntime,"ASXUI requires Lua 5.1")
class BuffOrderTests(unittest.TestCase):
 def setUp(self):
  self.lua=LuaRuntime();self.lua.execute(STUB)
  self.lua.execute('''available=true;fortune=true;function ASX_GetAPIVersion()return 1,2 end;function ASX_GetState()return {available=available,data={fortune=fortune and {} or nil}}end
  BuffFrame=CreateFrame("Frame","BuffFrame",UIParent)
  for n=0,3 do local f=CreateFrame("Button","BuffButton"..n,BuffFrame);f:SetID(n);f.buffIndex=n;f.scripts.OnClick=function()return n end;f.duration={owner=n};function f:GetNumPoints()return self.anchor and 1 or 0 end;function f:GetPoint()if self.anchor then return unpack(self.anchor)end end;function f:SetPoint(...)local p={unpack(arg)};local relative=p[2];local seen={};while type(relative)=="table" and not seen[relative]do assert(relative~=self,"anchor cycle");seen[relative]=true;relative=relative.anchor and relative.anchor[2]end;self.anchor=p end end
  objects.BuffButton0:SetPoint("TOPRIGHT",BuffFrame,"TOPRIGHT",0,0)
  objects.BuffButton1:SetPoint("RIGHT",objects.BuffButton0,"LEFT",-5,0)
  objects.BuffButton2:SetPoint("RIGHT",objects.BuffButton1,"LEFT",-5,0)
  objects.BuffButton3:SetPoint("TOPRIGHT",objects.BuffButton0,"BOTTOMRIGHT",0,-40)
  nativeBuffTextures[2]="Interface/Icons/INV_Misc_Orb_02.blp"
  function tick()now=now+.21;objects.ASXUIBuffOrderController.scripts.OnUpdate()end''')
  self.lua.execute((ROOT/'addons/ASXUI/Buffs.lua').read_text())
 def test_moves_fortune_to_native_first_slot_and_preserves_ids_clicks_and_wrap(self):
  self.lua.execute('tick();assert(objects.BuffButton2.anchor[2]==BuffFrame);assert(objects.BuffButton0.anchor[2]==objects.BuffButton2);assert(objects.BuffButton1.anchor[2]==objects.BuffButton0);assert(objects.BuffButton3.anchor[2]==objects.BuffButton2);for n=0,3 do local f=objects["BuffButton"..n];assert(f:GetID()==n and f.buffIndex==n and f.duration.owner==n and f.scripts.OnClick()==n)end;for i=1,10 do tick()end')
 def test_native_relayout_is_repaired_next_frame_without_waiting_for_metadata_poll(self):
  self.lua.execute('tick();for n=0,3 do objects["BuffButton"..n]:ClearAllPoints()end;objects.BuffButton0:SetPoint("TOPRIGHT",BuffFrame,"TOPRIGHT",0,0);objects.BuffButton1:SetPoint("RIGHT",objects.BuffButton0,"LEFT",-5,0);objects.BuffButton2:SetPoint("RIGHT",objects.BuffButton1,"LEFT",-5,0);objects.BuffButton3:SetPoint("TOPRIGHT",objects.BuffButton0,"BOTTOMRIGHT",0,-40);now=now+.01;objects.ASXUIBuffOrderController.scripts.OnUpdate();assert(objects.BuffButton2.anchor[2]==BuffFrame and objects.BuffButton0.anchor[2]==objects.BuffButton2)')
 def test_missing_state_restores_original_anchors_without_cycles(self):
  self.lua.execute('tick();available=false;tick();assert(objects.BuffButton0.anchor[2]==BuffFrame);assert(objects.BuffButton1.anchor[2]==objects.BuffButton0);assert(objects.BuffButton2.anchor[2]==objects.BuffButton1);assert(objects.BuffButton3.anchor[2]==objects.BuffButton0)')
 def test_native_layout_updates_are_preserved(self):
  self.lua.execute('tick();objects.BuffButton1:SetPoint("TOPRIGHT",BuffFrame,"TOPRIGHT",-500,0);fortune=false;tick();assert(objects.BuffButton1.anchor[2]==BuffFrame and objects.BuffButton1.anchor[4]==-500)')
 def test_hidden_or_unrelated_buff_is_not_promoted(self):
  self.lua.execute('objects.BuffButton2:Hide();tick();assert(objects.BuffButton0.anchor[2]==BuffFrame);objects.BuffButton2:Show();nativeBuffTextures[2]="native-class-buff";tick();assert(objects.BuffButton2.anchor[2]==objects.BuffButton1)')
if __name__=='__main__':unittest.main()
