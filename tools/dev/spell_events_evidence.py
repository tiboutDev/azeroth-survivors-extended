"""Read aggregate local spell-event evidence from the existing live run."""
from support_evidence import events

def assess(logs):
 players={}
 for number in range(1,5):
  rows=list(events(logs.get(number,''),'spell_events_summary'));previous=None;valid=bool(rows)
  for row in rows:
   values=[row.get(k,-1)for k in ('started','completed','replaced','healed','active')]
   valid=valid and all(isinstance(v,(int,float))and 0<=v<=1_000_000_000 and int(v)==v for v in values)
   if not valid:break
   started,completed,replaced,healed,active=values
   failed=row.get('failed',0)
   valid=valid and isinstance(failed,(int,float))and 0<=failed<=1_000_000_000 and int(failed)==failed
   valid=valid and active<=128 and completed+replaced+failed+active==started
   counters=values[:4]+[failed]
   if previous:valid=valid and all(a>=b for a,b in zip(counters,previous))
   previous=counters
  last=rows[-1]if rows else {}
  passed=valid and all(last.get(k,0)>0 for k in ('started','completed','healed'))
  players[number]=dict(samples=len(rows),last=last,balanced_and_monotonic=valid,passed=bool(passed))
 return dict(passed=all(p['passed']for p in players.values()),players=players,scope='Aggregate event invariants from one four-client support run; exact completion replay is verified by unit/native-memory tests')


def assess_impacts(logs,participants=(1,2,3,4)):
 """Three eligible casters, with an optional fourth Hunter control."""
 participants=tuple(participants)
 if participants not in ((1,2,3),(1,2,3,4)):raise ValueError("Unsupported combat participant scope")
 players={}
 for number in participants:
  rows=list(events(logs.get(number,''),'spell_events_summary'));counts=[r.get('impacts',-1)for r in rows]
  valid=bool(counts)and all(isinstance(v,(int,float))and 0<=v<=1_000_000_000 and int(v)==v for v in counts)
  valid=valid and all(a<=b for a,b in zip(counts,counts[1:]));last=counts[-1]if counts else -1
  passed=valid and (last>0 if number in (1,2,3)else last==0)
  players[number]=dict(samples=len(rows),impacts=last,monotonic=valid,passed=bool(passed))
 return dict(passed=all(p['passed']for p in players.values()),players=players,scope="Three reviewed offensive spells; Hunter unrelated-spell control included only with four participants",hunter_control_tested=4 in participants)


def assess_heal_commands(logs):
 """Each recipient must show bit-exact talent preservation after foreign HEAL."""
 players={}
 for number in range(1,5):
  rows=events(logs.get(number,''),'ally_heal_budget_preserved')
  valid=bool(rows)and all(r.get('target')==number and all(isinstance(r.get(k),(int,float))and 0<=r[k]<=0xffffffff and int(r[k])==r[k]for k in ('before_bits','after_bits'))and r['before_bits']==r['after_bits']for r in rows)
  players[number]=dict(samples=len(rows),passed=bool(valid))
 return dict(passed=all(p['passed']for p in players.values()),players=players,scope='Bit-exact native command restoration for all four recipients; native healing/meters and caster procs are assessed separately')


def assess_returns(logs, require_failure_per_player=True):
 """Require both outcomes and no stale casts; optionally require failure per player."""
 players={};ledger=assess(logs)
 for number in range(1,5):
  rows=events(logs.get(number,''),'spell_events_summary');last=rows[-1]if rows else {}
  passed=ledger['players'][number]['passed']and last.get('completed',0)>0 and (last.get('failed',0)>0 or not require_failure_per_player) and last.get('replaced',-1)==0 and last.get('active',-1)==0
  players[number]=dict(completed=last.get('completed',0),failed=last.get('failed',0),replaced=last.get('replaced',-1),active=last.get('active',-1),passed=bool(passed))
 failed_players=[n for n,p in players.items()if p['failed']>0]
 return dict(passed=all(p['passed']for p in players.values())and bool(failed_players),players=players,failure_players=failed_players,require_failure_per_player=require_failure_per_player,strict_failure_coverage_met=len(failed_players)==4,scope='Native success and balanced identities on every client; native failure on the common callback path. Per-player failure coverage is reported explicitly; exactly-once effects and nested isolation also covered by memory/Windows tests')


def assess_offensive(logs,participants=(1,2,3,4)):
 """The combat fixture places Hammer of Justice on guest 2."""
 participants=tuple(participants)
 if participants not in ((1,2,3),(1,2,3,4)):raise ValueError("Unsupported combat participant scope")
 players={}
 for number in participants:
  rows=events(logs.get(number,''),'spell_events_summary');previous=None;valid=bool(rows)
  for row in rows:
   counts=[row.get(k,-1)for k in ('effect_requests','effect_commands')]
   valid=valid and all(isinstance(v,(int,float))and 0<=v<=1_000_000_000 and int(v)==v for v in counts)
   if previous:valid=valid and all(a>=b for a,b in zip(counts,previous))
   previous=counts
  last=rows[-1]if rows else {};requests=last.get('effect_requests',-1);commands=last.get('effect_commands',-1)
  passed=valid and (requests==0 and commands>0 if number==1 else commands==0 and (requests>0 if number==2 else requests>=0))
  players[number]=dict(samples=len(rows),requests=requests,commands=commands,passed=bool(passed))
 controls=events(logs.get(1,''),'offensive_native_state')
 active=any(r.get('active')==1 and r.get('kind')==3 and r.get('spell')==853 for r in controls)
 return dict(passed=all(p['passed']for p in players.values())and active,players=players,native_hammer_control_observed=active,native_state_samples=len(controls),scope='Guest requests and host commands remain distinct; live positive control covers Hammer of Justice. Other reviewed variants are checked by semantic/native/wire fixtures, not claimed live.')


def assess_periodic(logs):
 """Require personal/foreign native ticks and the unchanged overheal shield."""
 import math,struct
 def f32(value):return struct.unpack('<f',struct.pack('<f',value))[0]
 players={};mixed=False;shield_gain=False
 for number in range(1,5):
  rows=events(logs.get(number,''),'periodic_budget_resolved');valid=bool(rows);personal=False;foreign=False
  for row in rows:
   keys=('player','foreign_rows','personal_rows','raw','foreign','personal','overheal','shield_before','shield_after','shield_expected')
   if not all(k in row and math.isfinite(row[k])for k in keys):valid=False;continue
   valid=valid and row['player']==number and all(0<=row[k]<=512 and int(row[k])==row[k]for k in ('foreign_rows','personal_rows'))
   try:expected=max(0.,f32(f32(row['raw'])-f32(row['foreign'])))
   except (OverflowError,struct.error):valid=False;continue
   valid=valid and f32(row['personal'])==expected and f32(row['shield_after'])==f32(row['shield_expected'])
   personal=personal or row['personal_rows']>0 and row['personal']>0
   foreign=foreign or row['foreign_rows']>0 and row['foreign']>0
   mixed=mixed or row['personal_rows']>0 and row['foreign_rows']>0
   shield_gain=shield_gain or row['shield_after']>row['shield_before']
  players[number]=dict(samples=len(rows),personal_observed=bool(personal),foreign_observed=bool(foreign),calculations_valid=bool(valid),passed=bool(valid and personal and foreign))
 return dict(passed=all(p['passed']for p in players.values())and mixed and shield_gain,players=players,mixed_batch_observed=bool(mixed),native_overheal_shield_gain=bool(shield_gain),scope='One four-client native periodic fixture; per-update personal talent contribution and bit-exact native overheal shield calculation. No hook retirement claimed.')
