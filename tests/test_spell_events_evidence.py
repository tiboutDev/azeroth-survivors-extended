import sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools/dev'))
from spell_events_evidence import assess,assess_impacts,assess_heal_commands,assess_returns,assess_offensive,assess_periodic
class Evidence(unittest.TestCase):
 def logs(self):return {n:'event spell_events_summary started=10 completed=6 replaced=3 healed=12 active=1\nevent spell_events_summary started=12 completed=8 replaced=3 healed=14 active=1'for n in range(1,5)}
 def test_all_four_players_must_have_completed_and_healed(self):
  logs=self.logs();self.assertTrue(assess(logs)['passed']);del logs[4];self.assertFalse(assess(logs)['passed'])
 def test_false_completion_or_counter_rewind_fails(self):
  logs=self.logs();logs[2]=logs[2].replace('completed=8','completed=9');self.assertFalse(assess(logs)['passed'])
  logs=self.logs();logs[2]+='\nevent spell_events_summary started=11 completed=8 replaced=3 healed=14 active=0';self.assertFalse(assess(logs)['passed'])
 def test_missing_or_noninteger_counts_fail(self):
  logs=self.logs();logs[3]=logs[3].replace('healed=14','healed=14.5');self.assertFalse(assess(logs)['passed'])

class ImpactEvidence(unittest.TestCase):
 def logs(self):return {n:'event spell_events_summary impacts='+str(10 if n<4 else 0)for n in range(1,5)}
 def test_three_casters_and_hunter_control_are_required(self):
  logs=self.logs();self.assertTrue(assess_impacts(logs)['passed']);logs[4]='event spell_events_summary impacts=1';self.assertFalse(assess_impacts(logs)['passed'])
 def test_missing_and_rewound_counts_cannot_pass(self):
  logs=self.logs();logs[2]+='\nevent spell_events_summary impacts=9';self.assertFalse(assess_impacts(logs)['passed']);del logs[3];self.assertFalse(assess_impacts(logs)['passed'])

class HealCommandEvidence(unittest.TestCase):
 def logs(self):return {n:f'event ally_heal_budget_preserved target={n} before_bits=2147483648 after_bits=2147483648'for n in range(1,5)}
 def test_requires_each_recipient_and_exact_saved_bits(self):
  logs=self.logs();self.assertTrue(assess_heal_commands(logs)['passed'])
  logs[2]=logs[2].replace('after_bits=2147483648','after_bits=2147483649');self.assertFalse(assess_heal_commands(logs)['passed'])
  logs=self.logs();del logs[4];self.assertFalse(assess_heal_commands(logs)['passed'])
 def test_rejects_invalid_or_wrong_recipient_observations(self):
  for before,after,target in [('nan','nan',1),('1.5','1.5',1),('1','1',2),('4294967296','4294967296',1)]:
   logs=self.logs();logs[1]=f'event ally_heal_budget_preserved target={target} before_bits={before} after_bits={after}'
   self.assertFalse(assess_heal_commands(logs)['passed'])

class ReturnEvidence(unittest.TestCase):
 def logs(self):return {n:'event spell_events_summary started=10 completed=6 failed=4 replaced=0 healed=12 active=0'for n in range(1,5)}
 def test_requires_true_and_false_native_outcomes_on_every_client(self):
  logs=self.logs();self.assertTrue(assess_returns(logs)['passed'])
  logs[2]=logs[2].replace('completed=6 failed=4','completed=10 failed=0');self.assertFalse(assess_returns(logs)['passed'])
 def test_replacement_or_pending_cast_is_not_an_observed_return(self):
  for fields in ['completed=6 failed=3 replaced=1 healed=12 active=0','completed=6 failed=3 replaced=0 healed=12 active=1','completed=7 failed=4 replaced=0 healed=12 active=0']:
   logs=self.logs();logs[3]='event spell_events_summary started=10 '+fields;self.assertFalse(assess_returns(logs)['passed'])
 def test_failed_counter_cannot_rewind(self):
  logs=self.logs();logs[1]+='\nevent spell_events_summary started=11 completed=8 failed=3 replaced=0 healed=12 active=0'
  self.assertFalse(assess_returns(logs)['passed'])

 def test_common_failure_path_mode_still_requires_real_failures_and_every_success(self):
  logs=self.logs();logs[4]=logs[4].replace('completed=6 failed=4','completed=10 failed=0')
  report=assess_returns(logs,require_failure_per_player=False)
  self.assertTrue(report['passed']);self.assertFalse(report['strict_failure_coverage_met']);self.assertEqual(report['failure_players'],[1,2,3])
  for n in logs:logs[n]=logs[n].replace('completed=6 failed=4','completed=10 failed=0')
  self.assertFalse(assess_returns(logs,require_failure_per_player=False)['passed'])
  logs=self.logs();logs[4]=logs[4].replace('completed=6 failed=4','completed=0 failed=10')
  self.assertFalse(assess_returns(logs,require_failure_per_player=False)['passed'])

