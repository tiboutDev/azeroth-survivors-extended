import importlib.util
from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('support_evidence',ROOT/'tools/dev/support_evidence.py')
evidence=importlib.util.module_from_spec(spec);spec.loader.exec_module(evidence)

class SupportEvidenceTests(unittest.TestCase):
    def logs(self):
        logs={}
        for n in (1,2):
            lines=[f'event run_rules_applied player={n} personal_luck={n*.25} options=[200.0, 200.0, 150.0, 100.0]']
            for spell in (635,331,8936,15237,6673,17,139,774,12042,13750,5171,11426,642,20925,324,467):
                lines.append(f'event ally_support_native_result target={n} spell={spell} before=20 after=40 buff_present=1')
            lines+=['event enemy_coop_scaled boss=0 factor=2.0','event enemy_coop_scaled boss=1 factor=1.85']
            logs[n]='\n'.join(lines)
        return logs
    def test_requires_actual_results_in_both_directions(self):
        logs=self.logs();self.assertTrue(evidence.assess(logs)['passed'])
        logs[2]=logs[2].replace('spell=331 before=20 after=40','spell=331 before=20 after=20')
        self.assertFalse(evidence.assess(logs)['passed'])
    def test_native_exception_cannot_pass_even_after_game_started(self):
        logs=self.logs();logs[2]+='\nfirst-chance: ACCESS_VIOLATION at native HEAL'
        self.assertFalse(evidence.assess(logs)['passed'])
    def test_requires_boss_scaling_and_host_options(self):
        logs=self.logs();logs[1]=logs[1].replace('boss=1 factor=1.85','boss=1 factor=1')
        self.assertFalse(evidence.assess(logs)['passed'])

    def test_previous_density_scaling_cannot_validate_current_build(self):
        logs=self.logs();logs[1]=logs[1].replace('boss=0 factor=2.0','boss=0 factor=1.55')
        self.assertFalse(evidence.assess(logs)['ordinary_scaling_verified'])
