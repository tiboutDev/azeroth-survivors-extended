import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools/dev"))
from treasure_goblin_evidence import assess


class GoblinEvidenceTests(unittest.TestCase):
    def fixture(self):
        host = "\n".join([
            "ASX active:",
            "treasure goblin arena: center=[0,0] radius=120 clearance=2",
            "treasure goblin equipment: native armor compositor selected",
            "treasure goblin spawned: game=a guid=10 clock=18",
            "treasure goblin spawned: game=a guid=11 clock=50",
            "treasure goblin fine motion: guid=10 samples=1000 min_projection=0 max_gap=0.02",
            "treasure goblin fine motion: guid=11 samples=800 min_projection=-0.001 max_gap=0.03",
            "treasure goblin escaped: guid=10",
            "treasure goblin rewarded: guid=11 recipients=2",
            'treasure goblin amulets: event=11 slot=0 guid=70 count=3 marks=1 keys=["a", "b", "c"]',
            'treasure goblin amulets: event=11 slot=1 guid=71 count=3 marks=1 keys=["d", "e", "f"]',
            "treasure goblin taunt control: normal=true goblin=false",
            "treasure goblin native: guid=10 clock=18 elite=true pursuers=[[-2,0,0]] position=[0,0,0]",
            "treasure goblin native: guid=10 clock=20 pursuers=[[-2,0,0]] position=[10,0,0]",
        ])
        guest = "ASX active:\ntreasure goblin equipment: native armor compositor selected\ntreasure goblin received: opcode=61 guid=0\n" + "\n".join(
            f"treasure goblin received: opcode={op} guid={guid}"
            for op in ("a9", "dd", "aa") for guid in (10, 11)
        )
        return host, guest

    def test_solo_and_party_require_actual_lifecycle(self):
        host, guest = self.fixture()
        self.assertTrue(assess([host.replace('treasure goblin amulets: event=11 slot=1 guid=71 count=3 marks=1 keys=["d", "e", "f"]', "")])["passed"])
        self.assertTrue(assess([host, guest])["passed"])
        self.assertFalse(assess(["ASX active:"])["passed"])
        self.assertFalse(assess([])["passed"])

    def test_wrong_creature_or_missing_movement_cannot_validate_replication(self):
        host, guest = self.fixture()
        self.assertFalse(assess([host, guest.replace("guid=11", "guid=99")])["passed"])
        self.assertFalse(assess([host, guest.replace("opcode=dd", "opcode=de")])["passed"])

    def test_duplicate_reward_escape_reward_and_failure_are_rejected(self):
        host, guest = self.fixture()
        for extra in ("treasure goblin rewarded: guid=11 recipients=2", "treasure goblin rewarded: guid=10 recipients=2", "Treasure Goblin event stopped: failure"):
            with self.subTest(extra=extra):
                self.assertFalse(assess([host + "\n" + extra, guest])["passed"])

    def test_disable_preserves_core_and_has_no_creatures(self):
        off = "ASX active:\nTreasure Goblins disabled; core ASX coop remains active."
        self.assertTrue(assess([off, off], disabled=True)["passed"])
        self.assertFalse(assess([off + "\ntreasure goblin spawned:"], disabled=True)["passed"])
        self.assertFalse(assess(["Treasure Goblins disabled;"], disabled=True)["passed"])

    def test_return_toward_player_and_missing_armor_are_rejected(self):
        host, guest = self.fixture()
        self.assertFalse(assess([host.replace("min_projection=0 ", "min_projection=-1 "), guest])["passed"])
        self.assertFalse(assess([host, guest.replace("treasure goblin equipment: native armor compositor selected", "")])["passed"])

    def test_missing_or_sparse_fine_evidence_cannot_pass(self):
        host, guest = self.fixture()
        for before, after in (("samples=1000", "samples=3"), ("max_gap=0.02", "max_gap=2"), ("min_projection=0 ", "min_projection=nan "), ("fine motion: guid=10", "fine motion: guid=99")):
            with self.subTest(before=before):
                self.assertFalse(assess([host.replace(before, after), guest])["passed"])

    def test_curved_sparse_chord_does_not_override_fine_movement(self):
        host, guest = self.fixture()
        host = host.replace("pursuers=[[-2,0,0]]", "pursuers=[[0,0,0]]").replace("position=[0,0,0]", "position=[8,0,0]").replace("position=[10,0,0]", "position=[-5,11,0]")
        self.assertTrue(assess([host, guest])["passed"])

    def test_leaving_the_arena_is_rejected(self):
        host, guest = self.fixture()
        self.assertFalse(assess([host.replace("position=[10,0,0]", "position=[121,0,0]"), guest])["passed"])

    def test_zero_marks_preserve_amulet_reward_and_fractions_are_rejected(self):
        host, guest = self.fixture()
        self.assertTrue(assess([host.replace("marks=1", "marks=0"), guest])["passed"])
        self.assertFalse(assess([host.replace("marks=1", "marks=0.5"), guest])["passed"])