class OffensiveEvidence(unittest.TestCase):
 def logs(self):
  logs={n:f'event spell_events_summary effect_requests={4 if n==2 else 0} effect_commands={4 if n==1 else 0}'for n in range(1,5)}
  logs[1]+='\nevent offensive_native_state kind=3 spell=853 active=1';return logs
 def test_requires_guest_capture_host_replay_and_native_control(self):
  logs=self.logs();self.assertTrue(assess_offensive(logs)['passed']);logs[1]=logs[1].replace('active=1','active=0');self.assertFalse(assess_offensive(logs)['passed'])
 def test_missing_rewound_or_wrong_phase_counts_cannot_pass(self):
  for bad in ['effect_requests=0 effect_commands=0','effect_requests=4 effect_commands=1','effect_requests=nan effect_commands=0']:
   logs=self.logs();logs[2]='event spell_events_summary '+bad;self.assertFalse(assess_offensive(logs)['passed'])
  logs=self.logs();logs[2]+='\nevent spell_events_summary effect_requests=3 effect_commands=0';self.assertFalse(assess_offensive(logs)['passed'])

class PeriodicEvidence(unittest.TestCase):
 def logs(self):return {n:f'event periodic_budget_resolved player={n} foreign_rows=1 personal_rows=1 raw=12 foreign=8 personal=4 overheal=7 shield_before=0 shield_after=7 shield_expected=7'for n in range(1,5)}
 def test_requires_both_ownerships_mixed_batch_and_real_shield_gain(self):
  logs=self.logs();self.assertTrue(assess_periodic(logs)['passed']);logs[4]=logs[4].replace('foreign_rows=1','foreign_rows=0');self.assertFalse(assess_periodic(logs)['passed'])
  logs={n:t.replace('shield_after=7 shield_expected=7','shield_after=0 shield_expected=0')for n,t in self.logs().items()};self.assertFalse(assess_periodic(logs)['passed'])
 def test_wrong_contribution_or_shield_and_missing_player_fail(self):
  for bad in [('personal=4','personal=5'),('shield_after=7','shield_after=8'),('personal=4','personal=nan')]:
   logs=self.logs();logs[2]=logs[2].replace(*bad);self.assertFalse(assess_periodic(logs)['passed'])
  logs=self.logs();del logs[3];self.assertFalse(assess_periodic(logs)['passed'])

class ThreeParticipantCombat(unittest.TestCase):
 def test_explicit_three_role_scope_does_not_fabricate_hunter_coverage(self):
  logs={n:'event spell_events_summary impacts=2 effect_requests='+str(2 if n==2 else 0)+' effect_commands='+str(2 if n==1 else 0)for n in (1,2,3)}
  logs[1]+='\nevent offensive_native_state kind=3 spell=853 active=1'
  result=assess_impacts(logs,participants=(1,2,3))
  self.assertTrue(result['passed']);self.assertFalse(result['hunter_control_tested'])
  self.assertTrue(assess_offensive(logs,participants=(1,2,3))['passed'])
  self.assertFalse(assess_impacts(logs)['passed']);self.assertFalse(assess_offensive(logs)['passed'])
  del logs[3];self.assertFalse(assess_impacts(logs,participants=(1,2,3))['passed']);self.assertFalse(assess_offensive(logs,participants=(1,2,3))['passed'])
  with self.assertRaises(ValueError):assess_impacts(logs,participants=())
  with self.assertRaises(ValueError):assess_offensive(logs,participants=(1,2))
