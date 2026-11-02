import json,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"tools/dev"))
from coop_ui_evidence import assess

def stream():
 rows=[]
 for clock,xp,gold in[(3,1.4,1.4),(7,1.4,1.5),(12,1.55,1.5),(30,1.55,1.5),(38,1.55,1.5),(42,1.55,1.5),(50,1.55,1.5)]:
  opened=clock in(30,42)
  rows.append("AZSX_DIAG "+json.dumps(dict(kind="ui.coop_hud",clock=clock,run_clock=clock,data=dict(phase="playing",available=True,sharedEconomy=dict(xp=xp,gold=gold),pinned="stats"if opened else None,uiActions=dict(stage={30:1,38:2,42:3,50:4}.get(clock,0)),character=dict(visible=opened),paperdoll=dict(visible=opened),hud=dict(xp="+55%",gold="+50%")if opened else{}))))
 return "\n".join(rows)
MARKERS="\n".join([f"coop economy fixture: stage={i} xp=1000"for i in(1,2,3)]+["native economy pool excluded: input=1152 output=1368","official inspection pause ignored: open=true"])
class CoopUiEvidence(unittest.TestCase):
 def test_complete_observations_require_actual_fixture_filter_and_both_clients(self):
  data=stream();self.assertTrue(assess(MARKERS+"\n"+data,data)["passed"])
  self.assertFalse(assess(data,data)["passed"]);self.assertFalse(assess(MARKERS+"\n"+data,"")["passed"])
 def test_transient_double_bonus_cannot_be_hidden_by_a_later_correct_value(self):
  data=stream();bad=data+"\nAZSX_DIAG "+json.dumps(dict(kind="ui.coop_hud",clock=40,run_clock=40,data=dict(phase="playing",available=True,sharedEconomy=dict(xp=1.8,gold=1.5))))
  self.assertFalse(assess(MARKERS+"\n"+data,bad)["passed"])
 def test_stats_only_or_a_real_host_menu_does_not_prove_the_requested_behavior(self):
  data=stream();self.assertFalse(assess(MARKERS+"\n"+data,data.replace('"pinned": "stats"','"pinned": null'))["passed"])
  self.assertFalse(assess(MARKERS+"\n"+data+"\nofficial menu request: open=true role=Host",data)["passed"])
