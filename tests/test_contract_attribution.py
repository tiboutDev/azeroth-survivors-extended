"""Ownership metadata cannot silently narrow shared contracts or invent validation."""
import copy
import json
import unittest
from pathlib import Path
from tools.dev import compatibility, contract_attribution, feature_catalog, scenario_suites
ROOT=Path(__file__).resolve().parents[1]

class ContractAttributionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.inventory=compatibility.inventory()
        cls.by_id={c['id']:c for c in cls.inventory['contracts']}
        cls.policy=json.loads((ROOT/'compat/contract-links.json').read_text())
        cls.validation=json.loads((ROOT/'compat/feature-validation.json').read_text())
        cls.features=feature_catalog.load_features()
        cls.suites=scenario_suites.load_suites()['suites']

    def validate(self,policy=None,validation=None):
        return contract_attribution.validate_policy(policy or self.policy,validation or self.validation,self.features,self.suites)

    def test_other_feature_families_have_explicit_reviewed_links(self):
        examples={'official_native.experience.replay_experience':'ASX-02',
                  'official_native.fatigue.project_fatigue_aura':'ASX-12',
                  'official_native.enchantments.ring_stats':'ASX-16',
                  'official_native.gems.gem_stats':'ASX-22',
                  'official_native.combat.write_combat_mark':'ASX-09',
                  'official_native.immunity_aggro.aggro_prepare':'ASX-11',
                  'official_native.raid_markers.set_raid_marker':'ASX-25',
                  'official_native.combat_text.draw_native_heal_text':'ASX-24'}
        for name,feature in examples.items():
            contract=self.by_id[name]
            self.assertEqual(contract['link_basis'],'explicit')
            self.assertIn(feature,contract['features'])
            self.assertTrue(contract['link_evidence']['reason'])
            self.assertTrue(contract['validation_profiles'])

    def test_shared_reward_hooks_preserve_goblin_and_jewelry_owners(self):
        self.assertTrue({'ASX-15','ASX-16','ASX-18','ASX-22','ASX-26'} <= set(self.by_id['official_native.bonus_ui.reward_entries']['features']))
        self.assertTrue({'ASX-22','ASX-26'} <= set(self.by_id['official_native.gems.amulet_selected']['features']))
        self.assertTrue({'ASX-03','ASX-21'} <= set(self.by_id['official_native.duplicate_loot.loot_actor_gold']['features']))

    def test_ambiguous_helpers_do_not_lose_shared_source_owners(self):
        helper=self.by_id['official_native.gems.q']
        self.assertEqual(helper['link_basis'],'conservative-domain')
        self.assertTrue({'ASX-22','ASX-26'} <= set(helper['features']))
        context=self.by_id['official_native.ui_identity.native_ui_participants']
        self.assertEqual(context['link_basis'],'conservative-domain')
        self.assertIn('shared',context['link_evidence']['reason'])

    def test_every_feature_has_candidate_profiles_without_invented_live_coverage(self):
        for feature in self.inventory['features']:
            self.assertTrue(feature['validation_profiles'])
            self.assertEqual(feature['live_suite_gap'],not bool(feature['declared_live_suites']))
        features={f['id']:f for f in self.inventory['features']}
        for name in ('ASX-20',):
            self.assertTrue(features[name]['live_suite_gap'])
        self.assertEqual(features['ASX-15']['runtime_disposition'],'disabled')
        self.assertEqual(features['ASX-21']['runtime_disposition'],'upstream')

    def test_invalid_ownership_missing_evidence_or_reference_is_rejected(self):
        key=next(iter(self.policy['contracts']))
        for mutation in ('unknown','duplicate','evidence','path','shared'):
            policy=copy.deepcopy(self.policy)
            if mutation=='unknown':policy['contracts'][key]=['ASX-99']
            if mutation=='duplicate':policy['contracts'][key]*=2
            if mutation=='evidence':policy['evidence'].pop(key)
            if mutation=='path':policy['evidence'][key]['references']=['../outside']
            if mutation=='shared':policy['evidence'][key]['shared']=not policy['evidence'][key]['shared']
            with self.subTest(mutation=mutation),self.assertRaises(ValueError):self.validate(policy=policy)

    def test_missing_feature_and_nonexistent_test_profile_are_rejected(self):
        for mutation in ('missing','profile'):
            value=copy.deepcopy(self.validation)
            if mutation=='missing':value['features'].pop('ASX-20')
            else:value['features']['ASX-12']['profiles']=['made-up-live-test']
            with self.assertRaises(ValueError):self.validate(validation=value)

    def test_new_contract_remains_conservative_and_stale_review_links_fail(self):
        contracts=copy.deepcopy(self.inventory['contracts'])
        sample=copy.deepcopy(self.by_id['official_native.gems.q']);sample['id']='official_native.gems.new_reader';contracts.append(sample)
        current,_=contract_attribution.apply_attribution(contracts,feature_catalog.load_features(),self.suites)
        self.assertEqual(current[-1]['link_basis'],'conservative-domain')
        self.assertIn('ASX-26',current[-1]['features'])
        current=[c for c in current if c['id']!='official_native.gems.amulet_selected']
        with self.assertRaises(ValueError):contract_attribution.apply_attribution(current,feature_catalog.load_features(),self.suites)

    def test_impact_keeps_shared_owners_profiles_gaps_and_review_requirement(self):
        report=compatibility.impact(dict(old_sha256=self.inventory['client_sha256'],new_sha256='b'*64,approved=False,offline_only=True,mapping_policy='instruction-alignment-v2',mapping={}),self.inventory)
        self.assertFalse(report['approved'])
        fatigue=next(f for f in report['features']if f['id']=='ASX-12')
        self.assertFalse(fatigue['live_suite_gap']);self.assertIn('coop-coverage',fatigue['declared_live_suites']);self.assertIn('fatigue',fatigue['validation_profiles']);self.assertTrue(fatigue['review_required'])
        reward=next(c for c in report['contracts']if c['id']=='official_native.bonus_ui.reward_entries')
        self.assertIn('ASX-26',reward['features']);self.assertIn('coop-events',reward['validation_suites'])

if __name__=='__main__':unittest.main()
