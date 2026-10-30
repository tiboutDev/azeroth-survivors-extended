import json,sys,tempfile,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/"tools/dev"))
from startup_profile import instrument_lua
from assess_startup_profile import assess
class StartupProfile(unittest.TestCase):
 def test_instrumentation_is_disposable_and_retains_activation_prefix(self):
  source=(ROOT/"native-ui/official-party.lua").read_text()
  profiled=instrument_lua(source)
  self.assertTrue(profiled.startswith("local asxNativeActive = 0 -- ASX_LOCAL_UI_GATE"))
  self.assertNotIn("ASXStartupPerformance",source)
  self.assertIn("perf.refreshes=perf.refreshes+1",profiled)
  self.assertEqual(__import__("re").findall(r'observer:RegisterEvent\("([^"\n]+)"\)',source),[])
  self.assertIn('controller:RegisterEvent("MINIMAP_PING")',(ROOT/"addons/ASXUI/Minimap.lua").read_text())
  self.assertNotIn("perf.pin_ms=perf.pin_ms+",profiled)
  self.assertIn("if now-last >= interval then",profiled)
 def test_summary_uses_elapsed_windows_and_rejects_missing_hook_samples(self):
  with tempfile.TemporaryDirectory()as tmp:
   root=Path(tmp);player=root/"player-1";player.mkdir()
   log=player/"azeroth-coop-native.log";log.write_text("")
   with self.assertRaises(ValueError):assess(root)
   rows=lambda n:json.dumps([dict(name="file_filter",calls=n,ns=n*1000000,max_ns=1000000)])
   log.write_text("[ INFO ] 1.0s ASX startup profile: "+rows(1)+"\n[ INFO ] 3.0s ASX startup profile: "+rows(5))
   (root/"process-cpu.json").write_text(json.dumps([dict(t=1,slot=1,pid=10,cpu_seconds=.2),dict(t=2,slot=1,pid=10,cpu_seconds=1.2)]))
   result=assess(root)
   self.assertEqual(result["players"][0]["peak_windows"]["file_filter"]["ms_per_second"],2)
   self.assertEqual(result["cpu_peak_cores_per_client"]["1"],1)
if __name__=="__main__":unittest.main()
