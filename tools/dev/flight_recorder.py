"""Read the native core's bounded AZFLIGHT ring; never open a live process."""
from pathlib import Path
import struct
import sys

HOOKS = ['TICK', 'SERIALIZE', 'WRITE', 'PAUSE', 'RESUME', 'START', 'AWARD_XP', 'CHOOSE', 'MOVE', 'DAMAGE', 'HURT', 'SPAWN', 'SPAWN_BOSS', 'FIND_SPAWN', 'SPAWN_LOOT', 'REMOVE_LOOT', 'OFFER_CHOICES', 'APPLY_EFFECT', 'DEFEAT_UI', 'UI_COMMAND', 'ENEMY_AOE', 'GROUND_AOE', 'ENEMY_TARGET', 'HUD_DELIVER', 'APPLY_DOT', 'MAP_SELECT', 'BOSS_DEATH', 'MENU_LOGIN', 'SUPPORT_CAST', 'SUPPORT_HEAL', 'SUPPORT_ELIGIBILITY', 'SUPPORT_BUFF_ELIGIBILITY', 'SUPPORT_METER', 'SUPPORT_CAST_END', 'SUPPORT_HEAL_PROC', 'SUPPORT_PERIODIC_PROC', 'SUPPORT_SHIELD_BREAK', 'SUPPORT_BURST_ORIGIN', 'ENEMY_DEATH_109', 'KILL_LABEL_109', 'REWARD_SKIP_110', 'REWARD_GOLD_110', 'KILL_RESULT_110', 'DAMAGE_RETURN_109', 'DAMAGE_ENTRY_109', 'ENEMY_CAST_AIM_112', 'ENEMY_AIM_112', 'PULL_112', 'HAZARDS_112', 'GUARDIAN_DEFINITION_112', 'REWARD_SKIP_SCALE_1133', 'REWARD_GOLD_SCALE_1133']
MAGIC = 0x544847494C465A41
RECORD = struct.Struct('<QQIIII')


def records(data):
    if len(data) < 64 or struct.unpack_from('<Q', data)[0] != MAGIC:
        raise ValueError('Not an AZFLIGHT file')
    count, size = struct.unpack_from('<II', data, 8)
    if size != RECORD.size or count > 8192 or len(data) < 64 + count * size:
        raise ValueError('Invalid or truncated AZFLIGHT ring')
    return sorted((row for row in RECORD.iter_unpack(data[64:64 + count * size]) if row[0]), key=lambda row: row[0])


def describe(row):
    sequence, tick, thread, kind, a, b = row
    hook = HOOKS[a] if a < len(HOOKS) else '?'
    action = {1: 'enter', 2: 'leave', 3: 'call start', 4: 'call end'}.get(kind)
    detail = f'{action} {hook} {b}' if action else f'kind {kind} {a:#x} {b:#x}'
    return f'#{sequence} t{thread} tick={tick} {detail}'


if __name__ == '__main__':
    if len(sys.argv) not in (2, 3):
        raise SystemExit('Usage: flight.py azcoop-flight.bin [tail-count]')
    with Path(sys.argv[1]).open('rb') as source:
        data = source.read(64 + 8192 * RECORD.size + 1)
    rows = records(data)
    tail = min(8192, max(1, int(sys.argv[2]))) if len(sys.argv) == 3 else 30
    print(f'{len(rows)} records')
    for row in rows[-tail:]:
        print(describe(row))
