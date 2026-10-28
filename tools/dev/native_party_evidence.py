"""Assess the native Lua frames, separately from packet emission and gameplay."""
import re

ROW = re.compile(r'AZCOOP_FRAME_PROBE members=(\d+) exists=(\d+) shown=(\d+) visible=(\d+) hp=(\d+) max=(\d+) value=(\d+) menu=(\d+)')


def assess(folder):
    players = {}
    for number in (1, 2):
        profile = folder/f'player-{number}'
        script = (profile/'stderr.log').read_text(encoding='utf-8', errors='replace')
        packets = (profile/'packets.log').read_text(encoding='utf-8', errors='replace')
        rows = [tuple(map(int, m.groups())) for m in ROW.finditer(script)]
        visible = [r for r in rows if r[:4] == (1, 1, 1, 1)]
        good = [r for r in visible if r[5] > 0 and 0 <= r[4] <= r[5] and r[4] == r[6]]
        health_values = sorted({r[4] for r in good})
        draw_values=re.findall(r'AZCOOP_FRAME_PROBE[^\n]* draw=(\S+)',script)
        players[number] = {
            'native_xml_loaded': 'PartyFrame.xml loaded (102 frames materialized)' in script,
            'group_list_decoded': '0x007d SMSG_GROUP_LIST ev=1' in packets,
            'samples': len(rows), 'visible_samples': len(visible),
            'matching_health_samples': len(good), 'health_values': health_values,
            'menu_hidden_in_world':bool(rows) and all(r[7]==0 for r in rows),
            'menu_composition_stopped':bool(draw_values) and all(v=='0' for v in draw_values),
            'passed': len(good) >= 3 and len(good) == len(visible) and len(health_values) > 1 and all(r[7]==0 for r in rows),
        }
        players[number]['passed'] &= players[number]['native_xml_loaded'] and players[number]['group_list_decoded']
        players[number]['passed'] &= players[number]['menu_composition_stopped']
    return {'players': players, 'passed': all(p['passed'] for p in players.values()),
            'pixel_reviewed': False, 'probe_transport': 'deliberately tagged native script diagnostics in disposable profiles'}
