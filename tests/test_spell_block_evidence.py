import sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools/dev'))
from spell_block_evidence import assess

class SpellBlockEvidence(unittest.TestCase):
 def test_missing_or_zero_damage_cannot_pass(self):
  self.assertFalse(all(assess('').values()))
  text='\n'.join(f'spell block polymorph: caster={n} result=Ok((123, true, true))\nspell block explosion: caster={n} guid=42 enemy=123 raw=10 effective=Some(0)'for n in (0,1))
  self.assertFalse(assess(text)['polymorph_0_native_damage'])
 def test_both_owners_and_all_mark_transitions_are_required(self):
  text='\n'.join(f'spell block polymorph: caster={n} result=Ok((123, true, true))\nspell block explosion: caster={n} guid=42 enemy=123 raw=10 effective=Some(9)'for n in (0,1))
  text+='\n'+'\n'.join(f'marks121 native: stage={stage} marked=true vulnerability={value}'for stage,value in ((1,.35),(2,.2),(3,0)))
  text+='\nspell block native hunters: Ok([(123, 0.35), (123, 0.35), (123, 0.2), (123, 0.0)])'
  self.assertTrue(all(assess(text).values()))
  self.assertFalse(all(assess(text.replace('caster=1','caster=0')).values()))
