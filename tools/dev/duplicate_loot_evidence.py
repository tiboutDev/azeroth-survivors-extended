"""Assess actual native loot receipts; synthetic tests do not validate gameplay."""
import ast,collections,json,math,re,struct

def f32(v):return struct.unpack("f",struct.pack("f",v))[0]
def assess(logs,players):
 host=logs[0]
 receipts=[]
 for ticket,boss,peer,items,awards in re.findall(r"duplicate loot settled: ticket=(\d+) boss=(\d+) peer=(None|Some\(\d+\)) items=(\[[^]]*\]) awards=(\[[^]]*\])",host):
  receipts.append(dict(ticket=int(ticket),boss=int(boss),slot=0 if peer=="None"else 1,items=ast.literal_eval(items),awards=ast.literal_eval(awards)))
 ui=[]
 for text in logs:
  ui.append([(int(item),int(amount),json.loads(value))for item,amount,value in re.findall(r"duplicate loot native UI: item=(\d+) amount=(\d+) payload=(\{[^\r\n]+\})",text)])
 grants=[tuple(map(int,row))for row in re.findall(r"duplicate loot gold: base=(\d+) count=(\d+) amount=(\d+) total=(\d+) balance=(\d+) earned=(\d+)",host)]
 def credited(slot,boss,item=101100):
  return [amount for r in receipts if(r["slot"],r["boss"])==(slot,boss)for ident,amount in zip(r["items"],r["awards"])if ident==item]
 checks=dict(unique_receipts=bool(receipts)and len({r["ticket"]for r in receipts})==len(receipts),matched_receipt_lengths=all(len(r["items"])==len(r["awards"])for r in receipts),five_host_compensations=all(credited(0,b)and all(a>0 for a in credited(0,b))for b in range(1,6)),native_gold_batch_totals=bool(grants)and all(count>0 and total==count*amount for base,count,amount,total,balance,earned in grants),base_curve={r[0]for r in grants}>={20,30,45,68,101},native_ui_matches_credited_gold=all(bool(rows)and all(amount>0 and f"+{amount} gold"in value.get("name","")and value.get("icon")and value.get("new")is False for ident,amount,value in rows)for rows in ui),final_boss_host_paid=bool(credited(0,5)))
 expected_ui=[collections.Counter()for _ in range(players)]
 for receipt in receipts:
  expected_ui[receipt["slot"]].update((i,a)for i,a in zip(receipt["items"],receipt["awards"])if a>0)
 checks["ui_receipts_match_wallet"]=all(collections.Counter((i,a)for i,a,v in ui[slot]if a>0)==expected_ui[slot]and bool(expected_ui[slot])for slot in range(players))
 checks["native_forced_item_icon_preserved"]=all(all(v.get("icon")=="INV_Shoulder_28"for i,a,v in rows if i==101100)for rows in ui)
 totals=[int(x)for x in re.findall(r"duplicate loot victory display: gold=(\d+)",host)]
 checks["victory_total_refreshed"]=bool(grants)and all(int(grants[-1][-1])in[int(x)for x in re.findall(r"duplicate loot victory display: gold=(\d+)",text)]for text in logs)
 checks["victory_refresh_bounded"]=all(0<len(re.findall(r"duplicate loot victory display:",text))<=2 for text in logs)
 if players==1:
  expected={1:24,2:45,3:54,4:82,5:121}
  checks["solo_bonus_once"]=all(credited(0,b)and all(a==expected[b]for a in credited(0,b))for b in expected)
 else:
  checks.update(first_guest_item_is_new=bool(re.search(r"native item: item=101100 known=false",logs[1])),new_guest_gets_no_gold=all(not r["awards"]for r in receipts if r["slot"]==1 and r["boss"]==1)and any(r["slot"]==1 and r["boss"]==1 for r in receipts),later_guest_duplicates_paid=all(credited(1,b)and all(a>0 for a in credited(1,b))for b in range(2,6)),per_player_full_credit=all(credited(0,b)and credited(1,b)and credited(0,b)[0]==credited(1,b)[0]for b in range(2,6)),final_boss_guest_paid=bool(credited(1,5)))
  normalized=[];current=None
  for line in host.splitlines():
   row=re.search(r"gold provenance: slot=(\d+) base=(\d+) multiplier=([0-9.eE+-]+) personal_fortune=([0-9.eE+-]+) shared_fortune=([0-9.eE+-]+)",line)
   if row:
    slot,base,mult,personal,shared=row.groups();effective=f32(f32(f32(float(mult))-f32(float(personal)))+f32(float(shared)));current=(int(slot),int(base),math.floor(f32(int(base)*effective)+.5))
   row=re.search(r"duplicate loot gold: base=(\d+) count=\d+ amount=(\d+)",line)
   if row and current:normalized.append((current,int(row[1]),int(row[2])));current=None
  checks["shared_bonus_once_no_division"]=len(normalized)>=9 and all(base==expected[1]and amount==expected[2]for expected,base,amount in normalized)
  pairs=[]
  for line in host.splitlines():
   if "official party: clock="not in line:continue
   actors=re.findall(r"ActorObservation \{ slot: (\d+), phase: \w+, gold: (\d+)",line)
   if len(actors)==2:pairs.append([int(g)for slot,g in actors])
  checks["shared_wallet_converged"]=len(pairs)>=3 and all(a[0]==a[1]for a in pairs[-3:])
 return dict(checks=checks,passed=all(checks.values()),receipts=receipts,gold_batches=grants,ui=ui)
