import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('chest_evidence', Path(__file__).resolve().parents[1]/'tools/dev/chest_evidence.py')
evidence = importlib.util.module_from_spec(spec)
spec.loader.exec_module(evidence)

class EvidenceTests(unittest.TestCase):
    def fixture(self):
        host = '\n'.join([
            'event party_chest_native_pickup picker=1 guid_exact=18446744073709551615',
            'event party_chest_purchased picker=2 guid_exact=18446744073709551614',
            'event party_wallet_published revision=3 gold=72 earned=100 cost=28 opened=2',
            'event party_chest_host_choice player=1'])
        guest = '\n'.join([
            'event party_chest_rewards player=2 count=1 opened=1',
            'event party_chest_rewards player=2 count=1 opened=2',
            'event party_wallet_applied player=2 revision=3 gold=72 earned=100 cost=28 opened=2'])
        return host,guest
    def test_boot_alone_cannot_pass(self):
        self.assertFalse(evidence.assess('event native_start player=1','event native_start player=2')['passed'])
    def test_two_chests_require_matching_money_and_both_rewards(self):
        host,guest=self.fixture()
        self.assertTrue(evidence.assess(host,guest)['passed'])
        self.assertFalse(evidence.assess(host,guest.replace('gold=72','gold=73'))['passed'])
        self.assertFalse(evidence.assess(host,guest+'\nevent party_chest_rewards count=1')['passed'])
    def test_same_guid_does_not_count_as_two_chests(self):
        host,guest=self.fixture()
        self.assertFalse(evidence.assess(host.replace('18446744073709551614','18446744073709551615'),guest)['passed'])
    def test_three_rounds_require_new_play_enemy_and_chest_evidence_each_time(self):
        h,g=self.fixture()
        h+='\nevent native_start player=1\nevent native_state player=1 phase=1\nevent enemy_sync common=3'
        g+='\nevent native_start player=2\nevent native_state player=2 phase=1\nevent enemy_sync common=3'
        host='\nevent round_state_reset player=1\n'.join([h]*3)
        guest='\nevent round_state_reset player=2\n'.join([g]*3)
        self.assertTrue(all(r['passed'] for r in evidence.assess_rounds(host,guest,3,True)))
        missing='\nevent round_state_reset player=2\n'.join([g,g,g.replace('event party_chest_rewards','event missing')])
        rows=evidence.assess_rounds(host,missing,3,True)
        self.assertEqual([r['passed'] for r in rows],[True,True,False])
        # A completed generation barrier never counts as a newly played round.
        rows=evidence.assess_rounds(h+'\nevent round_state_reset player=1',g+'\nevent round_state_reset player=2',3)
        self.assertEqual([r['passed'] for r in rows],[True,False,False])
