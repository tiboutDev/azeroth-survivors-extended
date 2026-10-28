"""Re-examine retained two-player evidence without overstating damage coverage."""
import json
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parent))
from support_evidence import events


def assess(logs):
    origins = {}
    nature = {}
    for player, text in logs.items():
        requested = events(text, 'ally_shield_break_native')
        used = events(text, 'ally_shield_burst_origin')
        origins[str(player)] = dict(
            requested=len(requested), used=len(used),
            forwarded=bool(requested) and len(requested) == len(used) and all(
                a.get('caster') == b.get('caster') == player
                and a.get('target') in (1, 2, 3, 4) and a['target'] != player
                and all(k in a and k in b and a[k] == b[k] for k in ('x', 'y'))
                for a, b in zip(requested, used)),
        )
        bursts = events(text, 'ally_nature_burst')
        nature[str(player)] = sum(
            r.get('caster') == player and r.get('target') != player
            and r.get('amount', 0) > 0 for r in bursts)
    oracle = [r for t in logs.values() for r in events(t, 'native_area106_oracle')]
    kinds = sorted({int(r['kind']) for t in logs.values()
                    for r in events(t, 'test_area109_eval')})
    return dict(
        shield_origin_forwarding=origins, nature_ally_bursts=nature,
        native_area_oracle_comparisons=len(oracle),
        native_area_oracle_all_match=bool(oracle) and all(r.get('matches') == 1 for r in oracle),
        hostile_area_kinds_observed=kinds,
        proves_per_enemy_proc_hp_loss=False,
        proves_authoritative_personal_kill_credit=False,
        proves_all_boss_abilities=False,
        scope='Retained controlled two-instance run; no new live run; no physical-PC trial',
    )
