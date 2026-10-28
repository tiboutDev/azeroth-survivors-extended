"""Evidence for the approved player-only rules; one game, two participants."""
from support_evidence import events

HEALS=(635,331,8936,15237,25912)
BUFFS=(6673,17,139,774,324,467)
PERSONAL=(11426,642,20925,12042,13750,5171,14278)

def assess(logs):
    native=[r for t in logs.values() for r in events(t,'ally_support_native_result')]
    results={str(s):sorted({int(r['target']) for r in native if r.get('spell')==s and
        ((r.get('after',0)>r.get('before',0)) if s in HEALS else r.get('buff_present')==1)}) for s in HEALS+BUFFS}
    meter=[r for t in logs.values() for r in events(t,'ally_healing_meter_native')]
    credits={str(s):sorted({int(r['caster']) for r in meter if r.get('spell')==s and r.get('amount',0)>0 and abs(r.get('after',0)-r.get('before',0)-r['amount'])<.02}) for s in HEALS+(139,774)}
    hybrids=[r for t in logs.values() for r in events(t,'ally_hybrid_elementary')]
    elementary=all(abs(r['amount']/r['caster_max']-({15237:.04,25912:.03}[int(r['spell'])]))<1e-6 for r in hybrids)
    elementary=elementary and all({int(r['target']) for r in hybrids if r.get('spell')==s}=={1,2} for s in (15237,25912))
    excluded=not any(r.get('spell') in PERSONAL for r in native)
    shield=all(any(r.get('spell')==17 and r.get('shield',0)>0 and r.get('target')==n for r in native) for n in (1,2))
    nature={int(r['caster']) for t in logs.values() for r in events(t,'ally_nature_burst') if r.get('amount',0)>0}=={1,2}
    bursts=[r for t in logs.values() for r in events(t,'ally_shield_burst_origin')]
    shield_bursts={int(r['caster']) for r in bursts}=={1,2}
    faults=[line for text in logs.values() for line in text.splitlines() if '[FAILED]' in line or 'ACCESS_VIOLATION' in line]
    passed=all(v==[1,2] for v in results.values()) and all(v==[1,2] for v in credits.values()) and elementary and excluded and shield and nature and shield_bursts and not faults
    return dict(passed=passed,scope='One game, two players; controlled native casts, no three-game restart series',
        verified_recipients_by_spell=results,native_meter_casters_by_spell=credits,
        elementary_hybrid_amounts=elementary,personal_spells_excluded=excluded,native_shield_absorption_present=shield,
        druid_bursts_both_casters=nature,priest_native_bursts_both_casters=shield_bursts,faults=faults)
