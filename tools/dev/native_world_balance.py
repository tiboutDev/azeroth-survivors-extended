"""Read-only native enemy/economy reference; refresh only on explicit request.

The recorded version is provenance, not a restriction on consulting the document.
Extraction itself requires the reviewed image to avoid decoding unknown layouts.
"""
import argparse
import json
import math
import struct
from pathlib import Path
from native_balance import Image, VERSION, SHA256
from native_balance_math import f32


def growth(minutes, coefficient, guardian=False):
    """Native float32 health polynomial, before spawn multipliers."""
    if not all(math.isfinite(v) and v >= 0 for v in (minutes, coefficient)):
        raise ValueError('Invalid growth inputs')
    m=f32(minutes)
    linear=f32(f32(f32(.1)*m)+1)
    quadratic=f32(f32(f32(.036)*m)*m)
    cubic=f32(f32(f32(f32(.00594 if guardian else .00704)*m)*m)*m)
    return f32(f32(cubic+f32(quadratic+linear))*f32(coefficient))


def party_factor(players, map_index, coefficient, host=False):
    if not isinstance(players,int) or not 1 <= players <= 5 or not 0 <= map_index < 12:
        raise ValueError('Invalid party/map')
    return 1 + coefficient*(players-1)*(.7 if map_index<3 else 1) if host else 1


def enemy_values(row, map_row, minutes, *, players=1, host=False,
                 run_multiplier=1, heroic_health=1, heroic_damage=1, endless_minutes=0):
    """Balancing estimates: full constructor formulas, not bit-identical outputs."""
    if not all(math.isfinite(v) and v >= 0 for v in (minutes,run_multiplier,heroic_health,heroic_damage,endless_minutes)):
        raise ValueError('Invalid encounter inputs')
    a=map_row['health_coefficient']*run_multiplier
    party=party_factor(players,map_row['index'],1.04 if row['kind']=='boss' else .72,host)
    if host:party*=.8
    if row['kind']=='boss':
        hp=row['health']*5.75*a*heroic_health*party*1.22**endless_minutes
        damage=row['damage']*math.sqrt(a)*heroic_damage*1.096**endless_minutes
        speed=row['speed']
    else:
        guardian=row['kind']=='guardian'
        hp=row['health']*(4.8 if guardian else 4.95)*growth(minutes,a,guardian)
        hp*=1+map_row['late_health_coefficient']*min(minutes/20,1)**2
        hp*=heroic_health*party*1.22**endless_minutes
        damage=row['damage']*(1 if guardian else 1.25)*math.sqrt(a)
        damage*=1+.07*minutes+.015*minutes**2+.0002*minutes**3
        damage*=heroic_damage*1.096**endless_minutes
        speed=row['speed']*1.05
    return dict(health=hp,damage=damage,speed=speed)


def cache_next(cost, difficulty=0):
    if not isinstance(cost,int) or not 0 <= cost <= 0xffffffff or not 0 <= difficulty <= 5:
        raise ValueError('Invalid cache inputs')
    return min(0xffffffff,int(f32(f32(f32(cost)*f32(1.3))+(6.25 if difficulty>=3 else 5))))


def extract(image):
    def u(a):return struct.unpack('<I',image.read(a,4))[0]
    def f(a):return struct.unpack('<f',image.read(a,4))[0]
    def vector(a):
        p,n=struct.unpack('<QQ',image.read(a,16))
        if not 0<n<=64:raise ValueError('Unexpected definition count')
        return p-image.base,n
    maps=[]
    for i in range(12):
        a=0x7102c30+i*240
        m=dict(index=i,key=image.string(a+8),name=image.string(a+24),rva=hex(a),
               health_coefficient=f(a+224),late_health_coefficient=f(a+228),enemies=[],bosses=[])
        p,n=vector(a+120)
        for j in range(n):
            b=p+j*80;minute=f(b+72);guardian=image.read(b+76,1)[0]
            if guardian not in (0,1):raise ValueError('Invalid guardian flag')
            m['enemies'].append(dict(name=image.string(b+16),rva=hex(b),kind='guardian' if guardian else 'ordinary',
                entry=u(b+32),display=u(b+36),scale=f(b+40),creature_type=u(b+44),health=f(b+48),damage=f(b+52),
                attack_interval=f(b+56),speed=f(b+60),xp_definition=u(b+64),unresolved_death_accumulator=f(b+68),
                minute=minute if math.isfinite(minute) else None))
        p,n=vector(a+136)
        for j in range(n):
            b=p+j*104
            m['bosses'].append(dict(name=image.string(b),rva=hex(b),kind='boss',entry=u(b+64),display=u(b+68),scale=f(b+72),
                 creature_type=u(b+76),minute=f(b+80),health=f(b+84),damage=f(b+88),speed=f(b+92)))
        maps.append(m)
    heroic=[dict(level=i,health=f(0x7142088+48*i-16),damage=f(0x7142088+48*i-12)) for i in range(1,6)]
    return dict(schema=1,source=dict(version=VERSION,sha256=SHA256),maps=maps,heroic=heroic)


