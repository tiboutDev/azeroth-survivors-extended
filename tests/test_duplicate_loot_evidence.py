import json,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"tools/dev"))
from duplicate_loot_evidence import assess

def fixture(players):
 logs=[[]for _ in range(players)];ticket=0
 for boss,(base,amount)in enumerate([(20,24),(30,45),(45,54),(68,82),(101,121)],1):
  for slot in range(players):
   ticket+=1;paid=slot==0 or boss>1;items=[101100]if paid else[];awards=[amount]if paid else[];peer="None"if slot==0 else"Some(0)"
   if paid:
    mult=1.5 if boss==2 else 1.2;fortune=.3 if boss==2 else 0
    if players==2:logs[0].append(f"duplicate loot gold provenance: slot={slot} base={base} multiplier={mult} personal_fortune={fortune} shared_fortune={fortune}")
    logs[0].append(f"duplicate loot gold: base={base} count=1 amount={amount} total={amount} balance=500 earned=500")
    value=dict(type="drop",name=f"Pauldrons — +{amount} gold",icon="INV_Shoulder_28",new=False)
    logs[slot].append(f"duplicate loot native UI: item=101100 amount={amount} payload={json.dumps(value)}")
   else:logs[slot].append("duplicate loot native item: item=101100 known=false")
   logs[0].append(f"duplicate loot settled: ticket={ticket} boss={boss} peer={peer} items={items} awards={awards}")
 logs[0]+=["official party: clock=30 [ActorObservation { slot: 0, phase: Playing, gold: 500 }, ActorObservation { slot: 1, phase: Playing, gold: 500 }]"for _ in range(3)]
 for text in logs:text.append("duplicate loot victory display: gold=500")
 return["\n".join(s)for s in logs]
class LootEvidence(unittest.TestCase):
 def test_complete_solo_and_two_recipient_paths(self):
  for players in(1,2):self.assertTrue(assess(fixture(players),players)["passed"])
 def test_missing_or_wrong_ui_is_not_payment_evidence(self):
  logs=fixture(2);logs[1]=logs[1].replace("+54 gold","+999 gold")
  self.assertFalse(assess(logs,2)["passed"])
 def test_native_final_boss_guest_claim_is_required(self):
  logs=fixture(2);logs[0]="\n".join(line for line in logs[0].splitlines()if not("boss=5 peer=Some"in line))
  self.assertFalse(assess(logs,2)["checks"]["final_boss_guest_paid"])
 def test_duplicate_ticket_fails(self):
  logs=fixture(2);logs[0]=logs[0].replace("ticket=10","ticket=9")
  self.assertFalse(assess(logs,2)["checks"]["unique_receipts"])
