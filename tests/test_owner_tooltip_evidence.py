import sys,tempfile,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools/dev'))
from owner_tooltip_evidence import assess
class TooltipEvidenceTests(unittest.TestCase):
    def fixture(self,root,wrong_owner=False):
        name='Smite'.encode().hex()
        for own,damage in ((1,42),(2,90)):
            other=3-own;other_damage=damage if wrong_owner else (90 if other==2 else 42)
            body=f'Deals {damage} damage with your bonuses.'.encode().hex()
            remote_body=f'Deals {other_damage} damage with your bonuses.'.encode().hex()
            wire=f'1|12\n585|1|{name}|{body}||255,209,0|255,255,255|'.encode().hex()
            line=f'AZCOOP_OWNER_TOOLTIP own={own} source={wire} remote={other}/spells/585/1/{name}/{remote_body}::255,209,0:255,255,255\n'
            folder=root/f'player-{own}';folder.mkdir();(folder/'stderr.log').write_text(line*3)
    def test_accepts_exact_native_owner_values_and_colors(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);self.fixture(root);report=assess(root)
            self.assertTrue(report['passed']);self.assertEqual(report['mismatch_count'],0)
    def test_rejects_tooltips_recalculated_with_observer_stats(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);self.fixture(root,True);report=assess(root)
            self.assertFalse(report['passed']);self.assertEqual(report['mismatch_count'],6)