def render(d):
    lines=['# Native enemies and encounter scaling','',
      'Native-only balancing reference. The recorded client version is provenance, not a strict usage requirement. Refresh values only when explicitly requested.',
      '',f"Source: {d['source']['version']}; SHA-256 `{d['source']['sha256']}`.",'',
      '## Reading the tables','',
      'HP and damage below are **definition inputs**, not final in-run values. Speed is native world units/second; intervals and times are seconds and simulation minutes respectively. Activation minutes indicate eligibility/scheduling, not a guaranteed ordinary spawn. A dash means scripted eligibility (infinite native threshold). XP is the raw definition input, before reward processing. Creature IDs and display IDs are retained in the JSON companion.',
      '', '## Spawn formulas','',
      'Let `m` be elapsed simulation minutes; `e` elapsed endless minutes (zero outside endless); `A = map health coefficient × run multiplier`; `L` the map late-health coefficient. The additional run multiplier is the native run field at `+0xf14`; it is an explicit input, not assumed universally equal to 1. `H` and `D` are heroic multipliers. `S = 0.7` on the first three maps, otherwise `1`.',
      '', 'For ordinary enemies / guardians:',
      '', '```text',
      'P(m) = 1 + 0.1m + 0.036m² + c m³',
      'c = 0.00704 ordinary; 0.00594 guardian',
      'HP = baseHP × (4.95 ordinary; 4.8 guardian) × A × P(m)',
      '     × (1 + L × clamp(m/20, 0, 1)²) × 1.22^e × H × partyHP',
      'partyHP = 1 solo; 0.8 × (1 + 0.72 × (N−1) × S) cooperative host',
      'Damage = baseDamage × (1.25 ordinary; 1 guardian) × sqrt(A)',
      '         × (1 + 0.07m + 0.015m² + 0.0002m³) × 1.096^e × D',
      'Movement speed = baseSpeed × 1.05',
      '```','',
      'Boss constructor:', '', '```text',
      'HP = baseHP × 5.75 × A × H × 1.22^e × bossPartyHP × spawnScale',
      'bossPartyHP = 1 solo; 0.8 × (1 + 1.04 × (N−1) × S) cooperative host',
      'Damage = baseDamage × sqrt(A) × D × 1.096^e',
      'Movement speed = baseSpeed',
      '```','',
      'Scheduled bosses use spawnScale=1. A highest-difficulty extra-spawn path uses 0.6. Bosses do not use the ordinary elapsed-time polynomial. These are constructor baselines; abilities, phases, attacks and temporary effects may have their own multipliers. No party damage multiplier appears in these constructor paths. A one-member cooperative host still gets the native 0.8 HP factor; it is distinct from solo.',
      '', 'Ordinary population target: 50 solo; `round(50 × (1 + 0.312 × (N−1) × S))` in coop. This is not total entities including bosses, summons and scripted enemies, nor a complete model of spawn pacing.',
      '', 'Formula evidence: ordinary/guardian constructor RVA `0x5cdee20`; health polynomials `0x5d55e90` / `0x5d55ee0`; boss constructor `0x5cf4170`; schedule `0x5cf54a0`. Calculated examples are balancing estimates; native arithmetic uses float32.',
      '', '## Heroic multipliers','', '| Level | HP | Damage |','| --- | ---: | ---: |']
    for h in d['heroic']:lines.append(f"| {h['level']} | {h['health']:.6g} | {h['damage']:.6g} |")
    lines+=['','Level 0 uses 1/1. Source: difficulty table at RVA `0x7142088`, stride 48, health/damage at row −16/−12.','']
    for m in d['maps']:
        lines += [f"## {m['name']}",'',f"Map coefficient **{m['health_coefficient']:.6g}**; late-health coefficient **{m['late_health_coefficient']:.6g}**.",'',
          '| Enemy | Kind | Minute | Base HP | Base damage | Interval | Speed | XP input |','| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |']
        for row in m['enemies']+m['bosses']:
            vals=[row['name'],row['kind'],row['minute'],row['health'],row['damage'],row.get('attack_interval'),row['speed'],row.get('xp_definition')]
            lines.append('| '+' | '.join('-' if v is None else format(v,'.6g') if isinstance(v,(int,float)) else v for v in vals)+' |')
    lines+=['','## Worked comparisons','',
      'Normal difficulty, solo, run multiplier=1, no endless. Guardians evaluated at their scheduled minute; bosses at their scheduled arrival. Values rounded for reading.',
      '', '| Map / enemy | Minute | Final HP estimate | Base attack damage estimate |','| --- | ---: | ---: | ---: |']
    for m in d['maps']:
        for row in [next(x for x in m['enemies'] if x['kind']=='guardian'),m['bosses'][0]]:
            v=enemy_values(row,m,row['minute'])
            lines.append(f"| {m['name']} / {row['name']} | {row['minute']:.3g} | {v['health']:.1f} | {v['damage']:.1f} |")
    lines+=['','## Limits','',
      'No class-specific damage model, boss ability catalog, phase mechanics, spawn-location probability or time-to-kill prediction is implied. The raw enemy field +68 feeds a decaying death accumulator; its full downstream meaning is unresolved and it must not be treated as gold per kill. Complete raw definitions and addresses are in [native-world-balance.json](native-world-balance.json). Economy and XP reward processing are described in [ECONOMY.md](ECONOMY.md).','']
    return '\n'.join(lines)


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--exe',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    d=extract(Image(a.exe.read_bytes()));a.output.mkdir(parents=True,exist_ok=True)
    (a.output/'native-world-balance.json').write_text(json.dumps(d,indent=2,allow_nan=False)+'\n')
    (a.output/'ENEMIES.md').write_text(render(d))
    print(f"Extracted {len(d['maps'])} maps, {sum(len(m['enemies']) for m in d['maps'])} enemy/guardian definitions, {sum(len(m['bosses']) for m in d['maps'])} bosses")
if __name__=='__main__':main()
