"""Explicit Lua 5.1 observer behavior; no native client or screenshot required."""
import json,unittest
from pathlib import Path
try:
 from lupa.lua51 import LuaRuntime
except ImportError:
 LuaRuntime=None
ROOT=Path(__file__).resolve().parents[1]
@unittest.skipIf(LuaRuntime is None,"Run explicitly with uv/lupa for required Lua 5.1 evidence")
class ObserverTests(unittest.TestCase):
 def setup_observer(self,enabled=True,limit=100):
  lua=LuaRuntime(unpack_returned_tuples=True);records=[];lua.globals().py_sink=records.append
  lua.execute("""now=0;widgets={};AZS_RUN={t=0};function GetTime()return now end
function getglobal(name)return widgets[name]end
function CreateFrame()return {SetScript=function(self,k,f)tick=f end}end
AZSXDiagnosticConfig={enabled="""+str(enabled).lower()+",session='unit',slot=0,max_records="+str(limit)+""",duration=10,sink=function(s)py_sink(s)end}
""")
  lua.execute((ROOT/'tools/dev/diagnostics/observer.lua').read_text())
  return lua,records
 def decoded(self,records):return [json.loads(row.split('AZSX_DIAG ',1)[1])for row in records]
 def test_visible_downed_ally_and_subsequent_hidden_marker(self):
  import sys
  sys.path.insert(0,str(ROOT/"tools/dev"))
  from diagnostic_records import revive_marker_cycle
  lua, records = self.setup_observer()
  lua.execute("""function AZS_GetPhase() return 'playing' end
function AZS_GetDownedPartyMembers() return friends end
friends={{name='Ally',anchorId='world-1',anchorStatus='ready'}}
widgets.AZSXUIReviveAnchor1={IsVisible=function()return true end,label={GetText=function()return 'Ally · 3 yd' end}}
now=1;tick()""")
  events=lambda: [json.loads(line.split("AZSX_DIAG ",1)[1]) for line in records]
  self.assertFalse(revive_marker_cycle(events()))
  lua.execute("friends={};widgets.AZSXUIReviveAnchor1.IsVisible=function() return false end;now=2;tick()")
  self.assertTrue(revive_marker_cycle(events()))

 def test_disabled_does_nothing(self):
  lua,records=self.setup_observer(False);self.assertEqual(records,[]);self.assertIsNone(lua.globals().tick)
 def test_change_only_sampling_and_real_widget_values(self):
  lua,records=self.setup_observer();lua.execute("now=1;tick();now=1.1;tick();now=2;tick()")
  before=len(records);lua.execute("now=3;tick()");self.assertEqual(len(records),before)
  lua.execute("""widgets.AZSXSupplyDiscoveryToast={IsVisible=function()return true end,IsShown=function()return true end,GetWidth=function()return 388 end,reward={GetText=function()return 'Arcane Crystal' end}};now=4;tick()""")
  rows=self.decoded(records);last=[r for r in rows if r['kind']=='ui.supply_toast'][-1]
  self.assertEqual(last['data']['reward'],'Arcane Crystal');self.assertTrue(last['data']['visible']);self.assertEqual(last['data']['width'],388)
  self.assertEqual([r['sequence']for r in rows],list(range(1,len(rows)+1)))
 def test_demon_record_passes_the_strict_collector(self):
  import sys
  sys.path.insert(0,str(ROOT/"tools/dev"))
  from diagnostic_records import collect
  lua,records=self.setup_observer()
  lua.execute("""AZSXDiagnosticConfig.demon=true
function AZSX_GetState()return {available=true,data={soloEvent={kind='banished_demon',stage=1,inside=true,held=100}}}end
now=1;tick()
""")
  stream=collect("\n".join(records),"unit",0)
  self.assertTrue(stream["passed"],stream["errors"])
  self.assertTrue(any(r["kind"]=="azsx.demon" and r["data"]["event"]["inside"] for r in stream["records"]))
 def test_budget_and_expiry_detach_observer(self):
  lua,records=self.setup_observer(limit=4);lua.execute("now=1;tick();now=2;tick()")
  self.assertLessEqual(len(records),4);self.assertIsNone(lua.globals().tick)
  lua,records=self.setup_observer();lua.execute("now=11;tick()");self.assertIsNone(lua.globals().tick)
 def test_native_nil_is_hidden_and_numeric_one_is_visible(self):
  lua,records=self.setup_observer()
  lua.execute("math.huge=nil;widgets.AZSXSaygeTrialProgress={IsVisible=function()return nil end,IsShown=function()return 1 end};now=1;tick()")
  row=[r for r in self.decoded(records)if r['kind']=='ui.sayge_progress'][0]
  self.assertFalse(row['data']['visible']);self.assertTrue(row['data']['shown'])
 def test_text_escaping_is_valid_json(self):
  lua,records=self.setup_observer();lua.globals().value='A\\B"C\nD'
  lua.execute("widgets.AZSXSupplyDiscoveryToast={reward={GetText=function()return value end}};now=1;tick()")
  row=[r for r in self.decoded(records)if r['kind']=='ui.supply_toast'][0]
  self.assertEqual(row['data']['reward'],lua.globals().value)
if __name__=='__main__':unittest.main()
