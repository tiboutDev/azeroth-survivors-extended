import importlib.util
from pathlib import Path
import struct
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('ui_archive',ROOT/'tools/dev/ui_archive.py')
mpq=importlib.util.module_from_spec(spec);spec.loader.exec_module(mpq)


class UIArchiveTests(unittest.TestCase):
    def test_standard_storm_table_keys_and_native_raw_file_flags(self):
        self.assertEqual(mpq.hash_name('(hash table)',3),0xc3af3770)
        self.assertEqual(mpq.hash_name('(block table)',3),0xec83b3a3)
        data=mpq.pack({'Interface/FrameXML/GlobalStrings.lua':b'native lua'})
        magic,hs,size,version,sector,hoff,boff,hcount,bcount=struct.unpack('<4sIIHHIIII',data[:32])
        self.assertEqual((magic,hs,size,version),(b'MPQ\x1a',32,len(data),0))
        blocks=mpq.crypt(data[boff:],0xec83b3a3)
        self.assertEqual(struct.unpack('<IIII',blocks)[3],0x80000000)
        with tempfile.TemporaryDirectory() as root:
            path=Path(root)/'patch-9.MPQ';path.write_bytes(data)
            self.assertEqual(mpq.read_raw(path,'interface\\framexml\\globalstrings.lua'),b'native lua')
            with self.assertRaises(KeyError):mpq.read_raw(path,'missing.lua')
            path.write_bytes(data[:33])
            with self.assertRaises((ValueError,KeyError,struct.error)):mpq.read_raw(path,'Interface/FrameXML/GlobalStrings.lua')
