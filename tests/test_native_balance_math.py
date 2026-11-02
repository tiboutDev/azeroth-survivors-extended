import importlib.util
import json
import math
import os
from pathlib import Path
import random
import struct
import sys
import unittest

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools/dev'))
from native_balance import Image, extract
from native_balance_math import amulet_effects, amulet_scale, f32, jewelry_gain, jewelry_history, jewelry_bonuses


class BalanceArithmeticTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.data=json.loads((ROOT/'docs/native-balance/native-balance.json').read_text())

    def test_mixed_history_cannot_be_replaced_by_best_rarity(self):
        row=self.data['jewelry'][0]
        mixed=jewelry_history(row,[0,2,3])
        uniform=jewelry_history(row,[3,3,3])
        self.assertNotAlmostEqual(mixed,uniform)
        self.assertAlmostEqual(jewelry_history(row,[0]),.255,places=6)

    def test_caps_and_compound_effects(self):
        self.assertEqual(jewelry_gain(.06,9,4,.6,20),0)
        self.assertEqual(jewelry_gain(.01,18,4,.03,20),0)
        row=next(r for r in self.data['jewelry'] if r['key']=='archimtiros')
        effects=jewelry_bonuses(row,.2)
        self.assertAlmostEqual(effects['Damage reduction (%)'],10.5,places=5)
        self.assertEqual(effects['Maximum health (%)'],80)

    def test_amulet_target_breakpoint_and_fixed_death_prevention(self):
        row=next(r for r in self.data['amulets'] if r['key']=='vek_nilash')
        self.assertEqual(amulet_effects(row,6)[0][1],1)
        self.assertEqual(amulet_effects(row,7)[0][1],2)
        self.assertEqual(amulet_effects(row,100)[0][1],2)
        row=next(r for r in self.data['amulets'] if r['key']=='fallen_dragon')
        self.assertEqual(amulet_effects(row,1)[:2],amulet_effects(row,20)[:2])
        self.assertEqual(amulet_scale(1,3),1)
        self.assertLess(amulet_scale(10,3),3)

    def test_cooldown_input_is_not_direct_percentage_reduction(self):
        row=next(r for r in self.data['jewelry'] if r['key']=='quick_strike')
        bonuses=jewelry_bonuses(row,1.0)
        self.assertEqual(bonuses['Cooldown recovery input (%)'],100)
        self.assertEqual(bonuses['Cooldown reduction alone (%)'],50)

    def test_invalid_inputs(self):
        for args in ((.1,0,-1),(.1,0,6),(.1,0,0,float('nan')),(-.1,0,0)):
            with self.assertRaises(ValueError):jewelry_gain(*args)
        with self.assertRaises(ValueError):amulet_scale(0,0)


