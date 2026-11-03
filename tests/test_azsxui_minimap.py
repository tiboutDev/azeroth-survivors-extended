"""AZSXUI marker ownership, native cache API and lifecycle under Lua 5.1."""
from pathlib import Path
import unittest
from test_official_party_lua import STUB, LuaRuntime
ROOT=Path(__file__).resolve().parents[1]

@unittest.skipUnless(LuaRuntime,"AZSXUI profile requires Lua 5.1")
class MinimapTests(unittest.TestCase):
    def setUp(self):
        self.lua=LuaRuntime()
        self.lua.execute(STUB)
        self.lua.execute('Minimap=CreateFrame("Frame","Minimap",UIParent);Minimap:SetWidth(200);Minimap:SetHeight(200);function Minimap:GetZoom()return 0 end;map={identity="1:11",x=10,y=20,radii={100,80,60,40,20,10},pingX=0,pingY=0};custom={};solo=nil;soloText=nil;function AZSX_GetAPIVersion()return 1,0 end;function AZSX_GetState()return {available=true,data={minimap=map,markers=custom,soloEvent=solo},texts={soloEvent=soloText}}end;AZS_RUN={chest=250};AZS_CHESTS={};function tick()now=now+.2;objects.AZSXUIMinimapController.scripts.OnUpdate()end')
        self.lua.execute((ROOT/'addons/AZSXUI/Minimap.lua').read_text())
    def run_lua(self,source): self.lua.execute(source)
    def test_escort_native_icon_pin_tracks_position_and_retires(self):
        self.run_lua('solo={event=7,kind="chicken_escort",stage=0,x=10,y=20};soloText={title="Mechanical Chicken Escort",note="Find the chicken"};tick();local p=AZSXUI_MinimapReadout();assert(table.getn(p)==1 and p[1].kind==4 and p[1].priority==4 and string.find(objects.AZSXEventPin1Icon.texture,"INV_Misc_EngGizmos_06",1,true));GameTooltip.lines={};GameTooltip.AddLine=function(self,text)table.insert(self.lines,text)end;enter("AZSXEventPin1");assert(GameTooltip.lines[1]=="Mechanical Chicken Escort" and GameTooltip.lines[2]=="Find the chicken");solo.x=20;solo.stage=2;tick();assert(AZSXUI_MinimapReadout()[1].y==10);solo.stage=3;tick();assert(table.getn(AZSXUI_MinimapReadout())==0);solo.stage=0;solo.x=0/0;tick();assert(table.getn(AZSXUI_MinimapReadout())==0)')

    def test_demon_native_icon_pin_retires_after_release(self):
        self.run_lua('solo={event=8,kind="banished_demon",stage=0,x=10,y=20};soloText={title="Banished Demon",note="Break the binding"};tick();local p=AZSXUI_MinimapReadout();assert(table.getn(p)==1 and p[1].kind==5 and p[1].priority==4 and string.find(objects.AZSXEventPin1Icon.texture,"Spell_Shadow_Cripple",1,true));solo.stage=3;tick();assert(table.getn(AZSXUI_MinimapReadout())==0)')
    def test_native_player_arrow_remains_unique_and_overlapping_pin_keeps_its_tooltip(self):
        self.run_lua('AZS_RUN.px=10;AZS_RUN.py=20;AZS_RUN.pf=0;AZS_CHESTS={{id=1,x=10,y=20}};tick();local pin=objects.AZSXEventPin1;assert(not objects.AZSXUIPlayerMinimapArrow and pin.alpha==.15 and pin:IsShown());enter("AZSXEventPin1");assert(GameTooltip.detail=="Next cache: 250 gold");AZS_RUN.px=40;tick();assert(pin.alpha==1)')
    def test_overlap_follows_live_player_without_moving_or_reallocating_the_pin(self):
        self.run_lua('sample={apiVersion=1,x=10,y=20,facing=0};function AZS_GetPlayerPosition()return sample end;custom={{event=1,kind=1,x=10,y=20,left=0}};tick();local pin=objects.AZSXEventPin1;local n=allocations;sample.x=25;sample.facing=1;now=now+1/60;objects.AZSXUIMinimapController.scripts.OnUpdate();assert(pin.alpha>.15 and pin.alpha<1 and allocations==n and AZSXUI_MinimapReadout()[1].y==0)')
    def test_foreign_or_missing_position_restores_pin_opacity(self):
        self.run_lua('custom={{event=1,kind=1,x=10,y=20,left=0}};AZS_ZONE={map=1,run="current"};sample={apiVersion=1,mapId=1,runId="current",x=10,y=20,facing=0};function AZS_GetPlayerPosition()return sample end;tick();local pin=objects.AZSXEventPin1;assert(pin.alpha==.15);sample.mapId=2;tick();assert(pin.alpha==1);sample.mapId=1;sample.runId="old";tick();assert(pin.alpha==1);sample.runId="current";sample.x=0/0;tick();assert(pin.alpha==1)')
    def test_native_caches_and_custom_events_share_art_projection_and_separate_identities(self):
        self.run_lua('custom={{event=1,kind=0,x=60,y=20,left=60},{event=2,kind=1,x=10,y=30,left=0}};AZS_CHESTS={{id=4294967386,x=60,y=20,z=0}};tick();local pins=AZSXUI_MinimapReadout();assert(table.getn(pins)==3 and pins[3].event==4294967386 and pins[1].kind~=pins[3].kind);assert(pins[1].x==pins[3].x and pins[1].y==pins[3].y);assert(objects.AZSXEventPin3.icon.portraitMasked and objects.AZSXEventPin3:GetWidth()==20 and objects.AZSXEventPin3.icon.texture~=objects.AZSXEventPin1.icon.texture);enter("AZSXEventPin3");assert(GameTooltip.detail=="Next cache: 250 gold")')
    def test_overlapping_caches_count_split_on_zoom_and_retire_without_hiding_supplies(self):
        self.run_lua("""
        AZS_CHESTS={{id=1,x=10,y=20},{id=2,x=24,y=20},{id=3,x=14,y=20}}
        custom={{event=8,kind=0,x=10,y=20,left=30}};tick()
        local pins=AZSXUI_MinimapReadout()
        assert(table.getn(pins)==2 and pins[2].quantity==3 and objects.AZSXEventPin2.badge:IsShown())
        function Minimap:GetZoom()return 5 end;tick()
        assert(table.getn(AZSXUI_MinimapReadout())==4 and not objects.AZSXEventPin2.badge:IsShown())
        function Minimap:GetZoom()return 0 end;AZS_CHESTS[3]=nil;tick()
        assert(AZSXUI_MinimapReadout()[2].quantity==2)
        """)
    def test_cache_and_supply_icons_are_distinct_native_resources(self):
        self.run_lua('custom={{event=1,kind=0,x=10,y=20,left=60}};AZS_CHESTS={{id=1,x=10,y=20}};tick()')
        supply=self.lua.eval('objects.AZSXEventPin1.icon.texture')
        cache=self.lua.eval('objects.AZSXEventPin2.icon.texture')
        self.assertTrue(supply.startswith("Interface"+chr(92)+"Icons"+chr(92)))
        self.assertTrue(cache.startswith("Interface"+chr(92)+"Icons"+chr(92)))
        self.assertNotEqual(supply,cache)
        self.assertTrue(cache.endswith("INV_Box_02"))
    def test_guardian_replaces_native_flag_with_native_icon_and_restores_preference(self):
        self.run_lua('flag=true;function AZS_IsHudShown()return flag end;function AZS_SetHudShown(k,v)flag=v end;lair={index=1,x=60,y=20,name="Guardian",active=true};function AZS_GetFlaggedLair()return lair end;tick();local pins=AZSXUI_MinimapReadout();assert(table.getn(pins)==1 and pins[1].kind==3 and not flag);assert(objects.AZSXEventPin1.icon.texture=="Interface\\\\Icons\\\\Ability_Warrior_Challange");AZSXUIDB={pins=false};tick();assert(flag and table.getn(AZSXUI_MinimapReadout())==0);flag=false;AZSXUIDB.pins=true;tick();AZSXUIDB.pins=false;tick();assert(not flag)')
    def test_all_active_lairs_are_shown_with_stable_semantic_layers(self):
        self.run_lua('custom={{event=1,kind=0,x=10,y=20,left=60},{event=2,kind=1,x=10,y=20,left=0}};AZS_CHESTS={{id=9,x=10,y=20}};function AZS_GetLairs()return {{index=1,name="First",active=true,x=10,y=20},{index=2,name="Second",active=true,x=60,y=20},{index=3,name="Returning",active=false,left=30,x=10,y=30},{index=4,active=false,left=61,x=10,y=30},{index=2,active=true,x=10,y=20}}end;tick();local p=AZSXUI_MinimapReadout();assert(table.getn(p)==6);assert(p[2].level>p[4].level and p[4].level>p[1].level and p[1].level>p[3].level and p[3].level>p[6].level);custom={};AZS_CHESTS={};tick();p=AZSXUI_MinimapReadout();assert(p[1].priority==3 and p[3].priority==0)')
    def test_native_purchase_removes_only_its_pin_and_reuses_widgets(self):
        self.run_lua('custom={{event=1,kind=1,x=60,y=20,left=0}};AZS_CHESTS={{id=1,x=10,y=20}};tick();local n=allocations;enter("AZSXEventPin2");AZS_CHESTS={};tick();assert(table.getn(AZSXUI_MinimapReadout())==1 and not objects.AZSXEventPin2.shown and GameTooltip.hidden);AZS_CHESTS={{id=2,x=10,y=20}};tick();assert(allocations==n)')
    def test_menu_expiry_disable_and_missing_bridge_hide_every_pin(self):
        self.run_lua('AZS_CHESTS={{id=1,x=10,y=20}};tick();for _,phaseName in ipairs({"menu","ended"})do phase=phaseName;tick();assert(table.getn(AZSXUI_MinimapReadout())==0)end;phase="playing";tick();assert(table.getn(AZSXUI_MinimapReadout())==1);AZSXUIDB={pins=false};tick();assert(table.getn(AZSXUI_MinimapReadout())==0);AZSXUIDB.pins=true;tick();AZSX_GetState=nil;tick();assert(table.getn(AZSXUI_MinimapReadout())==0)')
    def test_paused_and_choosing_preserve_available_caches(self):
        self.run_lua('AZS_CHESTS={{id=1,x=10,y=20}};for _,p in ipairs({"playing","paused","choosing"})do phase=p;tick();assert(table.getn(AZSXUI_MinimapReadout())==1)end')
    def test_invalid_native_cache_coordinates_are_ignored_and_view_is_bounded(self):
        self.run_lua('AZS_CHESTS={{id=1,x=0/0,y=0},{id=2,x=math.huge,y=0},{id=-1,x=0,y=0},{id=4,x="0",y=0}};tick();assert(table.getn(AZSXUI_MinimapReadout())==0);AZS_CHESTS={};for i=1,200 do AZS_CHESTS[i]={id=i,x=10,y=20}end;tick();assert(table.getn(AZSXUI_MinimapReadout())==1 and AZSXUI_MinimapReadout()[1].quantity==128)')
    def test_missing_or_invalid_live_zoom_hides_pins_without_using_payload_fallback(self):
        self.run_lua('AZS_CHESTS={{id=1,x=10,y=20}};map.zoom=0;tick();assert(table.getn(AZSXUI_MinimapReadout())==1);Minimap.GetZoom=nil;tick();assert(table.getn(AZSXUI_MinimapReadout())==0);for _,bad in ipairs({-1,6,.5,0/0})do function Minimap:GetZoom()return bad end;tick();assert(table.getn(AZSXUI_MinimapReadout())==0);arg2=.1;arg3=.2;objects.AZSXUIMinimapController.scripts.OnEvent()end;function Minimap:GetZoom()return 0 end;tick();assert(table.getn(AZSXUI_MinimapReadout())==1)')
    def test_missing_projection_is_not_replaced_with_hero_coordinates(self):
        self.run_lua('AZS_RUN.px=10;AZS_RUN.py=20;AZS_CHESTS={{id=1,x=10,y=20}};map=nil;tick();assert(table.getn(AZSXUI_MinimapReadout())==0)')
    def test_new_session_retires_previous_local_ping(self):
        self.run_lua('AZS_CHESTS={{id=1,x=60,y=20}};tick();arg2=.1;arg3=.2;objects.AZSXUIMinimapController.scripts.OnEvent();arg2=nil;arg3=nil;function Minimap:GetPingPosition()return -.1,.2 end;tick();assert(math.abs(AZSXUI_MinimapReadout()[1].y-50)<.01);map={identity="2:11",x=10,y=20,radii={100,80,60,40,20,10},pingX=0,pingY=0};function Minimap:GetPingPosition()return .1,-.05 end;tick();assert(math.abs(AZSXUI_MinimapReadout()[1].y-50)<.01)')

if __name__=='__main__':unittest.main()
