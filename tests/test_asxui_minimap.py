"""ASXUI marker ownership, native cache API and lifecycle under Lua 5.1."""
from pathlib import Path
import unittest
from test_official_party_lua import STUB, LuaRuntime
ROOT=Path(__file__).resolve().parents[1]

@unittest.skipUnless(LuaRuntime,"ASXUI profile requires Lua 5.1")
class MinimapTests(unittest.TestCase):
    def setUp(self):
        self.lua=LuaRuntime()
        self.lua.execute(STUB)
        self.lua.execute('Minimap=CreateFrame("Frame","Minimap",UIParent);Minimap:SetWidth(200);Minimap:SetHeight(200);function Minimap:GetZoom()return 0 end;map={identity="1:11",x=10,y=20,radii={100,80,60,40,20,10},pingX=0,pingY=0};custom={};function ASX_GetAPIVersion()return 1,0 end;function ASX_GetState()return {available=true,data={minimap=map,markers=custom}}end;AZS_RUN={chest=250};AZS_CHESTS={};function tick()now=now+.2;objects.ASXUIMinimapController.scripts.OnUpdate()end')
        self.lua.execute((ROOT/'addons/ASXUI/Minimap.lua').read_text())
    def run_lua(self,source): self.lua.execute(source)
    def test_native_caches_and_custom_events_share_art_projection_and_separate_identities(self):
        self.run_lua('custom={{event=1,kind=0,x=60,y=20,left=60},{event=2,kind=1,x=10,y=30,left=0}};AZS_CHESTS={{id=4294967386,x=60,y=20,z=0}};tick();local pins=ASXUI_MinimapReadout();assert(table.getn(pins)==3 and pins[3].event==4294967386 and pins[1].kind~=pins[3].kind);assert(pins[1].x==pins[3].x and pins[1].y==pins[3].y);assert(objects.ASXEventPin3.icon.portraitMasked and objects.ASXEventPin3:GetWidth()==20 and objects.ASXEventPin3.icon.texture~=objects.ASXEventPin1.icon.texture);enter("ASXEventPin3");assert(GameTooltip.detail=="Next cache: 250 gold")')
    def test_cache_and_supply_icons_are_distinct_native_resources(self):
        self.run_lua('custom={{event=1,kind=0,x=10,y=20,left=60}};AZS_CHESTS={{id=1,x=10,y=20}};tick()')
        supply=self.lua.eval('objects.ASXEventPin1.icon.texture')
        cache=self.lua.eval('objects.ASXEventPin2.icon.texture')
        self.assertTrue(supply.startswith("Interface"+chr(92)+"Icons"+chr(92)))
        self.assertTrue(cache.startswith("Interface"+chr(92)+"Icons"+chr(92)))
        self.assertNotEqual(supply,cache)
        self.assertTrue(cache.endswith("INV_Box_02"))
    def test_guardian_replaces_native_flag_with_native_icon_and_restores_preference(self):
        self.run_lua('flag=true;function AZS_IsHudShown()return flag end;function AZS_SetHudShown(k,v)flag=v end;lair={index=1,x=60,y=20,name="Guardian",active=true};function AZS_GetFlaggedLair()return lair end;tick();local pins=ASXUI_MinimapReadout();assert(table.getn(pins)==1 and pins[1].kind==3 and not flag);assert(objects.ASXEventPin1.icon.texture=="Interface\\\\Icons\\\\Ability_Warrior_Challange");ASXUIDB={pins=false};tick();assert(flag and table.getn(ASXUI_MinimapReadout())==0);flag=false;ASXUIDB.pins=true;tick();ASXUIDB.pins=false;tick();assert(not flag)')
    def test_native_purchase_removes_only_its_pin_and_reuses_widgets(self):
        self.run_lua('custom={{event=1,kind=1,x=60,y=20,left=0}};AZS_CHESTS={{id=1,x=10,y=20}};tick();local n=allocations;enter("ASXEventPin2");AZS_CHESTS={};tick();assert(table.getn(ASXUI_MinimapReadout())==1 and not objects.ASXEventPin2.shown and GameTooltip.hidden);AZS_CHESTS={{id=2,x=10,y=20}};tick();assert(allocations==n)')
    def test_menu_expiry_disable_and_missing_bridge_hide_every_pin(self):
        self.run_lua('AZS_CHESTS={{id=1,x=10,y=20}};tick();for _,phaseName in ipairs({"menu","ended"})do phase=phaseName;tick();assert(table.getn(ASXUI_MinimapReadout())==0)end;phase="playing";tick();assert(table.getn(ASXUI_MinimapReadout())==1);ASXUIDB={pins=false};tick();assert(table.getn(ASXUI_MinimapReadout())==0);ASXUIDB.pins=true;tick();ASX_GetState=nil;tick();assert(table.getn(ASXUI_MinimapReadout())==0)')
    def test_paused_and_choosing_preserve_available_caches(self):
        self.run_lua('AZS_CHESTS={{id=1,x=10,y=20}};for _,p in ipairs({"playing","paused","choosing"})do phase=p;tick();assert(table.getn(ASXUI_MinimapReadout())==1)end')
    def test_invalid_native_cache_coordinates_are_ignored_and_view_is_bounded(self):
        self.run_lua('AZS_CHESTS={{id=1,x=0/0,y=0},{id=2,x=math.huge,y=0},{id=-1,x=0,y=0},{id=4,x="0",y=0}};tick();assert(table.getn(ASXUI_MinimapReadout())==0);AZS_CHESTS={};for i=1,200 do AZS_CHESTS[i]={id=i,x=10,y=20}end;tick();assert(table.getn(ASXUI_MinimapReadout())==128)')
    def test_missing_or_invalid_live_zoom_hides_pins_without_using_payload_fallback(self):
        self.run_lua('AZS_CHESTS={{id=1,x=10,y=20}};map.zoom=0;tick();assert(table.getn(ASXUI_MinimapReadout())==1);Minimap.GetZoom=nil;tick();assert(table.getn(ASXUI_MinimapReadout())==0);for _,bad in ipairs({-1,6,.5,0/0})do function Minimap:GetZoom()return bad end;tick();assert(table.getn(ASXUI_MinimapReadout())==0);arg2=.1;arg3=.2;objects.ASXUIMinimapController.scripts.OnEvent()end;function Minimap:GetZoom()return 0 end;tick();assert(table.getn(ASXUI_MinimapReadout())==1)')
    def test_missing_projection_is_not_replaced_with_hero_coordinates(self):
        self.run_lua('AZS_RUN.px=10;AZS_RUN.py=20;AZS_CHESTS={{id=1,x=10,y=20}};map=nil;tick();assert(table.getn(ASXUI_MinimapReadout())==0)')
    def test_new_session_retires_previous_local_ping(self):
        self.run_lua('AZS_CHESTS={{id=1,x=60,y=20}};tick();arg2=.1;arg3=.2;objects.ASXUIMinimapController.scripts.OnEvent();arg2=nil;arg3=nil;function Minimap:GetPingPosition()return -.1,.2 end;tick();assert(math.abs(ASXUI_MinimapReadout()[1].y-50)<.01);map={identity="2:11",x=10,y=20,radii={100,80,60,40,20,10},pingX=0,pingY=0};function Minimap:GetPingPosition()return .1,-.05 end;tick();assert(math.abs(ASXUI_MinimapReadout()[1].y-50)<.01)')

if __name__=='__main__':unittest.main()
