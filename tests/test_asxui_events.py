"""ASX-owned event text with independently configurable ASXUI rendering."""
from pathlib import Path
import unittest
from test_official_party_lua import STUB,LuaRuntime
ROOT=Path(__file__).resolve().parents[1]
@unittest.skipUnless(LuaRuntime,"Explicit ASXUI profile requires Lua 5.1")
class EventTests(unittest.TestCase):
    def setUp(self):
        self.lua=LuaRuntime();self.lua.execute(STUB);self.lua.execute("SlashCmdList={}")
        self.lua.execute('view={epoch="1",own="11",owner={discovery={event=1,title="ASX supplies title",icon="Interface/Icons/INV_Crate_01",reward="ASX reward wording",note="ASX exact receipt",color={1,.5,0}},ritual={event=2,held=700,inside=true,maximum=2000,title="ASX ritual title",note="ASX progress wording"}},blessing={boss=1,title="ASX fortune title",icon="Interface/Icons/INV_Misc_Orb_02",reward="ASX exact fortune",note="ASX fortune note",color={1,1,1}}};function ASX_GetAPIVersion()return 1,0 end;function ASX_GetState()return {available=true,session={epoch=view.epoch,recipientGUID=view.own},data={discovery=view.owner.discovery,sayge=view.owner.ritual,fortune=view.blessing},texts={supplies=view.owner.discovery,sayge=view.owner.ritual,fortune=view.blessing}}end;function tick()now=now+.2;objects.ASXUIEventsController.scripts.OnUpdate()end')
        self.lua.execute((ROOT/'addons/ASXUI/Events.lua').read_text());self.lua.execute('tick()')
    def run_lua(self,s):self.lua.execute(s)
    def test_popups_use_classic_wow_backdrop_and_font(self):
        self.run_lua('local f=objects.ASXSupplyDiscoveryToast;assert(f.backdrop.bgFile=="Interface\\\\Tooltips\\\\UI-Tooltip-Background" and f.backdrop.edgeFile=="Interface\\\\Tooltips\\\\UI-Tooltip-Border");assert(f.title.fontSize==14 and string.find(f.title.font,"FRIZQT__",1,true))')
    def test_all_semantic_text_is_displayed_verbatim_from_asx(self):
        self.run_lua('assert(objects.ASXSupplyDiscoveryToast.title.text==view.owner.discovery.title and objects.ASXSupplyDiscoveryToast.reward.text==view.owner.discovery.reward and objects.ASXSupplyDiscoveryToast.note.text==view.owner.discovery.note);assert(objects.ASXDarkmoonBlessingToast.reward.text==view.blessing.reward and objects.ASXDarkmoonBlessingToast.note.text==view.blessing.note);assert(objects.ASXSaygeTrialProgress.title.text==view.owner.ritual.title and objects.ASXSaygeTrialProgress.note.text==view.owner.ritual.note)')
    def test_position_scale_opacity_visibility_and_reset_are_per_widget(self):
        self.run_lua('assert(ASXUI_ConfigureWidget("supplies","position","120","-210"));assert(ASXUI_ConfigureWidget("supplies","scale","1.4"));assert(ASXUI_ConfigureWidget("supplies","opacity","0.45"));local f=objects.ASXSupplyDiscoveryToast;assert(math.abs(f.anchor[4]-120/1.4)<.001 and f.anchor[2]==objects.ASXUIEventAnchor and f.scale==1.4 and f.alpha==.45);assert(objects.ASXDarkmoonBlessingToast.alpha==1);assert(ASXUI_ConfigureWidget("sayge","show","off"));assert(not objects.ASXSaygeTrialProgress.shown and view.owner.ritual.held==700);assert(ASXUI_ConfigureWidget("sayge","show","on"));assert(objects.ASXSaygeTrialProgress.shown);assert(ASXUI_ConfigureWidget("supplies","reset"));assert(f.anchor[4]==0 and f.anchor[5]==-82 and f.scale==1 and f.alpha==1)')
    def test_slash_commands_apply_immediately_to_persistent_preferences(self):
        self.lua.execute((ROOT/'addons/ASXUI/ASXUI.lua').read_text())
        self.run_lua('SlashCmdList.ASXUI("widget supplies position 90 -190");SlashCmdList.ASXUI("widget supplies opacity 0.6");assert(objects.ASXSupplyDiscoveryToast.anchor[4]==90 and objects.ASXSupplyDiscoveryToast.alpha==.6);saved=ASXUIDB.widgets;SlashCmdList.ASXUI("widget sayge show off");assert(not objects.ASXSaygeTrialProgress.shown);SlashCmdList.ASXUI("widget sayge reset");assert(objects.ASXSaygeTrialProgress.shown);SlashCmdList.ASXUI("widget sayge scale bad");assert(ASXUIDB.widgets.sayge.scale==1)')
    def test_shared_anchor_stacks_scaled_visible_widgets_and_tracks_hud_top(self):
        self.run_lua('local a=objects.ASXUIEventAnchor;assert(a.anchor[2]==UIParent and a.anchor[4]==0 and a.anchor[5]==-85);local sayge=objects.ASXSaygeTrialProgress;local supply=objects.ASXSupplyDiscoveryToast;local fortune=objects.ASXDarkmoonBlessingToast;assert(sayge.anchor[2]==a and sayge.anchor[5]==0 and supply.anchor[5]==-82 and fortune.anchor[5]==-200);ASXUI_ConfigureWidget("sayge","scale","2");assert(supply.anchor[5]==-152 and fortune.anchor[5]==-270);ASXUI_ConfigureWidget("sayge","show","off");assert(supply.anchor[5]==0 and fortune.anchor[5]==-118);assert(ASXUI_ConfigureWidget("anchor","position","10","-100"));assert(a.anchor[4]==10 and a.anchor[5]==-100)')
    def test_fortune_does_not_replay_on_missing_data_new_epoch_or_new_trial(self):
        self.run_lua('now=now+10;tick();assert(not objects.ASXDarkmoonBlessingToast.shown);local getter=ASX_GetState;ASX_GetState=nil;tick();ASX_GetState=getter;view.epoch="2";view.owner.ritual.event=3;tick();assert(not objects.ASXDarkmoonBlessingToast.shown);view.blessing.boss=2;tick();assert(objects.ASXDarkmoonBlessingToast.shown)')
    def test_old_absolute_positions_migrate_to_relative_offsets(self):
        self.run_lua('ASXUIDB={widgets={supplies={x=20,y=-180},fortune={y=-260},sayge={y=-75}}};tick();assert(ASXUIDB.widgetLayoutVersion==2 and ASXUIDB.widgets.supplies.y==-40 and ASXUIDB.widgets.fortune.y==0 and ASXUIDB.widgets.sayge.y==0)')
    def test_invalid_inputs_do_not_change_valid_preferences(self):
        self.run_lua('assert(ASXUI_ConfigureWidget("sayge","scale","1.5"));for _,v in ipairs({"0","3","nan","inf","bad"})do assert(not ASXUI_ConfigureWidget("sayge","scale",v))end;assert(not ASXUI_ConfigureWidget("sayge","opacity","2"));assert(not ASXUI_ConfigureWidget("sayge","position","5000","0"));assert(not ASXUI_ConfigureWidget("other","show","off"));assert(ASXUIDB.widgets.sayge.scale==1.5)')
    def test_saved_corrupt_preferences_are_repaired(self):
        self.run_lua('ASXUIDB={widgets={sayge={scale=0/0,opacity=math.huge,x="bad",visible="yes"}}};tick();local p=ASXUIDB.widgets.sayge;assert(p.scale==1 and p.opacity==1 and p.x==0 and p.visible==true)')
    def test_visibility_consumes_notifications_without_replay(self):
        self.run_lua('ASXUI_ConfigureWidget("supplies","show","off");now=now+5;tick();ASXUI_ConfigureWidget("supplies","show","on");assert(not objects.ASXSupplyDiscoveryToast.shown);view.owner.discovery.event=2;tick();assert(objects.ASXSupplyDiscoveryToast.shown)')
    def test_missing_bridge_hides_all_widgets_without_fabricating_text(self):
        self.run_lua('ASX_GetState=nil;tick();assert(not objects.ASXSupplyDiscoveryToast.shown and not objects.ASXDarkmoonBlessingToast.shown and not objects.ASXSaygeTrialProgress.shown)')
    def test_unsupported_public_major_hides_views_without_using_legacy_bridge(self):
        self.run_lua('function ASX_GetEventPresentation()error("legacy bridge used")end;function ASX_GetAPIVersion()return 2,0 end;tick();assert(not objects.ASXSupplyDiscoveryToast.shown and not objects.ASXDarkmoonBlessingToast.shown and not objects.ASXSaygeTrialProgress.shown)')
    def test_new_recipient_retires_old_queue_and_progress_identity(self):
        self.run_lua('view.owner.discovery={event=2,title="Queued old",icon="icon",reward="Old",note="Old",color={1,1,1}};tick();view.epoch="2";view.own="22";view.owner.discovery={event=1,title="New owner",icon="icon",reward="New owner",note="New",color={1,1,1}};view.owner.ritual.held=100;tick();assert(objects.ASXSupplyDiscoveryToast.reward.text=="New owner");assert(objects.ASXSaygeTrialProgress.rendered==100)')
if __name__=='__main__':unittest.main()
