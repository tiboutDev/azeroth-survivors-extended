import sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"tools/dev"))
from reward_wallet_evidence import assess
class RewardWalletEvidence(unittest.TestCase):
 def fixture(self):
  lines=["official party: clock=4 actors=[phase: Choosing, gold: 1000, phase: Choosing, gold: 1000]"]
  for s in range(2):
   for k in range(2):
    for a in range(3):
     old,earned=1000,1200;new=old if a==0 else old-100 if a==1 else old+6;ne=earned+(6 if a==2 else 0)
     wallet=lambda g,e:f"SharedWallet {{ gold: {g}, earned: {e} }}"
     lines.append(f"reward wallet fixture: slot={s} guid={111+s} kind={k} action={a} game=99 before={wallet(old,earned)} after={wallet(new,ne)} shared_before=Some((99, {wallet(old,earned)})) shared_after=Some((99, {wallet(new,ne)}))")
  return "\n".join(lines)
 def test_native_and_shared_reward_transactions(self):self.assertTrue(assess(self.fixture())["passed"])
 def test_missing_or_duplicate_transaction_cannot_pass(self):
  t=self.fixture();self.assertFalse(assess(t.rsplit("\n",1)[0])["passed"]);self.assertFalse(assess(t+"\n"+t.splitlines()[-1])["passed"])
 def test_local_only_change_is_rejected(self):self.assertFalse(assess(self.fixture().replace("shared_after=Some((99, SharedWallet { gold: 900", "shared_after=Some((99, SharedWallet { gold: 1000"))["checks"]["native_matches_shared"])
 def test_wrong_free_reroll_or_double_debit_is_rejected(self):
  self.assertFalse(assess(self.fixture().replace("gold: 900", "gold: 800"))["checks"]["paid_reroll_debits_once"])
  self.assertFalse(assess(self.fixture().replace("phase: Choosing, gold: 1000]", "phase: Choosing, gold: 999]"))["checks"]["both_native_projections_equal"])
if __name__=="__main__":unittest.main()
