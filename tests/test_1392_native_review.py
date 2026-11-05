"""Offline checks on the pinned 0.139.2 executable."""
import json
import os
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools/dev"))


def bindings(version):
    from binding_registry import parse_literal
    manifest = json.loads((ROOT / f"compat/clients/{version}.json").read_text())
    return {row["name"]: parse_literal(row["value"]) for row in manifest["constants"]}


class CandidateRegistration(unittest.TestCase):
    def test_partial_manifest_cannot_generate_runtime_bindings(self):
        from unittest.mock import patch
        import binding_registry
        from tempfile import TemporaryDirectory
        with TemporaryDirectory() as directory:
            manifest = Path(directory) / "incomplete.json"
            manifest.write_text(json.dumps({"version": "0.139.2", "schema": 1, "review_status": "partial"}))
            candidate = {"binding_manifest": manifest.name}
            with patch.object(binding_registry, "ROOT", manifest.parent), patch.object(
                    binding_registry, "selected_client", return_value=("0.139.2", candidate)):
                with self.assertRaisesRegex(ValueError, "Partial native bindings"):
                    binding_registry.render()


class Native1392Review(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        path = os.environ.get("AZSX_REVIEW_IMAGE_1392")
        if not path:
            raise unittest.SkipTest("Set AZSX_REVIEW_IMAGE_1392 to the private candidate executable")
        from compare_official import Image
        cls.values = bindings("0.139.2")
        cls.image = Image(Path(path))
        manifest = json.loads((ROOT / "compat/clients/0.139.2.json").read_text())
        pinned = "0bb6c557c5f6cfeea212761a33554f9a5647ce7cb7e03b8a163b95489496e908"
        if cls.image.sha != pinned or manifest["sha256"] != pinned:
            raise AssertionError("Wrong 0.139.2 review image")

    @classmethod
    def tearDownClass(cls):
        import gc
        # The bounded 0.138.0 checks run in the same process. Release both the
        # PE image and its cached Capstone instructions before loading the next.
        cls.image.ins.cache_clear()
        cls.image.tokens.cache_clear()
        del cls.image
        gc.collect()

    def body(self, owner):
        return {i.address: i for i in self.image.ins(owner)}

    def assert_log_call(self, caller, address, logger, label):
        call = caller[address]
        self.assertEqual((call.mnemonic, call.operands[0].imm), ("call", logger))
        for instruction in self.image.ins(logger):
            if instruction.mnemonic == "lea" and "[rip " in instruction.op_str:
                pointer = instruction.address + instruction.size + instruction.operands[1].mem.disp
                if self.image.pe.get_data(pointer, len(label)) == label:
                    return
        self.fail(f"Logger at {logger:#x} does not reference {label!r}")

    def test_engineering_grenade_appends_one_identifiable_projectile(self):
        from native_1392_machine import ReviewMachine
        m = ReviewMachine(self.image)
        game = m.scratch + 0x20000
        effect, origin, enemy = game + 0x4000, game + 0x4080, game + 0x5000
        guid = 0x100000042
        token = self.values["SOLO_MORTAR_TOKEN"] | 1
        spell = self.values["SOLO_MORTAR_SPELL"]
        vector = game + self.values["SOLO_PROJECTILE_VECTOR"]
        m.write(game + 0xea0, "f", 75.)
        m.write(game + 0x648, "QQ", enemy, 1)
        m.write(enemy + self.values["ENEMY_GUID"], "Q", guid)
        m.write(effect, "IQ", 8, token)
        m.write(vector, "3Q", 0, 8, 0)
        # Seventh Win64 argument is the additional half-second delay as f32.
        m.call(self.values["SOLO_PROJECTILE_CREATE"][0], game, spell, 2, effect,
               guid, 0, 0x3f000000, origin)
        cap, row, count = m.read(vector, "3Q")
        self.assertEqual(count, 1)
        self.assertEqual(m.allocations[row][0], cap * self.values["SOLO_PROJECTILE_STRIDE"])
        self.assertEqual(m.read(row + 112, "IQfI"), (8, token, 75.5, spell))

    def test_cosmetic_heal_queue_matches_native_producers(self):
        packet = self.values["HEAL_LOG_PACKET"][0]
        reserve = self.values["HEAL_LOG_RESERVE"][0]
        queue = self.values["HEAL_LOG_QUEUE"]
        for site in (0x5eda4ed, 0x5ef36f5, 0x5f2c8c7):
            ops = list(self.body(self.image.owner(site)).values())
            index = next(i for i, op in enumerate(ops) if op.address == site)
            self.assertEqual((ops[index].mnemonic, ops[index].operands[0].imm), ("call", packet))
            after = [op for op in ops[index+1:index+19] if op.mnemonic != "nop"]
            self.assertEqual(after[0].op_str, f"rdi, qword ptr [rsi + {queue+16:#x}]")
            self.assertEqual(after[1].op_str, f"rdi, qword ptr [rsi + {queue:#x}]")
            self.assertEqual((after[4].mnemonic, after[4].operands[0].imm), ("call", reserve))
            self.assertEqual(after[5].op_str, f"rax, qword ptr [rsi + {queue+8:#x}]")
            self.assertEqual(after[7].op_str, "rcx, 5")
            self.assertIn(f"qword ptr [rsi + {queue+16:#x}], rdi", [op.op_str for op in after])

    def test_direct_and_periodic_heal_consumers(self):
        direct = self.body(0x119ca60)
        periodic = self.body(0x119d5b0)
        self.assert_log_call(direct, 0x119cbbc, 0x7b0640, b"HEAL_CRIT")
        self.assert_log_call(periodic, 0x119d858, 0x7b34c0, b"PERIODIC_HEAL")
        self.assertEqual(direct[0x119cad9].op_str, "rax, 0x8000000000000097")
        self.assertEqual(periodic[0x119d632].op_str, "r14, 0x8000000000000096")
        self.assertEqual(direct[0x119cae7].operands[0].imm, 0x119cbc2)
        self.assertEqual(direct[0x119cbc2].op_str, "rcx, [rbp - 0x58]")
        self.assertEqual(periodic[0x119d62b].op_str, "rax, qword ptr [rbp + 0x2d0]")
        self.assertEqual(periodic[0x119d66c].op_str, "r15, qword ptr [rbp + 0x180]")
        self.assertEqual(self.body(0x7b34c0)[0x7b369c].op_str, "r13, 0x14")

    def test_heal_consumer_frames_hold_distinct_event_context_and_writer(self):
        from native_1392_machine import ReviewMachine
        from unicorn import UC_HOOK_CODE, x86_const as reg
        # Capture the direct frame before the logger consumes it and the periodic
        # frame at the hook. The direct destructor hook uses the same frame.
        for periodic, owner, boundary in ((0, 0x119ca60, 0x119cbbc), (1, 0x119d5b0, 0x119d66c)):
            machine = ReviewMachine(self.image)
            event = machine.scratch + 0x20000
            context, params = event + 0x1000, event + 0x2000
            tag = self.values["HEAL_TEXT_DISCRIMINANTS"][periodic]
            machine.write(event, "Q", tag)
            payload = bytes(range(1, 49))
            machine.cpu.mem_write(event + 8, payload)
            machine.write(context + 0x40, "Q", 0x1234ab)
            machine.write(context + 0xa0, "Q", context + 0x200)
            machine.write(params, "Q", 0x1234bc)
            machine.write(params + 0x20, "Q", 0x1234cd)
            captured = []

            def capture(cpu, address, size, unused):
                frame = cpu.reg_read(reg.UC_X86_REG_RBP)
                def slot(name):
                    return frame + self.values[name][periodic]
                captured.append((machine.read(slot("HEAL_TEXT_TAG"), "Q")[0],
                                 bytes(cpu.mem_read(slot("HEAL_TEXT_PAYLOAD"), 48)),
                                 *(machine.read(slot(name), "Q")[0] for name in
                                   ("HEAL_TEXT_OBJECTS", "HEAL_TEXT_PARAMS", "HEAL_TEXT_QUERY"))))
                cpu.reg_write(reg.UC_X86_REG_RIP, machine.stop)

            address = machine.base + boundary
            machine.cpu.hook_add(UC_HOOK_CODE, capture, begin=address, end=address)
            machine.call(owner, 0, event, context, params)
            self.assertEqual(captured, [(tag, payload, 0x1234ab, 0x1234bc, 0x1234cd)])

    def test_heal_packet_and_text_vectors_keep_distinct_row_sizes(self):
        from native_1392_machine import ReviewMachine
        machine = ReviewMachine(self.image)
        header = machine.scratch + 0x20000
        for critical in (0, 1):
            machine.call(self.values["HEAL_LOG_PACKET"][0], header, 0x100004202, 0x300002678, 17, 128, critical)
            self.assertEqual(machine.read(header + 24, "H"), (0x150,))
            self.assertEqual(machine.buffer(header), bytes.fromhex("13024201137826031100000080000000") + bytes([critical]))
            capacity, ptr, _ = machine.read(header, "3Q")
            machine.call(0x21d0, ptr, capacity, 1)
        for name, stride in (("TEXT_RESERVE", 56), ("HEAL_LOG_RESERVE", 32)):
            previous = machine.allocate(stride, 8)
            row = bytes(range(stride))
            machine.cpu.mem_write(previous, row)
            machine.write(header, "3Q", 1, previous, 1)
            machine.call(self.values[name][0], header)
            capacity, ptr, length = machine.read(header, "3Q")
            self.assertEqual((capacity, length), (4, 1))
            self.assertEqual(machine.allocations[ptr], (capacity * stride, 8))
            self.assertEqual(bytes(machine.cpu.mem_read(ptr, stride)), row)
            machine.call(0x21d0, ptr, capacity * stride, 8)
        self.assertFalse(machine.allocations)

    def test_entity_lookup_matches_periodic_target_map(self):
        standalone = list(self.image.ins(0x13d6890))
        periodic = list(self.image.ins(0x119d5b0))
        self.assertEqual(standalone[2].op_str, "r9, qword ptr [rcx]")
        for body in (standalone, periodic):
            literals = [i.operands[1].imm for i in body if i.mnemonic == "movabs"]
            mask = (1 << 64) - 1
            self.assertIn(0x9dc0fe5cef68def1, [v & mask for v in literals])
            self.assertIn(0xc0ac29b7c97c50dd, [v & mask for v in literals])
        self.assertEqual(standalone[-8].op_str, "rax, qword ptr [rsi - 8]")
        self.assertEqual(standalone[-4].op_str, "eax, eax")
        self.assertEqual(self.body(0x119d5b0)[0x119d751].op_str, "rdx, qword ptr [r11 - 8]")

    def test_heal_text_resolves_eligible_current_entity_only(self):
        from native_1392_machine import ReviewMachine
        m = ReviewMachine(self.image)
        root = m.scratch + 0x10000
        objects, control = root + 0x100, root + 0x500
        query, state, world = root + 0x600, root + 0x800, root + 0xb00
        rows, tables, columns, component = (root + n for n in (0xe00, 0x1000, 0x1200, 0x1400))
        guid, generation = 0x10000002a, 7
        handle = generation << 32 | 0xffffffff
        product = (0x9dc0fe5cef68def1 ^ guid) * 0xc0ac29b7c97c50dd
        hashed = (product ^ product >> 64) & 0xffffffffffffffff
        bucket = hashed & 15
        m.write(root, "Q", objects)
        m.write(objects, "4Q", control, 15, 0, 1)
        m.cpu.mem_write(control, bytes([255]) * 32)
        for mirror in (0, 16):
            m.write(control + bucket + mirror, "B", hashed >> 57)
        m.write(control - 16 * (bucket + 1), "2Q", guid, handle)
        m.write(query, "2Q", state, world)
        m.write(world + 8, "2Q", rows, 1)
        m.write(world + 0x1a8, "Q", tables)
        m.write(rows, "5I", 1, 0xffffffff, 0, 0, generation)
        m.write(state + 0x28, "Q", state + 0x200)
        m.write(state + 0x38, "Q", 1)
        m.write(state + 0x200, "Q", 1)
        m.write(tables + 0x18, "Q", columns)
        m.write(tables + 0x38, "Q", tables + 0x100)
        m.write(tables + 0x100, "Q", 0xffffffffffffffff)
        m.write(columns + 0x10, "Q", component)
        m.write(component + 8, "2Q", component + 0x80, 1)
        m.write(component + 0x20, "2Q", component + 0x100, 3)
        m.write(component + 0x80, "B", 4)
        lookup = self.values["TEXT_ENTITY"][0]
        predicate = self.values["TEXT_QUERY"][0]
        self.assertEqual(m.call(lookup, root, guid), handle)
        self.assertEqual(m.call(lookup, root, guid + 1), 0)
        self.assertEqual(m.call(predicate, guid + 1, objects, query) & 255, 0)
        for flags, accepted in ((0, 0), (8, 1), (16, 1)):
            m.write(component + 0x108, "I", flags)
            self.assertEqual(m.call(predicate, guid, objects, query) & 255, accepted)
        m.write(rows + 0x10, "I", generation + 1)
        self.assertEqual(m.call(predicate, guid, objects, query) & 255, 0)

    def test_hunters_mark_visual_updates_only_matching_living_enemy(self):
        from native_1392_machine import ReviewMachine
        m = ReviewMachine(self.image)
        game = m.scratch + 0x10000
        first, queue = game + 0x8000, game + 0x9000
        second = first + self.values["ENEMY_STRIDE"]
        apply = dict(self.values["ADDRESSES"])[0x5962610]
        m.write(game + 0x640, "3Q", 2, first, 2)
        m.write(game + 0x700, "3Q", 8, queue, 0)
        m.write(first + self.values["ENEMY_GUID"], "Q", 77)
        m.write(first + self.values["ENEMY_DEAD"], "I", 1)
        m.write(second + self.values["ENEMY_GUID"], "Q", 88)
        first_aura = first + self.values["ENEMY_AURAS_START"]
        second_aura = second + self.values["ENEMY_AURAS_START"]
        m.write(second_aura, "I", 42)
        for target, add, marked, packets in (
            (77, 1, False, 0), (99, 1, False, 0), (88, 1, True, 1),
            (88, 1, True, 1), (88, 0, False, 2), (88, 0, False, 2),
        ):
            m.call(apply, game, target, 1130, add)
            self.assertEqual(m.read(first_aura, "16I"), (0,) * 16)
            self.assertEqual(m.read(second_aura, "16I"), (42, 1130 if marked else 0) + (0,) * 14)
            self.assertEqual(m.read(game + 0x710, "Q"), (packets,))

    def test_single_field_writer_used_for_native_coinage(self):
        caller = self.body(0x5eaf0e0)
        self.assertEqual(caller[0x5eaf384].op_str, "dx, 0x498")
        self.assertEqual(caller[0x5eaf388].operands[0].imm, 0x5e08070)
        writer = self.body(0x5e08070)
        self.assertEqual(writer[0x5e080a5].op_str, "dx, word ptr [rax + r9*2 + 0x3a]")
        self.assertEqual(writer[0x5e080e5].op_str, "dword ptr [rax + r9*4 + 8], r8d")
        self.assertEqual(writer[0x5e0811d].operands[0].imm, 0x5ffae50)
        # The native gold award inlines the existing-key update and only calls
        # the entry helper when inserting. Hooking the public writer misses it.
        gold = list(self.image.ins(0x5f08680))
        calls = [i.operands[0].imm for i in gold if i.mnemonic == "call" and i.operands[0].type == 2]
        self.assertIn(0x5ffae50, calls)
        self.assertNotIn(0x5e08070, calls)

    def test_weapon_entity_handle_and_main_hand_slot(self):
        render = self.body(0xa659c0)
        for address, expected in (
            (0xa65efd, "rbx, qword ptr [rax + rcx*8]"),
            (0xa66414, "rbx, qword ptr [r11 + rax]"),
            (0xa6823c, "qword ptr [rax + rdi + 0x50], rbx"),
            (0xa6864a, "r12d, byte ptr [rcx + rax]"),
            (0xa68654, "edx, r12d"),
            (0xa6865c, "edx, edx"),
        ):
            self.assertEqual(render[address].op_str, expected)
        self.assertEqual(render[0xa68657].operands[0].imm, 0x60282c0)

    def test_spend_paths_project_the_new_balance_without_the_generic_setter(self):
        for owner, conversion, read_map, actor in (
            (0x5f29d50, 0x5f29de1, 0x5f29de8, "rsi"),
            (0x5f7a460, 0x5f7a57d, 0x5f7a584, "rcx"),
        ):
            with self.subTest(owner=hex(owner)):
                code = self.body(owner)
                self.assertEqual(code[conversion].op_str, "r8d, eax, 0x2710")
                self.assertEqual(code[read_map].op_str, f"rax, qword ptr [{actor} + 0xe38]")
                self.assertTrue(any(i.mnemonic == "mov" and i.op_str.endswith(", 0x498")
                                    for i in code.values()))
                self.assertFalse(any(i.mnemonic == "call" and i.op_str == "0x5e08070"
                                     for i in code.values()))

    def test_downed_xp_guard_entries_and_continuations(self):
        # The resume address must immediately follow the selected native branch.
        # The branch itself must still skip the reward path for the other state.
        contacts = (
            (0x5eaf535, 0x5eaf53e, "rsi", 4),
            (0x5eb03d0, 0x5eb03dd, "rsi", 1),
            (0x5eb22c2, 0x5eb22d0, "r9", 4),
            (0x5ed2981, 0x5ed298f, "r13", 4),
            (0x5ed3224, 0x5ed322e, "r13", 4),
            (0x5ed3bfd, 0x5ed3c0b, "r13", 1),
            (0x5f01042, 0x5f0104f, "rsi", 1),
        )
        self.assertCountEqual([(item[0], item[1]) for item in self.values["DOWNED_XP_GUARDS"]],
                              [(entry, resume) for entry, resume, _, _ in contacts])
        for entry, resume, actor, state in contacts:
            with self.subTest(entry=hex(entry)):
                owner = self.body(self.image.owner(entry))
                check = owner[entry]
                self.assertEqual((check.mnemonic, check.op_str),
                                 ("cmp", f"byte ptr [{actor} + 0xf63], {state}"))
                branch = owner[entry + check.size]
                self.assertEqual(branch.mnemonic, "je" if state == 4 else "jne")
                self.assertEqual(branch.address + branch.size, resume)
                self.assertIn(branch.operands[0].imm, owner)
                self.assertNotEqual(branch.operands[0].imm, resume)

    def test_ui_alias_writer_and_native_party_tokens(self):
        from native_1392_machine import ReviewMachine
        writer = self.body(0x2840140)
        self.assertEqual(writer[0x2840171].operands[0].imm, 0x2bf5ad0)
        self.assertEqual(writer[0x28401a0].op_str, "rcx, [rdi + 0x18]")
        self.assertEqual(writer[0x2840516].op_str, "qword ptr [r15 + 0x3308], rax")
        self.assertEqual([writer[a].op_str for a in (0x2840542, 0x284054a)],
                         ["xmmword ptr [r15 + 0x3330], xmm0", "xmmword ptr [r15 + 0x3340], xmm1"])
        machine = ReviewMachine(self.image)
        root = machine.scratch + 0x10000
        aliases = root + 0x18 + 0x3290
        out = root + 0x5000
        player = 0x100000010
        machine.write(aliases + 0x60, "Q", player)
        for index in range(4):
            for field, prefix in ((0x88, "party"), (0xa8, "partypet")):
                guid = player + 8 * field + index
                machine.write(aliases + field + 8 * index, "Q", guid)
                machine.call(0x29da440, out, aliases, guid)
                self.assertEqual(machine.buffer(out), f"{prefix}{index + 1}".encode())
        # This resolver returns party/pet labels only. Neither self nor an absent
        # identity should alias a party member, even when a departed slot is reused.
        machine.write(aliases + 0x88, "Q", 0)
        for absent in (0, player, player + 8 * 0x88):
            machine.call(0x29da440, out, aliases, absent)
            self.assertEqual(machine.read(out, "q"), (-1,))

    def test_run_info_missing_drops_uses_armory_contracts(self):
        from native_1392_machine import CatalogMachine
        m = CatalogMachine(self.image)
        # The missing-drops tooltip borrows armory rows and frees the same output
        # allocations as the native collection log. Verify its Win64 boundaries.
        contacts = dict(self.values["ADDRESSES"])
        callsite = self.body(self.image.owner(contacts[0x3c1ccf]))[contacts[0x3c1ccf]]
        self.assertEqual((callsite.mnemonic, callsite.operands[0].imm), ("call", contacts[0x5bffe00]))
        collection, drops, item = m.scratch + 0x10000, m.scratch + 0x20000, m.scratch + 0x30000
        for cls, zone, boss in ((1, 0, 0), (7, 2, 1), (11, 11, 4)):
            m.call(contacts[0x5bfd310], drops, cls, zone, boss)
            capacity, rows, count = m.read(drops, "3Q")
            self.assertTrue(0 < count <= capacity <= 64)
            ident = m.read(rows, "I")[0]
            m.call(0x21d0, rows, capacity * 8, 4)
            m.write(collection + 0x30, "3Q", 1, collection + 0x400, 1)
            m.write(collection + 0x400, "I", ident)
            self.assertEqual(m.call(contacts[0x5c4d7a0], collection, ident) & 255, 1)
            self.assertEqual(m.call(contacts[0x5c4d7a0], collection, 0) & 255, 0)
            m.cpu.mem_write(item, bytes([0x55]) * 160)
            self.assertEqual(m.call(contacts[0x5bffe00], item, ident), item)
            self.assertTrue(m.buffer(item).decode("utf-8").strip())
            self.assertEqual(bytes(m.cpu.mem_read(item + 144, 16)), bytes([0x55]) * 16)
            for offset, stride, align in ((0, 1, 1), (24, 8, 4)):
                cap, pointer, _ = m.read(item + offset, "3Q")
                if cap:
                    m.call(0x21d0, pointer, cap * stride, align)

    def test_shared_awards_use_the_callers_observed_by_azsx(self):
        from compare_official import MD
        entries = dict(self.values["ADDRESSES"])
        gold = (0x58dc5f7, 0x58ee94e, 0x5978a43, 0x597e5c0, 0x597ee62)
        xp = (0x595e653, 0x5949b3b, 0x59763dd, 0x58fa00b, 0x58dac9f)
        for callers, callee in ((gold, 0x5920470), (xp, 0x591db30)):
            for key in callers:
                ret = entries[key]
                ins, = MD.disasm(self.image.pe.get_data(ret-5, 5), ret-5)
                self.assertEqual((ins.mnemonic, ins.operands[0].imm), ("call", entries[callee]))
        # These are the pre-multiplier inputs AZSX reads at the native award
        # function, including the host's already-summed guest receipts in ESI.
        for key, producer, source in ((xp[0], 0x5f20900, "ebx"), (xp[1], 0x5f5a035, "r12d"),
                                      (xp[2], 0x5f6e063, "ebx"), (xp[3], 0x5ed2a10, "esi")):
            from capstone import x86_const
            widths = {"ebx": ("EBX", "RBX"), "r12d": ("R12D", "R12"), "esi": ("ESI", "RSI")}
            registers = {getattr(x86_const, "X86_REG_" + name) for name in widths[source]}
            code = self.image.pe.get_data(producer, entries[key]-5-producer)
            instructions = list(MD.disasm(code, producer))
            self.assertEqual(instructions[0].op_str, "eax, " + source)
            self.assertTrue(any(i.mnemonic == "cvtsi2ss" for i in instructions))
            self.assertTrue(all(registers.isdisjoint(i.regs_access()[1]) for i in instructions))
        self.assertEqual(self.image.owner(entries[xp[-1]]), entries[0x58daa10])

    def test_spec_looks_classify_the_same_native_talent_keys(self):
        import struct
        from compare_official import Image
        previous_path = os.environ.get("AZSX_REVIEWED_IMAGE_138")
        if not previous_path:
            self.skipTest("Set AZSX_REVIEWED_IMAGE_138 for paired talent catalogs")
        previous = Image(Path(previous_path))
        self.assertEqual(previous.sha, "eb5fd1c1b96e6180fcf5f3d685d90a61471af28d22e5f060c02b7b7663fbf79e")
        def keys(image, tree):
            base, count = tree
            names = []
            for i in range(count):
                pointer, size = struct.unpack("<QQ", image.pe.get_data(base + 176*i + 64, 16))
                self.assertTrue(0 < size <= 128)
                names.append(image.pe.get_data(pointer - image.pe.OPTIONAL_HEADER.ImageBase, size))
            return names
        baseline = bindings("0.138.0")["WEAPON_TALENT_TREES"]
        current = self.values["WEAPON_TALENT_TREES"]
        self.assertEqual(len(current), 3)
        for old_class, new_class in zip(baseline, current, strict=True):
            self.assertEqual(len(new_class), 3)
            for before, after in zip(old_class, new_class, strict=True):
                self.assertEqual(keys(previous, before), keys(self.image, after))

    def test_party_spell_definition_keys_match_the_candidate_catalog(self):
        import struct
        roles = dict(self.values["ADDRESSES"])
        for key, spell in ((0x6cc5c78, 139), (0x6cc72f8, 774), (0x6cc7370, 8936),
                           (0x6cc5c00, 17), (0x6cc61a0, 324), (0x6cc73e8, 467),
                           (0x6cc4df0, 635), (0x6cc6380, 331)):
            self.assertEqual(self.image.pe.get_data(roles[key] + 0x40, 4), struct.pack("<I", spell))
        shield = dict(self.values["EXTENSION_ADDRESSES"])[0x72a0ca0]
        self.assertEqual(self.image.pe.get_data(shield + 0x40, 4), struct.pack("<I", 11426))

    def test_diagnostic_weapon_catalog_uses_live_hash_buckets(self):
        address, guard = self.values["WEAPON_CATALOG_PROBE"]
        frame = self.values["WEAPON_CATALOG_FRAME"]
        body = self.body(self.image.owner(address))
        self.assertEqual(self.image.pe.get_data(address, len(guard)), bytes(guard))
        self.assertEqual(body[address].op_str, f"xmm0, xmmword ptr [rbp + {frame:#x}]")
        self.assertEqual(body[0x7e0236].op_str, f"rcx, [rbp + {frame:#x}]")
        loader = self.body(body[0x7e0244].operands[0].imm)
        self.assertEqual(loader[0x61f8bff].op_str, "r10, r10, 0x168")
        self.assertEqual(loader[0x61f8c06].op_str, "dword ptr [rcx + r10 - 0x168], 0x88f5")

    def test_weapon_preview_uses_resolver_result_and_scoped_collection(self):
        from native_1392_machine import CatalogMachine
        m = CatalogMachine(self.image)
        entries = dict(self.values["ADDRESSES"])
        looks = m.scratch + 0x10000
        equipped, visible = looks + 0x400, looks + 0x800
        for slot in (15, 16):
            m.write(looks + slot*8, "2I", 1, 101008)
            self.assertEqual(m.call(entries[0x2828c0], looks, 101008) & 255, 1)
            self.assertEqual(m.call(entries[0x2828c0], looks, 101009) & 255, 0)
            m.write(looks + slot*8, "Q", 0)
        m.write(equipped + 60, "I", 101008)
        resolved = m.call(entries[0x5bff970], visible, 1, 1, equipped, looks)
        self.assertEqual(resolved, visible)
        self.assertEqual(m.read(resolved + 60, "I"), (101008,))
        for key, returns_after_nop in ((0x5ceedf2, True), (0x2a2e7a, False)):
            address = entries[key]
            caller = self.body(self.image.owner(address))
            if returns_after_nop:
                self.assertEqual(caller[address-1].mnemonic, "nop")
            call = caller[address-5-int(returns_after_nop)]
            self.assertEqual((call.mnemonic, call.operands[0].imm), ("call", entries[0x5bff970]))
        preview = self.body(self.image.owner(entries[0x2a2e7a]))
        slot = self.values["WEAPON_PREVIEW_COLLECTION_SLOT"]
        self.assertEqual(preview[0x13632b].op_str, f"qword ptr [rsp + {slot:#x}], rcx")
        self.assertEqual(preview[0x1363ea].op_str, f"rdx, qword ptr [rsp + {slot:#x}]")
        self.assertEqual(preview[0x1363d5].op_str, "r8d, byte ptr [rbp + 0x30]")
        self.assertEqual(preview[0x136405].op_str, "qword ptr [rsp + 0x20], rbx")
        armory = self.body(self.image.owner(entries[0x3c18e0]))
        self.assertEqual(armory[entries[0x3c18e0]].op_str, "rcx, [rbp + 0x2a8]")
        self.assertEqual(armory[0x383b7d].operands[0].imm, entries[0x296900])

    def test_party_ui_receives_native_run_line_and_owner_spell_text(self):
        from native_1392_machine import CatalogMachine
        m = CatalogMachine(self.image)
        game, settings, output = (m.scratch + n for n in (0x10000, 0x18000, 0x2a000))
        m.call(0x5efe1f0, game, settings, settings + 0x2000, settings + 0x4000, settings + 0x6000)
        entries = dict(self.values["ADDRESSES"])
        self.assertEqual(m.call(entries[0x5821710], output, game, 0), output)
        tag, length = m.read(output + 40, "2Q")
        self.assertEqual(m.bytes_at(tag, length), b"AZSRUN")
        native = m.buffer(output + 56)
        self.assertIn(b";hp=", native)
        self.assertNotIn(b";azsx=", native)
        ability = output + 0x3000
        m.write(ability, "QI", m.base + entries[0x6cc5c78], 1)
        stats = next(start + 0x490 - first for first, last, start in self.values["GAME_RESOLVED_FIELDS"]
                     if first <= 0x490 <= last)
        spell = output + 0x1000
        self.assertEqual(m.call(entries[0x58629b0], spell, ability, game + stats, 0, ability + 0x2000), spell)
        self.assertEqual(m.buffer(spell), b"Renew")
        self.assertEqual(m.buffer(spell + 72), b"Rank 1")
        self.assertTrue(m.buffer(spell + 24))

    def test_retained_reward_hooks_match_native_widget_calls(self):
        from native_1392_machine import ReviewMachine
        entries = dict(self.values["EXTENSION_ADDRESSES"])
        renderer = self.body(entries[self.values["BONUS_UI_RVA"]])
        separator = self.values["BONUS_SEPARATOR_CALLER"]
        self.assertEqual(renderer[separator-5].operands[0].imm, self.values["BONUS_SEPARATOR_BUILD"][0])
        self.assertEqual(renderer[0x358c4a].operands[0].imm, self.values["BONUS_SEPARATOR_SPAWN"][0])
        self.assertEqual(renderer[0x358fc0].operands[0].imm, entries[self.values["BONUS_LINE_RVA"]])
        self.assertEqual(renderer[0x358130].operands[0].imm, entries[self.values["BONUS_ICON_RVA"]])
        header = entries[self.values["BONUS_UI_HEADER_CALLER"]]
        self.assertEqual(header, self.values["BONUS_HEADER_CALLER"])
        constructor = self.body(self.image.owner(header))
        self.assertEqual(constructor[header-5].operands[0].imm, self.values["BONUS_COLUMN_SPAWN"][0])
        tooltip = self.values["KEPT_TOOLTIP_BINDINGS"]
        local_row = constructor[tooltip[0][0]]
        self.assertEqual(local_row.op_str, f"rcx, [rbp - {-self.values['KEPT_TOOLTIP_ROW']:#x}]")

        # AZSX supplies a plain Node and a color. The native builder consumes
        # exactly one bundle and allocates the gradient's owned backing storage.
        m = ReviewMachine(self.image)
        bundle = m.scratch + 0x18000
        node, color = bundle + 0x1000, bundle + 0x2000
        m.cpu.mem_write(bundle, bytes([85])*0x1c0)
        for i in range(4):
            m.write(node + i*24, "3Q", 0, 4, 0)
        width = self.values["BONUS_UI_HEADER_WIDTH"]
        m.write(node + width, "If", 1, 208.0)
        m.write(color, "I4f", 0, 0.1, 0.2, 0.3, 0.7)
        self.assertEqual(m.call(self.values["BONUS_SEPARATOR_BUILD"][0], bundle, node, color), bundle)
        self.assertEqual(m.read(bundle + width, "If"), (1, 208.0))
        cap, ptr, count = m.read(bundle + self.values["NODE_SIZE"], "3Q")
        self.assertTrue(0 < count <= cap and ptr in m.allocations)
        self.assertEqual(bytes(m.cpu.mem_read(bundle + 0x198, 8)), bytes([85])*8)
        # The kept-bonus tooltip reserves space for a complete 24-byte command.
        m.write(node, "3Q", 0, 1, 0)
        m.call(tooltip[4][0], node, 0, 24, 1, 1)
        cap, ptr, count = m.read(node, "3Q")
        self.assertTrue(cap >= 24 and ptr in m.allocations)
        self.assertEqual(count, 0)

    def test_talent_panel_notes_match_native_talent_keys(self):
        import struct
        hook = dict(self.values["ADDRESSES"])[0x1dca71]
        rows = list(self.body(self.image.owner(hook)).values())
        index = next(i for i, op in enumerate(rows) if op.address == hook)
        self.assertEqual([(op.mnemonic, op.op_str) for op in rows[index-4:index-2]],
                         [("lea", "rcx, [rbp - 0x30]"), ("mov", "rdx, r14")])
        self.assertEqual([op.mnemonic for op in rows[index-2:index]], ["call", "nop"])
        self.assertEqual(rows[index].op_str, "r12d, byte ptr [rsi]")
        seen = set()
        for address, key in self.values["COOP_TALENT_KEYS"]:
            pointer, length = struct.unpack("<QQ", self.image.pe.get_data(address, 16))
            self.assertEqual(self.image.pe.get_data(pointer - self.image.pe.OPTIONAL_HEADER.ImageBase, length),
                             key.encode("ascii"))
            self.assertNotIn(key, seen)
            seen.add(key)
        self.assertEqual(len(seen), 13)

    def test_polymorph_eligibility_and_native_attacker_context(self):
        roles = dict(self.values["ADDRESSES"])
        begin, end, scan, resume = (roles[k] for k in
                                    (0x5cfeb6c, 0x5cfed67, 0x5cfeadd, 0x5cfeb08))
        body = self.body(self.image.owner(begin))
        self.assertEqual(body[scan].op_str, "rax, qword ptr [r13 + 0x400]")
        self.assertEqual(body[resume].op_str, f"rsi, qword ptr [rbp + {self.values['ENCOUNTER_GAME_FRAME']:#x}]")
        self.assertEqual(body[resume + body[resume].size].op_str, "r10d, byte ptr [rsi + 0x236]")
        self.assertEqual(body[begin].op_str, "rbx, [rsi + 0xad0]")
        self.assertIn("r14b, al", [i.op_str for a, i in body.items() if begin-16 <= a < begin])
        self.assertIn("qword ptr [rdx + r9*8], r12", [i.op_str for a, i in body.items() if begin < a < begin+80])
        self.assertEqual(body[end].op_str, "r14b, r14b")

    def test_downed_fatigue_projects_into_the_native_aura_allocation(self):
        roles = dict(self.values["EXTENSION_ADDRESSES"])
        site = roles[self.values["FATIGUE_AURA_RVA"]]
        body = self.body(self.image.owner(site))
        slot = self.values["AURA_ROWS_FRAME_SLOT"]
        self.assertEqual(body[0x5ee1f71].op_str, "ecx, 0x300")
        self.assertEqual(body[0x5ee1f83].op_str, f"qword ptr [rbp + {slot:#x}], rax")
        self.assertEqual(body[site].op_str, "qword ptr [rax + 0x9c0], 0x30")
        allocated = [op.op_str for a, op in body.items() if 0x5ee1f83 < a < site]
        self.assertIn("r13, rax", allocated)
        self.assertIn("dword ptr [r13 + 0x2f0], 1", allocated)

    def test_set_dot_ownership_only_at_native_dot_builder_returns(self):
        roles = dict(self.values["ADDRESSES"])
        builder = self.image.owner(roles[0x5b8e99b])
        callers = self.values["SET_DOT_OWNER_CALLERS"]
        self.assertEqual(self.values["SET_DOT_STACK_FIELDS"], [0xe8, 0xb0])
        self.assertEqual(self.values["SET_DOT_ROW_REGISTER"], 1)
        self.assertEqual([(kind, slot) for _, kind, slot in callers],
                         [(0, 0xc0), (1, 0x1a0), (1, 0x1a0), (1, 0x1b8), (1, 0x1b8)])
        for address, kind, slot in callers:
            body = self.body(self.image.owner(address))
            self.assertEqual((body[address-5].mnemonic, body[address-5].operands[0].imm),
                             ("call", builder))
            self.assertIn(f"qword ptr [rbp + {slot:#x}], r15" if kind else "rsi, rcx",
                          [op.op_str for a, op in body.items() if a < address])

    def test_retained_pause_callbacks_still_target_pause_controls(self):
        contacts = dict(self.values["ADDRESSES"])
        ui = self.body(self.image.owner(contacts[0x19d63c]))
        paint = self.body(contacts[0x2e23d0])
        check = ui[contacts[0x19d63c]]
        self.assertEqual(check.op_str, "byte ptr [r12 + 0xee], 1")
        self.assertEqual(ui[check.address + check.size].operands[0].imm, contacts[0x19d661])
        visible = contacts[0x19d66d]
        inputs = [ins.op_str for addr, ins in ui.items() if visible - 32 <= addr < visible]
        for op in ("byte ptr [rbp + 0x2fe], 0", "esi, esi", "edx, byte ptr [rcx]"):
            self.assertIn(op, inputs)
        click = self.body(self.image.owner(contacts[0x19d26e]))
        self.assertEqual(click[contacts[0x19d26e]].op_str, "rcx, rcx")
        self.assertTrue(any(ins.mnemonic == "jne" and ins.operands[0].imm == contacts[0x19cd74]
                            for addr, ins in click.items() if contacts[0x19d26e]-80 < addr < contacts[0x19d26e]))
        for key, label in ((0x2e299c, b"picker.menu"), (0x2e2b79, b"menu.escape_back")):
            point = contacts[key]
            self.assertEqual(paint[point].op_str, "rbx, rdx")
            before = [ins for addr, ins in paint.items() if point-30 <= addr < point]
            ref = next(ins for ins in reversed(before) if ins.mnemonic == "lea" and "rip" in ins.op_str)
            text = ref.address + ref.size + ref.operands[1].mem.disp
            self.assertEqual(self.image.pe.get_data(text, len(label)), label)
        font = contacts[0x2e2b19]
        self.assertTrue(any(ins.op_str == "dword ptr [rbp + 0x470], 0x42380000"
                            for addr, ins in paint.items() if font-140 < addr < font))
        for ret, target in ((0x2e2913, 0x2b72f0), (0x2e3253, 0x2bc790)):
            call = paint[contacts[ret] - 5]
            self.assertEqual((call.mnemonic, call.operands[0].imm), ("call", contacts[target]))

    def test_paid_reroll_then_goblin_amulets_and_acceptance(self):
        from native_1392_machine import CatalogMachine, UC_HOOK_CODE, reg
        m = CatalogMachine(self.image)
        game, settings, borrowed = (m.scratch + n for n in (0x10000, 0x18000, 0x28000))
        m.call(0x5efe1f0, game, settings, settings + 0x2000, settings + 0x4000, settings + 0x6000)
        m.write(game + 0xf63, "B", 2)
        m.write(game + 0xf61, "B", 0)
        m.write(game + 0x578, "I", 350)
        m.write(game + 0x62c, "2I", 0, 0)
        ext = dict(self.values["EXTENSION_ADDRESSES"])

        def cards():
            capacity, start, count = m.read(game + 0x730, "3Q")
            self.assertEqual(count, 3)
            self.assertGreaterEqual(capacity, count)
            return [start + i * 0xa0 for i in range(count)]

        m.call(ext[self.values["REWARD_REROLL_RVA"]], game)
        self.assertEqual(m.read(game + 0x578, "I"), (250,))
        self.assertEqual(m.read(game + 0x62c, "2I"), (0, 1))
        self.assertEqual(len({m.buffer(row) for row in cards()} - {b""}), 3)

        seen = []
        def force_amulets(cpu, address, size, user):
            frame = cpu.reg_read(reg.UC_X86_REG_RBP)
            self.assertEqual(m.read(frame + self.values["GOBLIN_REWARD_GAME_FRAME"], "Q"), (game,))
            header = frame + self.values["GOBLIN_REWARD_POOL_FRAME"]
            capacity, ptr, length = m.read(header, "3Q")
            self.assertTrue(3 <= length <= capacity <= 256)
            self.assertEqual(cpu.reg_read(reg.UC_X86_REG_R15), length)
            for i in range(3):
                m.write(ptr + 24*i, "fI2Q", 1, 0, 1, 0)
            m.write(header + 16, "Q", 3)
            cpu.reg_write(reg.UC_X86_REG_R15, 3)

        def exclude(cpu, address, size, user):
            native = cpu.reg_read(reg.UC_X86_REG_R8)
            _, ptr, n = m.read(native, "3Q")
            self.assertLess(n, 128)
            rows = [m.read(ptr + i*24, "3Q") for i in range(n)] + seen
            if rows:
                for i, row in enumerate(rows):
                    m.write(borrowed + 32 + i*24, "3Q", *row)
                m.write(borrowed, "3Q", len(rows), borrowed + 32, len(rows))
                cpu.reg_write(reg.UC_X86_REG_R8, borrowed)

        def record_card(cpu, address, size, user):
            out = cpu.reg_read(reg.UC_X86_REG_RAX)
            tag, item = m.read(out + 0x78, "2Q")
            self.assertEqual(tag, self.values["AMULET_CHOICE_TAG"])
            key = m.buffer(item)
            at = borrowed + 0x2000 + len(seen)*128
            self.assertTrue(0 < len(key) < 128)
            m.cpu.mem_write(at, key)
            seen.append((len(key), at, len(key)))

        handles = []
        def hook(rva, handler):
            handles.append(m.cpu.hook_add(UC_HOOK_CODE, handler, begin=m.base + rva, end=m.base + rva))
        hook(self.values["GOBLIN_AMULET_POOL"][0], force_amulets)
        hook(self.values["GOBLIN_AMULET_EXCLUDE"][0], exclude)
        builder = ext[self.values["REWARD_CARD_RVA"]]
        for ins in self.image.ins(builder):
            if ins.mnemonic == "ret":
                hook(ins.address, record_card)
        m.call(self.values["ITEM_CHOICE_RVA"], game, 1, 0)
        for handle in handles:
            m.cpu.hook_del(handle)
        offered = [m.buffer(m.read(row + 0x80, "Q")[0]) for row in cards()]
        self.assertEqual(offered, [bytes(m.cpu.mem_read(ptr, n)) for _, ptr, n in seen])
        self.assertEqual(len(set(offered)), 3)
        self.assertTrue(all(key.decode() in self.values["AMULET_KEYS"] for key in offered))
        # The state of the owning actor changes only when an in-range card is accepted.
        def field(name):
            key = self.values[name]
            return next(start + key - first for first, last, start in self.values["FIELD1201_RESOLVED_FIELDS"]
                        if first <= key <= last)
        accept = ext[self.values["REWARD_ACCEPT_RVA"]]
        m.call(accept, game, 3)
        self.assertEqual(m.read(game + field("AMULET_OWNED"), "Q"), (2**64 - 1,))
        m.call(accept, game, 1)
        self.assertEqual(m.buffer(game + field("AMULET_OWNED")), offered[1])
        self.assertEqual(m.read(game + field("AMULET_RANK"), "I"), (1,))
        self.assertEqual(m.read(game + 0x740, "Q"), (0,))

        # Start a fresh run with no queued level-up or goblin choices.
        game += 0x38000
        m.call(0x5efe1f0, game, settings, settings + 0x2000, settings + 0x4000, settings + 0x6000)
        m.write(game + 0xf63, "B", 1)
        m.write(game + 0xf61, "B", 255)
        contacts = dict(self.values["ADDRESSES"])
        chest, convert = contacts[0x5963be0], contacts[0x597e580]
        m.write(game + 0xf00, "I", 100)
        m.write(game + 0x578, "I", 99)
        self.assertEqual(m.call(chest, game) & 255, 0)
        self.assertEqual(m.read(game + 0x578, "I"), (99,))
        m.call(contacts[0x5920470], game, 126)
        balance, = m.read(game + 0x578, "I")
        self.assertGreaterEqual(balance, 225)
        self.assertEqual(m.call(chest, game) & 255, 1)
        self.assertEqual(m.read(game + 0x578, "I"), (balance - 100,))
        self.assertEqual(m.read(game + 0xf00, "I"), (135,))
        # Present a cache reward. Conversion must consume the offer once.
        m.call(self.values["ITEM_CHOICE_RVA"], game, 1, 0)
        m.write(game + 0x352, "B", 1)
        m.write(game + 0xf63, "B", 2)
        m.call(convert, game)
        after, = m.read(game + 0x578, "I")
        self.assertGreater(after, balance - 100)
        self.assertEqual(m.read(game + 0x740, "Q"), (0,))
        m.call(convert, game)
        self.assertEqual(m.read(game + 0x578, "I"), (after,))

    def test_goblin_native_placement_and_creature_creation(self):
        import math
        import struct
        from native_1392_machine import CatalogMachine, UC_HOOK_CODE, reg
        m = CatalogMachine(self.image)
        game, settings, out, terrain = (m.scratch + n for n in (0x10000, 0x18000, 0x24000, 0x28000))
        m.call(0x5efe1f0, game, settings, settings + 0x2000, settings + 0x4000, settings + 0x6000)
        m.write(game + self.values["EVENT_TERRAIN_OFFSET"], "Q", terrain)
        m.write(terrain + 0xd0, "fB", 5, 1)
        hero_x, hero_y = m.read(game + 0x550, "2f")
        m.write(terrain + 0x40, "4Q2f", 4096, terrain + 0x1000, 4096, 64, hero_x - 32, hero_y - 32)
        def bits(f):
            return struct.unpack("<I", struct.pack("<f", f))[0]
        m.cpu.reg_write(reg.UC_X86_REG_XMM2, bits(6))
        m.cpu.reg_write(reg.UC_X86_REG_XMM3, bits(9))
        m.call(self.values["GOBLIN_POSITION"][0], out, game, 0, 0, bits(0.5))
        x, y, z = m.read(out, "3f")
        self.assertTrue(6 <= math.hypot(x - hero_x, y - hero_y) <= 9)
        self.assertEqual(z, 5)
        table = self.values["GOBLIN_MAP_TABLE"]
        name, length = struct.unpack("<2Q", self.image.pe.get_data(table + 8, 16))
        m.write(game + 0x358, "2Q", name, length)
        start, count = struct.unpack("<2Q", self.image.pe.get_data(table + 0x78, 16))
        rows = self.image.pe.get_data(start - m.base, count * 80)
        template = bytearray(next(rows[i:i+80] for i in range(0, len(rows), 80) if rows[i + 76] == 1))
        definition = out + 256
        template[:16] = bytes(16)
        name = b"Venture Co. Treasurer"
        m.cpu.mem_write(definition + 128, name)
        struct.pack_into("<2Q2I", template, 16, definition + 128, len(name),
                         self.values["GOBLIN_ENTRY"], self.values["GOBLIN_DISPLAY"])
        struct.pack_into("<f", template, 40, 2)
        guardian_health, = struct.unpack_from("<f", template, 48)
        struct.pack_into("<f", template, 48, min(1e9, max(1, guardian_health * 1.5)))
        struct.pack_into("<3f", template, 52, 0, 5, 5.2)
        template[64:] = bytes(16)
        m.cpu.mem_write(definition, bytes(template))
        def substitute(cpu, address, size, user):
            self.assertEqual(cpu.reg_read(reg.UC_X86_REG_RSI), game)
            self.assertTrue(start <= cpu.reg_read(reg.UC_X86_REG_R15) < start + count*80)
            cpu.reg_write(reg.UC_X86_REG_R15, definition)
        site = m.base + self.values["GOBLIN_DEFINITION"][0]
        handle = m.cpu.hook_add(UC_HOOK_CODE, substitute, begin=site, end=site)
        m.write(out, "I3f", 1, x, y, z)
        spawn = self.values["GOBLIN_SPAWN"][0]
        m.call(spawn, game, 0, out)
        m.cpu.hook_del(handle)
        m.call(spawn, game, 0, out)
        _, first, count = m.read(game + 0x640, "3Q")
        self.assertEqual(count, 2)
        guids = [m.read(first + i*self.values["ENEMY_STRIDE"] + self.values["ENEMY_GUID"], "Q")[0]
                 for i in range(2)]
        self.assertEqual((guids[0] >> 24) & 0xffffff, self.values["GOBLIN_ENTRY"])
        self.assertNotEqual((guids[1] >> 24) & 0xffffff, self.values["GOBLIN_ENTRY"])
        self.assertEqual(m.read(first + self.values["ENEMY_POSITION"], "3f"), (x, y, z))

    def test_goblin_recycling_motion_and_texture_input(self):
        from native_1392_machine import ReviewMachine
        m = ReviewMachine(self.image)
        game = m.scratch + 0x10000
        out, iterator, rows = [game + n for n in (0x10000, 0x10100, 0x11000)]
        stride = self.values["ENEMY_STRIDE"]
        for i, distance in enumerate((500, 1, 400)):
            row = rows + i * stride
            m.write(row + 0x100, "Q", 123)
            for offset in (0x18, 0x58):
                m.write(row + offset, "Q", 2**64 - 1)
            m.write(row + self.values["ENEMY_POSITION"], "3f", distance, 0, 0)
            m.write(row + self.values["ENEMY_GUID"], "Q", 20 + i)
        m.write(iterator, "5Q", rows, rows + 3 * stride, game, iterator + 64, 4)
        self.assertEqual(m.call(self.values["GOBLIN_RECYCLE"][0], out, iterator), out)
        _, ptr, length = m.read(out, "3Q")
        self.assertEqual(length, 2)
        self.assertEqual(m.read(ptr, "2Q"), (20, 22))
        self.assertEqual(m.read(iterator + 16, "Q"), (game,))

        key = b"3ef7f3eb90fa64d85635bc771b647bee.blp"
        m.cpu.mem_write(iterator, key)
        m.call(self.values["GOBLIN_SKIN"][0], out, iterator, len(key))
        self.assertEqual(m.buffer(out), b"mpq://textures/bakednpctextures/" + key)
        body = self.body(self.image.owner(self.values["GOBLIN_FINAL_MOTION"][0]))
        for site, role in ((0x5f1a590, "BASIC_GAME"), (0x5f16f37, "BASIC_ENEMY"),
                           (0x5f17de3, "BASIC_POSITION"), (0x5f16d69, "BASIC_ORIGIN_XY"),
                           (0x5f1a006, "BASIC_ORIGIN_Z")):
            operand, = [o for o in body[site].operands if o.type == 3]
            self.assertEqual((body[site].reg_name(operand.mem.base), operand.mem.disp),
                             ("rbp", self.values[role]))
        self.assertEqual(body[0x5f1a006].op_str, "xmm14, dword ptr [rbp + 0x24c]")
        self.assertEqual(body[0x5f1a200].op_str, "xmm13, xmm1")
        self.assertEqual(body[0x5f1a210].op_str, "xmm9, xmm13")

    def test_goblin_mark_rewards_follow_native_factors(self):
        import struct
        from native_1392_machine import ReviewMachine, reg
        m = ReviewMachine(self.image)
        game = m.scratch + 0x10000
        fields = self.values["GOBLIN_MARK_FIELDS"]
        for index, endless, kills, heroic in ((0, 0, 0, 0), (9, 1, 0, 2), (10, 1, 3, 5)):
            m.write(game + self.values["EVENT_ENDLESS_OFFSET"], "I", endless)
            m.write(game + self.values["GOBLIN_ENDLESS_KILLS_OFFSET"], "I", kills)
            m.write(game + self.values["HEROIC_LEVEL"], "B", heroic)
            for at, value in zip(fields, (5, 11)):
                m.write(game + at, "f", value)
            m.cpu.reg_write(reg.UC_X86_REG_RSI, game)
            m.cpu.reg_write(reg.UC_X86_REG_RBX, game + 0x2000)
            m.cpu.reg_write(reg.UC_X86_REG_RCX, index & ~1)
            m.cpu.emu_start(m.base + 0x5f08b5d, m.base + 0x5f08c00, count=100)
            self.assertEqual(m.cpu.reg_read(reg.UC_X86_REG_RIP), m.base + 0x5f08c00)
            factor = (self.values["HEROIC_REWARD_TABLE"] + heroic * 48 - 8 if heroic
                      else self.values["NORMAL_MARKS_FACTOR_RVA"])
            gain = (index // 2 + 1) * max(kills, 1) * struct.unpack("<f", self.image.pe.get_data(factor, 4))[0]
            for at, start in zip(fields, (5, 11)):
                self.assertAlmostEqual(m.read(game + at, "f")[0], start + gain, places=5)

    def test_native_heap_alignment_and_release_at_windows_imports(self):
        import pefile
        from compare_official import MD
        from native_1392_machine import ReviewMachine
        from unicorn import UC_HOOK_CODE, x86_const as reg
        machine = ReviewMachine(self.image)
        pe = self.image.pe
        pe.parse_data_directories(directories=[pefile.DIRECTORY_ENTRY["IMAGE_DIRECTORY_ENTRY_IMPORT"]])
        imports = {item.address: item.name for dll in pe.DIRECTORY_ENTRY_IMPORT for item in dll.imports}
        # The usual review machine isolates native allocations at their thunks.
        # Here execute their actual tail targets, isolating only the OS heap.
        def target(key):
            thunk = dict(self.values["ADDRESSES"])[key]
            jump = next(MD.disasm(pe.get_data(thunk, 8), thunk))
            self.assertEqual(jump.mnemonic, "jmp")
            return jump.operands[0].imm
        heap, storage = machine.scratch + 0x20000, machine.scratch + 0x24010
        requested, released = [], []
        def service(cpu, address, size, name):
            if name == b"GetProcessHeap":
                result = heap
            else:
                self.assertEqual(cpu.reg_read(reg.UC_X86_REG_RCX), heap)
                self.assertEqual(cpu.reg_read(reg.UC_X86_REG_RDX), 0)
                argument = cpu.reg_read(reg.UC_X86_REG_R8)
                if name == b"HeapAlloc":
                    requested.append(argument)
                    result = storage
                else:
                    released.append(argument)
                    result = 1
            cpu.reg_write(reg.UC_X86_REG_RAX, result)
        targets = {}
        for site, name in ((0x630c081, b"GetProcessHeap"), (0x630c09b, b"HeapAlloc"),
                           (0x62f4888, b"GetProcessHeap"), (0x62f489c, b"HeapFree")):
            ins = next(MD.disasm(pe.get_data(site, 16), site))
            iat = machine.base + ins.address + ins.size + ins.operands[0].mem.disp
            self.assertEqual(imports[iat], name)
            if name not in targets:
                address = heap + 16 * len(targets)
                targets[name] = address
                machine.cpu.mem_write(address, bytes([0xc3]))
                machine.cpu.hook_add(UC_HOOK_CODE, service, name, begin=address, end=address)
            if not any(first <= iat <= last for first, last, _ in machine.cpu.mem_regions()):
                machine.map_page(machine.cpu, 0, iat, 8, 0, None)
            machine.write(iat, "Q", targets[name])
        for align in (1, 8, 16, 32, 64):
            pointer = machine.call(target(0x21c0), 113, align)
            self.assertEqual(pointer % align, 0)
            self.assertEqual(requested[-1], 113 if align <= 16 else 113 + align)
            self.assertTrue(storage <= pointer <= storage + requested[-1] - 113)
            machine.call(target(0x21d0), pointer, 113, align)
            self.assertEqual(released[-1], storage)
        self.assertEqual(len(requested), len(released))

    def test_virtual_script_reader_returns_owned_cached_script(self):
        from compare_official import MD
        from native_1392_machine import ReviewMachine
        machine = ReviewMachine(self.image)
        output = machine.scratch + 0x10000
        path = b"Interface\\FrameXML\\WorldStateFrame.lua"
        machine.cpu.mem_write(output + 64, path)
        cache = output + 0x2000

        def global_at(site):
            ins = next(MD.disasm(self.image.pe.get_data(site, 16), site))
            operand, = [op for op in ins.operands if op.type == 3]
            return machine.base + ins.address + ins.size + operand.mem.disp

        # Exercise the initialized cache as the reader sees it after startup.
        control = global_at(0x625f17e)
        for address, value in ((control, cache), (control + 8, 15),
                               (global_at(0x625f151), 1), (global_at(0x625f116), output + 0x1000)):
            if not any(first <= address <= last for first, last, _ in machine.cpu.mem_regions()):
                machine.map_page(machine.cpu, 0, address, 8, 0, None)
            machine.write(address, "Q", value)
        machine.write(global_at(0x625f0f6), "I", 0)
        machine.cpu.mem_write(cache, bytes([255]) * 32)
        key = output + 0x400
        machine.call(0x625f530, key, output + 64, len(path))
        self.assertEqual(machine.buffer(key), path.lower())
        hashed = machine.call(0x11d50, global_at(0x625f15f), key)
        bucket = hashed & 15
        for index in (bucket, bucket + 16):
            machine.write(cache + index, "B", hashed >> 57)
        script = b"AZSApiFrame = CreateFrame('Frame')"
        machine.cpu.mem_write(output + 0x800, script)
        machine.write(cache - 48 * (bucket + 1), "6Q", *machine.read(key, "3Q"),
                      len(script), output + 0x800, len(script))
        result = machine.call(self.values["ADDON_API_READER"], output, 0, output + 64, len(path))
        self.assertEqual(result, output)
        self.assertEqual(machine.buffer(output), script)
        cap, ptr, _ = machine.read(output, "3Q")
        self.assertNotEqual(ptr, output + 0x800)
        machine.call(0x21d0, ptr, cap, 1)
        self.assertEqual(bytes(machine.cpu.mem_read(output + 0x800, len(script))), script)

    def test_native_host_submission_queues_hero_and_named_lobby(self):
        from native_1392_machine import ReviewMachine
        machine = ReviewMachine(self.image)
        resource = machine.scratch + 0x10000
        writer, shared, rows, hero, text = [resource + i for i in (0x400, 0x500, 0x800, 0x1000, 0x1200)]
        for header, value in ((resource + self.values["LOBBY_NAME"], b"[AZSX] Trial"),
                              (resource + 0xe8, b"private"), (resource + 0x100, b"fixture"),
                              (hero, b"Test hero"), (hero + 24, b"Test class")):
            machine.cpu.mem_write(text, value)
            machine.write(header, "3Q", len(value), text, len(value))
            text += 32
        machine.write(resource + 0x1fd, "B", 5)
        machine.write(writer, "Q", shared)
        machine.write(shared + 0x38, "3Q", 4, rows, 0)
        machine.call(self.values["LOBBY_SUBMIT"], resource, writer, hero, 0)
        self.assertEqual(machine.read(shared + 0x48, "Q"), (0,))
        machine.call(self.values["LOBBY_SUBMIT"], resource, writer, hero, 1)
        self.assertEqual(machine.read(shared + 0x48, "Q"), (2,))
        self.assertEqual(machine.read(rows, "I"), (12,))
        self.assertEqual(machine.buffer(rows + 8), b"Test hero")
        room = rows + 0xe8
        self.assertEqual(machine.read(room, "I"), (23,))
        self.assertEqual(machine.buffer(room + 8), b"[AZSX] Trial")
        self.assertEqual(machine.buffer(room + 32), b"private")
        self.assertEqual(machine.buffer(room + 56), b"fixture")

    def test_event_map_bounds_and_marks_tables(self):
        import struct
        from compare_official import Image
        old_path = os.environ.get("AZSX_REVIEWED_IMAGE_138")
        if not old_path:
            self.skipTest("Set AZSX_REVIEWED_IMAGE_138 for the reviewed map catalog")
        previous = bindings("0.138.0")
        old = Image(Path(old_path))
        self.assertEqual(old.sha, "eb5fd1c1b96e6180fcf5f3d685d90a61471af28d22e5f060c02b7b7663fbf79e")

        def maps(image, values):
            def read(at, fmt):
                return struct.unpack(fmt, image.pe.get_data(at, struct.calcsize(fmt)))
            def name(at):
                pointer, length = read(at, "<2Q")
                self.assertTrue(0 < length < 256)
                return image.pe.get_data(pointer - image.pe.OPTIONAL_HEADER.ImageBase, length)
            table = dict(values["EXTENSION_ADDRESSES"])[values["BLESSING_MAP_TABLE"]]
            self.assertEqual(table, values["GOBLIN_MAP_TABLE"])
            result = []
            for index in range(values["BLESSING_MAP_COUNT"]):
                row = table + index * values["BLESSING_MAP_STRIDE"]
                bosses, count = read(row + 136, "<2Q")
                self.assertEqual(count, 5)
                bosses -= image.pe.OPTIONAL_HEADER.ImageBase
                result.append((name(row + 8), read(row + 0xc8, "<2f"), read(row + 0xd8, "<f"),
                               [name(bosses + 104 * i) for i in range(count)]))
            return result

        self.assertEqual(maps(old, previous), maps(self.image, self.values))
        for role, offset, size in (("BOSS_MARKS_TABLE", 0, 24), ("NORMAL_MARKS_FACTOR_RVA", 0, 4),
                                   *(("HEROIC_REWARD_TABLE", h * 48 - 8, 4) for h in range(1, 6))):
            self.assertEqual(old.pe.get_data(previous[role] + offset, size),
                             self.image.pe.get_data(self.values[role] + offset, size))

    def test_damage_renderer_row_and_size_operands(self):
        from capstone import x86_const
        body = self.body(0xe7f920)
        rva, _ = self.values["DAMAGE_TEXT_SIZE"]
        self.assertEqual(self.values["DAMAGE_TEXT_ROW_REGISTER"], 14)
        self.assertEqual(self.values["DAMAGE_TEXT_XMM"], 8)
        self.assertEqual(body[0xe80d12].op_str, "ecx, byte ptr [r14 + 0x38]")
        self.assertEqual(body[0xe80eac].op_str, "eax, dword ptr [r14 + 0x34]")
        self.assertEqual(body[rva].op_str, "xmm10, xmm8")
        for instruction in body.values():
            if 0xe80d12 < instruction.address < 0xe80eac:
                self.assertNotIn(x86_const.X86_REG_R14, instruction.regs_access()[1])

    def test_codex_auto_discovery_preserves_higher_rarity(self):
        from native_1392_machine import ReviewMachine
        machine = ReviewMachine(self.image)
        entries = dict(self.values["ADDRESSES"])
        collection = machine.scratch + 0x10000
        text = collection + 0x400
        for key, rarity, changed in ((b"insignia", 2, 1), (b"eldre_thalas", 0, 1),
                                     (b"chosen", 0, 1), (b"insignia", 0, 0), (b"chosen", 0, 0)):
            machine.cpu.mem_write(text, key)
            result = machine.call(entries[0x5c4d7f0], collection, text, len(key), rarity)
            self.assertEqual(result & 255, changed)
        _, rows, count = machine.read(collection + 0x90, "3Q")
        self.assertEqual([(machine.buffer(rows + 32*i), machine.read(rows + 32*i + 24, "B")[0])
                          for i in range(count)], [(b"chosen", 0), (b"eldre_thalas", 0), (b"insignia", 2)])
        load = self.body(0x5e839d0)
        self.assertEqual(load[0x5e839ef].op_str, "rsi, rcx")
        self.assertEqual(load[entries[0x5c4dd97]].op_str, "rdx, qword ptr [rbp + 0x10]")
        self.assertEqual(load[entries[0x5c4ddf8]].op_str, "rax, rsi")
        save = self.body(entries[0x5c56eb0])
        self.assertEqual(save[0x5e8cdc6].op_str, "rsi, rcx")
        self.assertEqual(save[0x5e8cdd0].op_str, "rdx, rsi")
        message = save[0x5e8ce58]
        string = message.address + message.size + message.operands[1].mem.disp
        self.assertTrue(self.image.pe.get_data(string, 48).startswith(b"rogue: sealing the save failed"))

    def test_pause_key_queue_guest_delivery_and_native_apply(self):
        import struct
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        machine = ReviewMachine(self.image)
        server = machine.scratch + 0x10000
        writer, shared, rows = (server + n for n in (0x4000, 0x4200, 0x4600))
        machine.write(writer, "Q", shared)
        machine.write(shared + 0x38, "3Q", 4, rows, 0)
        key_system = self.body(self.image.owner(self.values["MENU_KEY"][0]))
        ret = self.values["MENU_ESCAPE_QUEUE_RETURN"]
        self.assertEqual(key_system[ret - 5].operands[0].imm, self.values["MENU_UI_QUEUE"][0])
        self.assertEqual(key_system[0x25f619].operands[0].imm, self.values["MENU_KEY_NATIVE_PAUSE"])
        game, args = server + 16, server + 0x8000
        machine.call(0x5efe1f0, game, args, args + 0x2000, args + 0x4000, args + 0x6000)
        machine.write(game + 0xf63, "B", 1)
        machine.write(game + 0xf5e, "B", 0)
        for count, (start, paused) in enumerate(((self.values["MENU_KEY_NATIVE_PAUSE"], 1), (0x25f59e, 0)), 1):
            stack = machine.stack - 8
            machine.cpu.reg_write(reg.UC_X86_REG_RSP, stack)
            machine.cpu.reg_write(reg.UC_X86_REG_RSI, writer)
            machine.write(stack + 0x58, "Q", writer)
            machine.cpu.emu_start(machine.base + start, machine.base + ret, count=100000)
            self.assertEqual(machine.cpu.reg_read(reg.UC_X86_REG_RIP), machine.base + ret)
            self.assertEqual(machine.read(shared + 0x48, "Q"), (count,))
            action = rows + (count - 1) * 0xe8
            self.assertEqual(machine.read(action, "IB"), (7, paused))
            for role, active, consumed in ((0, 1, 1), (3, 1, 0), (0, 0, 0)):
                machine.write(server + 0xf78, "I", role)
                machine.write(server + 0x13b5, "B", active)
                self.assertEqual(machine.call(self.values["MENU_INPUT"][0], server, action) & 255, consumed)
            machine.call(self.values["MENU_INPUT_APPLY"][0], server, action)
            self.assertEqual(machine.read(game + 0xf63, "B"), (3 if paused else 1,))
        payload, queue = server + 0x6000, server + 0x6200
        machine.write(server + 0x13b5, "B", 1)
        machine.write(server + 0xfe8, "3Q", 4, queue, 0)
        machine.write(payload, "QB", 57, 1)
        opcode = self.values["MENU_OPCODE"]
        machine.call(self.values["MENU_SEND"][0], server, 0, opcode, payload, 9)
        self.assertEqual(machine.read(server + 0xff8, "Q"), (1,))
        self.assertEqual(machine.buffer(queue), struct.pack("<BHIQB", 7, opcode, 9, 57, 1))

    def test_scene_visual_unit_messages_and_buffer_ownership(self):
        import struct
        from native_1392_machine import ReviewMachine
        machine = ReviewMachine(self.image)
        out = machine.scratch + 0x20000
        fields, block, point, name, title, goal = [out + n for n in (64, 128, 256, 400, 500, 600)]
        guid, entry, display = 0xf130a50010000008, 0xa50010, 1234
        machine.write(point, "3f", 17, -13.25, 4.5)
        machine.write(goal, "3f", 18, -13.25, 4.5)
        machine.cpu.mem_write(name, b"Sayge")
        machine.cpu.mem_write(title, b"Test title")

        def call(role, *arguments):
            return machine.call(self.values[role][0], *arguments)

        def take(opcode):
            self.assertEqual(machine.read(out+24, "H"), (opcode,))
            body = machine.buffer(out)
            capacity, ptr, _ = machine.read(out, "3Q")
            machine.call(0x21d0, ptr, capacity, 1)
            return body

        call("GOBLIN_EVENT", out, name, 5, title, 10)
        notice = take(0x96)
        self.assertIn(b"Sayge\0", notice)
        self.assertIn(b"Test title\0", notice)
        call("GOBLIN_QUERY_BUILD", out, entry, name, 5, title, 10, 7, 0, display)
        metadata = take(0x61)
        self.assertTrue(metadata.startswith(struct.pack("<I", entry) + b"Sayge\0"))
        self.assertIn(b"Test title\0", metadata)
        self.assertIn(struct.pack("<I", display), metadata)
        call("SOLO_SCENE_SPLINE", out, guid, point, goal, 17, 400)
        movement = take(0xdd)
        self.assertIn(struct.pack("<3fI", 17, -13.25, 4.5, 17), movement)
        self.assertTrue(movement.endswith(struct.pack("<II3f", 400, 1, 18, -13.25, 4.5)))

        for index, value in ((0, guid & 0xffffffff), (1, guid >> 32), (2, 9), (3, entry),
                             (4, 0x3f800000), (37, 23173), (38, 23322), (131, display), (164, 1)):
            call("EVENT_UNIT_FIELDS_SET", fields, index, value)
        machine.write(block, "4Q", 0, 1, 0, 0)
        call("EVENT_UNIT_CREATE", block, guid, 3, 0, point, 0, point + 32, fields)
        unit = machine.buffer(block)
        self.assertIn(struct.pack("<3f", 17, -13.25, 4.5), unit)
        self.assertTrue(unit.endswith(struct.pack("<4I", 23173, 23322, display, 1)))
        call("EVENT_UPDATE_PACKET", out, block)
        call("EVENT_UNIT_FIELDS_DROP", fields)
        self.assertEqual(take(0xa9), bytes([1, 0, 0, 0, 0]) + unit)
        self.assertFalse(machine.allocations)

    def test_event_ground_height_and_obstructed_scene_movement(self):
        import struct
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        machine = ReviewMachine(self.image)
        terrain = machine.scratch + 0x10000
        height = terrain + 0x10
        output = terrain + 0x1000
        cells = output + 0x80
        machine.write(height + 0x78, "3Q", cells + 256, 4, 2)
        machine.write(cells + 256, "4f", -4, 2, 3, 7)
        for x, y, expected in ((0, 0, -4), (0.5, 0, 2), (0, 0.5, 3), (0.5, 0.5, 7), (1, 0, None)):
            for register, coord in ((reg.UC_X86_REG_XMM1, x), (reg.UC_X86_REG_XMM2, y)):
                machine.cpu.reg_write(register, struct.unpack("<I", struct.pack("<f", coord))[0])
            present = machine.call(self.values["EVENT_GROUND_HEIGHT"][0], height) & 255
            self.assertEqual(present, int(expected is not None))
            if present:
                bits = machine.cpu.reg_read(reg.UC_X86_REG_XMM0) & 0xffffffff
                self.assertEqual(struct.unpack("<f", struct.pack("<I", bits))[0], expected)
        machine.write(terrain + 0x40, "4Q", 64, cells, 64, 8)
        machine.write(output + 16, "3f", 1.5, 1.5, 2)
        machine.write(output + 32, "3f", 6, 1.5, 8)
        for flag in (0, 1):
            machine.write(cells + 11, "B", flag)
            machine.call(self.values["GOBLIN_CLIP"][0], output, terrain + 0x40, output + 16, output + 32)
            x, y, z = machine.read(output, "3f")
            self.assertAlmostEqual(x, 2.2 if flag else 6, places=5)
            self.assertEqual(y, 1.5)
            self.assertAlmostEqual(z, 2 + (x - 1.5) * 6 / 4.5, places=5)

    def test_native_event_creation_and_per_slot_delivery(self):
        import struct
        from native_1392_machine import ReviewMachine
        machine = ReviewMachine(self.image)
        game = machine.scratch + 0x10000
        server = game - 16
        descriptor, position, metadata, name = (game + n for n in (0x2000, 0x2100, 0x2200, 0x2300))
        peer, stream = game + 0x30000, game + 0x40000
        queue, outbound = machine.allocate(32, 8), game + 0x60000
        machine.write(game + 0xed4, "I", 7)
        machine.write(game + 0xe00, "4Q", 133, 211, 343, 555)
        machine.write(game + 0x700, "3Q", 1, queue, 1)
        prior = bytes(range(32))
        machine.cpu.mem_write(queue, prior)
        machine.write(server + 0xf78, "I", 3)
        machine.write(server + 0xfb0, "2Q", peer, 1)
        machine.write(peer, "I", 1)
        machine.write(peer + 8, "Q", 1)
        machine.write(peer + 0x86d, "B", 1)
        machine.write(peer + 0x390, "3Q", 8, outbound, 0)
        machine.write(server + 0x13f0, "2Q", stream, 1)
        machine.write(stream + 0xa8, "B", 1)
        machine.write(stream + 0x90, "Q", stream + 0x800)

        guest = ReviewMachine(self.image)
        guest_server = guest.scratch + 0x10000
        message = guest_server + 0x8000
        guest_stream = guest_server + 0x10000
        guest.write(guest_server + 0x13b5, "B", 1)
        guest.write(guest_server + 0x13f0, "2Q", guest_stream, 1)
        guest.write(guest_stream + 0xa8, "B", 1)
        guest.write(guest_stream + 0x90, "Q", guest_stream + 0x800)
        guest.write(guest_server + 0x1020, "Q", message + 0x400)
        guest.cpu.mem_write(message + 0x400, bytes([255]) * 32)

        def deliver(packet, opcode, slots=(0, 1, 2)):
            self.assertEqual(machine.read(packet + 24, "H"), (opcode,))
            payload = machine.buffer(packet)
            count = machine.read(peer + 0x3a0, "Q")[0]
            local_count = len(machine.local_packets)
            for slot in slots:
                machine.call(self.values["GOBLIN_PACKET_SEND"][0], server, slot, packet)
            self.assertEqual(len(machine.local_packets), local_count + int(0 in slots))
            if 0 in slots:
                self.assertEqual(machine.local_packets[-1], (stream + 0x810,
                    struct.pack(">H", len(payload) + 2) + struct.pack("<H", opcode) + payload))
            self.assertEqual(machine.read(peer + 0x3a0, "Q"), (count + 1,))
            self.assertEqual(machine.buffer(outbound + 24 * count),
                             struct.pack("<BHI", 7, opcode, len(payload)) + payload)

            # Feed the connected peer's queued payload to the native guest
            # dispatcher. Verify the exact packet reaching its local game client.
            wire = machine.buffer(outbound + 24 * count)
            variant, guest_opcode, length = struct.unpack_from("<BHI", wire)
            self.assertEqual(variant, 7)
            owned = guest.allocate(length, 1)
            guest.cpu.mem_write(owned, wire[7:])
            guest.write(message, "4QH", 0x8000000000000006, length, owned, length, guest_opcode)
            guest.call(self.values["GOBLIN_PACKET_PROBE"][0], guest_server, message)
            self.assertNotIn(owned, guest.allocations)
            self.assertEqual(guest.local_packets[-1], (guest_stream + 0x810,
                struct.pack(">H", len(payload) + 2) + struct.pack("<H", opcode) + payload))

        for serial, (entry, display, scale, title) in enumerate((
            (0xa50001, 36, 1.0, b"Lost Supplies"),
            (0xa50007, 2972, 0.75, b"Green Crystal"),
        ), 8):
            machine.cpu.mem_write(name, title)
            machine.call(0x5da7220, metadata, entry, 5, display, name, len(title))
            self.assertTrue(machine.buffer(metadata).startswith(struct.pack("<3I", entry, 5, display) + title + b"\0"))
            deliver(metadata, 0x5f)
            machine.write(descriptor, "IIf", entry, display, scale)
            xyz = (17.0, -13.25, 4.5)
            machine.write(position, "3f", *xyz)
            guid = machine.call(0x5f290f0, game, descriptor, position, 5, 0)
            self.assertEqual(guid, (0xf110 << 48) | (entry << 24) | serial)
            capacity, current_queue, length = machine.read(game + 0x700, "3Q")
            self.assertGreaterEqual(capacity, length)
            self.assertEqual(length, 2)
            if serial == 8:
                self.assertNotEqual(current_queue, queue)
            queue = current_queue
            self.assertEqual(bytes(machine.cpu.mem_read(queue, 32)), prior)
            row = queue + 32
            body = machine.buffer(row)
            guid_bytes = guid.to_bytes(8, "little")
            mask = sum(1 << i for i, byte in enumerate(guid_bytes) if byte)
            self.assertTrue(body.startswith(b"\x01\0\0\0\0\x02" + bytes([mask]) +
                                            bytes(b for b in guid_bytes if b) + b"\x05"))
            self.assertIn(struct.pack("<3f", *xyz), body)
            # Copy and detach the appended row before invoking either sender.
            borrowed = game + 0x70000
            machine.cpu.mem_write(borrowed + 32, body)
            machine.write(borrowed, "4Q", len(body), borrowed + 32, len(body), 0xa9)
            machine.write(game + 0x710, "Q", 1)
            capacity, buffer, _ = machine.read(row, "3Q")
            machine.call(0x21d0, buffer, capacity, 1)
            deliver(borrowed, 0xa9)
            # Personal hide goes to the guest only. A scene-wide removal reaches
            # both transports. Both carry the full GUID from native creation.
            machine.cpu.mem_write(borrowed + 32, guid_bytes)
            machine.write(borrowed, "4Q", 8, borrowed + 32, 8, 0xaa)
            deliver(borrowed, 0xaa, (1,) if serial == 8 else (0, 1))
        self.assertEqual(len(machine.local_packets), 5)

    def test_server_routes_by_assigned_slot_across_peer_removal(self):
        from native_1392_machine import ReviewMachine
        machine = ReviewMachine(self.image)
        server, peers, packet = (machine.scratch + n for n in (0x10000, 0x20000, 0x30000))
        stride = self.values["SERVER_PEER_STRIDE"]
        machine.write(server + self.values["SERVER_ROLE"], "I", 3)
        machine.write(server + self.values["SERVER_PEER_PTR"], "2Q", peers, 2)
        machine.write(packet, "4Q", 1, packet + 32, 1, 0xaa)
        machine.write(packet + 32, "B", 42)
        for index, slot in enumerate((3, 1)):
            row = peers + index * stride
            machine.write(row, "I", 1)
            machine.write(row + 8, "Q", slot)
            machine.write(row + 0x86d, "B", 1)
            machine.write(row + 0x390, "3Q", 8, packet + 256 + 256 * index, 0)

        def send(slot):
            machine.call(self.values["GOBLIN_PACKET_SEND"][0], server, slot, packet)

        def lengths():
            return [machine.read(peers + n * stride + 0x3a0, "Q")[0] for n in (0, 1)]

        send(1)
        self.assertEqual(lengths(), [0, 1])
        machine.write(peers + 0x86d, "B", 0)
        send(3)
        self.assertEqual(lengths(), [0, 1])
        machine.write(peers + 0x86d, "B", 1)
        machine.write(peers, "I", 0)
        send(3)
        self.assertEqual(lengths(), [0, 1])
        # Present the surviving connection at a different vector index.
        machine.cpu.mem_write(peers, bytes(machine.cpu.mem_read(peers + stride, stride)))
        machine.write(server + self.values["SERVER_PEERS"], "Q", 1)
        send(3)
        self.assertEqual(lengths(), [1, 1])
        send(1)
        self.assertEqual(lengths(), [2, 1])
        for other_role in (0, 1, 2):
            machine.write(server + self.values["SERVER_ROLE"], "I", other_role)
            send(1)
            self.assertEqual(lengths(), [2, 1])

    def test_candidate_guards_and_server_sink(self):
        from compare_official import MD
        contacts = dict(self.values["ADDRESSES"])
        guards = [(contacts[key], guard) for key, guard in self.values["ENTRIES"]]
        extensions = dict(self.values["EXTENSION_ADDRESSES"])
        guards.extend((extensions[key], guard) for key, guard in self.values["EXTENSION_ENTRIES"])
        manifest = json.loads((ROOT / "compat/clients/0.139.2.json").read_text())
        guards.extend(self.values[row["name"]] for row in manifest["constants"]
                      if row["type"] == "(u64, &[u8])")
        reroll = self.values["REWARD_REROLL_RVA"]
        self.assertEqual(dict(self.values["EXTENSION_ENTRIES"])[reroll], self.values["REWARD_REROLL_BYTES"])
        for entry, resume, entry_guard, resume_guard in self.values["DOWNED_XP_GUARDS"]:
            guards.extend(((entry, entry_guard), (resume, resume_guard)))
        for name in ("ADDON_API_READER", "LOBBY_SUBMIT"):
            guards.append((self.values[name], self.values[name + "_BYTES"]))
        for rva, guard in guards:
            with self.subTest(rva=hex(rva)):
                data = self.image.pe.get_data(rva, len(guard))
                self.assertEqual(data, bytes(guard))
                self.assertEqual(sum(i.size for i in MD.disasm(data, rva)), len(data))
        import struct
        base = self.image.pe.OPTIONAL_HEADER.ImageBase
        vtable = struct.unpack("<9Q", self.image.pe.get_data(contacts[0x5ede960], 72))
        self.assertEqual(vtable[1:3], (32, 8))
        self.assertEqual(vtable[3] - base, 0x5e24450)
        update = self.body(0x5e50b10)
        self.assertEqual(update[0x5e567d7].operands[1].mem.disp, self.values["SERVER_UI_SINK"])
        self.assertEqual(update[0x5e54777].operands[0].mem.disp, self.values["SERVER_GAME_STREAM"])

    def test_native_raid_mark_command_sets_and_moves_one_guid(self):
        from native_1392_machine import ReviewMachine
        machine = ReviewMachine(self.image)
        server = machine.scratch + 0x10000
        packet = server + 0x9000
        guid = 0xf00100abc
        machine.write(server + self.values["SERVER_ROLE"], "I", 3)
        command = dict(self.values["ADDRESSES"])[0x5873b00]
        marks = server + self.values["SERVER_RAID_MARKS"]
        for icon in (3, 7):
            machine.write(packet, "BQ", icon, guid)
            machine.call(command, server, 0, 0, self.values["RAID_MARKER_COMMAND"], packet, 9)
            self.assertEqual(machine.read(marks, "8Q"), tuple(guid if i == icon else 0 for i in range(8)))

    def test_private_context_swap_across_engine_versions(self):
        from compare_official import Image
        from native_1392_machine import ReviewMachine
        old_path = os.environ.get("AZSX_REVIEWED_IMAGE_138")
        if not old_path:
            self.skipTest("Set AZSX_REVIEWED_IMAGE_138 for the cross-version context swap")
        old = Image(Path(old_path))
        self.assertEqual(old.sha, "eb5fd1c1b96e6180fcf5f3d685d90a61471af28d22e5f060c02b7b7663fbf79e")
        old_values = bindings("0.138.0")

        def swap_fields(image, values):
            machine = ReviewMachine(image)
            game = machine.scratch + 0x20000
            peer = game + 0x4000
            size = values["CONTEXT_SIZE"]
            observations = []
            for shift in (0, 8):
                # Two passes encode the origin of every byte, including padding.
                machine.cpu.mem_write(game, bytes((i >> shift) & 255 for i in range(0x1100)))
                machine.cpu.mem_write(peer, bytes(((0x4000 + i) >> shift) & 255 for i in range(size)))
                machine.call(dict(values["ADDRESSES"])[0x5920ed0], game, peer)
                observations.append((bytes(machine.cpu.mem_read(game, 0x1100)),
                                     bytes(machine.cpu.mem_read(peer, size))))
            host, private = ([lo | hi << 8 for lo, hi in zip(observations[0][k], observations[1][k])]
                             for k in (0, 1))
            for index, source in enumerate(private[:-3]):
                self.assertLess(source, len(host))
                self.assertEqual(host[source], index + 0x4000)
            self.assertEqual(private[-3:], list(range(0x4000 + size - 3, 0x4000 + size)))
            return private[:-3]

        def offsets(ranges):
            return {a: start + a - first for first, last, start in ranges for a in range(first, last + 1)}

        before = swap_fields(old, old_values)
        after = swap_fields(self.image, self.values)
        self.assertEqual(len(before), len(after))
        self.assertEqual(len(after), 2901)
        self.assertEqual(self.values["ACTOR_RESOLVED_FIELDS"], old_values["ACTOR_RESOLVED_FIELDS"])
        translation = dict(zip(before, after))
        previous = offsets(old_values["GAME_RESOLVED_FIELDS"])
        proposed = offsets(self.values["GAME_RESOLVED_FIELDS"])
        for canonical, address in previous.items():
            if address in translation:
                self.assertEqual(proposed.get(canonical), translation[address], hex(canonical))

    def test_game_allocation_headers_survive_the_engine_update(self):
        from compare_official import Image
        from native_1392_machine import ReviewMachine
        old_path = os.environ.get("AZSX_REVIEWED_IMAGE_138")
        if not old_path:
            self.skipTest("Set AZSX_REVIEWED_IMAGE_138 for the cross-image Game destructor")
        old = Image(Path(old_path))
        self.assertEqual(old.sha, "eb5fd1c1b96e6180fcf5f3d685d90a61471af28d22e5f060c02b7b7663fbf79e")
        # 0.138.0 capacity offsets, element sizes and alignments. Test each vector
        # alone so two neighboring allocations of the same size cannot mask a swap.
        vectors = (
            (0x640, 520, 8), (0x670, 128, 8), (0x688, 64, 8), (0x6a0, 32, 8),
            (0x6e8, 72, 8), (0x700, 32, 8), (0x718, 24, 8), (0x748, 40, 8),
            (0x968, 8, 4), (0xb28, 16, 8), (0xb88, 0xb58, 8), (0xba0, 16, 8),
            (0xbb8, 16, 8), (0xbd0, 72, 8), (0xbe8, 80, 8), (0xc48, 16, 4),
            (0xc60, 4, 4), (0xc78, 56, 8), (0xc90, 40, 8), (0xca8, 32, 8),
            (0xcc0, 32, 8), (0xcd8, 8, 4), (0xcf0, 40, 8), (0xd08, 16, 8),
            (0xd20, 24, 8),
        )
        base_ranges = bindings("0.138.0")["GAME_RESOLVED_FIELDS"]
        proposed = {k: native + k - first for first, last, native in self.values["GAME_RESOLVED_FIELDS"]
                    for k in range(first, last + 1)}
        for image, destructor, arc_offset, insertion in (
            (old, 0x5ba5810, 0xde0, 0),
            (self.image, 0x5dbab50, 0xdf8, 24),
        ):
            machine = ReviewMachine(image)
            game = machine.scratch + 0x10000
            arc = game + 0x8000
            for offset, size, align in vectors:
                with self.subTest(image=image.sha[:8], offset=hex(offset)):
                    native = offset + (insertion if offset >= 0xcf0 else 0)
                    machine.cpu.mem_write(game, bytes(0x1400))
                    machine.write(game + arc_offset, "Q", arc)
                    machine.write(arc, "Q", 2)
                    ptr = machine.allocate(size * 3, align)
                    machine.write(game + native, "3Q", 3, ptr, 0)
                    machine.call(destructor, game)
                    self.assertEqual(machine.allocations, {})
                    self.assertEqual(machine.read(arc, "Q"), (1,))
                    if image is self.image:
                        for first, last, start in base_ranges:
                            for old_byte in range(max(start, offset), min(start + last - first + 1, offset + 16)):
                                self.assertEqual(proposed[first + old_byte - start], native + old_byte - offset)

    def test_host_fields_in_native_game_constructors(self):
        from compare_official import Image
        from native_1392_machine import ReviewMachine
        old_path = os.environ.get("AZSX_REVIEWED_IMAGE_138")
        if not old_path:
            self.skipTest("Set AZSX_REVIEWED_IMAGE_138 for both native constructors")
        old = Image(Path(old_path))
        previous = bindings("0.138.0")

        def construct(image, site, size, fill, seed):
            machine = ReviewMachine(image)
            game = machine.scratch + 0x10000
            args = game + 0x8000
            machine.cpu.mem_write(machine.stack - 0x18000, bytes([fill]) * 0x18000)
            # Seed individual input bytes of the settings block copied by both constructors.
            settings = (*range(0x32d, 0x330), *range(0x348, 0x351), *range(0x354, 0x358))
            for offset in settings:
                machine.write(args + offset - 0x238, "B", (offset + fill) & 255)
            machine.write(args + 0x138, "Q", seed)
            machine.call(site, game, args, args + 0x2000, args + 0x4000, args + 0x6000)
            value = bytearray(machine.cpu.mem_read(game, size))
            for offset in settings:
                self.assertEqual(value[offset], (offset + fill) & 255)
            # These slots hold native image references, which relocate with each build.
            # Normalize only their addresses, leaving adjacent fields untouched.
            for offset in (8, 0x18, 0x4e0):
                pointer = int.from_bytes(value[offset:offset+8], "little")
                self.assertTrue(machine.base <= pointer < machine.end)
                value[offset:offset+8] = bytes(8)
            return value

        def normalized(value, ranges):
            return {i: value[start + i - first] for first, last, start in ranges
                    for i in range(first, last + 1)}

        for fill, seed in ((0xa5, 0), (0x5a, 0), (0x5a, 0x765432100)):
            with self.subTest(fill=fill, seed=seed):
                before = construct(old, 0x5cee940, 0xf50, fill, seed)
                after = construct(self.image, 0x5efe1f0, 0xf68, fill, seed)
                self.assertEqual(before, after[:0xcf0] + after[0xd08:])
                self.assertEqual(normalized(before, previous["GAME_RESOLVED_FIELDS"]),
                                 normalized(after, self.values["GAME_RESOLVED_FIELDS"]))

                for table in ("FIELD117_RESOLVED_FIELDS", "FIELD1201_RESOLVED_FIELDS"):
                    original = normalized(before, previous[table])
                    candidate = normalized(after, self.values[table])
                    self.assertEqual(candidate, original)

        # An adjacent vector was inserted at the old header. Execute both native
        # append blocks with a nonempty buffer and verify row stride and length.
        from unicorn import x86_const as reg
        for image, start, end, header, frame_source in (
            (old, 0x5cfc354, 0x5cfc3c0, 0xcf0, 0x900),
            (self.image, 0x5f0c8a4, 0x5f0c910, 0xd08, 0x930),
        ):
            machine = ReviewMachine(image)
            game = machine.scratch + 0x10000
            frame, rows = game + 0x4000, game + 0x8000
            machine.write(game + header, "3Q", 2, rows, 1)
            machine.write(frame + frame_source, "Q", 0x123456789abcdef)
            machine.cpu.mem_write(rows, bytes([0x55]) * 80)
            machine.cpu.reg_write(reg.UC_X86_REG_R14, game)
            machine.cpu.reg_write(reg.UC_X86_REG_RBP, frame)
            machine.cpu.emu_start(machine.base + start, machine.base + end, count=100)
            self.assertEqual(machine.cpu.reg_read(reg.UC_X86_REG_RIP), machine.base + end)
            self.assertEqual(machine.read(game + header, "3Q"), (2, rows, 2))
            self.assertEqual(bytes(machine.cpu.mem_read(rows, 40)), bytes([0x55]) * 40)
            self.assertEqual(machine.read(rows + 40, "Q"), (0x123456789abcdef,))

        # The empty variant leaves these three floats unwritten in the fixture.
        # Check the live payload operand in each version, independently of defaults.
        from compare_official import MD
        for old_site, new_site, old_offset, new_offset, operation in (
            (0x5cfa766, 0x5f0acb6, 0xd9c, 0xdb4, "subss"),
            (0x5c9f77f, 0x5eaf09f, 0xdc4, 0xddc, "movss"),
            (0x5cf144e, 0x5f00d0e, 0xddc, 0xdf4, "addss"),
        ):
            for image, site, offset in ((old, old_site, old_offset), (self.image, new_site, new_offset)):
                inst = next(MD.disasm(image.pe.get_data(site, 16), site))
                self.assertEqual(inst.mnemonic, operation)
                operand, = [p for p in inst.operands if p.type == 3]
                self.assertEqual((inst.reg_name(operand.mem.base), operand.mem.disp, operand.size),
                                 ("rsi", offset, 4))

    def test_sink_copies_borrowed_ui_text_into_native_ring(self):
        from native_1392_machine import ReviewMachine
        machine = ReviewMachine(self.image)
        sink = machine.scratch + 0x10000
        ring, slots, message = (sink + n for n in (0x1000, 0x2000, 0x3000))
        machine.write(sink + 0x10, "Q", ring)
        machine.write(ring + 0x18, "4Q", 4, slots, 3, 1)
        untouched = bytes(range(24))
        machine.cpu.mem_write(slots + 3 * 24, untouched)
        payload = b'{"type":"unpaused"}'
        machine.cpu.mem_write(message, payload + b"tail")
        sink_event = dict(self.values["ADDRESSES"])[0x586d550]
        machine.call(sink_event, sink, message, len(payload))
        machine.cpu.mem_write(message, bytes(len(payload)))
        self.assertEqual(machine.buffer(slots), payload)
        self.assertEqual(machine.read(ring + 0x28, "2Q"), (3, 2))
        self.assertEqual(bytes(machine.cpu.mem_read(slots + 3 * 24, 24)), untouched)
        self.assertEqual(machine.read(ring + 0x10, "B"), (0,))

    def test_reroll_charge_before_native_choice_generation(self):
        from native_1392_machine import ReviewMachine, reg
        from unicorn import UC_HOOK_CODE
        machine = ReviewMachine(self.image)
        game = machine.scratch + 0x10000
        machine.write(game + 0xf63, "B", 2)
        machine.write(game + 0xf61, "B", 0)
        entries = dict(self.values["EXTENSION_ADDRESSES"])
        reroll = entries[self.values["REWARD_REROLL_RVA"]]
        handoffs = []

        def reached_generator(cpu, addr, size, user):
            handoffs.append(tuple(cpu.reg_read(r) for r in (
                reg.UC_X86_REG_RCX, reg.UC_X86_REG_RDX, reg.UC_X86_REG_R8, reg.UC_X86_REG_RSP)))
            cpu.reg_write(reg.UC_X86_REG_RIP, machine.stop)

        start = machine.base + 0x5f6c500
        machine.cpu.hook_add(UC_HOOK_CODE, reached_generator, begin=start, end=start)
        for gold, free, next_gold, next_free, paid, generates in (
            (99, 0, 99, 0, 0, False),
            (350, 1, 350, 0, 0, True),
            (350, 0, 250, 0, 1, True),
        ):
            with self.subTest(gold=gold, free=free):
                handoffs.clear()
                machine.write(game + 0x578, "I", gold)
                machine.write(game + 0x62c, "2I", free, 0)
                machine.call(reroll, game)
                self.assertEqual(machine.read(game + 0x578, "I"), (next_gold,))
                self.assertEqual(machine.read(game + 0x62c, "2I"), (next_free, paid))
                self.assertEqual(handoffs, [(game, 0, 0, machine.stack)] if generates else [])

    def test_native_single_field_replacement_preserves_adjacent_key(self):
        from native_1392_machine import ReviewMachine
        machine = ReviewMachine(self.image)
        fields = machine.scratch + 0x10000
        writer = self.values["EVENT_UNIT_FIELDS_SET"][0]
        self.assertEqual(writer, dict(self.values["ADDRESSES"])[0x58092f0])
        for key, value, existed in ((0x498, 10000, 0), (0x499, 17, 0), (0x498, 20000, 1)):
            self.assertEqual(machine.call(writer, fields, key, value), existed)
        node, height, count = machine.read(fields, "3Q")
        self.assertEqual((height, count), (0, 2))
        self.assertEqual(machine.read(node + 0x38, "2H"), (0x498, 0x499))
        self.assertEqual(machine.read(node + 8, "2I"), (20000, 17))

    def test_goblin_compositor_keeps_its_entity_across_both_callbacks(self):
        code = self.body(0x7edef0)
        c = self.values
        for name in ("GOBLIN_OUTFIT_METADATA", "GOBLIN_OUTFIT_KEY"):
            address, guard = c[name]
            self.assertEqual(self.image.pe.get_data(address, len(guard)), bytes(guard))
        self.assertIsNone(c["GOBLIN_OUTFIT_ENTITY"])
        self.assertEqual(code[0x7ee404].op_str, "r12, qword ptr [r8 + rdx*8]")
        self.assertEqual(code[0x7ee6c8].op_str, "r12, qword ptr [r8 + rdx]")
        self.assertEqual(code[0x7ef379].op_str, "r9, r12")
        metadata = code[c["GOBLIN_OUTFIT_METADATA"][0]]
        key = code[c["GOBLIN_OUTFIT_KEY"][0]]
        self.assertEqual(metadata.operands[1].mem.disp, c["GOBLIN_OUTFIT_ATLAS"])
        self.assertEqual(key.op_str, "byte ptr [rbp + 0xcd0], 1")
        for ins in code.values():
            if metadata.address <= ins.address <= key.address:
                self.assertFalse({"r12", "r12d", "r12w", "r12b"} &
                                 {ins.reg_name(r) for r in ins.regs_access()[1]})
        self.assertEqual(code[0x7ef4c8].operands[1].mem.disp, c["GOBLIN_OUTFIT_APPEARANCE"])
        self.assertEqual(code[0x7ef5cf].operands[1].mem.disp, c["GOBLIN_OUTFIT_ARMOR_KEY"])
        self.assertEqual(code[0x7ef5d6].operands[1].mem.disp, c["GOBLIN_OUTFIT_MODELS"])
        self.assertEqual(code[0x7ef53b].operands[0].mem.disp,
                         c["GOBLIN_OUTFIT_ARMOR_KEY"] + c["GOBLIN_OUTFIT_KEY_RACE_OFFSET"])
        self.assertEqual(code[0x7ef55b].operands[0].mem.disp + 2,
                         c["GOBLIN_OUTFIT_ARMOR_KEY"] + c["GOBLIN_OUTFIT_KEY_SIZE"])
        self.assertEqual(code[0x7ef616].address + code[0x7ef616].size, c["GOBLIN_OUTFIT_SKIN_CALLER"])
