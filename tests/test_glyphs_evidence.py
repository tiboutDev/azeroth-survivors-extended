import importlib.util,tempfile,unittest,json
from pathlib import Path
spec=importlib.util.spec_from_file_location("glyphs_evidence",Path(__file__).resolve().parents[1]/"tools/dev/glyphs_evidence.py");m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Evidence(unittest.TestCase):
 def test_missing_or_unrelated_logs_cannot_pass(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder)
   for i in(1,2):
    p=root/f"player-{i}";p.mkdir();(p/"azeroth-coop-native.log").write_text("");(p/"native-output.log").write_text('rogue: {"type":"choices","choices":[{"lines":"normal upgrade"}]}\n')
   self.assertFalse(any(m.assess(root)["checks"].values()))
 def test_wrong_native_transform_fails_even_if_a_log_exists(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder)
   for i in(1,2):
    p=root/f"player-{i}";p.mkdir();(p/"azeroth-coop-native.log").write_text("glyph fixture modifiers: slot=0 spell=845 glyph=Sweeping Cleave before=[1,1,0,1,1,0,1] after=Ok([1,1,2,1,1,0,1])\n");(p/"native-output.log").write_text("")
   self.assertFalse(m.assess(root)["checks"]["glyph_native_modifiers"])

 def test_removal_requires_the_native_rebuild_and_a_real_cast(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder)
   for i in(1,2):
    p=root/f"player-{i}";p.mkdir();(p/"azeroth-coop-native.log").write_text("");(p/"native-output.log").write_text("")
   host=root/"player-1/azeroth-coop-native.log"
   host.write_text("glyph fixture removal rebuilt: slot=0 spell=845 baseline=Some([1,1,0,1,1,0,1]) current=Ok([1,1,0,1,1,0,1])\n")
   checks=m.assess(root)["checks"];self.assertTrue(checks["glyph_removal_rebuilt"]);self.assertFalse(checks["glyph_removed_cast"])
   host.write_text(host.read_text()+"glyph fixture native cast: slot=0 spell=845 rank=4 glyph=Unglyphed modifiers=[1,1,0,1,1,0,1]\n")
   self.assertTrue(m.assess(root)["checks"]["glyph_removed_cast"])
   matching=host.read_text();host.write_text(matching.replace("native cast: slot=0", "native cast: slot=1"))
   self.assertFalse(m.assess(root)["checks"]["glyph_removed_cast"]);host.write_text(matching)
   host.write_text(host.read_text().replace("current=Ok([1,1,0,1,1,0,1])","current=Ok([1.4,1.5,0,1,1,0,1])"))
   self.assertFalse(m.assess(root)["checks"]["glyph_removal_rebuilt"])

 def test_complete_native_matrix_requires_correct_effect_channel_and_spell_identity(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder)
   for i in(1,2):
    p=root/f"player-{i}";p.mkdir();(p/"azeroth-coop-native.log").write_text("");(p/"native-output.log").write_text("")
   lines=[]
   for name,(d,c,t,r,l,e)in m.NAMES.items():
    b=[2,.5,4,1.2,1.3,.1,1.75];a=[2*d,.5*c,4+t,1.2*r,1.3*l,.1*d,1.75*e]
    lines.append(f"glyph fixture warrior matrix: slot=0 spell={m.SPELLS[name]} glyph={name} before=Ok({json.dumps(b)}) after=Ok({json.dumps(a)})\n")
   host=root/"player-1/azeroth-coop-native.log";host.write_text("".join(lines));self.assertTrue(m.assess(root)["checks"]["glyph_warrior_matrix"])
   host.write_text("".join(lines[:-1]));self.assertFalse(m.assess(root)["checks"]["glyph_warrior_matrix"])
   host.write_text("".join(lines).replace("spell=6673", "spell=635"));self.assertFalse(m.assess(root)["checks"]["glyph_warrior_matrix"])
   # Locate an effect-scaling row explicitly, independent of float repr details.
   for i,line in enumerate(lines):
    if "glyph=Fierce Rally " in line:
     prefix,after=line.split(" after=Ok(");values=json.loads(after[:-2]);values[6]=1.75;lines[i]=prefix+" after=Ok("+json.dumps(values)+")\n"
   host.write_text("".join(lines));self.assertFalse(m.assess(root)["checks"]["glyph_warrior_matrix"])

 def test_representative_cast_gate_accepts_attacks_plus_support_without_exact_old_spell_set(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder)
   for i in(1,2):
    p=root/f"player-{i}";p.mkdir();(p/"azeroth-coop-native.log").write_text("");(p/"native-output.log").write_text("")
   host=root/"player-1/azeroth-coop-native.log"
   buff="glyph fixture native cast: slot=1 spell=6673 rank=3 glyph=Lasting Rally modifiers=[1,1,0,1,1.5,0,1]\n"
   attack="glyph fixture native cast: slot=0 spell=23922 rank=2 glyph=Crushing Shield modifiers=[1.4,1,0,1,1,0,.6]\n"
   host.write_text(buff);checks=m.assess(root)["checks"];self.assertFalse(checks["glyph_successful_native_casts"]);self.assertTrue(checks["glyph_representative_support_cast"])
   host.write_text(buff+attack);checks=m.assess(root)["checks"];self.assertTrue(checks["glyph_successful_native_casts"]);self.assertTrue(checks["glyph_representative_support_cast"])
   host.write_text(attack);self.assertFalse(m.assess(root)["checks"]["glyph_representative_support_cast"])

 def test_paladin_matrix_requires_healing_square_without_health_damage_scaling(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder)
   for i in(1,2):
    p=root/f"player-{i}";p.mkdir();(p/"azeroth-coop-native.log").write_text("");(p/"native-output.log").write_text("")
   lines=[]
   for name,(d,c,t,r,l,e)in m.PALADIN_NAMES.items():
    b=[2,.5,4,1.2,1.3,.1,1.75];h=m.HEALING.get(name,1)if m.PALADIN_SPELLS[name]==635 else 1
    a=[2*d*h*h,.5*c,4+t,1.2*r,1.3*l,.1*d,1.75*e]
    lines.append(f"glyph fixture paladin matrix: slot=0 spell={m.PALADIN_SPELLS[name]} glyph={name} before=Ok({json.dumps(b)}) after=Ok({json.dumps(a)})\n")
   host=root/"player-1/azeroth-coop-native.log";host.write_text("".join(lines))
   self.assertTrue(m.assess(root,"paladin")["checks"]["glyph_warrior_matrix"])
   # Explicitly corrupt Holy Light's native damage word, leaving the heal text intact.
   for i,line in enumerate(lines):
    if 'glyph=Deep Prayer 'in line:
     prefix,after=line.split(' after=Ok(');a=json.loads(after[:-2]);a[0]=2*1.4;lines[i]=prefix+' after=Ok('+json.dumps(a)+')\n'
   host.write_text(''.join(lines));self.assertFalse(m.assess(root,"paladin")["checks"]["glyph_warrior_matrix"])

 def test_paladin_heals_require_both_routes_real_hp_and_both_aura_profiles(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder)
   for i in(1,2):
    p=root/f"player-{i}";p.mkdir();(p/"azeroth-coop-native.log").write_text("");(p/"native-output.log").write_text("")
   host=root/"player-1/azeroth-coop-native.log"
   text=("glyph fixture healing: game=1 spell=25912 delivery=false before=0.03 after=0.045 factor=1.5\n"
    "glyph fixture healing: game=1 spell=25912 delivery=true before=3 after=4.5 factor=1.5\n"
    "glyph fixture self heal applied: spell=25912 before_hp=98 after_hp=100 maximum=100 fraction=0.045\n"
    "glyph fixture holy shield aura: glyph=Barbed Aegis before_power=10 after_power=14 before_duration=10 after_duration=6.5\n"
    "glyph fixture holy shield aura: glyph=Lasting Aegis before_power=10 after_power=7.5 before_duration=10 after_duration=15\n")
   host.write_text(text);checks=m.assess(root,"paladin")["checks"]
   for key in('glyph_paladin_healing','glyph_paladin_self_hp','glyph_paladin_shield_auras'):self.assertTrue(checks[key])
   host.write_text(text.replace('delivery=false','delivery=true'));self.assertFalse(m.assess(root,"paladin")["checks"]['glyph_paladin_healing'])
   host.write_text(text.replace('after_hp=100','after_hp=99'));self.assertFalse(m.assess(root,"paladin")["checks"]['glyph_paladin_self_hp'])
   host.write_text(text.replace('after_power=14','after_power=10'));self.assertFalse(m.assess(root,"paladin")["checks"]['glyph_paladin_shield_auras'])

 def test_hunter_matrix_keeps_mark_vulnerability_and_explosion_has_own_evidence(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder)
   for i in(1,2):
    p=root/f"player-{i}";p.mkdir();(p/"azeroth-coop-native.log").write_text("");(p/"native-output.log").write_text("")
   lines=[]
   for name,(d,c,t,r,l,e)in m.HUNTER_NAMES.items():
    spell=m.HUNTER_SPELLS[name];b=[2,.5,4,1.2,1.3,.1,1.75];damage=1 if spell==1130 else d
    a=[2*damage,.5*c,4+t,1.2*r,1.3*l,.1*damage,1.75*e]
    lines.append(f"glyph fixture hunter matrix: slot=0 spell={spell} glyph={name} before=Ok({json.dumps(b)}) after=Ok({json.dumps(a)})\n")
   host=root/"player-1/azeroth-coop-native.log";text=''.join(lines);host.write_text(text)
   checks=m.assess(root,"hunter")["checks"];self.assertTrue(checks['glyph_warrior_matrix']);self.assertFalse(checks['glyph_hunter_mark_explosion'])
   hit='glyph fixture mark explosion: game=1 before=200 after=150 factor=0.75\n'
   host.write_text(text+hit);self.assertTrue(m.assess(root,"hunter")["checks"]['glyph_hunter_mark_explosion'])
   host.write_text(text+hit.replace('after=150','after=200'));self.assertFalse(m.assess(root,"hunter")["checks"]['glyph_hunter_mark_explosion'])
   host.write_text(''.join(lines[:-1])+hit);self.assertFalse(m.assess(root,"hunter")["checks"]['glyph_warrior_matrix'])

 def test_rogue_evidence_requires_all_profiles_buff_ratios_and_an_owned_seal_hit(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder)
   for i in(1,2):
    p=root/f"player-{i}";p.mkdir();(p/"azeroth-coop-native.log").write_text("");(p/"native-output.log").write_text("")
   lines=[]
   for name,(d,c,t,r,l,e)in m.ROGUE_NAMES.items():
    b=[2,.5,4,1.2,1.3,.1,1.75];a=[2*d,.5*c,4+t,1.2*r,1.3*l,.1*d,1.75*e]
    lines.append(f"glyph fixture rogue matrix: slot=0 spell={m.ROGUE_SPELLS[name]} glyph={name} before=Ok({json.dumps(b)}) after=Ok({json.dumps(a)})\n")
   for name in('Enduring Rush','Intense Rush','Lasting Edge','Keen Edge'):
    d,c,t,r,l,e=m.ROGUE_NAMES[name];lines.append(f"glyph fixture rogue aura: spell={m.ROGUE_SPELLS[name]} glyph={name} before_power=0.6 after_power={0.6*e} before_duration=10 after_duration={10*l}\n")
   hit='glyph fixture seal hit: source=Some((1, 222)) enemy=123 amount=100 glyph=Shared Fate\n'
   host=root/'player-1/azeroth-coop-native.log';text=''.join(lines);host.write_text(text+hit);checks=m.assess(root,'rogue')['checks']
   for key in('glyph_warrior_matrix','glyph_rogue_buff_auras','glyph_rogue_seal_hits'):self.assertTrue(checks[key])
   host.write_text(text+hit.replace('Shared Fate','Unglyphed'));self.assertFalse(m.assess(root,'rogue')['checks']['glyph_rogue_seal_hits'])
   host.write_text(text.replace('after_duration=6.5','after_duration=10')+hit);self.assertFalse(m.assess(root,'rogue')['checks']['glyph_rogue_buff_auras'])
   host.write_text(''.join(lines[1:])+hit);self.assertFalse(m.assess(root,'rogue')['checks']['glyph_warrior_matrix'])

 def test_priest_matrix_and_support_require_true_healing_ratios_and_both_routes(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder)
   for i in(1,2):
    p=root/f"player-{i}";p.mkdir();(p/"azeroth-coop-native.log").write_text("");(p/"native-output.log").write_text("")
   lines=[]
   for name,(d,c,t,r,l,e)in m.PRIEST_NAMES.items():
    spell=m.PRIEST_SPELLS[name];h=m.HEALING.get(name,1)if spell==139 else 1;b=[2,.5,4,1.2,1.3,.1,1.75];a=[2*d*h*h,.5*c,4+t,1.2*r,1.3*l,.1*d,1.75*e]
    lines.append(f"glyph fixture priest matrix: slot=0 spell={spell} glyph={name} before=Ok({json.dumps(b)}) after=Ok({json.dumps(a)})\n")
   for spell,name,factor,duration in [(17,'Deep Shelter',1.4,0),(17,'Fleeting Shelter',.6,0),(139,'Deep Renewal',1.4,10),(139,'Swift Renewal',.6,10)]:lines.append(f"glyph fixture priest support: spell={spell} glyph={name} before_power=100 after_power={100*factor} before_duration={duration} after_duration={duration}\n")
   lines+=['glyph fixture healing: game=1 spell=15237 delivery=false before=0.04 after=0.06 factor=1.5\n','glyph fixture healing: game=1 spell=15237 delivery=true before=4 after=6 factor=1.5\n','glyph fixture self heal applied: spell=15237 before_hp=90 after_hp=96 maximum=100 fraction=0.06\n','glyph fixture lightwell hit: source=Ok(Some((0, 111))) amount=100\n']
   host=root/'player-1/azeroth-coop-native.log';text=''.join(lines);host.write_text(text);checks=m.assess(root,'priest')['checks']
   for key in('glyph_warrior_matrix','glyph_priest_support_coefficients','glyph_priest_nova_healing','glyph_priest_nova_self_hp','glyph_priest_lightwell_hits'):self.assertTrue(checks[key])
   host.write_text(text.replace('after_power=140.0','after_power=100.0'));self.assertFalse(m.assess(root,'priest')['checks']['glyph_priest_support_coefficients'])
   host.write_text(text.replace('delivery=false','delivery=true'));self.assertFalse(m.assess(root,'priest')['checks']['glyph_priest_nova_healing'])
   host.write_text(text.replace('after_hp=96','after_hp=100'));self.assertFalse(m.assess(root,'priest')['checks']['glyph_priest_nova_self_hp'])

 def test_shaman_evidence_rejects_wrong_healing_shield_or_missing_totem(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder)
   for i in(1,2):
    p=root/f"player-{i}";p.mkdir();(p/"azeroth-coop-native.log").write_text("");(p/"native-output.log").write_text("")
   lines=[]
   for name,(d,c,t,r,l,e)in m.SHAMAN_NAMES.items():
    sp=m.SHAMAN_SPELLS[name];b=[2,.5,4,1.2,1.3,.1,1.75]
    if sp==8056:l*=e;e=1
    h=m.HEALING.get(name,1);a=[2*d*h*h,.5*c,4+t,1.2*r,1.3*l,.1*d,1.75*e]
    lines.append(f"glyph fixture shaman matrix: slot=0 spell={sp} glyph={name} before=Ok({json.dumps(b)}) after=Ok({json.dumps(a)})\n")
   for name in('Overcharged Shield','Volatile Shield'):
    d,c,t,r,l,e=m.SHAMAN_NAMES[name];lines.append(f"glyph fixture lightning shield aura: glyph={name} before_power=10 after_power={10*d} before_duration=15 after_duration={15*l}\n")
   for name in('Deep Waters','Swift Waters'):lines.append(f"glyph fixture shaman healing: glyph={name} before_amount=100 after_amount={100*m.HEALING[name]}\n")
   for sp in(8187,3606,16038):lines.append(f"glyph fixture shaman hit: spell={sp} source=Ok(Some((1, 111))) amount=100\n")
   host=root/'player-1/azeroth-coop-native.log';text=''.join(lines);host.write_text(text);checks=m.assess(root,'shaman')['checks']
   for key in('glyph_warrior_matrix','glyph_shaman_shield_auras','glyph_shaman_healing','glyph_shaman_totem_hits','glyph_shaman_expiration_explosion'):self.assertTrue(checks[key],key)
   host.write_text(text.replace('after_amount=140.0','after_amount=100.0'));self.assertFalse(m.assess(root,'shaman')['checks']['glyph_shaman_healing'])
   host.write_text(text.replace('after_duration=7.5','after_duration=15'));self.assertFalse(m.assess(root,'shaman')['checks']['glyph_shaman_shield_auras'])
   host.write_text(text.replace('spell=3606 source','spell=324 source'));self.assertFalse(m.assess(root,'shaman')['checks']['glyph_shaman_totem_hits'])
   host.write_text(text.replace('spell=16038 source','spell=324 source'));self.assertFalse(m.assess(root,'shaman')['checks']['glyph_shaman_expiration_explosion'])
   host.write_text(''.join(lines[1:]));self.assertFalse(m.assess(root,'shaman')['checks']['glyph_warrior_matrix'])

 def test_mage_evidence_requires_owned_sheep_gate_burst_and_barrier(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder)
   for i in(1,2):
    d=root/f"player-{i}";d.mkdir();(d/"azeroth-coop-native.log").write_text("");(d/"native-output.log").write_text("")
   lines=[]
   for name,(d,c,t,r,l,e)in m.MAGE_NAMES.items():
    sp=m.MAGE_SPELLS[name];b=[2,.5,4,1.2,1.3,.1,1.75];damage=1 if sp==118 else d
    if sp in(116,120,11113):l*=e;e=1
    a=[2*damage,.5*c,4+t,1.2*r,1.3*l,.1*damage,1.75*e]
    lines.append(f"glyph fixture mage matrix: slot=0 spell={sp} glyph={name} before=Ok({json.dumps(b)}) after=Ok({json.dumps(a)})\n")
   for name,fac in [('Deep Barrier',1.4),('Fleeting Barrier',.6)]:lines.append(f"glyph fixture mage support: spell=11426 glyph={name} before_power=100 after_power={100*fac} before_duration=15 after_duration=15\n")
   lines+=['glyph fixture mage tooltip: glyph=Flock Master strings=Ok([(24, and 5 at once. burst of 7.8% of its maximum health)])\n','glyph fixture sheep explosion: game=1 before=100 after=65 factor=0.65 source=Ok(Some((0, 111)))\n','glyph fixture sheep gate: slot=0 count=3 limit=5 glyph=Flock Master allowed=true\n']
   host=root/'player-1/azeroth-coop-native.log';text=''.join(lines);host.write_text(text);checks=m.assess(root,'mage')['checks']
   for key in('glyph_warrior_matrix','glyph_mage_barrier','glyph_mage_sheep_damage','glyph_mage_owned_sheep_gate'):self.assertTrue(checks[key],key)
   host.write_text(text.replace('after=65','after=100'));self.assertFalse(m.assess(root,'mage')['checks']['glyph_mage_sheep_damage'])
   host.write_text(text.replace('limit=5','limit=3'));self.assertFalse(m.assess(root,'mage')['checks']['glyph_mage_owned_sheep_gate'])
   host.write_text(text.replace('after_power=140.0','after_power=100.0'));self.assertFalse(m.assess(root,'mage')['checks']['glyph_mage_barrier'])

 def test_warlock_requires_native_buff_and_real_shard_damage_and_leech(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder)
   for i in(1,2):
    d=root/f"player-{i}";d.mkdir();(d/"azeroth-coop-native.log").write_text("");(d/"native-output.log").write_text("")
   lines=[]
   for name,(d,c,t,r,l,e)in m.WARLOCK_NAMES.items():
    sp=m.WARLOCK_SPELLS[name];b=[2,.5,4,1.2,1.3,.1,1.75];a=[2*d,.5*c,4+t,1.2*r,1.3*l,.1*d,1.75*e]
    lines.append(f"glyph fixture warlock matrix: slot=0 spell={sp} glyph={name} before=Ok({json.dumps(b)}) after=Ok({json.dumps(a)})\n")
   for name in('Unbound Pact','Enduring Pact'):
    d,c,t,r,l,e=m.WARLOCK_NAMES[name];lines.append(f"glyph fixture demonic aura: glyph={name} before_power=0.4 after_power={.4*e} before_duration=8 after_duration={8*l}\n")
   lines+=['glyph fixture fear effect: spell=5782 glyph=Spreading Dread before=4 after=2.6\n','glyph fixture warlock hit: slot=0 spell=6353 glyph=Hungry Souls amount=100\n','glyph fixture warlock hit: slot=0 spell=689 glyph=Shared Hunger amount=100\n','glyph fixture warlock leech HP: spell=689 before=20 after=80 maximum=100 damage=100 fraction=0.6\n','glyph fixture warlock tooltip: glyph=Hungry Souls strings=Ok([(24, Your kills gather soul shards, at most one every 0.3 sec. At 5 shards a free Soulfire strikes.)])\n']
   host=root/'player-1/azeroth-coop-native.log';text=''.join(lines);host.write_text(text);checks=m.assess(root,'warlock')['checks']
   for key in('glyph_warrior_matrix','glyph_warlock_demonic_auras','glyph_warlock_soulfire','glyph_warlock_leech_damage','glyph_warlock_leech_healing','glyph_warlock_shard_tooltip'):self.assertTrue(checks[key],key)
   host.write_text(text.replace('spell=6353 glyph','spell=686 glyph'));self.assertFalse(m.assess(root,'warlock')['checks']['glyph_warlock_soulfire'])
   host.write_text(text.replace('after_power=0.5599999999999999','after_power=0.4'));self.assertFalse(m.assess(root,'warlock')['checks']['glyph_warlock_demonic_auras'])
   host.write_text(text.replace('after=80','after=20'));self.assertFalse(m.assess(root,'warlock')['checks']['glyph_warlock_leech_healing'])

 def test_druid_aura_and_arc_evidence_rejects_missing_or_wrong_scaling(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder)
   for i in(1,2):
    p=root/f"player-{i}";p.mkdir();(p/"azeroth-coop-native.log").write_text("");(p/"native-output.log").write_text("")
   host=root/"player-1/azeroth-coop-native.log"
   text=("glyph fixture druid support: spell=774 glyph=Abundant Growth before_power=10 after_power=9.1 before_duration=12 after_duration=7.8\n"
    "glyph fixture druid support: spell=774 glyph=Persistent Growth before_power=10 after_power=10.5 before_duration=12 after_duration=18\n"
    "glyph fixture thorns shield aura: glyph=Barbed Thorns before_power=20 after_power=30 before_duration=20 after_duration=13\n"
    "glyph fixture thorns shield aura: glyph=Lasting Thorns before_power=20 after_power=14 before_duration=20 after_duration=30\n"
    "glyph fixture faerie explosion: game=1 before=100 after=150 factor=1.5 source=Ok(Some((0, 1)))\n")
   host.write_text(text);checks=m.assess(root,"druid")["checks"]
   for key in("glyph_druid_rejuvenation","glyph_druid_thorns","glyph_druid_faerie_arcs"):self.assertTrue(checks[key],key)
   host.write_text(text.replace("after_power=9.1", "after_power=14"));self.assertFalse(m.assess(root,"druid")["checks"]["glyph_druid_rejuvenation"])
   host.write_text(text.replace("after=150", "after=100"));self.assertFalse(m.assess(root,"druid")["checks"]["glyph_druid_faerie_arcs"])

 def test_passive_removal_is_explicit_rebuild_evidence_and_never_claims_a_cast(self):
  with tempfile.TemporaryDirectory()as folder:
   root=Path(folder)
   for i in(1,2):
    p=root/f"player-{i}";p.mkdir();(p/"azeroth-coop-native.log").write_text("");(p/"native-output.log").write_text("")
   host=root/"player-1/azeroth-coop-native.log"
   rebuilt="glyph fixture removal rebuilt: slot=0 spell=770 baseline=Some([1,1,0,1,1,0,1]) current=Ok([1,1,0,1,1,0,1])\n"
   accepted="glyph accepted: slot=0 guid=11 spell=770 rank=4 glyph=Unglyphed\n"
   host.write_text(rebuilt);self.assertFalse(m.assess(root,"druid")["passive_removal_rebuilt"])
   host.write_text(rebuilt+accepted);r=m.assess(root,"druid");self.assertTrue(r["passive_removal_rebuilt"]);self.assertFalse(r["checks"]["glyph_removed_cast"])
   self.assertFalse(m.assess(root,"warrior")["passive_removal_rebuilt"])
   host.write_text((rebuilt+accepted).replace("current=Ok([1,1,0,1,1,0,1])","current=Ok([1,1,0,1.4,1,0,1])"));self.assertFalse(m.assess(root,"druid")["passive_removal_rebuilt"])
   host.write_text((rebuilt+accepted).replace("spell=770","spell=774"));self.assertFalse(m.assess(root,"druid")["passive_removal_rebuilt"])
