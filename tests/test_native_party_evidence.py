import importlib.util
from pathlib import Path
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('native_party_evidence', ROOT/'tools/dev/native_party_evidence.py')
evidence=importlib.util.module_from_spec(spec);spec.loader.exec_module(evidence)


class NativePartyEvidenceTests(unittest.TestCase):
    def trial(self,root,shown=1,value_delta=0,opcode='0x007d SMSG_GROUP_LIST ev=1',draw='0'):
        for number in (1,2):
            folder=Path(root)/f'player-{number}';folder.mkdir()
            (folder/'packets.log').write_text(opcode)
            rows=['PartyFrame.xml loaded (102 frames materialized)']
            for hp in (100,99,98):
                rows.append(f'AZCOOP_FRAME_PROBE members=1 exists=1 shown={shown} visible={shown} hp={hp} max=100 value={hp+value_delta} menu=0 draw={draw}')
            (folder/'stderr.log').write_text('\n'.join(rows))
        return evidence.assess(Path(root))

    def test_requires_native_visibility_and_matching_dynamic_health_on_both_clients(self):
        with tempfile.TemporaryDirectory() as root:
            self.assertTrue(self.trial(root)['passed'])
        with tempfile.TemporaryDirectory() as root:
            self.assertFalse(self.trial(root,shown=0)['passed'])
        with tempfile.TemporaryDirectory() as root:
            self.assertFalse(self.trial(root,value_delta=1)['passed'])
        with tempfile.TemporaryDirectory() as root:
            self.assertFalse(self.trial(root,opcode='0x0067 SMSG_FRIEND_LIST ev=1')['passed'])
        with tempfile.TemporaryDirectory() as root:
            self.assertFalse(self.trial(root,draw='1')['passed'])
