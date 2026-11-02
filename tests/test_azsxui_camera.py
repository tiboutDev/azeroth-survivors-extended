"""Persistent camera behavior under the game's Lua 5.1 contract."""
from pathlib import Path
import unittest
try:
    from lupa.lua51 import LuaRuntime
except ImportError:
    LuaRuntime = None

ROOT = Path(__file__).resolve().parents[1]
STUB = r"""
frames={}
function getglobal(name)return _G[name]end
function newFrame(name)
 local f={scripts={},shown=false}
 function f:SetPoint(...)end
 function f:SetText(v)self.text=v end
 function f:Show()self.shown=true end
 function f:Hide()self.shown=false end
 function f:IsShown()return self.shown end
 function f:CreateFontString()return newFrame()end
 function f:RegisterEvent(e)end
 function f:GetScript(e)return self.scripts[e]end
 function f:SetScript(e,fn)self.scripts[e]=fn end
 if name then _G[name]=f end
 return f
end
function CreateFrame(kind,name,parent)return newFrame(name)end
UIParent=newFrame("UIParent");WorldFrame=newFrame("WorldFrame")
TalentFrame=newFrame("TalentFrame");GameMenuFrame=newFrame("GameMenuFrame")
look=false;starts=0;stops=0
function MouselookStart()look=true;starts=starts+1 end
function MouselookStop()look=false;stops=stops+1 end
function IsMouselooking()return look end
function AZS_GetPhase()return phase or "playing"end
function click(button)
 arg1=button
 WorldFrame.scripts.OnMouseDown()
 WorldFrame.scripts.OnMouseUp()
 arg1=.11
 AZSXUICameraController.scripts.OnUpdate()
 arg1=nil
end
function tick(dt)arg1=dt or .11;AZSXUICameraController.scripts.OnUpdate();arg1=nil end
"""

@unittest.skipUnless(LuaRuntime, "Lua 5.1 profile requires lupa")
class CameraTests(unittest.TestCase):
    def setUp(self):
        self.lua=LuaRuntime(unpack_returned_tuples=True)
        self.lua.execute(STUB)
        self.lua.execute((ROOT/"addons/AZSXUI/Camera.lua").read_text())

    def run_lua(self, code):
        self.lua.execute(code)

    def test_right_click_toggles_without_auto_reacquire(self):
        self.run_lua('click("RightButton");assert(look and starts==1, tostring(look)..":"..tostring(starts)..":"..AZSXUI_CameraStatus().mode..":"..tostring(AZSXUI_CameraStatus().pending));click("RightButton");assert(not look);for i=1,30 do tick()end;assert(starts==1)')

    def test_talents_release_and_closing_stays_in_inspection(self):
        self.run_lua('click("RightButton");TalentFrame:Show();tick();assert(not look and AZSXUI_CameraStatus().mode=="inspect");TalentFrame:Hide();for i=1,30 do tick()end;assert(not look);click("RightButton");assert(look)')

    def test_other_menus_and_choices_release(self):
        self.run_lua('click("RightButton");GameMenuFrame:Show();tick();assert(not look);GameMenuFrame:Hide();click("RightButton");phase="choosing";tick();assert(not look);phase="playing";tick();assert(not look)')

    def test_native_handlers_and_left_click_remain_intact(self):
        self.run_lua('downs=0;ups=0;WorldFrame:SetScript("OnMouseDown",function()downs=downs+1 end);WorldFrame:SetScript("OnMouseUp",function()ups=ups+1 end);tick(1);click("RightButton");click("LeftButton");assert(downs==2 and ups==2 and look)')

    def test_conflicting_addon_disables_controller(self):
        self.run_lua('function SCM_Toggle()end;click("RightButton");assert(not look and AZSXUI_CameraStatus().conflict);SCM_Toggle=nil;click("RightButton");assert(look)')

    def test_bindings_and_missing_api(self):
        self.run_lua('AZSXUI_CameraToggle();assert(look);AZSXUI_CameraInspect();assert(not look);MouselookStart=nil;click("RightButton");assert(not look)')

if __name__=="__main__": unittest.main()
