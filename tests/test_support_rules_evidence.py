import importlib.util
from pathlib import Path
import sys
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools/dev'))
import support_rules_evidence as evidence

class ApprovedSupportTests(unittest.TestCase):
    def logs(self):
        logs={}
        for n in (1,2):
            rows=[]
            for s in evidence.HEALS+evidence.BUFFS:
                rows.append(f'event ally_support_native_result target={n} spell={s} before=20 after=40 buff_present=1 shield=50')
            for s in evidence.HEALS+(139,774):
                rows.append(f'event ally_healing_meter_native caster={n} spell={s} amount=20 before=100 after=120')
            for s,amount in ((15237,4),(25912,3)):
                rows.append(f'event ally_hybrid_elementary target={n} spell={s} amount={amount} caster_max=100')
            rows.append(f'event ally_nature_burst caster={n} amount=50 x=9 y=8')
            rows.append(f'event ally_shield_burst_origin caster={n} x=9 y=8')
            logs[n]='\n'.join(rows)
        return logs
    def test_requires_native_counter_credit_and_elementary_values(self):
        logs=self.logs();self.assertTrue(evidence.assess(logs)['passed'])
        logs[2]=logs[2].replace('spell=25912 amount=3 caster_max=100','spell=25912 amount=30 caster_max=100')
        self.assertFalse(evidence.assess(logs)['passed'])
    def test_native_counter_mismatch_or_personal_target_is_a_failure(self):
        logs=self.logs();logs[1]=logs[1].replace('after=120','after=140')
        self.assertFalse(evidence.assess(logs)['passed'])
        logs=self.logs();logs[1]+='\nevent ally_support_native_result target=2 spell=11426 before=20 after=40 buff_present=1'
        self.assertFalse(evidence.assess(logs)['passed'])
    def test_buff_icon_without_real_absorption_is_not_a_shield_proof(self):
        logs=self.logs();logs[1]=logs[1].replace('shield=50','shield=0')
        self.assertFalse(evidence.assess(logs)['passed'])
