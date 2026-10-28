"""Four-process native integration evidence for the stable support policy."""
from support_rules_evidence import HEALS,BUFFS,PERSONAL
from support_evidence import events

def assess(logs):
    native=[r for t in logs.values() for r in events(t,'ally_support_native_result')]
    results={str(s):sorted({int(r['target']) for r in native if r.get('spell')==s and ((r.get('after',0)>r.get('before',0)) if s in HEALS else r.get('buff_present')==1)}) for s in HEALS+BUFFS}
    meter=[r for t in logs.values() for r in events(t,'ally_healing_meter_native')]
    credits=sorted({int(r['caster']) for r in meter if r.get('amount',0)>0 and abs(r.get('after',0)-r.get('before',0)-r['amount'])<.02})
    applied=[r for t in logs.values() for r in events(t,'ally_support_applied')]
    identities=[(r.get('caster'),r.get('target'),r.get('seq')) for r in applied]
    nature=sorted({int(r['caster']) for t in logs.values() for r in events(t,'ally_nature_burst') if r.get('amount',0)>0})
    shield=sorted({int(r['caster']) for t in logs.values() for r in events(t,'ally_shield_burst_origin')})
    faults=[line for text in logs.values() for line in text.splitlines() if '[FAILED]' in line or 'ACCESS_VIOLATION' in line]
    checks=dict(all_reviewed_spells_applied=all(results.values()),all_four_native_meters=credits==[1,2,3,4],no_duplicate_delivery=len(identities)==len(set(identities)),no_personal_spells=not any(r.get('spell') in PERSONAL for r in native),druid_proc=bool(nature),priest_proc=bool(shield),no_native_faults=not faults)
    return dict(passed=all(checks.values()),scope='Four native processes on one PC; controlled spell inputs, automatic player targeting and native effects',checks=checks,recipients_by_spell=results,native_meter_casters=credits,nature_casters=nature,shield_casters=shield,faults=faults)