@unittest.skipUnless(os.environ.get('AZSX_REVIEWED_IMAGE_1351') and importlib.util.find_spec('unicorn') and importlib.util.find_spec('pefile'), 'Native-image emulation dependencies not supplied')
class NativeArithmeticOracle(unittest.TestCase):
    """Independent oracle: execute original arithmetic in an isolated emulator.

    No native process, DLL injection, game assets or installation writes.
    Only ceilf/floorf imports are implemented for the amulet transform loop.
    """
    @classmethod
    def setUpClass(cls):
        import unicorn
        from unicorn import x86_const as reg
        cls.reg=reg
        blob=Path(os.environ['AZSX_REVIEWED_IMAGE_1351']).read_bytes()
        cls.image=Image(blob);cls.data=extract(blob);cls.base=cls.image.base
        cls.uc=unicorn.Uc(unicorn.UC_ARCH_X86,unicorn.UC_MODE_64)
        extent=max(a+n for a,n,_ in cls.image.sections)
        cls.uc.mem_map(cls.base,(extent+4095)&~4095)
        for va,size,raw in cls.image.sections:cls.uc.mem_write(cls.base+va,blob[raw:raw+size])
        cls.scratch=0x200000000;cls.uc.mem_map(cls.scratch,0x20000)
        cls.stop=cls.scratch+0x1f000
        def math_import(uc,address,size,user):
            if address not in (cls.base+0x6114055,cls.base+0x611408b):return
            value=struct.unpack('<f',struct.pack('<I',uc.reg_read(reg.UC_X86_REG_XMM0)&0xffffffff))[0]
            answer=math.ceil(value) if address==cls.base+0x6114055 else math.floor(value)
            uc.reg_write(reg.UC_X86_REG_XMM0,struct.unpack('<I',struct.pack('<f',answer))[0])
            sp=uc.reg_read(reg.UC_X86_REG_RSP)
            target=struct.unpack('<Q',uc.mem_read(sp,8))[0]
            uc.reg_write(reg.UC_X86_REG_RSP,sp+8);uc.reg_write(reg.UC_X86_REG_RIP,target)
        cls.uc.hook_add(unicorn.UC_HOOK_CODE,math_import,begin=cls.base+0x6114055,end=cls.base+0x611408b)

    def run_native(self,rva,rcx,rdx,r8=0):
        uc,r=self.uc,self.reg;sp=self.scratch+0x18000
        uc.mem_write(sp,struct.pack('<Q',self.stop))
        for key,value in ((r.UC_X86_REG_RCX,rcx),(r.UC_X86_REG_RDX,rdx),(r.UC_X86_REG_R8,r8),(r.UC_X86_REG_RSP,sp),(r.UC_X86_REG_MXCSR,0x1f80)):
            uc.reg_write(key,value)
        uc.emu_start(self.base+rva,self.stop,count=10000)
        self.assertEqual(uc.reg_read(r.UC_X86_REG_RIP),self.stop,'Native routine did not return within its bound')

    def native_gain(self,index,rarity,value,rank):
        self.uc.mem_write(self.scratch,struct.pack('<IfI',int(rank>0),value,rank))
        self.run_native(0x5c64c10,self.base+0x747c720+index*72,rarity,self.scratch)
        return struct.unpack('<f',struct.pack('<I',self.uc.reg_read(self.reg.UC_X86_REG_XMM0)&0xffffffff))[0]

    def test_generic_reduction_input_does_not_grant_item_compound_health(self):
        # The generic input dispatch is separate from the jewelry-kind dispatch.
        # Execute original branches, with distinct sentinels in every accumulator.
        for kind,changed in ((5,12),(2,9)):
            table=0x77fb97c
            target=table+struct.unpack('<i',self.image.read(table+kind*4,4))[0]
            self.assertEqual(target,{5:0x5cc5d1a,2:0x5cc5d87}[kind])
            for bonus in (.03,.05,.07):
                regs=[getattr(self.reg,'UC_X86_REG_XMM'+str(i)) for i in range(16)]
                before=[struct.unpack('<I',struct.pack('<f',.125*(i+1)))[0] for i in range(16)]
                for reg,value in zip(regs,before):self.uc.reg_write(reg,value)
                self.uc.mem_write(self.scratch,struct.pack('<f',bonus))
                self.uc.reg_write(self.reg.UC_X86_REG_RCX,self.scratch)
                self.uc.emu_start(self.base+target,self.base+0x5cc5cd0,count=10)
                self.assertEqual(self.uc.reg_read(self.reg.UC_X86_REG_RIP),self.base+0x5cc5cd0)
                for i,reg in enumerate(regs):
                    actual=self.uc.reg_read(reg)
                    expected=before[i] if i!=changed else struct.unpack('<I',struct.pack('<f',.125*(i+1)+f32(bonus)))[0]
                    self.assertEqual(actual,expected,(kind,bonus,i))

    def test_jewelry_matches_native_for_5520_uniform_and_920_mixed_picks(self):
        rng=random.Random(1351)
        for i,row in enumerate(self.data['jewelry']):
            for sequence in [*[([rarity]*40) for rarity in range(6)], [rng.randrange(5) for _ in range(40)]]:
                value=0
                for rank,rarity in enumerate(sequence):
                    native=self.native_gain(i,rarity,value,rank)
                    model=jewelry_gain(row['coefficient'],row['stat_kind'],rarity,value,rank)
                    self.assertAlmostEqual(native,model,delta=max(1e-7,abs(native)*1e-6),msg=(row['key'],rank,rarity))
                    value=f32(value+native)

    def test_all_amulet_effect_fields_match_native_transform(self):
        for row in self.data['amulets']:
            for rank in (1,2,3,5,6,7,10,20,100):
                s=self.scratch;raw=b''.join(bytes.fromhex(e['raw_hex']) for e in row['effects'])
                self.uc.mem_write(s+0x1000,raw)
                scale=amulet_scale(rank,row['rarity'])
                self.uc.mem_write(s+0x100,struct.pack('<fI',scale,rank))
                self.uc.mem_write(s+0x200,struct.pack('<4Q',s+0x1000,s+0x1000+len(raw),s+0x100,s+0x104))
                self.uc.mem_write(s+0x300,struct.pack('<3Q',s+0x400,0,s+0x5000))
                self.uc.mem_write(s+0x400,bytes(8))
                self.run_native(0x5dd34c0,s+0x200,s+0x300)
                for i,e in enumerate(row['effects']):
                    original=bytes.fromhex(e['raw_hex']);actual=bytes(self.uc.mem_read(s+0x5000+i*64,64));tag=e['tag']
                    self.assertEqual(actual[0],tag)
                    fields={43:([4],[]),4:([4],[8]),6:([8],[]),9:([8],[12]),11:([4],[8]),19:([],[4,8]),44:([],[])}[tag]
                    for offset in fields[0]:
                        before=struct.unpack_from('<f',original,offset)[0];after=struct.unpack_from('<f',actual,offset)[0]
                        self.assertEqual(after,f32(before*scale),(row['key'],rank,offset))
                    for offset in fields[1]:self.assertEqual(actual[offset:offset+4],original[offset:offset+4])
                    if tag==44:
                        actual_count=struct.unpack_from('<f',actual,32)[0]
                        predicted=next(v for label,v,unit in amulet_effects(row,rank) if label=='Additional spell targets')
                        self.assertEqual(actual_count,predicted)


if __name__=='__main__':unittest.main()
