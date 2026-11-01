import importlib.util
import json
import os
from pathlib import Path
import struct
import unittest
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools/dev'))
spec = importlib.util.spec_from_file_location('native_balance', ROOT / 'tools/dev/native_balance.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class NativeBalanceTests(unittest.TestCase):
    def test_unknown_image_rejected_before_parsing(self):
        with self.assertRaisesRegex(ValueError, 'Unsupported'):
            module.Image(b'not the reviewed executable')

    def test_reads_cannot_cross_raw_section_or_truncation(self):
        image = object.__new__(module.Image)
        image.sections = [(100, 4, 0)]
        image.blob = b'abcd'
        self.assertEqual(image.read(101, 2), b'bc')
        for offset, size in [(99, 1), (103, 2), (100, -1), (-1, 1)]:
            with self.assertRaises(ValueError):
                image.read(offset, size)
        image.blob = b'a'
        with self.assertRaises(ValueError):
            image.read(100, 4)

    def test_special_effect_is_not_guessed_as_stat(self):
        raw = bytes([44]) + bytes(63)
        decoded = module.effect(raw)
        self.assertNotIn('coefficient', decoded)
        self.assertEqual(decoded['raw_hex'], raw.hex())

    def test_nonfinite_and_truncated_effects_rejected(self):
        with self.assertRaises(ValueError):
            module.effect(bytes(63))
        with self.assertRaises(ValueError):
            module.effect(struct.pack('<IfI', 43, float('nan'), 0) + bytes(52))

    def test_snapshot_coverage_and_document(self):
        folder = ROOT / 'docs/native-balance'
        data = json.loads((folder / 'native-balance.json').read_text())
        self.assertEqual(len(data['stats']), 29)
        self.assertEqual(len(data['jewelry']), 23)
        self.assertEqual(len(data['amulets']), 16)
        self.assertEqual(len({r['key'] for r in data['jewelry']}), 23)
        self.assertEqual(module.render(data), (folder / 'NATIVE-BALANCE.md').read_text())
        self.assertNotIn('ASX', module.render(data))

    @unittest.skipUnless(os.environ.get('ASX_REVIEWED_IMAGE_1351'), 'Exact native image not supplied')
    def test_exact_image_reproduces_snapshot(self):
        data = module.extract(Path(os.environ['ASX_REVIEWED_IMAGE_1351']).read_bytes())
        expected = json.loads((ROOT / 'docs/native-balance/native-balance.json').read_text())
        self.assertEqual(data, expected)
        accuria = next(r for r in data['jewelry'] if r['key'] == 'accuria')
        self.assertAlmostEqual(accuria['coefficient'], .17, places=6)
        woven = next(r for r in data['amulets'] if r['key'] == 'woven_ivy')
        self.assertEqual([e['stat_id'] for e in woven['effects']], [6, 2, 7])
        self.assertEqual(woven['effects'][0]['coefficient'], 6)


if __name__ == '__main__':
    unittest.main()
