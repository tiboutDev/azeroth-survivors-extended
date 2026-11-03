"""Native HUD invariants using the actual Lua 5.1 renderer."""
from pathlib import Path
import unittest
from test_official_party_lua import STUB, LuaRuntime
ROOT=Path(__file__).resolve().parents[1]
HUD_STUB=r'''
CharacterFrame=CreateFrame("Frame","CharacterFrame",UIParent);PaperDollFrame=CreateFrame("Frame","PaperDollFrame",CharacterFrame);CharacterFrame:Hide();PaperDollFrame:Hide()
nativeCharacterCalls={};function ToggleCharacter(tab)table.insert(nativeCharacterCalls,tab);if tab=="PaperDollFrame"then if opaqueCharacterBackend then nativeCharacterShown=not nativeCharacterShown elseif CharacterFrame:IsVisible()and PaperDollFrame:IsVisible()then CharacterFrame:Hide();PaperDollFrame:Hide()else CharacterFrame:Show();PaperDollFrame:Show()end end;return "native:"..tostring(tab),nil,"retained"end;nativeCharacterToggle=ToggleCharacter
AZSXUIDB={};hudPrefs={build=true,tally=true,clock=true};xpWrites=0
function AZS_IsHudShown(k)return hudPrefs[k]end
function AZS_SetHudShown(k,v)hudPrefs[k]=v end
function GameTooltip:SetText(v)self.title=v;self.lines={}end
function GameTooltip:ClearLines()self.lines={}end
function GameTooltip:AddLine(value,r,g,b,wrap)table.insert(self.lines,{value,r,g,b});self.detail=value end
function GameTooltip:AddDoubleLine(left,right)self.title=left;self.right=right end
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
 if template=="TargetDebuffButtonTemplate"then
  create("Font",name.."Count",f)
  f.scripts.OnEnter=function()GameTooltip:SetUnitDebuff("target",this:GetID())end
 end
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
function AZSX_GetAPIVersion()return 1,14 end
function AZSX_GetState()return view end
function hudCall(name,event)local previous=this;this=objects[name];this.scripts[event]();this=previous end
function hudTick(n)for i=1,n or 4 do now=now+.1;arg1=.1;AZSXUIHudController.scripts.OnUpdate()end end
'''
@unittest.skipUnless(LuaRuntime,'HUD requires Lua 5.1')
class HUDTests(unittest.TestCase):
    def setUp(self):
        self.lua=LuaRuntime(unpack_returned_tuples=True);self.lua.execute(STUB);self.lua.execute(HUD_STUB)
        self.lua.execute((ROOT/'addons/AZSXUI/Theme.lua').read_text())
        self.lua.execute((ROOT/'addons/AZSXUI/Hud.lua').read_text());self.lua.execute('arg1="AZSXUI";AZSXUIHudController.scripts.OnEvent();hudTick()')
    def run_lua(self,s):self.lua.execute(s)
    def test_first_equipment_payload_populates_existing_slots(self):
        self.run_lua('''phase="ready";hudTick();phase="playing";view.available=false;view.reason="missing_or_invalid_payload"
arg1=.001;AZSXUIHudController.scripts.OnUpdate()
assert(AZSXUIHudEquipment:IsShown() and not hudPrefs.build and not AZSXUIHudEquipment1.item)
view.available=true;hudTick();assert(AZSXUIHudEquipment1.item.name=="Native ring")
view.available=false;hudTick();assert(not AZSXUIHudEquipment:IsShown() and hudPrefs.build)''')
    def test_kill_tooltip_updates_and_shares_ranks_on_ties(self):
        self.run_lua('''function GameTooltip:ClearAllPoints()end
function GameTooltip:SetPoint(...)self.anchor=arg end
function GameTooltip:ClearLines()self.ranks={};self.lines={}end
function GameTooltip:AddDoubleLine(name,count)table.insert(self.ranks,{name,count})end
view.data.killRanking={{name="First",kills=5},{name="Second",kills=5},{name="Third",kills=1}}
hudCall("AZSXUIKillCounter","OnEnter");assert(GameTooltip.ranks[2][1]=="1.  Second" and GameTooltip.ranks[3][1]=="3.  Third")
view.data.killRanking[1].kills=8;hudTick();assert(GameTooltip.ranks[1][2]=="8" and GameTooltip.ranks[2][1]=="2.  Second")
hudCall("AZSXUIKillCounter","OnLeave");view.data.killRanking=nil
hudCall("AZSXUIKillCounter","OnEnter");assert(GameTooltip.title=="Kills")''')
    def test_native_party_keeps_native_stats_and_build_without_supported_provider(self):
        self.run_lua('''function GetNumPartyMembers()return 1 end;AZS_STATS.xp=1.25;AZS_STATS.gold=1.5
function AZSX_GetState()error("unsupported API called")end
for _,major in ipairs({0,2})do
 if major==0 then AZSX_GetAPIVersion=nil else function AZSX_GetAPIVersion()return major end end
 hudTick();assert(AZSXUI_HudReadout().enabled and hudPrefs.build and not AZSXUIHudEquipment:IsShown())
 ToggleCharacter("PaperDollFrame");hudTick()
 local vals={};for _,r in ipairs(AZSXUI_HudReadout().rows)do vals[r.key]=r.value end
 assert(vals.xp=="+25%" and vals.gold=="+50%")
 ToggleCharacter("PaperDollFrame");hudTick()
end''')
    def test_native_xp_and_action_handlers_stay_native(self):
        self.run_lua('assert(AZSXUI_HudReadout().enabled);assert(MainMenuExpBar:GetWidth()==393 and MainMenuExpBar:GetHeight()==13);ActionButton1.scripts.OnClick();assert(nativeClickCount==1);assert(AZSXUIHudRoot:GetFrameLevel()<MainMenuBarArtFrame:GetFrameLevel());assert(MainMenuExpBar.anchor[2]==MainMenuBar and ActionButton1.anchor[2]==MainMenuBarArtFrame and MainMenuBarTexture0:GetAlpha()==1);assert(AZSXUIUtilityStrip==nil)')
    def test_top_counters_are_independent_of_bottom_stack_and_clock_restores(self):
        self.run_lua('assert(AZSXUIHudCounters.anchor[1]=="TOP" and AZSXUIHudCounters.anchor[2]==UIParent and AZSXUIHudCounters.anchor[5]==-8);assert(AZSXUIHudEquipment.anchor[2]==AZSXUIHudBase and AZSXUIHudBase.anchor[2]==MainMenuExpBar);assert(not hudPrefs.clock and AZSXUIHudClock:IsShown());AZSXUI_ConfigureHud("off");assert(hudPrefs.clock and not AZSXUIHudClock:IsShown());hudPrefs.clock=false;AZSXUI_ConfigureHud("on");hudTick();assert(not AZSXUIHudClock:IsShown())')
    def test_character_sheet_opens_29_stats_and_preserves_native_command(self):
        self.run_lua('ToggleCharacter("PaperDollFrame");hudTick();local h=AZSXUI_HudReadout();assert(table.getn(h.rows)==29);local seen={};for _,r in ipairs(h.rows)do seen[r.key]=r.value end;assert(seen.dmg=="+20%" and seen.echo=="0%" and seen.pick and seen.luck);assert(AZSXUIHudStats.anchor[2]==PaperDollFrame);hudCall("AZSXUIHudStats","OnClick");hudTick();assert(not AZSXUIHudDrawer:IsShown());ToggleCharacter("PaperDollFrame");hudTick();ToggleCharacter("PaperDollFrame");hudTick();assert(AZSXUIHudDrawer:IsShown())')

    def test_counter_cells_keep_their_vertical_center_after_activation(self):
        self.run_lua('for _,v in ipairs(AZSXUIHudCounters.values)do assert(v.button.anchor[1]=="LEFT" and v.button.anchor[3]=="LEFT" and v.button.anchor[5]==0)end;AZSXUI_ConfigureHud("off");AZSXUI_ConfigureHud("on");for _,v in ipairs(AZSXUIHudCounters.values)do assert(v.button.anchor[1]=="LEFT" and v.button.anchor[5]==0)end')
    def test_gold_glow_tracks_actual_affordability_and_invalid_data(self):
        self.run_lua('assert(not AZSXUI_HudReadout().affordable);AZS_RUN.gold=40;hudTick();assert(AZSXUI_HudReadout().affordable);AZS_RUN.chest=41;hudTick();assert(not AZSXUI_HudReadout().affordable);AZS_RUN.gold=100;AZS_RUN.chest=0;hudTick();assert(not AZSXUI_HudReadout().affordable);AZS_RUN.chest=40;AZS_RUN.gold=0/0;hudTick();assert(not AZSXUI_HudReadout().affordable);AZS_RUN.gold=100;hudTick();AZSXUI_ConfigureHud("off");assert(not AZSXUI_HudReadout().affordable)')
    def test_utility_section_uses_shared_values_and_preserves_personal_stats(self):
        self.run_lua('ToggleCharacter("PaperDollFrame");function GetNumPartyMembers()return 1 end;AZS_STATS.xp=1.2;AZS_STATS.gold=1.7;view.data.sharedEconomy={xp=1.35};hudTick();local r=AZSXUI_HudReadout().rows;assert(r[26].key=="xp" and r[26].value=="+35%" and r[27].key=="pick" and r[28].key=="luck" and r[29].key=="gold" and r[29].value=="—");view.data.sharedEconomy={xp=1,gold=1.6};hudTick();r=AZSXUI_HudReadout().rows;assert(r[26].key=="gold" and r[26].value=="+60%" and r[27].key=="pick" and r[28].key=="xp" and r[28].value=="+0%");view.available=false;hudTick();assert(AZSXUI_HudReadout().rows[29].value=="—" and AZS_STATS.gold==1.7)')

    def test_item_tooltip_preserves_native_lines_rank_and_actual_enchantment(self):
        self.run_lua('view.texts.equipment[1].text="Native owner effect\\n\\nEnchantment: Lesser Might\\n+9% damage";hudTick();hudCall("AZSXUIHudEquipment1","OnEnter");assert(GameTooltip.right=="Rank 2" and GameTooltip.lines[1][1]=="Native owner effect" and GameTooltip.lines[3][1]=="Enchantment: Lesser Might" and GameTooltip.lines[3][2]==.12 and GameTooltip.lines[4][1]=="+9% damage")')
    def test_native_item_effects_get_separate_tooltip_lines(self):
        self.run_lua('view.texts.equipment[1].text="+31% max health, +5% damage reduction\\nHeals you for 3% of all damage dealt, Each kill heals 1.0 health\\n\\nEnchantment: Minor Warding\\n+1% damage reduction";hudTick();hudCall("AZSXUIHudEquipment1","OnEnter");assert(GameTooltip.lines[1][1]=="+31% max health" and GameTooltip.lines[2][1]=="+5% damage reduction" and GameTooltip.lines[3][1]=="Heals you for 3% of all damage dealt" and GameTooltip.lines[4][1]=="Each kill heals 1.0 health" and GameTooltip.lines[5][1]=="" and GameTooltip.lines[6][1]=="Enchantment: Minor Warding" and GameTooltip.lines[7][1]=="+1% damage reduction")')
    def test_hovered_item_tooltip_hides_on_changed_or_missing_owner_data(self):
        self.run_lua('hudCall("AZSXUIHudEquipment1","OnEnter");assert(not GameTooltip.hidden);view.texts.equipment[1].text="Changed owner effect";hudTick();assert(GameTooltip.hidden);hudCall("AZSXUIHudEquipment1","OnEnter");assert(not GameTooltip.hidden);view.available=false;hudTick();assert(GameTooltip.hidden)')
    def test_partial_native_categories_reserve_all_five_equipment_positions(self):
        self.run_lua('view.data.equipment={{slot=1,rank=2,category="ring"},{slot=2,rank=1,category="trinket"}};view.texts.equipment={{name="Ring",icon="INV_Ring",text="Ring owner"},{name="Trinket",icon="INV_Trinket",text="Trinket owner"}};hudTick();local r=AZSXUI_HudReadout().equipment;assert(r[1]==nil and r[2].name=="Ring" and r[3]==nil and r[4].name=="Trinket" and r[5]==nil);hudCall("AZSXUIHudEquipment2","OnEnter");assert(GameTooltip.title=="Ring");view.data.equipment[1].category="amulet";hudTick();assert(AZSXUI_HudReadout().equipment[1].name=="Ring" and AZSXUI_HudReadout().equipment[2]==nil)')
    def test_native_equipment_text_is_verbatim_and_missing_data_restores_native_build(self):
        self.run_lua('assert(not hudPrefs.build);hudCall("AZSXUIHudEquipment1","OnEnter");assert(GameTooltip.title=="Native ring" and GameTooltip.right=="Rank 2" and GameTooltip.lines[1][1]=="Exact native tooltip" and GameTooltip.lines[2][1]=="+10 damage");view.available=false;hudTick();assert(hudPrefs.build and not AZSXUIHudEquipment:IsShown());assert(AZSXUI_HudReadout().equipment[1]==nil)')
    def test_toggle_restores_owned_anchors_and_preferences_but_preserves_external_changes(self):
        self.run_lua('ActionButton2:ClearAllPoints();ActionButton2:SetPoint("TOP",UIParent,"TOP",123,9);assert(AZSXUI_ConfigureHud("off"));assert(not AZSXUI_HudReadout().enabled and hudPrefs.build and hudPrefs.tally and MainMenuExpBar:GetWidth()==393);assert(ActionButton1.anchor[2]==MainMenuBarArtFrame);assert(ActionButton2.anchor[1]=="TOP" and ActionButton2.anchor[4]==123);assert(not AZSXUI_ConfigureHud("bad"))')
    def test_missing_values_are_not_invented_and_retained_frames_do_not_allocate_each_tick(self):
        self.run_lua('local count=allocations;AZS_RUN.chest=nil;hudTick(40);assert(allocations==count and hudPrefs.tally);phase="ready";hudTick();assert(AZSXUI_HudReadout().enabled);assert(AZSXUIHudRoot:IsShown());assert(not AZSXUIHudEquipment:IsShown());assert(not AZSXUIHudClock:IsShown());ToggleCharacter("PaperDollFrame");hudTick();assert(AZSXUI_HudReadout().rows[26].value=="—")')
if __name__=='__main__':unittest.main()
