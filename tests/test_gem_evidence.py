import importlib.util,json,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location("gem_evidence",Path(__file__).resolve().parents[1]/"tools/dev/gem_evidence.py");m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Evidence(unittest.TestCase):
 def sample(self):
  host="gem fixture rebuild: stable=true restored=true\n";base=[1.,1.,100.,.05,1.,.08,.8,1.]
  for kind,value in enumerate(m.VALUES):
   a=base.copy()
   if kind==1:a[1]=1/(1+value)
   elif kind==2:a[2]=120.
   else:a[{0:0,3:3,4:4,5:5,6:6,7:7}[kind]]+=value
   host+=f"gem matrix: kind={kind} base=Ok({base}) actual=Ok({a}) repeat=Ok({a}) restored=true\n"
  for slot in range(2):host+=f"gem locked: slot={slot} guid={slot+10} amulet=0 previous=None name=Bold Ruby\ngem retained: slot={slot} guid={slot+10} amulet=1 previous=Some(0) name=Bold Ruby\n"
  return host,["gem native UI block: name=Bold Ruby\nrogue: "+json.dumps(dict(choices=[dict(kind="amulet",rank=2,lines="Gem: Bold Ruby\n+12% damage\nGem retained")]))]*2
 def test_complete_evidence_and_missing_matrix_owner_or_replacement(self):
  h,n=self.sample();self.assertTrue(all(m.assess(h,n,2)["checks"].values()))
  for before,after,key in [("kind=7","kind=99","all_eight_native_bonuses"),("gem locked: slot=1","ignored: slot=1","one_gem_lock_per_player"),("gem retained: slot=1","ignored: slot=1","every_player_replaces"),("restored=true","restored=false","rebuilds_stable_and_inputs_restored")]:self.assertFalse(m.assess(h.replace(before,after),n,2)["checks"][key])
 def test_changed_gem_and_absent_ui_are_rejected(self):
  h,n=self.sample();self.assertFalse(m.assess(h.replace("previous=Some(0) name=Bold Ruby","previous=Some(0) name=Renewing Emerald"),n,2)["checks"]["replacement_keeps_gem"]);self.assertFalse(m.assess(h,["",n[1]],2)["checks"]["native_card_annotation"])
