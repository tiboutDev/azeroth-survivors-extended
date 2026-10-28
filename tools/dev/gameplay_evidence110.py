"""Assess real, bounded two-player 0.110 gameplay logs and native packet traces.
This does not claim a mouse-input test or automatically approve rendered pixels.
"""
import argparse,json,re
from pathlib import Path

def assess(root):
    texts={n:(root/f'native-{n}.log').read_text(errors='replace') for n in (1,2)}
    def events(n,name):
        return [(float(m[1]),m[2]) for m in re.finditer(r'\[ INFO \]\s+([0-9.]+)s event '+re.escape(name)+r'(?: (.*))?$',texts[n],re.M)]
    def number(text,key,default=0.):
        m=re.search(r'(?:^| )'+key+r'=([-0-9.]+)',text);return float(m[1]) if m else default
    checks={}
    checks['no_native_faults']=all(not re.search(r'\[FAILED\]|ACCESS_VIOLATION|unhandled exception',t,re.I) for t in texts.values())
    checks['two_chests']=all(events(n,'party_chest_native_pickup') or events(n,'party_chest_rewards') for n in (1,2)) and bool(events(1,'party_chest_purchased'))
    checks['guest_waited_on_two_chests']=len(events(2,'test_chest_guest_hold'))>=2 and all(number(t,'seconds')>=4 for _,t in events(2,'test_chest_guest_hold'))
    checks['host_waited_for_guest']=len(events(1,'talent_wait_started'))>=2 and len(events(1,'talent_wait_ended'))>=2
    frozen=[]
    starts=events(1,'talent_wait_started');ends=events(1,'talent_wait_ended')
    for at,_ in starts:
        end=next((stop for stop,_ in ends if stop>at),None)
        if end is None or not 3.5<=end-at<=6.0:continue
        samples=[number(t,'next',-1) for t0,t in events(1,'test_encounter_hud') if at+.25<=t0<=end-.1]
        if len(samples)>=2:frozen.append(max(samples)-min(samples)<=.05)
    checks['world_clock_frozen_while_guest_chooses']=len(frozen)>=2 and all(frozen)
    kills=events(1,'test_guardian_kill_requested')
    reward_kinds={}
    if kills:
        start=kills[0][0]
        for n in (1,2):
            # Both process clocks start together; allow native call/transport jitter
            # and the deliberately held guest item chooser.
            reward_kinds[n]=[int(number(t,'kind')) for at,t in events(n,'test_reward_choice_kind') if start-.8<=at<=start+8]
    checks['guardian_one_skill_one_item']=all(sorted(reward_kinds.get(n,[]))==[0,2] for n in (1,2))
    checks['guardian_award_once']=len(events(1,'authoritative_guardian_reward'))==1 and len(events(2,'independent_guardian_reward'))==1
    gold=events(1,'party_reward_gold');gross=sum(number(t,'gross') for _,t in gold);credited=sum(number(t,'credited') for _,t in gold)
    checks['both_players_skipped']=all(len(events(n,'test_native_skip_reward'))==1 for n in (1,2))
    checks['gold_divided_once']=gross>0 and credited==gross//2 and all(number(t,'participants')==2 for _,t in gold)
    checks['live_kill_labels']=all(any(re.search(r'text=\d+ \(\d+\)',t) for _,t in events(n,'coop_kill_label')) for n in (1,2))
    result_labels={n:events(n,'coop_result_kill_label') for n in (1,2)}
    checks['native_result_kill_labels']=all(result_labels.values())
    poi={}
    for n in (1,2):
        poi[n]={}
        for line in (root/f'player-{n}/packets.log').read_text(errors='replace').splitlines():
            m=re.search(r' in\s+0x0224 .*hex=([0-9a-f]+)',line)
            if not m:continue
            body=bytes.fromhex(m[1])
            if body.startswith((99).to_bytes(4,'little')):poi[n][body[4:12].hex()]=body.hex()
    checks['native_guardian_minimap_markers']=bool(poi[1]) and poi[1]==poi[2]
    boss=events(1,'test_unique_boss_kill_requested')
    checks['boss_killed']=bool(boss)
    checks['guest_guardian_bar']=any(number(t,'bars')>0 for _,t in events(2,'test_encounter_hud'))
    checks['guest_timer_after_boss']=bool(boss) and any(at>boss[0][0]+1 and number(t,'next',-1)>0 and number(t,'bars')==0 for at,t in events(2,'test_encounter_hud'))
    checks['muted_test_profiles']=all(all(re.search(r'^'+key+r'\s*=\s*"0"', (root/f'player-{n}/config.toml').read_text(),re.M) for key in ('MasterVolume','MusicVolume','SoundVolume','EnableMusic')) for n in (1,2))
    return dict(passed=all(checks.values()),checks=checks,guardian_reward_kinds=reward_kinds,gold=dict(gross=gross,credited=credited),
                result_labels=result_labels,poi_counts={n:len(poi[n]) for n in (1,2)},computer_use=False,
                scope='Native diagnostic actions in one private co-op run; renderer captures require separate visual review')

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('trial',type=Path);args=parser.parse_args()
    report=assess(args.trial)
    (args.trial/'gameplay-evidence.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))
    raise SystemExit(0 if report['passed'] else 1)
