"""Crystal pickup notices consume personal snapshot deltas without replay."""
from pathlib import Path
import unittest
from test_official_party_lua import STUB,LuaRuntime
ROOT=Path(__file__).resolve().parents[1]
@unittest.skipUnless(LuaRuntime,"Explicit ASXUI profile requires Lua 5.1")
class CrystalPopupTests(unittest.TestCase):
 def setUp(self):
  self.lua=LuaRuntime();self.lua.execute(STUB)
  self.lua.execute('''available=true;epoch="1";recipient="11";clock=10;phase="playing";crystals=nil;discovery=nil
  function AZS_GetPhase()return phase end;function AZS_GetRunTime()return clock end
  texts={};for _,key in ipairs({"red","blue","green","yellow"})do texts[key]={title=key.." ASX title",reward=key.." ASX effect",note="ASX permanence note",icon="native crystal icon",color={.12,1,0},titleColor={1,.25,.2}}end
  function ASX_GetAPIVersion()return 1,4 end
  function ASX_GetState()return {available=available,session={epoch=epoch,recipientGUID=recipient},data={crystals=crystals,discovery=discovery},texts={crystals={pickups=texts},supplies=discovery}}end
  function tick(dt)now=now+(dt or .2);objects.ASXUIEventsController.scripts.OnUpdate()end
  function pickup(key)if not crystals then crystals={red=0,blue=0,green=0,yellow=0}end;crystals[key]=crystals[key]+1 end
  function toast()return objects.ASXCrystalDiscoveryToast end''')
  self.lua.execute((ROOT/'addons/ASXUI/Events.lua').read_text())
 def test_personal_pickup_uses_asx_text_native_style_and_shared_anchor(self):
  self.lua.execute('tick();assert(not toast());pickup("red");tick();local f=toast();assert(f.shown and f.title.text==texts.red.title and f.reward.text==texts.red.reward and f.note.text==texts.red.note and f.icon.texture==texts.red.icon);assert(f.anchor[2]==objects.ASXUIEventAnchor and f.anchor[5]==0 and f.title.fontSize==14);assert(f.backdrop.edgeSize==16);tick(3.1);assert(not f.shown);for i=1,10 do tick()end;assert(not f.shown and crystals.red==1)')
 def test_restored_totals_do_not_announce_history(self):
  self.lua.execute('pickup("red");pickup("green");tick();assert(not toast());pickup("blue");tick();assert(toast().shown and toast().title.text==texts.blue.title)')
 def test_coalesced_colors_and_repeated_pickups_are_queued_once(self):
  self.lua.execute('tick();pickup("red");pickup("red");pickup("yellow");tick();assert(toast().title.text==texts.red.title);tick(3.1);assert(toast().shown and toast().title.text==texts.red.title);tick(3.1);assert(toast().shown and toast().title.text==texts.yellow.title);tick(3.1);assert(not toast().shown)')
 def test_stale_reconnect_and_recipient_change_discard_old_notices(self):
  self.lua.execute('tick();pickup("red");pickup("blue");tick();available=false;tick();assert(not toast().shown);pickup("red");available=true;tick();assert(not toast().shown);pickup("green");tick();assert(toast().shown and toast().title.text==texts.green.title);recipient="22";tick();assert(not toast().shown);pickup("yellow");tick();assert(toast().shown and toast().title.text==texts.yellow.title);epoch="2";tick();assert(not toast().shown)')
 def test_run_restart_and_regressing_totals_clear_pending_notices(self):
  self.lua.execute('tick();pickup("red");pickup("blue");tick();clock=0;tick();assert(not toast().shown);crystals=nil;tick();pickup("green");tick();assert(toast().shown and toast().title.text==texts.green.title)')
 def test_choices_and_pause_defer_readable_notice_without_replay(self):
  self.lua.execute('tick();phase="choosing";pickup("blue");tick();assert(not toast());phase="playing";tick();assert(toast().shown);phase="paused";tick(5);assert(not toast().shown);phase="playing";tick();assert(toast().shown);tick(3.1);assert(not toast().shown)')
 def test_hidden_notice_is_consumed_and_widget_preferences_are_independent(self):
  self.lua.execute('tick();assert(ASXUI_ConfigureWidget("crystals","show","off"));pickup("red");tick();assert(not toast().shown);tick(4);ASXUI_ConfigureWidget("crystals","show","on");assert(not toast().shown);ASXUI_ConfigureWidget("crystals","scale","1.2");ASXUI_ConfigureWidget("crystals","opacity","0.4");pickup("blue");tick();assert(toast().shown and toast().scale==1.2 and toast().alpha==.4);assert(ASXUI_ConfigureWidget("crystals","reset"));assert(toast().scale==1 and toast().alpha==1)')
 def test_missing_optional_pickup_texts_are_not_fabricated_or_replayed(self):
  self.lua.execute('tick();saved=texts;texts=nil;pickup("red");tick();assert(not toast());texts=saved;tick();assert(not toast());pickup("blue");tick();assert(toast().shown and toast().title.text==texts.blue.title)')
 def test_invalid_counts_fail_closed_and_burst_queue_is_bounded(self):
  self.lua.execute('tick();pickup("red");crystals.red=0/0;tick();assert(not toast());crystals.red=0;tick();crystals.red=1000000;tick();assert(toast().shown);for i=1,16 do tick(3.1)end;assert(not toast().shown)')
 def test_supplies_and_crystals_stack_without_sharing_receipts(self):
  self.lua.execute('tick();discovery={event=1,title="Supply",reward="ASX supply",note="supply note",icon="native supply",color={1,1,1}};pickup("red");tick();assert(objects.ASXSupplyDiscoveryToast.shown and toast().shown);assert(objects.ASXSupplyDiscoveryToast.anchor[5]==0 and toast().anchor[5]==-118);tick(4);assert(not toast().shown);discovery.event=2;tick();assert(objects.ASXSupplyDiscoveryToast.shown and not toast().shown)')
if __name__=='__main__':unittest.main()
