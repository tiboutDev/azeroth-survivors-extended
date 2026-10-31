"""Native HUD invariants using the actual Lua 5.1 renderer."""
from pathlib import Path
import unittest
from test_official_party_lua import STUB, LuaRuntime
ROOT=Path(__file__).resolve().parents[1]
HUD_STUB=r'''
nativeCharacterCalls={};function ToggleCharacter(tab)table.insert(nativeCharacterCalls,tab);return "native:"..tostring(tab)end;nativeCharacterToggle=ToggleCharacter
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
 function f:SetBlendMode(v)self.blend=v end
 function f:GetFrameLevel()return self.level or 5 end
 function f:GetFrameStrata()return self.strata or "LOW"end
 function f:GetValue()return self.value or 0 end
 function f:GetScript(event)return self.scripts[event]end
 function f:GetText()return self.text end
 function f:SetChecked(v)self.checked=v end
 function f:SetCheckedTexture(t)self.checkedTexture=t end
 function f:SetPushedTexture(t)self.pushedTexture=t end
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
        self.run_lua('assert(ASXUI_HudReadout().enabled);assert(MainMenuExpBar:GetWidth()==437 and MainMenuExpBar:GetHeight()==13);ActionButton1.scripts.OnClick();assert(nativeClickCount==1);assert(ASXUIHudRoot:GetFrameLevel()<MainMenuBarArtFrame:GetFrameLevel());assert(ASXUIHudDrawer:GetFrameLevel()<MainMenuBarArtFrame:GetFrameLevel());assert(ASXUIHudStats.anchor[4]==403 and ASXUIHudCombat==nil and ASXUIHudCharacter==nil)')
    def test_top_counters_are_independent_of_bottom_stack_and_clock_restores(self):
        self.run_lua('assert(ASXUIHudCounters.anchor[1]=="TOP" and ASXUIHudCounters.anchor[2]==UIParent and ASXUIHudCounters.anchor[5]==-8);assert(ASXUIHudEquipment.anchor[2]==ASXUIHudBase and ASXUIHudBase.anchor[2]==MainMenuExpBar);assert(not hudPrefs.clock and ASXUIHudClock:IsShown());ASXUI_ConfigureHud("off");assert(hudPrefs.clock and not ASXUIHudClock:IsShown());hudPrefs.clock=false;ASXUI_ConfigureHud("on");hudTick();assert(not ASXUIHudClock:IsShown())')
    def test_displayed_zero_stats_are_hidden_multiplier_one_is_zero_not_critical_two(self):
        self.run_lua('hudCall("ASXUIHudStats","OnEnter");hudTick();local r=ASXUI_HudReadout().rows;local seen={};for _,v in ipairs(r)do seen[v.key]=v.value end;assert(seen.dmg=="+20%" and seen.cm=="200%" and seen.dots=="1");assert(seen.cd==nil and seen.area==nil and seen.auto==nil)')
    def test_single_panel_opens_and_closes_immediately_and_includes_every_stat(self):
        self.run_lua('for k,v in pairs(AZS_STATS)do AZS_STATS[k]=2 end;hudCall("ASXUIHudStats","OnClick");local r=ASXUI_HudReadout();assert(r.progress==1 and table.getn(r.rows)==29 and ASXUIHudDrawerMask:IsShown());local h=ASXUIHudDrawer:GetHeight();hudTick(1);assert(ASXUI_HudReadout().progress==1 and ASXUIHudDrawer:GetHeight()==h);hudCall("ASXUIHudStats","OnClick");assert(ASXUI_HudReadout().progress==0 and not ASXUIHudDrawerMask:IsShown())')
    def test_joined_panel_has_four_columns_short_labels_and_native_foreground(self):
        self.run_lua('for k in pairs(AZS_STATS)do AZS_STATS[k]=2 end;hudCall("ASXUIHudStats","OnClick");hudTick();assert(ASXUIHudDrawer:GetWidth()==MainMenuExpBar:GetWidth()+32 and ASXUIHudDrawerMask:GetWidth()==469);local rows=ASXUI_HudReadout().rows;assert(table.getn(rows)==29 and rows[1].label=="DMG" and rows[18].label=="Max HP");assert(ASXUIHudDrawer:GetFrameLevel()>ASXUIHudEquipment1:GetFrameLevel() and ASXUIHudDrawer:GetFrameLevel()<MainMenuBarArtFrame:GetFrameLevel());assert(ASXUIHudCounters:GetWidth()==300)')
    def test_native_checked_state_tracks_only_pinned_panel_and_disable(self):
        self.run_lua('hudCall("ASXUIHudStats","OnEnter");assert(not ASXUIHudStats.checked);hudCall("ASXUIHudStats","OnClick");assert(ASXUIHudStats.checked);hudCall("ASXUIHudStats","OnLeave");hudTick();assert(ASXUIHudStats.checked);hudCall("ASXUIHudStats","OnClick");assert(not ASXUIHudStats.checked);hudCall("ASXUIHudStats","OnClick");ASXUI_ConfigureHud("off");assert(not ASXUIHudStats.checked)')
    def test_columns_have_vertical_rules_and_xp_art_draws_above_the_clipped_bottom(self):
        self.run_lua('hudCall("ASXUIHudStats","OnClick");hudTick();assert(table.getn(ASXUIHudDrawer.dividers)==3 and ASXUIHudDrawer.separator==nil);for _,line in ipairs(ASXUIHudDrawer.dividers)do assert(line:GetWidth()==1 and line.anchor[1]=="BOTTOMLEFT")end;assert(MainMenuExpBar:GetFrameLevel()>ASXUIHudDrawer:GetFrameLevel() and MainMenuExpBar:GetFrameLevel()<MainMenuBarArtFrame:GetFrameLevel());assert(ASXUIHudDrawer:GetHeight()==ASXUI_HudReadout().height+8);ASXUI_ConfigureHud("off");assert(MainMenuExpBar:GetFrameLevel()==5)')
    def test_xp_layer_is_ready_before_the_first_update_after_reactivation(self):
        self.run_lua('ASXUI_ConfigureHud("off");ASXUI_ConfigureHud("on");hudCall("ASXUIHudStats","OnClick");assert(MainMenuExpBar:GetFrameLevel()>ASXUIHudDrawer:GetFrameLevel() and MainMenuExpBar:GetFrameLevel()<MainMenuBarArtFrame:GetFrameLevel());ASXUI_ConfigureHud("off");assert(MainMenuExpBar:GetFrameLevel()==5)')
    def test_character_command_toggles_our_panel_and_forwards_other_tabs_and_disabled_hud(self):
        self.run_lua('ToggleCharacter("PaperDollFrame");assert(ASXUI_HudReadout().pinned=="stats" and table.getn(nativeCharacterCalls)==0);ToggleCharacter("PaperDollFrame");assert(not ASXUIHudDrawerMask:IsShown());assert(ToggleCharacter("PetPaperDollFrame")=="native:PetPaperDollFrame");ASXUI_ConfigureHud("off");assert(ToggleCharacter==nativeCharacterToggle and ToggleCharacter("PaperDollFrame")=="native:PaperDollFrame")')
    def test_character_command_preserves_an_external_replacement_on_disable(self):
        self.run_lua('local external=function()return "external"end;ToggleCharacter=external;ASXUI_ConfigureHud("off");assert(ToggleCharacter==external)')
    def test_counter_cells_keep_their_vertical_center_after_activation(self):
        self.run_lua('for _,v in ipairs(ASXUIHudCounters.values)do assert(v.button.anchor[1]=="LEFT" and v.button.anchor[3]=="LEFT" and v.button.anchor[5]==0)end;ASXUI_ConfigureHud("off");ASXUI_ConfigureHud("on");for _,v in ipairs(ASXUIHudCounters.values)do assert(v.button.anchor[1]=="LEFT" and v.button.anchor[5]==0)end')
    def test_gold_glow_tracks_actual_affordability_and_invalid_data(self):
        self.run_lua('assert(not ASXUI_HudReadout().affordable);AZS_RUN.gold=40;hudTick();assert(ASXUI_HudReadout().affordable);AZS_RUN.chest=41;hudTick();assert(not ASXUI_HudReadout().affordable);AZS_RUN.gold=100;AZS_RUN.chest=0;hudTick();assert(not ASXUI_HudReadout().affordable);AZS_RUN.chest=40;AZS_RUN.gold=0/0;hudTick();assert(not ASXUI_HudReadout().affordable);AZS_RUN.gold=100;hudTick();ASXUI_ConfigureHud("off");assert(not ASXUI_HudReadout().affordable)')
    def test_membership_is_frozen_while_values_refresh_and_rebuilt_on_reopen(self):
        self.run_lua('hudCall("ASXUIHudStats","OnClick");hudTick();local n=table.getn(ASXUI_HudReadout().rows);AZS_STATS.echo=.2;AZS_STATS.dmg=1;hudTick();local r=ASXUI_HudReadout().rows;assert(table.getn(r)==n and r[1].value=="+0%");hudCall("ASXUIHudStats","OnClick");hudTick();hudCall("ASXUIHudStats","OnClick");hudTick();local seen={};for _,v in ipairs(ASXUI_HudReadout().rows)do seen[v.key]=v.value end;assert(seen.dmg==nil and seen.echo=="20%")')
    def test_partial_native_categories_reserve_all_five_equipment_positions(self):
        self.run_lua('view.data.equipment={{slot=1,rank=2,category="ring"},{slot=2,rank=1,category="trinket"}};view.texts.equipment={{name="Ring",icon="INV_Ring",text="Ring owner"},{name="Trinket",icon="INV_Trinket",text="Trinket owner"}};hudTick();local r=ASXUI_HudReadout().equipment;assert(r[1]==nil and r[2].name=="Ring" and r[3]==nil and r[4].name=="Trinket" and r[5]==nil);hudCall("ASXUIHudEquipment2","OnEnter");assert(GameTooltip.title=="Ring");view.data.equipment[1].category="amulet";hudTick();assert(ASXUI_HudReadout().equipment[1].name=="Ring" and ASXUI_HudReadout().equipment[2]==nil)')
    def test_native_equipment_text_is_verbatim_and_missing_data_restores_native_build(self):
        self.run_lua('assert(not hudPrefs.build);hudCall("ASXUIHudEquipment1","OnEnter");assert(GameTooltip.title=="Native ring" and GameTooltip.detail=="Exact native tooltip\\n+10 damage");view.available=false;hudTick();assert(hudPrefs.build and not ASXUIHudEquipment:IsShown());assert(ASXUI_HudReadout().equipment[1]==nil)')
    def test_toggle_restores_owned_anchors_and_preferences_but_preserves_external_changes(self):
        self.run_lua('ActionButton2:ClearAllPoints();ActionButton2:SetPoint("TOP",UIParent,"TOP",123,9);assert(ASXUI_ConfigureHud("off"));assert(not ASXUI_HudReadout().enabled and hudPrefs.build and hudPrefs.tally and MainMenuExpBar:GetWidth()==393);assert(ActionButton1.anchor[2]==MainMenuBarArtFrame);assert(ActionButton2.anchor[1]=="TOP" and ActionButton2.anchor[4]==123);assert(not ASXUI_ConfigureHud("bad"))')
    def test_missing_values_are_not_invented_and_retained_frames_do_not_allocate_each_tick(self):
        self.run_lua('local count=allocations;AZS_RUN.chest=nil;hudTick(40);assert(allocations==count and hudPrefs.tally);phase="ready";hudTick();assert(not ASXUI_HudReadout().enabled and not ASXUIHudRoot:IsShown())')
if __name__=='__main__':unittest.main()
