import importlib.util
import json
import os
from pathlib import Path
import struct
import sys
import unittest
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools/dev'))
from native_world_balance import extract,render,growth,cache_next,enemy_values
from native_balance import Image

class WorldReferenceTests(unittest.TestCase):
    def test_saved_reference_and_comparisons(self):
        d=json.loads((ROOT/'docs/native-balance/native-world-balance.json').read_text())
        self.assertEqual((ROOT/'docs/native-balance/ENEMIES.md').read_text(),render(d))
        self.assertEqual(len(d['maps']),12)
        self.assertEqual(sum(len(m['enemies']) for m in d['maps']),148)
        self.assertEqual(sum(len(m['bosses']) for m in d['maps']),60)
        m=d['maps'][0];e=m['enemies'][0]
        solo=enemy_values(e,m,0)
        self.assertAlmostEqual(solo['health'],24*4.95*.82,places=4)
        coop=enemy_values(e,m,0,players=2,host=True)
        self.assertAlmostEqual(coop['health']/solo['health'],.8*(1+.72*.7))
        self.assertEqual(coop['damage'],solo['damage'])
        self.assertGreater(enemy_values(e,m,15)['health'],solo['health'])
        with self.assertRaises(ValueError):enemy_values(e,m,1,players=6)

    def test_cache_sequences(self):
        for difficulty,expected in [(0,[20,31,45,63,86,116]),(3,[20,32,47,67,93,127])]:
            actual=[20]
            for _ in range(5):actual.append(cache_next(actual[-1],difficulty))
            self.assertEqual(actual,expected)
        self.assertEqual(cache_next(0xffffffff),0xffffffff)
        with self.assertRaises(ValueError):cache_next(-1)

@unittest.skipUnless(os.environ.get('ASX_REVIEWED_IMAGE_1351') and importlib.util.find_spec('unicorn') and importlib.util.find_spec('pefile'),'Reviewed image and emulator required')
class WorldNativeOracle(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        import unicorn
        from unicorn import x86_const as reg
        cls.reg=reg;cls.image=Image(Path(os.environ['ASX_REVIEWED_IMAGE_1351']).read_bytes());cls.base=cls.image.base
        cls.uc=unicorn.Uc(unicorn.UC_ARCH_X86,unicorn.UC_MODE_64)
        extent=max(a+n for a,n,_ in cls.image.sections);cls.uc.mem_map(cls.base,(extent+4095)&~4095)
        for va,size,raw in cls.image.sections:cls.uc.mem_write(cls.base+va,cls.image.blob[raw:raw+size])
        cls.scratch=0x200000000;cls.uc.mem_map(cls.scratch,0x20000)

    def test_extraction_matches_saved_data(self):
        self.assertEqual(extract(self.image),json.loads((ROOT/'docs/native-balance/native-world-balance.json').read_text()))

    def test_original_health_polynomials(self):
        uc,r=self.uc,self.reg;stop=self.scratch+0x18000;sp=self.scratch+0x10000
        for guardian,rva in [(True,0x5d55e90),(False,0x5d55ee0)]:
            for minute in (0,.5,3.5,8,13,15,20,45,120):
                for coefficient in (.82,1,2.33,6.25):
                    uc.mem_write(sp,struct.pack('<Q',stop));uc.reg_write(r.UC_X86_REG_RSP,sp);uc.reg_write(r.UC_X86_REG_MXCSR,0x1f80)
                    for register,value in [(r.UC_X86_REG_XMM0,minute),(r.UC_X86_REG_XMM1,coefficient)]:uc.reg_write(register,struct.unpack('<I',struct.pack('<f',value))[0])
                    uc.emu_start(self.base+rva,stop,count=50)
                    self.assertEqual(uc.reg_read(r.UC_X86_REG_RIP),stop)
                    actual=struct.unpack('<f',struct.pack('<I',uc.reg_read(r.UC_X86_REG_XMM0)&0xffffffff))[0]
                    self.assertEqual(actual,growth(minute,coefficient,guardian))

    def test_original_cache_price_and_wallet_debit(self):
        uc,r=self.uc,self.reg;game=self.scratch
        for difficulty in range(6):
            for cost in (0,1,20,31,32,100,999,10000,0xffffffff):
                uc.mem_write(game+0x3f7,bytes([difficulty]));uc.mem_write(game+0x578,struct.pack('<I',cost))
                uc.reg_write(r.UC_X86_REG_RSI,game);uc.reg_write(r.UC_X86_REG_R15,cost);uc.reg_write(r.UC_X86_REG_RAX,0)
                uc.emu_start(self.base+0x5cfbbea,self.base+0x5cfbc3d,count=40)
                self.assertEqual(uc.reg_read(r.UC_X86_REG_RIP),self.base+0x5cfbc3d)
                self.assertEqual(struct.unpack('<I',uc.mem_read(game+0xee8,4))[0],cache_next(cost,difficulty))
                self.assertEqual(struct.unpack('<I',uc.mem_read(game+0x578,4))[0],0)
if __name__=='__main__':unittest.main()
