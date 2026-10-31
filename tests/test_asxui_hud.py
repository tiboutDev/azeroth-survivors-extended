"""Native HUD invariants using the actual Lua 5.1 renderer."""
from pathlib import Path
import unittest
from test_official_party_lua import STUB, LuaRuntime
ROOT=Path(__file__).resolve().parents[1]
HUD_STUB=r'''
ASXUIDB={};hudPrefs={build=true,tally=true,clock=true};xpWrites=0
function AZS_IsHudShown(k)return hudPrefs[k]end
function AZS_SetHudShown(k,v)hudPrefs[k]=v end
function GameTooltip:SetText(v)self.title=v end
local create=CreateFrame
function CreateFrame(kind,name,parent,template)
 local f=create(kind,name,parent,template)
 function f:GetNumPoints()return self.anchor and 1 or 0 end
 function f:GetPoint()if self.anchor then return unpack(self.anchor)end end
 function f:GetTop()return 0 end
 function f:GetBottom()return 0 end
 function f:GetFrameLevel()return self.level or 5 end
 function f:GetFrameStrata()return self.strata or "LOW"end
 function f:GetScript(event)return self.scripts[event]end
 function f:GetText()return self.text end
 function f:SetNormalTexture(t)self.normal=t end
 function f:SetHighlightTexture(t)self.highlight=t end
 function f:SetScrollChild(v)self.scrollChild=v end
 function f:SetVerticalScroll(v)self.scroll=v end
 if name then _G[name]=f end
 return f
end
MainMenuBar=CreateFrame("Frame","MainMenuBar",UIParent);MainMenuBar:SetWidth(397);MainMenuBar:SetHeight(66);MainMenuBar:SetPoint("BOTTOM",UIParent,"BOTTOM",0,0)
MainMenuBarArtFrame=CreateFrame("Frame","MainMenuBarArtFrame",MainMenuBar)
MainMenuExpBar=CreateFrame("StatusBar","MainMenuExpBar",MainMenuBar);MainMenuExpBar:SetWidth(393);MainMenuExpBar:SetHeight(13);MainMenuExpBar:SetPoint("BOTTOMLEFT",MainMenuBar,"BOTTOMLEFT",2,43)
for _,key in ipairs({"SetHeight","SetTexture","SetStatusBarTexture","SetScript","SetAlpha","Hide","Show"})do MainMenuExpBar[key]=function()error("native XP modified")end end
for i=0,4 do local name=i==4 and "AZS_MainMenuBarTexture4"or("MainMenuBarTexture"..i);local f=CreateFrame("Texture",name,MainMenuBarArtFrame);f:SetPoint("LEFT",MainMenuBarArtFrame,"LEFT",i*40,0)end
for _,key in ipairs({"MainMenuBarLeftEndCap","MainMenuBarRightEndCap","CharacterMicroButton","TalentMicroButton"})do local f=CreateFrame("Frame",key,MainMenuBarArtFrame);f:SetPoint("LEFT",MainMenuBarArtFrame,"LEFT",10,0)end
for i=1,12 do local f=CreateFrame("Button","ActionButton"..i,MainMenuBarArtFrame);f:SetPoint("LEFT",MainMenuBarArtFrame,"LEFT",i*40,4);f.scripts.OnClick=function()nativeClickCount=(nativeClickCount or 0)+1 end end
AZS_MAIN_BAR_SLICES={};AZS_MAIN_BAR_ROWS={}
function AZS_MainBar_Slice(t,file,rows,slice,anchor,point,y)t:SetTexture(file);t:SetPoint("BOTTOMLEFT",getglobal(anchor),point,slice[4],y)end
AZS_RUN={t=10,kills=12,gold=34,chest=40,nb=170}
function AZS_GetRunTime()return AZS_RUN.t end
AZS_STATS={dmg=1.2,cd=1,crit=.05,cm=2,area=1,cnt=0,echo=0,bp=0,dots=1,doth=0,big=0,exe=0,cc=0,dotd=0,shd=0,auto=0,pet=0,hp=100,arm=0,reg=1,spd=1,pick=9,lch=0,heal=0,thn=0,liv=0,xp=1,gold=1,luck=0}
view={available=true,session={epoch="one",recipientGUID="11"},data={equipment={{slot=1,rank=2}}},texts={equipment={{name="Native ring",icon="INV_Jewelry_Ring_01",text="Exact native tooltip\n+10 damage"}}}}
function ASX_GetState()return view end
function hudCall(name,event)local previous=this;this=objects[name];this.scripts[event]();this=previous end
function hudTick(n)for i=1,n or 4 do now=now+.1;arg1=.1;ASXUIHudController.scripts.OnUpdate()end end
'''
@unittest.skipUnless(LuaRuntime,'HUD requires Lua 5.1')
class HUDTests(unittest.TestCase):
    def setUp(self):
        self.lua=LuaRuntime(unpack_returned_tuples=True);self.lua.execute(STUB);self.lua.execute(HUD_STUB)
        self.lua.execute((ROOT/'addons/ASXUI/Theme.lua').read_text())
        self.lua.execute((ROOT/'addons/ASXUI/Hud.lua').read_text());self.lua.execute('arg1="ASXUI";ASXUIHudController.scripts.OnEvent();hudTick()')
    def run_lua(self,s):self.lua.execute(s)
    def test_native_xp_and_action_handlers_stay_native(self):
        self.run_lua('assert(ASXUI_HudReadout().enabled);assert(MainMenuExpBar:GetWidth()==481 and MainMenuExpBar:GetHeight()==13);ActionButton1.scripts.OnClick();assert(nativeClickCount==1);assert(ASXUIHudRoot:GetFrameLevel()<MainMenuBarArtFrame:GetFrameLevel());assert(ASXUIHudDrawer:GetFrameLevel()<MainMenuBarArtFrame:GetFrameLevel());assert(ASXUIHudCombat.anchor[4]==403 and ASXUIHudCharacter.anchor[4]==447)')
    def test_top_counters_are_independent_of_bottom_stack_and_clock_restores(self):
        self.run_lua('assert(ASXUIHudCounters.anchor[1]=="TOP" and ASXUIHudCounters.anchor[2]==UIParent and ASXUIHudCounters.anchor[5]==0);assert(ASXUIHudEquipment.anchor[2]==ASXUIHudBase and ASXUIHudBase.anchor[2]==MainMenuExpBar);assert(not hudPrefs.clock and ASXUIHudClock:IsShown());ASXUI_ConfigureHud("off");assert(hudPrefs.clock and not ASXUIHudClock:IsShown());hudPrefs.clock=false;ASXUI_ConfigureHud("on");hudTick();assert(not ASXUIHudClock:IsShown())')
    def test_displayed_zero_stats_are_hidden_multiplier_one_is_zero_not_critical_two(self):
        self.run_lua('hudCall("ASXUIHudCombat","OnEnter");hudTick();local r=ASXUI_HudReadout().rows;local seen={};for _,v in ipairs(r)do seen[v.key]=v.value end;assert(seen.dmg=="+20%" and seen.cm=="200%" and seen.dots=="1");assert(seen.cd==nil and seen.area==nil and seen.auto==nil)')
    def test_slide_moves_scroll_without_changing_panel_height_or_native_xp(self):
        self.run_lua('hudCall("ASXUIHudCombat","OnClick");local h=ASXUIHudDrawer:GetHeight();hudTick(1);local r=ASXUI_HudReadout();assert(r.progress>0 and r.progress<1 and ASXUIHudDrawerMask.scroll>0);assert(ASXUIHudDrawer:GetHeight()==h);hudTick(5);assert(ASXUI_HudReadout().progress==1);hudCall("ASXUIHudCombat","OnClick");hudTick(1);assert(ASXUI_HudReadout().progress>0 and ASXUI_HudReadout().progress<1);assert(ASXUIHudDrawer:GetHeight()==h)')
    def test_membership_is_frozen_while_values_refresh_and_rebuilt_on_reopen(self):
        self.run_lua('hudCall("ASXUIHudCombat","OnClick");hudTick();local n=table.getn(ASXUI_HudReadout().rows);AZS_STATS.echo=.2;AZS_STATS.dmg=1;hudTick();local r=ASXUI_HudReadout().rows;assert(table.getn(r)==n and r[1].value=="+0%");hudCall("ASXUIHudCombat","OnClick");hudTick();hudCall("ASXUIHudCombat","OnClick");hudTick();local seen={};for _,v in ipairs(ASXUI_HudReadout().rows)do seen[v.key]=v.value end;assert(seen.dmg==nil and seen.echo=="20%")')
    def test_native_equipment_text_is_verbatim_and_missing_data_restores_native_build(self):
        self.run_lua('assert(not hudPrefs.build);hudCall("ASXUIHudEquipment1","OnEnter");assert(GameTooltip.title=="Native ring" and GameTooltip.detail=="Exact native tooltip\\n+10 damage");view.available=false;hudTick();assert(hudPrefs.build and not ASXUIHudEquipment:IsShown());assert(ASXUI_HudReadout().equipment[1]==nil)')
    def test_toggle_restores_owned_anchors_and_preferences_but_preserves_external_changes(self):
        self.run_lua('ActionButton2:ClearAllPoints();ActionButton2:SetPoint("TOP",UIParent,"TOP",123,9);assert(ASXUI_ConfigureHud("off"));assert(not ASXUI_HudReadout().enabled and hudPrefs.build and hudPrefs.tally and MainMenuExpBar:GetWidth()==393);assert(ActionButton1.anchor[2]==MainMenuBarArtFrame);assert(ActionButton2.anchor[1]=="TOP" and ActionButton2.anchor[4]==123);assert(not ASXUI_ConfigureHud("bad"))')
    def test_missing_values_are_not_invented_and_retained_frames_do_not_allocate_each_tick(self):
        self.run_lua('local count=allocations;AZS_RUN.chest=nil;hudTick(40);assert(allocations==count and hudPrefs.tally);phase="ready";hudTick();assert(not ASXUI_HudReadout().enabled and not ASXUIHudRoot:IsShown())')
if __name__=='__main__':unittest.main()
