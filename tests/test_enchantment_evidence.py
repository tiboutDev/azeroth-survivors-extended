import importlib.util,json,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location("enchantment_evidence",Path(__file__).resolve().parents[1]/"tools/dev/enchantment_evidence.py");m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Evidence(unittest.TestCase):
 def sample(self):
  b=[1.,1.,100.,.05,1.,.08,.8,1.];t=""
  for name in m.NAMES:
   a=b.copy()
   if name=="Alacrity":a[1]=1/1.08
   elif name=="Fortitude":a[2]=120.
   else:
    k,v={"Might":(0,.15),"Precision":(3,.05),"Ferocity":(4,.35),"Warding":(5,.04),"Renewal":(6,1.5),"Reach":(7,.15)}[name];a[k]+=v
   t+=f"enchantment matrix: name={name} base=Ok({b}) actual=Ok({a}) repeat=Ok({a}) restored=true\n"
  for ring,name in[(0,"Might"),(1,"Reach")]:t+=f'enchantment locked: slot=0 guid=11 ring={ring} rank=2 name={name}\nenchantment fixture retained: [(0, 11, {ring}, "{name}", Ok(Some((3, 100))))]\n'
  t+="enchantment fixture rebuild: stable=true restored=true\n"
  native=['rogue: '+json.dumps(dict(choices=[dict(lines="Enchantment: Might\n+15% damage\nApplies enchantment"),dict(lines="Enchantment: Might\n+15% damage")]))]
  return t,native
 def test_complete_observations_required(self):
  h,n=self.sample();self.assertTrue(all(m.assess(h,n,1)["checks"].values()));self.assertFalse(m.assess(h.replace("name=Reach base=","absent base="),n,1)["checks"]["all_eight_native_bonuses"])
 def test_duplicate_lock_or_changed_later_enchantment_fails(self):
  h,n=self.sample();self.assertFalse(m.assess(h+'enchantment locked: slot=0 guid=11 ring=0 rank=3 name=Ferocity',n,1)["checks"]["one_lock_per_ring"]);self.assertFalse(m.assess(h.replace('0, "Might", Ok','0, "Ferocity", Ok'),n,1)["checks"]["later_upgrades_preserve_enchantment"])
 def test_later_rank_is_required_for_each_player_but_not_every_random_ring(self):
  h,n=self.sample();one_later=h.replace('1, "Reach", Ok(Some((3, 100)))','1, "Reach", Ok(Some((2, 100)))');self.assertTrue(m.assess(one_later,n,1)["checks"]["later_upgrades_preserve_enchantment"])
  no_later=one_later.replace('0, "Might", Ok(Some((3, 100)))','0, "Might", Ok(Some((2, 100)))');self.assertFalse(m.assess(no_later,n,1)["checks"]["later_upgrades_preserve_enchantment"])
 def test_accumulation_missing_guest_and_unrestored_input_fail(self):
  h,n=self.sample();self.assertFalse(m.assess(h.replace('stable=true','stable=false'),n,1)["checks"]["rebuilds_do_not_accumulate"]);self.assertFalse(m.assess(h,n,2)["checks"]["two_rings_per_player"]);self.assertFalse(m.assess(h.replace('restored=true','restored=false'),n,1)["checks"]["all_eight_native_bonuses"])

 def ranked_sample(self):
  h,n=self.sample();b=[1.,1.,100.,.05,1.,.08,.8,1.];h="\n".join(line for line in h.splitlines()if not line.startswith("enchantment matrix:"))
  for name,values in m.RANK_VALUES.items():
   for tier,v in enumerate(values):
    a=b.copy()
    if name=="Alacrity":a[1]=1/(1+v)
    elif name=="Fortitude":a[2]=100*(1+v)
    else:a[{"Might":0,"Precision":3,"Ferocity":4,"Warding":5,"Renewal":6,"Reach":7}[name]]+=v
    h+=f"\nenchantment ranked matrix: tier={tier} name={name} base=Ok({b}) actual=Ok({a}) repeat=Ok({a}) restored=true"
  for r,t in [(4,"Lesser"),(7,""),(10,"Greater"),(14,"Superior")]:h+=f"\nenchantment promoted: slot=0 guid=11 ring=0 rank={r} name={t+" "if t else""}Might"
  n[0]+='\nrogue: '+json.dumps(dict(choices=[dict(lines="Enchantment: Lesser Might\n+15% damage → +20% damage\nImproves enchantment")]),ensure_ascii=False)
  return h,n
 def test_all_rank_values_and_actual_promotions_are_required(self):
  h,n=self.ranked_sample();self.assertTrue(all(m.assess_ranked(h,n,1)["checks"].values()));self.assertFalse(m.assess_ranked(h.replace("tier=4 name=Reach","tier=7 name=Reach"),n,1)["checks"]["all_40_native_rank_values"])
  self.assertFalse(m.assess_ranked(h.replace("rank=14 name=Superior","rank=14 name=Absent"),n,1)["checks"]["all_four_promotions_observed"])

 def test_fixture_gold_must_converge_and_xp_must_always_match(self):
  def line(t,a,b):return f"{t:.3f}s official party: clock=1 actors=[phase: Playing, gold: {a[0]}, level: {a[1]}, xp: {a[2]}, phase: Playing, gold: {b[0]}, level: {b[1]}, xp: {b[2]}]"
  text="\n".join([line(1,(10,1,0),(10,1,0)),line(2,(22,1,0),(10,1,0)),line(3,(22,1,0),(22,1,0))]);self.assertTrue(all(m.assess_fixture_party(text)["checks"].values()))
  self.assertFalse(m.assess_fixture_party(text.replace('3.000s','9.000s'))["checks"]["gold_converges_after_fixture"])
  self.assertFalse(m.assess_fixture_party(text.replace('xp: 0','xp: 1',1))["checks"]["equal_progression"])
  self.assertFalse(m.assess_fixture_party(text.rsplit('\n',1)[0])["checks"]["gold_converges_after_fixture"])
