import importlib.util
from pathlib import Path
import unittest
try:
    import capstone as cs
except ImportError:
    cs=None

@unittest.skipIf(cs is None,'Capstone analysis dependency is installed in WSL only')
class OwnershipTests(unittest.TestCase):
    def analyze(self,hex_code,root='game'):
        spec=importlib.util.spec_from_file_location('native_ownership',Path(__file__).resolve().parents[1]/'tools/dev/native_ownership.py')
        module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
        md=cs.Cs(cs.CS_ARCH_X86,cs.CS_MODE_64);md.detail=True
        return module.analyze(md,list(md.disasm(bytes.fromhex(hex_code),0x1000)),root)
    def test_enemy_root_does_not_follow_loaded_heap_pointers(self):
        refs,_=self.analyze('488b7990 8b4708 c3','enemy')
        self.assertEqual(refs[0x1000,1]['domain'],'enemy')
        self.assertNotIn((0x1004,1),refs)
    def test_call_preserves_game_copy_but_not_volatile_register(self):
        # mov rdi,rcx; call next; mov eax,[rdi+20]; mov eax,[rcx+20]; ret
        refs,_=self.analyze('4889cf e800000000 8b4720 8b4120 c3')
        self.assertEqual(refs[0x1008,1]['offset'],0x20)
        self.assertNotIn((0x100b,1),refs)
    def test_conflicting_predecessor_erases_ownership(self):
        # mov rdi,rcx; test edx,edx; je read; xor edi,edi; read: mov eax,[rdi+20]
        refs,_=self.analyze('4889cf 85d2 7402 31ff 8b4720 c3')
        self.assertNotIn((0x1009,1),refs)
    def test_heap_pointer_load_does_not_inherit_game_ownership(self):
        refs,_=self.analyze('488b7920 8b4708 c3') # mov rdi,[rcx+20]; mov eax,[rdi+8]
        self.assertIn((0x1000,1),refs)
        self.assertNotIn((0x1004,1),refs)
    def test_partial_stack_write_invalidates_spilled_pointer(self):
        # frame setup; spill Game; overwrite four bytes; reload; dereference
        refs,_=self.analyze('4883ec28 4889e5 48894d08 c7450c00000000 488b7d08 8b4710 c3')
        self.assertNotIn((0x1016,1),refs)
    def test_simd_copy_cannot_turn_rip_literals_into_game_fields(self):
        refs,_=self.analyze('66480f6ec1 8b0500000000 c3')
        self.assertNotIn((0x1005,1),refs)
    def test_high_byte_write_invalidates_the_whole_pointer(self):
        refs,_=self.analyze('b501 8b4120 c3')
        self.assertNotIn((0x1002,1),refs)
