"""Strict acceptance checks for the controlled two-player native chest fixture."""
import re

def events(text, name):
    for line in text.splitlines():
        match = re.search(r'event ' + re.escape(name) + r' (.*)$', line)
        if match:
            yield dict(re.findall(r'(\w+)=(\S+)', match[1]))

def assess(host, guest):
    host_pickups = list(events(host, 'party_chest_native_pickup'))
    guest_pickups = [e for e in events(host, 'party_chest_purchased') if e.get('picker') == '2']
    snapshots = {e['revision']: e for e in events(host, 'party_wallet_published')}
    applied = list(events(guest, 'party_wallet_applied'))
    matching = []
    for e in applied:
        authoritative = snapshots.get(e.get('revision'))
        if authoritative and all(e.get(k) == authoritative.get(k) for k in ('gold','earned','cost','opened')) and int(e['opened']) >= 2:
            matching.append(e)
    rewards = sum(int(e.get('count', 0)) for e in events(guest, 'party_chest_rewards'))
    host_group_rewards = list(events(host, 'party_chest_host_choice'))
    guids = {e.get('guid_exact') for e in host_pickups + guest_pickups}
    return dict(passed=bool(host_pickups and guest_pickups and len(guids) >= 2 and matching and rewards == 2 and host_group_rewards),
                host_pickups=len(host_pickups), guest_pickups=len(guest_pickups), guest_rewards=rewards,
                host_group_rewards=len(host_group_rewards), matching_wallet_after_two_chests=bool(matching))


def assess_rounds(host,guest,count,chests=False,guest_number=2):
    def split(text):
        return re.split(r'^.*event round_state_reset player=\d+\s*$',text,flags=re.M)
    hosts,guests=split(host),split(guest)
    result=[]
    for index in range(count):
        h=hosts[index] if index<len(hosts) else ''
        g=guests[index] if index<len(guests) else ''
        starts=all(any(events(text,'native_start')) for text in (h,g))
        playing=all(any(e.get('phase')=='1' and e.get('player')==str(n) for e in events(text,'native_state')) for n,text in [(1,h),(guest_number,g)])
        enemies=all(any(int(e.get('common',0))>0 for e in events(text,'enemy_sync')) for text in (h,g))
        row=dict(round=index+1,native_starts=starts,both_playing=playing,shared_enemies=enemies)
        if chests:row['chests']=assess(h,g)
        row['passed']=starts and playing and enemies and (not chests or row['chests']['passed'])
        result.append(row)
    return result
