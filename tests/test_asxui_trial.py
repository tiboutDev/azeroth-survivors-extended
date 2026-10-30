"""Evidence assessor rejects incomplete real API/widget observations."""
import copy,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools/dev'))
from asx_api_trial import assess,parse_args,adapter_rejections,duration_checks,assess_wow_visual
class TrialAssessorTests(unittest.TestCase):
    def test_expected_taunt_immunity_is_not_an_adapter_error(self):
        expected='[ INFO ]   64.775s treasure goblin taunt rejected: enemy=17379661029583094021 spell=23922'
        self.assertEqual(adapter_rejections(expected),[])
        for line in ('[ WARN ]   1.000s treasure goblin taunt rejected: enemy=1 spell=2','[ INFO ]   1.000s Native inputs rejected: invalid context','[ ERROR ]   1.000s error'):
            self.assertEqual(adapter_rejections(line),[line])
    def test_claimed_recipient_needs_no_pin_but_observed_pin_duration_is_bounded(self):
        host=[{'structured':{'consumables':{'0':{'remaining':60}},'markers':{}}}]
        checks,maximum=duration_checks(host);self.assertTrue(all(checks.values()));self.assertEqual(maximum,0)
        guest=copy.deepcopy(host);guest[0]['structured']['markers']={'1':{'kind':0,'left':90}}
        checks,maximum=duration_checks(guest);self.assertTrue(all(checks.values()));self.assertEqual(maximum,90)
        guest[0]['structured']['markers']['1']['left']=91;self.assertFalse(duration_checks(guest)[0]['supply_pin_bounds'])
        self.assertFalse(duration_checks([])[0]['sixty_second_buffs'])
    def test_visual_order_only_covers_active_asx_fortunes(self):
        native={'structured':{},'buffOrder':{'fortuneShown':True,'first':False}}
        active={'structured':{'fortune':{'kind':1}},'buffOrder':{'fortuneShown':True,'first':True,'tooltip':"Sayge's Dark Fortune of Wealth"}}
        self.assertTrue(all(assess_wow_visual([native,active]).values()))
        self.assertFalse(all(assess_wow_visual([native]).values()))
        active['buffOrder']['first']=False
        self.assertFalse(assess_wow_visual([active])['fortune_in_first_native_position'])
    def sample(self):
        widget=dict(shown=True,textMatches=True,hideWorks=True,showWorks=True,displayOnly=True,scale=1,alpha=1)
        widgets={k:dict(widget)for k in ('supplies','fortune','sayge')}
        widgets['supplies']['scale']=.85;widgets['fortune']['alpha']=.75;widgets['sayge']['scale']=.9
        return dict(available=True,major=1,globals=True,recipient='11',copyIsolated=True,deepCopyIsolated=True,callbacks=3,badCallbacks=3,callbackMismatch=0,structured={'discovery':{'event':1,'gold':33}},texts={'supplies':{'reward':'ASX text'}},widgets=widgets)
    def test_game_launch_requires_explicit_api_mode(self):
        import contextlib,io
        with contextlib.redirect_stderr(io.StringIO()):
            for argv in ([],['--resource-probe'],['--api-trial','3'],['--api-trial','1','--resource-probe']):
                with self.assertRaises(SystemExit):parse_args(argv)
        self.assertEqual(parse_args(['--api-trial','2','--seconds','180']).api_trial,2)
    def test_complete_observation_passes(self):self.assertTrue(all(assess([self.sample()]).values()))
    def test_empty_or_unavailable_evidence_cannot_pass(self):
        self.assertFalse(all(assess([]).values()))
        value=self.sample();value['available']=False;self.assertFalse(all(assess([value]).values()))
    def test_each_missing_widget_or_text_match_fails(self):
        for kind in ('supplies','fortune','sayge'):
            value=self.sample();value['widgets'][kind]['shown']=False;self.assertFalse(all(assess([value]).values()))
            value=self.sample();value['widgets'][kind]['textMatches']=False;self.assertFalse(all(assess([value]).values()))
    def test_mutable_copies_callback_errors_or_wrong_preferences_fail(self):
        for field in ('copyIsolated','deepCopyIsolated','globals'):
            value=self.sample();value[field]=False;self.assertFalse(all(assess([value]).values()))
        value=self.sample();value['callbackMismatch']=1;self.assertFalse(all(assess([value]).values()))
        value=self.sample();value['widgets']['fortune']['alpha']=1;self.assertFalse(all(assess([value]).values()))
if __name__=='__main__':unittest.main()
