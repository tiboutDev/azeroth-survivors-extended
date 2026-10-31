"""Native health/aura ownership, allied build children and objective lifecycle."""
from pathlib import Path
import unittest
import test_asxui_hud as hud
ROOT=hud.ROOT

class FramesTests(unittest.TestCase):
    def run_lua(self,s):self.lua.execute(s)
    def setUp(self):
        hud.HUDTests.setUp(self)
        self.lua.execute('''
for i=1,4 do local p=objects["PartyMemberFrame"..i];_G["PartyMemberFrame"..i]=p;function p:GetFrameLevel()return 5 end;function p:GetNumPoints()return self.anchor and 1 or 0 end;function p:GetPoint()return unpack(self.anchor or{})end end
TargetFrame=CreateFrame("Frame","TargetFrame",UIParent)
function UnitHealth()return playerHP or 125 end
function UnitHealthMax()return 250 end
function TargetFrame:IsMouseEnabled()return self.mouse~=false end
function PlayerFrame:IsMouseEnabled()return self.mouse~=false end
Minimap=CreateFrame("Frame","Minimap",UIParent)
guardians={{name="Ancient Guardian",hp=100},{name="Fallen Guardian",hp=0}}
function AZS_GetGuardians()return guardians end
view.data.sayge={event=1,held=0,maximum=2000}
for i=0,3 do local f=CreateFrame("Button","BuffButton"..i,UIParent);f.buffFilter=i==3 and "HARMFUL"or"HELPFUL";f.buffIndex=i;f:SetWidth(32);f:SetHeight(32);f:SetPoint("TOP",UIParent,"TOP",i*40,0);f.nativeClick=function()end;f.scripts.OnClick=f.nativeClick;local t=CreateFrame("FontString","BuffButton"..i.."Duration",UIParent);t:SetPoint("TOP",f,"BOTTOM",0,0)end
''')
        self.lua.execute((ROOT/'addons/ASXUI/Theme.lua').read_text())
        self.lua.execute((ROOT/'addons/ASXUI/Frames.lua').read_text())
        self.lua.execute('arg1=.1;ASXUIFramesController.scripts.OnUpdate();ASXUI_DockAuras(view);arg1=.1;ASXUIFramesController.scripts.OnUpdate()')
    def test_hp_only_native_values_and_root_restoration(self):
        self.run_lua('assert(PlayerFrame:GetAlpha()==0 and TargetFrame:GetAlpha()==0);assert(ASXUIHudHealth.fraction==.5 and ASXUIHudHealth:GetWidth()==158);assert(ASXUIHudHealth:GetHeight()==22 and ASXUIHudHealthFill:GetWidth()==316 and ASXUIHudHealth.anchor[1]=="LEFT");playerHP=0;arg1=.1;ASXUIFramesController.scripts.OnUpdate();assert(ASXUIHudHealth.fraction==0 and not ASXUIHudHealth:IsShown());ASXUI_ConfigureHud("off");ASXUIFramesController.scripts.OnUpdate();assert(PlayerFrame:GetAlpha()==1 and TargetFrame:GetAlpha()==1 and PlayerFrame:IsMouseEnabled() and TargetFrame:IsMouseEnabled())')
    def test_pill_fill_clips_its_fixed_silhouette_at_low_full_and_missing_health(self):
        self.run_lua('assert(ASXUIHudHealthFillLeft1.portraitMasked and ASXUIHudHealthFillRight8.portraitMasked and ASXUIHudHealthFill.centerTexture.texture=="Interface\\\\TargetingFrame\\\\UI-StatusBar");playerHP=1;arg1=.1;ASXUIFramesController.scripts.OnUpdate();assert(math.abs(ASXUIHudHealth:GetWidth()-316/250)<.001 and ASXUIHudHealthFill:GetWidth()==316);playerHP=300;ASXUIFramesController.scripts.OnUpdate();assert(ASXUIHudHealth:GetWidth()==316);function UnitHealthMax()return nil end;ASXUIFramesController.scripts.OnUpdate();assert(not ASXUIHudHealth:IsShown() and ASXUI_FramesReadout().health=="—")')
    def test_auras_keep_native_ids_scripts_and_separate_harmful_row(self):
        self.run_lua('assert(BuffButton0.scripts.OnClick==BuffButton0.nativeClick);assert(BuffButton3.buffIndex==3 and BuffButton3.buffFilter=="HARMFUL");assert(BuffButton3.anchor[5]>BuffButton0.anchor[5]);ASXUI_ConfigureHud("off");assert(not ASXUI_DockAuras(view));assert(BuffButton0.anchor[2]==UIParent and BuffButton0Duration.anchor[1]=="TOP" and BuffButton0Duration.anchor[3]=="BOTTOM")')
    def test_compact_vitals_center_each_aura_row_without_shrinking_native_buttons(self):
        self.run_lua('assert(ASXUIHudVitals:GetWidth()==320 and ASXUI_GetHudAnchors().width==481);assert(BuffButton0.anchor[3]=="BOTTOM" and BuffButton0.anchor[4]==59);assert(BuffButton2.anchor[4]==-27 and BuffButton3.anchor[4]==16);for i=4,10 do local f=CreateFrame("Button","BuffButton"..i,UIParent);f:SetWidth(32);f:SetHeight(32);f.buffFilter="HELPFUL"end;ASXUI_DockAuras(view);assert(BuffButton9.anchor[4]==37.5 and BuffButton10.anchor[4]==-5.5 and BuffButton9.anchor[5]>BuffButton0.anchor[5]);assert(BuffButton0:GetWidth()==32 and BuffButton0.scripts.OnClick==BuffButton0.nativeClick);ASXUI_ConfigureHud("off");ASXUI_DockAuras(view);assert(BuffButton0.anchor[2]==UIParent and BuffButton0Duration.anchor[1]=="TOP" and BuffButton0Duration.anchor[3]=="BOTTOM")')
    def test_fixed_health_and_drawer_covering_the_upper_auras(self):
        self.run_lua('local before=ASXUIHudVitals.anchor[5];hudCall("ASXUIHudCombat","OnClick");hudTick();arg1=.1;ASXUIFramesController.scripts.OnUpdate();assert(ASXUIHudVitals.anchor[5]==before);assert(ASXUIHudAuras.anchor[2]==ASXUIHudVitals and ASXUIHudAuras.anchor[3]=="TOP");assert(BuffButton0Duration.anchor[1]=="BOTTOM" and BuffButton0Duration.anchor[2]==BuffButton0 and BuffButton0Duration.anchor[3]=="TOP");assert(ASXUIHudDrawer:GetFrameLevel()>BuffButton0:GetFrameLevel());local anchor,y=ASXUI_GetDrawerAnchor();assert(anchor==ASXUIHudVitals and y==12)')
    def test_objectives_only_include_living_guardians_and_available_ritual(self):
        self.run_lua('assert(ASXUIObjectives:IsShown() and ASXUIObjectives:GetHeight()==93);guardians={};view.data.sayge=nil;arg1=.1;ASXUIFramesController.scripts.OnUpdate();assert(not ASXUIObjectives:IsShown())')
    def test_guardians_group_names_but_deduplicate_native_entity_ids(self):
        self.run_lua('guardians={{id=1,name="Guardian",hp=10},{id=2,name="Guardian",hp=20},{id=2,name="Guardian",hp=20}};view.data.sayge=nil;arg1=.1;ASXUIFramesController.scripts.OnUpdate();assert(ASXUIObjectives:GetHeight()==65)')
    def test_held_lairs_include_distant_guardians_without_requiring_the_minimap(self):
        self.run_lua('function AZS_GetLairs()return {{active=true,id=99,name="Distant Guardian"},{active=false,id=0,name="Absent Guardian"}}end;guardians={};view.data.sayge=nil;Minimap:Hide();arg1=.1;ASXUIFramesController.scripts.OnUpdate();assert(ASXUIObjectives:IsShown() and ASXUIObjectives:GetHeight()==65 and ASXUI_FramesReadout().objectives[1]=="Defeat Distant Guardian")')
    def test_allied_buffs_stay_outside_build_frame_with_native_identity_and_tooltips(self):
        self.run_lua('local spell=CreateFrame("Button","ASXOfficialParty1Spell1",PartyMemberFrame1);spell.data={owner="22"};local handler=function()end;for i=1,9 do local b=CreateFrame("Button","ASXOfficialParty1Buff"..i,PartyMemberFrame1);b.unit="party1";b.buffIndex=i;b:SetPoint("LEFT",PartyMemberFrame1,"RIGHT",i*18,0);b.scripts.OnEnter=handler end;arg1=.1;ASXUIFramesController.scripts.OnUpdate();local panel=ASXUIAllyBuild1;assert(panel:GetWidth()==100 and panel:GetHeight()==49);assert(spell.anchor[3]=="TOPLEFT");assert(ASXOfficialParty1Buff1.anchor[2]==panel and ASXOfficialParty1Buff1.anchor[3]=="TOPRIGHT" and ASXOfficialParty1Buff1.anchor[4]==6);assert(ASXOfficialParty1Buff9.anchor[4]==6 and ASXOfficialParty1Buff9.anchor[5]<ASXOfficialParty1Buff1.anchor[5]);assert(ASXOfficialParty1Buff1.unit=="party1" and ASXOfficialParty1Buff1.buffIndex==1 and ASXOfficialParty1Buff1.scripts.OnEnter==handler);ASXUI_ConfigureHud("off");ASXUIFramesController.scripts.OnUpdate();assert(ASXOfficialParty1Buff1.anchor[2]==PartyMemberFrame1 and ASXOfficialParty1Buff1.anchor[3]=="RIGHT")')
    def test_allied_panels_retain_native_owner_tooltips_and_restore_anchors(self):
        self.run_lua('local b=CreateFrame("Button","ASXOfficialParty1Spell1",PartyMemberFrame1);b.data={owner="22",text="Owner native tooltip"};b:SetPoint("TOPLEFT",PartyMemberFrame1,"TOPRIGHT",4,0);local click=function()end;b.scripts.OnEnter=click;arg1=.1;ASXUIFramesController.scripts.OnUpdate();assert(ASXUIAllyBuild1:IsShown() and b.anchor[2]==ASXUIAllyBuild1 and b.data.text=="Owner native tooltip" and b.scripts.OnEnter==click);ASXUI_ConfigureHud("off");ASXUIFramesController.scripts.OnUpdate();assert(b.anchor[2]==PartyMemberFrame1)')

if __name__=='__main__':unittest.main()
