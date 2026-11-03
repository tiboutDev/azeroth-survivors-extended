"""Native health/aura ownership, allied build children and objective lifecycle."""
from pathlib import Path
import unittest
import test_azsxui_hud as hud
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
        self.lua.execute((ROOT/'addons/AZSXUI/Theme.lua').read_text())
        self.lua.execute((ROOT/'addons/AZSXUI/Frames.lua').read_text())
        self.lua.execute('arg1=.1;AZSXUIFramesController.scripts.OnUpdate();AZSXUI_DockAuras(view);arg1=.1;AZSXUIFramesController.scripts.OnUpdate()')
    def test_first_playing_update_applies_the_whole_hud(self):
        self.run_lua('''phase="menu";arg1=.001;AZSXUIHudController.scripts.OnUpdate()
assert(not AZSXUIHudVitals:IsShown() and PlayerFrame:GetAlpha()==1)
phase="playing";MainMenuBar:Hide();AZSXUIHudController.scripts.OnUpdate()
assert(AZSXUI_HudReadout().enabled and AZSXUIHudRoot:GetParent()==MainMenuBar)
MainMenuBar:Show();AZSXUIHudController.scripts.OnUpdate()
assert(AZSXUI_HudReadout().enabled and AZSXUIHudVitals:IsShown() and PlayerFrame:GetAlpha()==0)
assert(BuffButton0.anchor[2]==AZSXUIHudAuras and MainMenuExpBar:GetWidth()==437)
AZSXUI_ConfigureHud("off");AZSXUIHudController.scripts.OnUpdate()
assert(not AZSXUIHudVitals:IsShown() and PlayerFrame:GetAlpha()==1)''')

    def test_cooperative_information_is_first_and_returns_to_native_fallback(self):
        self.lua.execute((ROOT/'native-ui/official-party.lua').read_text().replace('local azsxNativeActive = 0 -- AZSX_LOCAL_UI_GATE','local azsxNativeActive = 1 -- AZSX_LOCAL_UI_GATE',1))
        self.lua.globals().wire="1|1|11|1\nM|11|0|506c61796572\nO|11|-|-\n".encode().hex()
        self.lua.execute('''AZS_RUN={azsx=wire,lvl=1};AZS_RUN_UPDATED=now;update();local info=AZSX_SetCooperativeRulesDocked("AZSXUI",true)
AZSXUI_DockAuras(AZSX_GetState());assert(info.anchor[2]==AZSXUIHudCounters and info.anchor[3]=="RIGHT" and info.anchor[4]>0)
assert(BuffButton0.scripts.OnClick==BuffButton0.nativeClick and info:GetWidth()==26)
AZSXUI_ConfigureHud("off");AZSXUI_DockAuras(AZSX_GetState());assert(info:IsShown() and info.anchor[2]==PartyMemberFrame1 and BuffButton0.anchor[2]==UIParent)
phase="ready";update();assert(not info:IsShown() and BuffButton0.anchor[2]==UIParent)''')

    def test_character_level_uses_native_run_value_and_clears_invalid_data(self):
        self.lua.execute('MainMenuBarExpText=MainMenuExpBar:CreateFontString();MainMenuBarExpText:SetText("21 / 27");MainMenuBarExpText:Show();AZS_RUN.lvl=42;arg1=.2;AZSXUIFramesController.scripts.OnUpdate();assert(AZSXUI_FramesReadout().level=="|cffffd578Level 42|r · 21 / 27" and MainMenuBarExpText:GetAlpha()==0 and AZSXUIExperienceLevel:IsShown() and AZSXUIExperienceLevel.anchor[2]==MainMenuExpBar);AZS_RUN.lvl=0/0;arg1=.2;AZSXUIFramesController.scripts.OnUpdate();assert(AZSXUI_FramesReadout().level=="" and MainMenuBarExpText:GetAlpha()==1 and not AZSXUIExperienceLevel:IsShown())')
    def test_hp_only_native_values_and_root_restoration(self):
        self.run_lua('assert(PlayerFrame:GetAlpha()==0 and TargetFrame:GetAlpha()==0);assert(AZSXUIHudHealth.fraction==.5 and AZSXUIHudHealth:GetWidth()==143.5);assert(AZSXUIHudHealth:GetHeight()==22 and AZSXUIHudHealthFill:GetWidth()==287 and AZSXUIHudHealth.anchor[1]=="LEFT");playerHP=0;arg1=.1;AZSXUIFramesController.scripts.OnUpdate();assert(AZSXUIHudHealth.fraction==0 and not AZSXUIHudHealth:IsShown());AZSXUI_ConfigureHud("off");AZSXUIFramesController.scripts.OnUpdate();assert(PlayerFrame:GetAlpha()==1 and TargetFrame:GetAlpha()==1 and PlayerFrame:IsMouseEnabled() and TargetFrame:IsMouseEnabled())')
    def test_pill_fill_clips_its_fixed_silhouette_at_low_full_and_missing_health(self):
        self.run_lua('assert(AZSXUIHudHealthFillLeft1.texture=="Interface\\\\CharacterFrame\\\\TempPortraitAlphaMask" and AZSXUIHudHealthFillRight22.texture==AZSXUIHudHealthFillLeft1.texture and AZSXUIHudHealthFill.centerTexture.texture=="Interface\\\\ChatFrame\\\\ChatFrameBackground");playerHP=1;arg1=.1;AZSXUIFramesController.scripts.OnUpdate();assert(math.abs(AZSXUIHudHealth:GetWidth()-287/250)<.001 and AZSXUIHudHealthFill:GetWidth()==287);playerHP=300;AZSXUIFramesController.scripts.OnUpdate();assert(AZSXUIHudHealth:GetWidth()==287);function UnitHealthMax()return nil end;AZSXUIFramesController.scripts.OnUpdate();assert(not AZSXUIHudHealth:IsShown() and AZSXUI_FramesReadout().health=="—")')
    def test_auras_keep_native_ids_scripts_and_separate_harmful_row(self):
        self.run_lua('assert(BuffButton0.scripts.OnClick==BuffButton0.nativeClick);assert(BuffButton3.buffIndex==3 and BuffButton3.buffFilter=="HARMFUL");assert(BuffButton3.anchor[5]>BuffButton0.anchor[5]);AZSXUI_ConfigureHud("off");assert(not AZSXUI_DockAuras(view));assert(BuffButton0.anchor[2]==UIParent and BuffButton0Duration.anchor[1]=="TOP" and BuffButton0Duration.anchor[3]=="BOTTOM")')
    def test_compact_vitals_center_each_aura_row_without_shrinking_native_buttons(self):
        self.run_lua('assert(AZSXUIHudVitals:GetWidth()==291 and AZSXUI_GetHudAnchors().width==437);assert(BuffButton0.anchor[3]=="BOTTOM" and BuffButton0.anchor[4]==59);assert(BuffButton2.anchor[4]==-27 and BuffButton3.anchor[4]==16);for i=4,10 do local f=CreateFrame("Button","BuffButton"..i,UIParent);f:SetWidth(32);f:SetHeight(32);f.buffFilter="HELPFUL"end;AZSXUI_DockAuras(view);assert(BuffButton9.anchor[4]==37.5 and BuffButton10.anchor[4]==-5.5 and BuffButton9.anchor[5]>BuffButton0.anchor[5]);assert(BuffButton0:GetWidth()==32 and BuffButton0.scripts.OnClick==BuffButton0.nativeClick);AZSXUI_ConfigureHud("off");AZSXUI_DockAuras(view);assert(BuffButton0.anchor[2]==UIParent and BuffButton0Duration.anchor[1]=="TOP" and BuffButton0Duration.anchor[3]=="BOTTOM")')
    def test_fixed_health_and_drawer_between_equipment_and_native_gryphons(self):
        self.run_lua('assert(AZSXUIHudVitals.anchor[2]==UIParent and AZSXUIHudVitals.anchor[5]==252);local before=AZSXUIHudVitals.anchor[5];hudCall("AZSXUIHudStats","OnClick");hudTick();arg1=.1;AZSXUIFramesController.scripts.OnUpdate();assert(AZSXUIHudVitals.anchor[5]==before);assert(AZSXUIHudAuras.anchor[2]==AZSXUIHudVitals and AZSXUIHudAuras.anchor[3]=="TOP");assert(BuffButton0Duration.anchor[1]=="BOTTOM" and BuffButton0Duration.anchor[2]==BuffButton0 and BuffButton0Duration.anchor[3]=="TOP");assert(AZSXUIHudDrawer:GetFrameLevel()>AZSXUIHudEquipment1:GetFrameLevel() and AZSXUIHudDrawer:GetFrameLevel()<MainMenuBarArtFrame:GetFrameLevel());local anchor,y=AZSXUI_GetDrawerAnchor();assert(anchor==AZSXUIHudBase and y==0 and AZSXUIHudDrawerMask.anchor[2]==AZSXUIHudBase)')
    def test_five_stats_rows_keep_the_raised_health_capsule_clear(self):
        self.run_lua("""
for _,key in ipairs({"dmg","cd","crit","cm","area","cnt","echo","bp","dots","doth","big","exe","cc","dotd","shd","auto","pet"})do AZS_STATS[key]=1.2 end
AZS_STATS.cd=.8;AZS_STATS.cnt=2;AZS_STATS.dots=2
hudCall("AZSXUIHudStats","OnClick");hudTick();arg1=.1;AZSXUIFramesController.scripts.OnUpdate()
assert(AZSXUIHudDrawer.visibleHeight>=133 and AZSXUIHudVitals.anchor[5]==252)
""")
    def test_bosses_and_awake_guardians_follow_top_hud_and_fail_open(self):
        self.run_lua('hudPrefs.bosses=true;bosses={{id=1,name="Boss",hp=30,hpm=100}};function AZS_GetBosses()return bosses end;guardians={{id=2,name="Guardian",hp=50,hpm=200,awake=true},{id=3,name="Sleeping",hp=100,hpm=100,awake=false}};arg1=.2;AZSXUIFramesController.scripts.OnUpdate();assert(not hudPrefs.bosses and AZSXUIEncounters:IsShown() and AZSXUIEncounter1.title.text=="Boss" and AZSXUIEncounter1.bar.value==.3 and AZSXUIEncounter2.title.text=="Guardian" and AZSXUIEncounter2.bar.value==.25);assert(AZSXUIEncounters.anchor[5]==-132 and AZSXUI_GetEncounterOffset()==206);bosses[1].hpm=nil;AZSXUIFramesController.scripts.OnUpdate();assert(hudPrefs.bosses and not AZSXUIEncounters:IsShown() and AZSXUI_GetEncounterOffset()==137);bosses[1].hpm=100;AZSXUIFramesController.scripts.OnUpdate();AZSXUI_ConfigureHud("off");AZSXUIFramesController.scripts.OnUpdate();assert(hudPrefs.bosses and not AZSXUIEncounters:IsShown())')
    def test_native_goblin_health_joins_encounters_without_selecting_other_elites(self):
        self.run_lua('hudPrefs.bosses=true;function AZS_GetBosses()return {}end;guardians={};AZS_UNITS={{kind="elite",id=123,name="Venture Co. Treasurer",hp=30,hpm=120},{kind="elite",id=124,name="Other elite",hp=50,hpm=100},{kind="normal",id=125,name="Venture Co. Treasurer",hp=50,hpm=100}};arg1=.2;AZSXUIFramesController.scripts.OnUpdate();assert(AZSXUIEncounters:IsShown() and AZSXUIEncounter1.title.text=="Venture Co. Treasurer" and AZSXUIEncounter1.bar.value==.25 and AZSXUI_GetEncounterOffset()==175);AZS_UNITS[1].hp=0;AZSXUIFramesController.scripts.OnUpdate();assert(not AZSXUIEncounters:IsShown());AZS_UNITS[1].hp=30;AZS_UNITS[1].hpm=nil;AZSXUIFramesController.scripts.OnUpdate();assert(hudPrefs.bosses and not AZSXUIEncounters:IsShown())')
    def test_native_debuffs_follow_each_encounter_and_clear_on_disappearance(self):
        self.run_lua("""
UIParent:SetWidth(1600)
hudPrefs.bosses=true
function AZS_GetBosses()return {{id=1,name="Boss",hp=50,hpm=100,debuffs={{name="Rend",icon="Ability_Gouge",stacks=2,remaining=8},{name="Curse",icon="Interface/Icons/Spell_Shadow_Curse",remaining=12}}}}end
guardians={{id=2,name="Guardian",hp=25,hpm=100,awake=true,debuffs={{name="Slow",icon="Spell_Frost_Slow",remaining=4}}}}
AZS_UNITS={{kind="elite",id=3,name="Venture Co. Treasurer",hp=20,hpm=100}}
function AZS_GetDebuffs(id)if id==3 then return {{name="Poison",icon="Ability_Poisons",stacks=1,remaining=6}}end return {}end
arg1=.1;AZSXUIFramesController.scripts.OnUpdate()
local r=AZSXUI_FramesReadout().encounters
assert(r.height==106 and r.rows[1].debuffs==2 and r.rows[2].debuffs==1 and r.rows[3].debuffs==1)
assert(string.find(AZSXUIEncounter1.debuffs[1].icon.texture,"Ability_Gouge",1,true))
assert(AZSXUIEncounter1.debuffs[1].icon==objects.AZSXUIEncounter1Debuff1Icon and AZSXUIEncounter1.debuffs[1].border==objects.AZSXUIEncounter1Debuff1Border and AZSXUIEncounter1.debuffs[1].border:IsShown())
assert(AZSXUIEncounter1.debuffs[2].icon.texture=="Interface/Icons/Spell_Shadow_Curse")
assert(AZSXUIEncounter1:GetWidth()==AZSXUIHudCounters:GetWidth())
assert(AZSXUIEncounter1.anchor[4]==-155 and AZSXUIEncounter2.anchor[4]==155 and AZSXUIEncounter3.anchor[4]==0 and AZSXUIEncounter3.anchor[5]==-53)
UIParent:SetWidth(1100);AZSXUIFramesController.scripts.OnUpdate()
assert(AZSXUIEncounter1.anchor[4]==-155 and AZSXUIEncounter2.anchor[4]==155 and AZSXUIEncounter3.anchor[4]==0 and AZSXUIEncounter3.anchor[5]==-53)
UIParent:SetWidth(800);AZSXUIFramesController.scripts.OnUpdate()
assert(AZSXUIEncounter2.anchor[5]==-53 and AZSXUIEncounter3.anchor[5]==-106)
this=AZSXUIEncounter1.debuffs[1];this.scripts.OnEnter();assert(GameTooltip.title=="Rend" and not GameTooltip.hidden)
AZS_TARGET={id=1};function UnitDebuff(unit,index)assert(unit=="target");if index==2 then return "Interface/Icons/Ability_Gouge"end end
this.scripts.OnEnter();assert(GameTooltip.title=="Native")
function UnitDebuff()return "Interface/Icons/Ability_Gouge.blp"end
this.scripts.OnEnter();assert(GameTooltip.title=="Rend")
function UnitDebuff()error("unavailable")end
this.scripts.OnEnter();assert(GameTooltip.title=="Rend")
AZS_TARGET.id=99;this.scripts.OnEnter();assert(GameTooltip.title=="Rend" and not GameTooltip.hidden)
local many={};for i=1,25 do many[i]={name="Effect "..i,icon="Ability_Gouge",remaining=10}end
AZS_UNITS={};guardians={};function AZS_GetBosses()return {{id=1,name="Boss",hp=50,hpm=100,debuffs=many}}end
AZSXUIFramesController.scripts.OnUpdate();assert(AZSXUIEncounter1.debuffs[13].anchor[5]==-25 and AZSXUIEncounter1.more:IsShown() and AZSXUIEncounter1.more:GetText()=="+9")
function AZS_GetBosses()return {}end
AZSXUIFramesController.scripts.OnUpdate();assert(not AZSXUIEncounters:IsShown() and not AZSXUIEncounter1.debuffs[1]:IsShown())
""")
    def test_encounter_overflow_preserves_complete_native_display(self):
        self.run_lua('hudPrefs.bosses=true;function AZS_GetBosses()local out={};for i=1,9 do out[i]={id=i,name="Boss",hp=1,hpm=2}end;return out end;guardians={};arg1=.2;AZSXUIFramesController.scripts.OnUpdate();assert(hudPrefs.bosses and not AZSXUIEncounters)')
    def test_solo_escort_objective_uses_provider_text(self):
        self.run_lua('''view.data.soloEvent={event=2,kind="chicken_escort",stage=2,progress=40,total=120}
view.texts.soloEvent={title="Mechanical Chicken Escort",note="Stay close to the chicken"}
arg1=.1;AZSXUIFramesController.scripts.OnUpdate()
local r=AZSXUI_FramesReadout();assert(r.objectives[table.getn(r.objectives)]=="Stay close to the chicken")
view.data.soloEvent=nil;arg1=.1;AZSXUIFramesController.scripts.OnUpdate()
local r=AZSXUI_FramesReadout();for _,v in ipairs(r.objectives)do assert(v~="Stay close to the chicken")end''')
    def test_spawned_bosses_appear_once_and_retire_individually(self):
        self.run_lua("""guardians={};view.data.sayge=nil
local first={id=11,name="First",hp=1,hpm=1};local second={id=12,name="Second",hp=1,hpm=1}
local list={first,first,second};function AZS_GetBosses()return list end
arg1=.2;AZSXUIFramesController.scripts.OnUpdate()
assert(AZSXUI_FramesReadout().objectives[1]=="Defeat First; Defeat Second")
first.hp=0;AZSXUIFramesController.scripts.OnUpdate();assert(AZSXUI_FramesReadout().objectives[1]=="Defeat Second")
list={};AZSXUIFramesController.scripts.OnUpdate();assert(AZSXUI_FramesReadout().objectives[1]=="No active objectives")""")
    def test_objectives_only_include_living_guardians_and_available_ritual(self):
        self.run_lua('assert(AZSXUIObjectives:IsShown() and AZSXUIObjectives:GetHeight()==84);guardians={};view.data.sayge=nil;arg1=.1;AZSXUIFramesController.scripts.OnUpdate();assert(AZSXUIObjectives:IsShown() and AZSXUI_FramesReadout().objectives[1]=="No active objectives")')
    def test_objective_tracker_follows_minimap_without_moving_allies_across_screen(self):
        self.run_lua('assert(AZSXUIObjectives.anchor[1]=="TOPRIGHT" and AZSXUIObjectives.anchor[2]==Minimap and AZSXUIObjectives.anchor[3]=="BOTTOMRIGHT");assert(PartyMemberFrame1.anchor[2]==UIParent and PartyMemberFrame1.anchor[1]=="TOPLEFT");MinimapCluster=CreateFrame("Frame","MinimapCluster",UIParent);arg1=.1;AZSXUIFramesController.scripts.OnUpdate();assert(AZSXUIObjectives.anchor[2]==MinimapCluster);MinimapCluster=nil;Minimap=nil;AZSXUIFramesController.scripts.OnUpdate();assert(AZSXUIObjectives.anchor[2]==UIParent and AZSXUIObjectives.anchor[4]==-20)')
    def test_upcoming_guardians_use_native_lair_countdowns_and_retire_when_held(self):
        self.run_lua('view.data.sayge=nil;function AZS_GetLairs()return lairs end;lairs={{state="empty",active=false,name="Next",left=45},{state="empty",active=false,name="Later",left=61}};arg1=.1;AZSXUIFramesController.scripts.OnUpdate();assert(AZSXUI_FramesReadout().objectives[1]=="Next: Incoming" and AZSXUI_FramesReadout().objectives[2]==nil);lairs[1]={state="asleep",active=true,name="Next",left=-1};AZSXUIFramesController.scripts.OnUpdate();assert(AZSXUI_FramesReadout().objectives[1]=="Defeat Next");lairs[1]={state="empty",active=false,name="Next",left=0};AZSXUIFramesController.scripts.OnUpdate();assert(AZSXUI_FramesReadout().objectives[1]=="Next: Arriving soon")')
    def test_guardian_objectives_share_one_quest_and_preserve_every_name(self):
        self.run_lua('view.data.sayge=nil;guardians={{id=1,name="First",hp=1},{id=2,name="Second",hp=2}};arg1=.1;AZSXUIFramesController.scripts.OnUpdate();local r=AZSXUI_FramesReadout();assert(table.getn(r.objectives)==1 and r.objectives[1]=="Defeat First; Defeat Second");assert(r.health=="125")')
    def test_encounter_names_are_centered_without_percentage_labels(self):
        self.run_lua('hudPrefs.bosses=true;function AZS_GetBosses()return {{id=1,name="Boss",hp=50,hpm=100}}end;guardians={};arg1=.1;AZSXUIFramesController.scripts.OnUpdate();assert(AZSXUIEncounter1.value==nil and AZSXUIEncounter1.title.anchor[1]=="CENTER" and AZSXUI_FramesReadout().encounters.rows[1].health==.5)')
    def test_arrival_timer_has_its_own_line_only_during_the_last_thirty_seconds(self):
        self.run_lua('view.data.sayge=nil;lair={state="empty",active=false,name="Next guardian",left=45};function AZS_GetLairs()return {lair}end;arg1=.1;AZSXUIFramesController.scripts.OnUpdate();local text=AZSXUI_FramesReadout().objectiveTexts[1];assert(string.find(text,"Await Next guardian",1,true) and not string.find(text,"45",1,true));lair.left=30;AZSXUIFramesController.scripts.OnUpdate();text=AZSXUI_FramesReadout().objectiveTexts[1];assert(string.find(text,"Await Next guardian (30 s)",1,true));lair.left=31;AZSXUIFramesController.scripts.OnUpdate();assert(not string.find(AZSXUI_FramesReadout().objectiveTexts[1],"Arrives in",1,true))')
    def test_guardians_group_names_but_deduplicate_native_entity_ids(self):
        self.run_lua('guardians={{id=1,name="Guardian",hp=10},{id=2,name="Guardian",hp=20},{id=2,name="Guardian",hp=20}};view.data.sayge=nil;arg1=.1;AZSXUIFramesController.scripts.OnUpdate();assert(AZSXUIObjectives:GetHeight()==59)')
    def test_held_lairs_include_distant_guardians_without_requiring_the_minimap(self):
        self.run_lua('function AZS_GetLairs()return {{active=true,id=99,name="Distant Guardian"},{active=false,id=0,name="Absent Guardian"}}end;guardians={};view.data.sayge=nil;Minimap:Hide();arg1=.1;AZSXUIFramesController.scripts.OnUpdate();assert(AZSXUIObjectives:IsShown() and AZSXUIObjectives:GetHeight()==59 and AZSXUI_FramesReadout().objectives[1]=="Defeat Distant Guardian")')
    def test_allied_buffs_stay_outside_build_frame_with_native_identity_and_tooltips(self):
        self.run_lua('local spell=CreateFrame("Button","AZSXOfficialParty1Spell1",PartyMemberFrame1);spell.data={owner="22"};local handler=function()end;for i=1,9 do local b=CreateFrame("Button","AZSXOfficialParty1Buff"..i,PartyMemberFrame1);b.unit="party1";b.buffIndex=i;b:SetPoint("LEFT",PartyMemberFrame1,"RIGHT",i*18,0);b.scripts.OnEnter=handler end;arg1=.1;AZSXUIFramesController.scripts.OnUpdate();local panel=AZSXUIAllyBuild1;assert(panel:GetWidth()==100 and panel:GetHeight()==49);assert(spell.anchor[3]=="TOPLEFT");assert(AZSXOfficialParty1Buff1.anchor[2]==panel and AZSXOfficialParty1Buff1.anchor[3]=="TOPRIGHT" and AZSXOfficialParty1Buff1.anchor[4]==6);assert(AZSXOfficialParty1Buff9.anchor[4]==6 and AZSXOfficialParty1Buff9.anchor[5]<AZSXOfficialParty1Buff1.anchor[5]);assert(AZSXOfficialParty1Buff1.unit=="party1" and AZSXOfficialParty1Buff1.buffIndex==1 and AZSXOfficialParty1Buff1.scripts.OnEnter==handler);AZSXUI_ConfigureHud("off");AZSXUIFramesController.scripts.OnUpdate();assert(AZSXOfficialParty1Buff1.anchor[2]==PartyMemberFrame1 and AZSXOfficialParty1Buff1.anchor[3]=="RIGHT")')
    def test_allied_panels_retain_native_owner_tooltips_and_restore_anchors(self):
        self.run_lua('local b=CreateFrame("Button","AZSXOfficialParty1Spell1",PartyMemberFrame1);b.data={owner="22",text="Owner native tooltip"};b:SetPoint("TOPLEFT",PartyMemberFrame1,"TOPRIGHT",4,0);local click=function()end;b.scripts.OnEnter=click;arg1=.1;AZSXUIFramesController.scripts.OnUpdate();assert(AZSXUIAllyBuild1:IsShown() and b.anchor[2]==AZSXUIAllyBuild1 and b.data.text=="Owner native tooltip" and b.scripts.OnEnter==click);AZSXUI_ConfigureHud("off");AZSXUIFramesController.scripts.OnUpdate();assert(b.anchor[2]==PartyMemberFrame1)')

if __name__=='__main__':unittest.main()
