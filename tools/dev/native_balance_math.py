"""Portable common-item arithmetic, reviewed against native machine code.

RVA evidence and source-image identity are recorded in the balance reference.
No client installation, runtime memory or game process is accessed here.
"""
import math
import struct

RARITIES = ('Common', 'Uncommon', 'Rare', 'Epic', 'Legendary')
MULTIPLIERS = (1.0, 1.4, 1.8, 2.4, 3.2)
AMULET_RATES = (.6, .5, .35, .25, .2)


def f32(x):
    return struct.unpack('<f', struct.pack('<f', x))[0]


def rarity_index(rarity):
    if not isinstance(rarity, int) or not 0 <= rarity <= 5:
        raise ValueError('Rarity must be an integer from 0 to 5')
    return min(rarity, 4)


def jewelry_gain(base, kind, rarity, value=0.0, rank=0):
    """Next gain, not final total. rank is the number already acquired."""
    r = rarity_index(rarity)
    if not isinstance(rank, int) or rank < 0 or not math.isfinite(value) or value < 0:
        raise ValueError('Invalid jewelry history')
    if not math.isfinite(base) or base <= 0 or not 0 <= kind <= 19:
        raise ValueError('Invalid jewelry definition')
    b, x = f32(base), f32(value)
    if kind == 16:
        return 2.0 if r >= 3 else 1.0
    q = f32(f32(f32(MULTIPLIERS[r]) * b) * (1.5 if rank == 0 else 1.0))
    if kind in (2, 5, 6):
        threshold = f32(3.0 * b)
        step = f32(q * .0625)
        y = x
        for _ in range(16):
            if y > threshold:
                t = f32(threshold / y)
                weight = f32(f32(t * t) * .25)
            else:
                weight = f32(f32(f32(-.75 * y) / threshold) + 1.0)
            y = f32(y + f32(weight * step))
        return f32(y - x)
    if kind in (0, 1, 3, 4, 8, 9):
        denominator = f32(1 + f32(f32(rank) * f32(.08))) if rank else 1.0
        gain = f32(q / denominator)
        return min(gain, max(f32(f32(.6) - x), 0.0)) if kind == 9 else gain
    factor = 3.0 if kind in (7, 15, 18) else 10.0 if kind == 19 else 5.0
    cap = f32(factor * b)
    room = max(f32(cap - x), 0.0)
    return min(f32(f32(q * room) / cap), room)


def jewelry_history(row, rarities):
    value = 0.0
    for rank, rarity in enumerate(rarities):
        gain = jewelry_gain(row['coefficient'], row['stat_kind'], rarity, value, rank)
        value = f32(value + gain)
        if row['stat_kind'] == 16:
            value = min(value, 2.0)  # Unspent lives only; spent lives are separate state.
    return value


def jewelry_bonuses(row, value):
    kind, b = row['stat_kind'], row['coefficient']
    if kind == 1:
        return {'Cooldown recovery input (%)':100*value, 'Cooldown reduction alone (%)':100*(1-1/(1+value))}
    if kind == 5:
        return {'Damage reduction (%)': 100 * min(value, f32(3*b)), 'Maximum health (%)': 400*value}
    if kind == 6:
        return {'Maximum health (%)': 100*value, 'Damage reduction (%)': 100*min(f32(value*f32(.16)), f32(.12))}
    names = {0:'Damage (%)',1:'Cooldown reduction (%)',2:'Maximum health (%)',3:'Critical chance (pp)',
             4:'Critical damage (pp)',7:'Health / sec',8:'Area (%)',9:'Echo (pp)',10:'Movement speed (%)',
             11:'Pickup radius (%)',12:'Experience (%)',13:'Gold (%)',14:'Luck (pp)',15:'Health / kill',
             16:'Unspent lives',17:'Buff power (%)',18:'Lifesteal (%)',19:'DoT haste (%)'}
    return {names[kind]:value * (1 if kind in (7,15,16) else 100)}


def amulet_scale(rank, rarity):
    if not isinstance(rank, int) or rank < 1:
        raise ValueError('Amulet rank starts at one')
    r = rarity_index(rarity)
    decay = f32(f32(f32(AMULET_RATES[r]) * -.5) + 1)
    return f32(3 - f32(2 * f32(decay ** (rank-1))))


def amulet_effects(row, rank):
    s = amulet_scale(rank, row['rarity'])
    values = []
    for e in row['effects']:
        raw = bytes.fromhex(e['raw_hex'])
        def f(at): return struct.unpack_from('<f',raw,at)[0]
        def scaled(at): return f32(f(at)*s)
        tag = e['tag']
        if tag == 43:
            v = scaled(4)
            unit = ' HP/sec' if e['stat_id']==6 else ' HP/kill' if e['stat_id']==15 else '%'
            values.append(('Cooldown recovery input' if e['stat_id']==1 else e['stat'],v if unit!='%' else 100*v,unit))
        elif tag == 9:
            values += [('Critical-hit DoT',100*scaled(8),'% of triggering damage'),('DoT duration',f(12),'sec')]
        elif tag == 6:
            values += [('Additional chain targets',1,'target'),('Chain damage',100*scaled(8),'% of triggering damage')]
        elif tag == 44:
            targets = min(math.floor(scaled(32)+.0001), math.ceil(3*f(32)-.001)-1)
            values += [('Additional spell targets',targets,'targets')]
        elif tag == 19:
            values += [('Death-prevention cooldown',f(4),'sec'),('Health after death prevention',100*f(8),'% of maximum health')]
        elif tag == 4:
            values += [('Critical-hit splash',100*scaled(4),'% of triggering damage'),('Splash radius',f(8),'yd'),('Proc limit',struct.unpack_from('<I',raw,20)[0],'/sec')]
        elif tag == 11:
            values += [('Kill explosion',100*scaled(4),'% of victim maximum health'),('Explosion radius',f(8),'yd'),('Proc limit',struct.unpack_from('<I',raw,16)[0],'/sec')]
        else:
            raise ValueError(f'Unreviewed common amulet effect {tag}')
    return values
