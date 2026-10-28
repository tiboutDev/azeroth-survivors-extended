import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools/dev'))
from support_proc_evidence import assess


class ProcEvidenceTests(unittest.TestCase):
    def test_missing_mismatched_and_duplicate_origins_are_not_proof(self):
        request = 'event ally_shield_break_native caster=1 target=2 x=10 y=20\n'
        used = 'event ally_shield_burst_origin caster=1 x=10 y=20\n'
        self.assertTrue(assess({1: request + used})['shield_origin_forwarding']['1']['forwarded'])
        for log in (request, used, request + used.replace('x=10', 'x=11'), request + used * 2,
                    (request + used).replace('x=10', ''), request.replace('target=2', '') + used):
            self.assertFalse(assess({1: log})['shield_origin_forwarding']['1']['forwarded'])

    def test_proc_notifications_do_not_prove_hp_loss_or_kill_credit(self):
        report = assess({1: 'event ally_nature_burst caster=1 target=2 amount=50 x=10 y=20'})
        self.assertEqual(report['nature_ally_bursts']['1'], 1)
        self.assertFalse(report['proves_per_enemy_proc_hp_loss'])
        self.assertFalse(report['proves_authoritative_personal_kill_credit'])
        self.assertFalse(report['proves_all_boss_abilities'])
