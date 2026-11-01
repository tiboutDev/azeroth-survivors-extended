import json,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"tools/dev"))
from escape_evidence import assess
class NativeEscapeEvidence(unittest.TestCase):
 def test_ownership_without_native_key_request_does_not_pass(self):
  guest="official pause UI: "+json.dumps({"view":{"active":True,"local_menu":True}})
  self.assertFalse(assess("official peer menu: slot=1 open=true",guest)["passed"])
 def test_requires_guest_request_host_ack_and_matching_local_ownership(self):
  host="official peer menu: slot=1 open=true";request="official menu request: open=true role=Guest"
  view="official pause UI: "+json.dumps({"view":{"active":True,"local_menu":True}})
  self.assertTrue(assess(host,request+"\n"+view)["passed"])
  for h,g in [("",request+"\n"+view),(host,request),(host,"official menu request: open=false role=Guest\n"+view)]:self.assertFalse(assess(h,g)["passed"])
 def test_waiting_panel_and_malformed_ui_are_not_personal_menu_evidence(self):
  guest='official menu request: open=true role=Guest\nofficial pause UI: {bad}\nofficial pause UI: {"view":{"active":true,"local_menu":false}}'
  self.assertFalse(assess('official peer menu: slot=1 open=true',guest)["passed"])
