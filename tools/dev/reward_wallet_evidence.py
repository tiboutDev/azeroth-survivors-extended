"""Assess bounded native reward actions against the authoritative ASX wallet."""
import re
ROW=re.compile(r"reward wallet fixture: slot=(\d+) guid=(\d+) kind=(\d+) action=(\d+) game=(\d+) before=SharedWallet \{ gold: (\d+), earned: (\d+) \} after=SharedWallet \{ gold: (\d+), earned: (\d+) \} shared_before=Some\(\((\d+), SharedWallet \{ gold: (\d+), earned: (\d+) \}\)\) shared_after=Some\(\((\d+), SharedWallet \{ gold: (\d+), earned: (\d+) \}\)\)")
def assess(text):
 rows=[tuple(map(int,m))for m in ROW.findall(text)]
 keys=[(r[0],r[2],r[3])for r in rows]
 expected={(s,k,a)for s in range(2)for k in range(2)for a in range(3)}
 samples=[re.findall(r"phase: (?:Playing|Choosing), gold: (\d+)",line)for line in text.splitlines()if "official party: clock="in line]
 checks=dict(all_reward_actions=set(keys)==expected and len(keys)==12,
  native_matches_shared=bool(rows)and all(r[4]==r[9]==r[12]and r[5:7]==r[10:12]and r[7:9]==r[13:15]for r in rows),
  free_reroll_unchanged=bool(rows)and all(r[5:7]==r[7:9]for r in rows if r[3]==0),
  paid_reroll_debits_once=bool(rows)and all(r[5]-r[7]in(100,300)and r[6]==r[8]for r in rows if r[3]==1),
  skip_credits_once=bool(rows)and all(r[7]>r[5]and r[7]-r[5]==r[8]-r[6]for r in rows if r[3]==2),
  both_native_projections_equal=any(len(x)==2 for x in samples)and all(x[0]==x[1]for x in samples if len(x)==2))
 return dict(checks=checks,passed=all(checks.values()),actions=[dict(slot=r[0],guid=r[1],kind=r[2],action=r[3],before=dict(gold=r[5],earned=r[6]),after=dict(gold=r[7],earned=r[8]))for r in rows])
