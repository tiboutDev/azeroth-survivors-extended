"""Exact-image 0.140.0 contracts; artifact activation and live evidence remain separate."""
import os
import json
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools/dev"))
PINNED = "27c8df53c87e38a98d474bf7106d2444e11e7634ca70dbffc5c2b0aecb894fd9"


class Native1400Review(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        path = os.environ.get("AZSX_REVIEW_IMAGE_1400")
        if not path:
            raise unittest.SkipTest("Set AZSX_REVIEW_IMAGE_1400 to the private review executable")
        from compare_official import Image
        cls.image = Image(Path(path))
        from binding_registry import parse_literal
        manifest = json.loads((ROOT / "compat/clients/0.140.0.json").read_text())
        if manifest["sha256"] != PINNED:
            raise AssertionError("Wrong 0.140.0 review manifest")
        cls.values = {row["name"]: parse_literal(row["value"]) for row in manifest["constants"]}
        if cls.image.sha != PINNED:
            raise AssertionError("Wrong 0.140.0 review image")

    @classmethod
    def tearDownClass(cls):
        cls.image.ins.cache_clear()
        cls.image.tokens.cache_clear()
        del cls.image
        import gc
        gc.collect()

    def tearDown(self):
        import gc
        gc.collect()

    def field(self, canonical, namespace="GAME_RESOLVED_FIELDS"):
        hits = [native + canonical - start for start, end, native in self.values[namespace]
                if start <= canonical <= end]
        self.assertEqual(len(hits), 1, (namespace, hex(canonical)))
        return hits[0]

    def test_guardian_reward_context_reads_selected_index_and_none_sentinel(self):
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        m = ReviewMachine(self.image)
        enemy, frame = m.scratch + 0x20000, m.scratch + 0x24000
        self.assertEqual(self.values["ENEMY_GUARDIAN_INDEX"], 0x78)
        # Actual native damage/death owner snapshots this optional guardian
        # context before its reward branches; the old +0x58 is unrelated.
        for guardian in (3, 0xffffffffffffffff):
            m.write(enemy + 0x58, "Q", 999)
            m.write(enemy + 0x78, "Q", guardian)
            m.write(enemy + 0xb0, "Q", 12345)
            m.write(frame + 0x320, "Q", 0)
            m.cpu.reg_write(reg.UC_X86_REG_RCX, enemy)
            m.cpu.reg_write(reg.UC_X86_REG_RBP, frame)
            m.cpu.emu_start(m.base + 0x5f448cc, m.base + 0x5f448f8, count=20)
            self.assertEqual(m.read(frame + 0x318, "Q")[0], guardian)
            self.assertEqual(m.read(frame + 0x320, "Q")[0], 0 if guardian == 0xffffffffffffffff else 12345)

    def test_native_xp_receipt_append_uses_the_reviewed_queue_and_preserves_prior_rows(self):
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        self.assertEqual(self.field(0x990), 0xce0)
        for count in (0, 1):
            m = ReviewMachine(self.image)
            game, records = m.scratch + 0x10000, m.scratch + 0x20000
            m.write(game + 0xce0, "3Q", 2, records, count)
            m.write(game + 0xfc0, "Q", 2)
            m.cpu.mem_write(records, bytes([0xa5]) * 32)
            m.cpu.reg_write(reg.UC_X86_REG_RSI, game)
            m.cpu.reg_write(reg.UC_X86_REG_R15, 37)
            m.cpu.emu_start(m.base + 0x5fd2f7a, m.base + 0x5fd2fbf, count=30)
            self.assertEqual(m.read(game + 0xce0, "3Q"), (2, records, count + 1))
            self.assertEqual(m.read(records + count * 16, "QI"), (2, 37))
            if count:
                self.assertEqual(bytes(m.cpu.mem_read(records, 16)), bytes([0xa5]) * 16)

    def test_native_cache_mutations_match_host_and_transferred_ally_fields(self):
        from native_1392_machine import ReviewMachine
        self.assertEqual([self.field(k) for k in (0xbb4, 0xbf4, 0x7b0)], [0x1028, 0x1070, 0xa9c])
        self.assertEqual([self.field(k, "ACTOR_RESOLVED_FIELDS") for k in (0x92c, 0x96c, 0x6f0)], [0xba8, 0xbf0, 0x944])
        for gold, cost, granted, remaining, next_cost in ((19, 20, 0, 19, 20), (20, 20, 1, 0, 31), (100, 40, 1, 60, 57)):
            m = ReviewMachine(self.image)
            game, actor = m.scratch + 0x10000, m.scratch + 0x14000
            m.write(game + 0x5f0, "I", gold)
            m.write(game + 0x1070, "I", cost)
            m.write(game + 0xa98, "I", 42)
            self.assertEqual(m.call(0x5f60ac0, game) & 255, granted)
            self.assertEqual(m.read(game + 0x5f0, "I"), (remaining,))
            self.assertEqual(m.read(game + 0xa98, "I"), (42,))
            self.assertEqual(m.read(game + 0x1070, "I"), (next_cost,))
            self.assertEqual(m.read(game + 0x1028, "I"), (granted,))
            self.assertEqual(m.read(game + 0xa9c, "I"), (granted,))
            m.call(0x5f614f0, game, actor)
            self.assertEqual(m.read(actor + 0xba8, "I"), (granted,))
            self.assertEqual(m.read(actor + 0xbf0, "I"), (next_cost,))
            self.assertEqual(m.read(actor + 0x944, "I"), (granted,))

    def test_native_character_constructor_retains_gender_in_both_actor_namespaces(self):
        from native_1392_machine import CatalogMachine
        m = CatalogMachine(self.image)
        game, profile, actor = m.scratch + 0x10000, m.scratch + 0x20000, m.scratch + 0x30000
        self.assertEqual(self.field(0x32e), 0x44e)
        self.assertEqual(self.field(0x32e, "ACTOR_RESOLVED_FIELDS"), 0x44e)
        for sex in (0, 1):
            m.write(profile + 0x1a4, "3B", 1, 1, sex)
            m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
            self.assertEqual(m.read(game + 0x44c, "3B"), (1, 1, sex))
            m.call(0x5f614f0, game, actor)
            self.assertEqual(m.read(actor + 0x44c, "3B"), (1, 1, sex))

    def test_native_set_stat_variant_keeps_its_kind_and_controlled_damage_value(self):
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        m = ReviewMachine(self.image)
        out = m.scratch + 0x10000
        for class_id in (2, 3, 4, 5, 8, 9, 11):
            m.cpu.reg_write(reg.UC_X86_REG_XMM3, 0x3f800000)
            m.call(0x5e32350, out, class_id, 5)
            capacity, pointer, count = m.read(out, "3Q")
            stats = [pointer + i * 64 for i in range(count) if m.read(pointer + i * 64, "I")[0] == 72]
            self.assertEqual(len(stats), 1)
            self.assertEqual(m.read(stats[0] + 8, "I"), (18,))
            self.assertAlmostEqual(m.read(stats[0] + 4, "f")[0], .6 if class_id == 9 else 1., places=6)
            m.call(0x21d0, pointer, capacity * 64, 8)

    def test_native_minimap_projection_reads_the_selected_borrowed_context_fields(self):
        import struct
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        self.assertEqual(self.values["MINIMAP_STATE_FIELDS"], [0x6768, 0x677c, 0x676c, 0x6770, 0x6760, 0x6764])
        m = ReviewMachine(self.image)
        root = m.scratch + 0x10000
        for visible, indoor, expected in ((0, 0, 0.), (1, 0, 30. / (14. * 16.666666 * 2.)), (1, 1, .1)):
            m.write(root + 0x6780, "B", visible)
            m.write(root + 0x6794, "B", indoor)
            m.write(root + 0x6784, "2f", 10., 20.)
            m.write(root + 0x6778, "2f", 40., 50.)
            m.write(root + 0x678c, "2I", 0, 0)
            m.cpu.reg_write(reg.UC_X86_REG_RAX, root)
            m.cpu.emu_start(m.base + 0x2ee1c22, m.base + 0x2ee1cf8, count=100)
            scalar = lambda register: struct.unpack("<d", m.cpu.reg_read(register).to_bytes(16, "little")[:8])[0]
            self.assertAlmostEqual(scalar(reg.UC_X86_REG_XMM7), expected, places=7)
            self.assertAlmostEqual(scalar(reg.UC_X86_REG_XMM6), -expected, places=7)

    def test_embedded_native_world_anchor_api_drives_azsxui_with_scoped_downed_members(self):
        from test_official_party_lua import LuaRuntime, STUB
        if LuaRuntime is None:
            self.skipTest("Required marker behavior uses Lua 5.1")
        # Read only this executable's embedded API implementation. No archives,
        # installed addons, MPQs or native assets are extracted or modified.
        raw = self.image.raw
        start = raw.index(b"function AZS_GetWorldAnchors(...)")
        end = raw.index(b"function AZS_SetHudShown(", start)
        self.assertLess(end - start, 10000)
        lua = LuaRuntime()
        lua.execute(STUB)
        lua.execute(r'''
Minimap=CreateFrame("Frame","Minimap",UIParent);Minimap:SetWidth(200);Minimap:SetHeight(200)
function Minimap:GetZoom()return 0 end
WorldFrame=CreateFrame("Frame","WorldFrame",UIParent)
map={identity="1:11",x=10,y=20,radii={100,80,60,40,20,10},pingX=0,pingY=0}
function AZSX_GetAPIVersion()return 1,0 end
function AZSX_GetState()return {available=true,data={minimap=map,markers={}},texts={}}end
AZS_RUN={chest=250};AZS_CHESTS={};AZS_ZONE_UPDATED=true;AZS_ZONE={map=2}
player={runId="R",mapId=2,spaceId="S",x=10,y=20,facing=0}
function AZS_GetPlayerPosition()return player end
function AZS_CTErr(code,message)return nil,{code=code,message=message}end
ensures=0;function AZS_WAEnsure()ensures=ensures+1 end
function AZS_CTSeat()return 3 end
AZS_DOWNED={{id=12,name="Ally",state="down",anchorStatus="ready",anchorId="A",x=17,y=30,z=2,spaceId="S"},
 {id=13,name="Other space",state="down",anchorStatus="ready",anchorId="B",x=17,y=30,z=2,spaceId="old"},
 {id=14,name="Alive",state="up",anchorStatus="ready",anchorId="C",x=17,y=30,z=2,spaceId="S"}}
function AZS_NativeWorldAnchorQuery(query,cursor,limit,stamp)
 assert(query.kinds[1]=="cache" and query.states[1]=="present" and cursor==nil and limit==128 and stamp==now)
 return {runId="R",mapId=2,entries={{cacheId=4,kind="cache",x=17,y=30,positionAvailable=true,spaceId="S"},
 {cacheId=5,kind="cache",x=20,y=40,positionAvailable=true,spaceId="old"}}}
end
function AZS_NativeWorldAnchorProject(requests,stamp,seat)
 assert(table.getn(requests)==1 and requests[1].anchorId=="A" and requests[1].attachment=="position")
 assert(requests[1].offset.z==2 and stamp==now and seat==3)
 return {frameCurrent=true,entries={{anchorId="A",state="onScreen",x=20,y=30,playerDistance=12.3}}}
end
''')
        lua.execute(raw[start:end].decode())
        lua.execute((ROOT / "addons/AZSXUI/Minimap.lua").read_text())
        lua.execute('''
local friends=AZS_GetDownedPartyMembers()
assert(table.getn(friends)==2 and friends[1].positionAvailable and friends[1].anchorId=="A")
assert(friends[1].runId=="R" and friends[1].mapId==2 and friends[1].spaceId=="S")
assert(not friends[2].positionAvailable)
friends[1].name="Mutated copy";assert(AZS_DOWNED[1].name=="Ally")
now=now+.2;objects.AZSXUIMinimapController.scripts.OnUpdate()
local pins=AZSXUI_MinimapReadout();assert(table.getn(pins)==1 and pins[1].event==4)
assert(table.getn(AZS_WA_HANDLERS)==1);AZS_WorldAnchorFrame()
local pin=objects.AZSXUIReviveAnchor1;assert(pin:IsShown() and pin.label:GetText()=="Ally · 12 yd")
AZS_DOWNED={};now=now+.2;objects.AZSXUIMinimapController.scripts.OnUpdate()
assert(not pin:IsShown() and table.getn(AZS_WA_HANDLERS)==0)
AZS_ZONE.map=99;local members,err=AZS_GetDownedPartyMembers();assert(not members and err.code=="NOT_READY")
local members,err=AZS_GetDownedPartyMembers(1);assert(not members and err.code=="INVALID_ARGUMENT")
assert(ensures>=3)
''')

    def test_native_pause_builder_keeps_the_retained_presentation_field_extents(self):
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        m = ReviewMachine(self.image)
        frame = m.scratch + 0x10000
        m.cpu.mem_write(frame, bytes([0xa5]) * 0x800)
        m.cpu.reg_write(reg.UC_X86_REG_RBP, frame)
        # Native title style writes size, weight and smoothing into this exact
        # stack payload. AZSX's retained title-size override targets size only.
        m.cpu.emu_start(m.base + 0x392217, m.base + 0x392231, count=10)
        self.assertEqual(m.read(frame + 0x470, "fHB"), (46., 400, 1))
        self.assertEqual(bytes(m.cpu.mem_read(frame + 0x477, 9)), bytes([0xa5]) * 9)
        # Native menu presentation resets the byte immediately before the
        # visibility hook; this byte is distinct from the game phase.
        m.cpu.emu_start(m.base + 0x124d31, m.base + 0x124d38, count=2)
        self.assertEqual(m.read(frame + 0x2fe, "B"), (0,))
        self.assertEqual(bytes(m.cpu.mem_read(frame + 0x2ff, 9)), bytes([0xa5]) * 9)

    def test_native_host_submission_queues_only_the_requested_named_lobby(self):
        from native_1392_machine import ReviewMachine
        m = ReviewMachine(self.image)
        resource = m.scratch + 0x10000
        writer, shared, rows, hero, text = [resource + n for n in (0x400, 0x500, 0x800, 0x1000, 0x1200)]
        self.assertEqual(self.values["LOBBY_NAME"], 0xd0)
        for header, value in ((resource + 0xd0, b"[AZSX] Trial"),
                              (resource + 0xe8, b"private"), (resource + 0x100, b"fixture"),
                              (hero, b"Test hero"), (hero + 24, b"Test class")):
            m.cpu.mem_write(text, value)
            m.write(header, "3Q", len(value), text, len(value))
            text += 32
        m.write(resource + 0x1fd, "B", 5)
        m.write(writer, "Q", shared)
        m.write(shared + 0x38, "3Q", 4, rows, 0)
        m.call(self.values["LOBBY_SUBMIT"], resource, writer, hero, 0)
        self.assertEqual(m.read(shared + 0x48, "Q"), (0,))
        m.call(self.values["LOBBY_SUBMIT"], resource, writer, hero, 1)
        self.assertEqual(m.read(shared + 0x48, "Q"), (2,))
        self.assertEqual(m.read(rows, "I"), (12,))
        self.assertEqual(m.buffer(rows + 8), b"Test hero")
        room = rows + 0xe8
        self.assertEqual(m.read(room, "I"), (23,))
        self.assertEqual([m.buffer(room + n) for n in (8, 32, 56)], [b"[AZSX] Trial", b"private", b"fixture"])
        self.assertEqual(m.read(room + 81, "B"), (5,))

    def test_native_look_slots_and_generated_weapons_keep_their_catalog_boundaries(self):
        from native_1392_machine import CatalogMachine
        m = CatalogMachine(self.image)
        out, looks, equipped, visible = [m.scratch + n for n in (0x10000, 0x12000, 0x14000, 0x16000)]
        for class_id in (1, 2, 7):
            for hand in (8, 9):
                for tier in range(8):
                    entry = 100000 + class_id * 1000 + hand * 100 + tier * 10
                    m.cpu.mem_write(out, bytes([0xa7]) * 128)
                    self.assertEqual(m.call(0x5e2ca90, out, entry), out)
                    capacity, pointer, length = m.read(out, "3Q")
                    if capacity != 0xffffffffffffffff:
                        self.assertGreater(length, 0)
                        self.assertEqual(m.read(out + 0x50, "I"), (entry,))
                    self.assertEqual(bytes(m.cpu.mem_read(out + 106, 22)), bytes([0xa7]) * 22)
        for slot in range(19):
            m.cpu.mem_write(looks, bytes(19 * 8))
            m.write(looks + slot * 8, "2I", 1, 101800)
            self.assertEqual(m.call(0x224f80, looks, 101800) & 255, int(slot >= 6))
            self.assertEqual(m.call(0x224f80, looks, 101801) & 255, 0)
        m.write(equipped + 60, "I", 101800)
        m.cpu.mem_write(looks, bytes(19 * 8))
        self.assertEqual(m.call(0x5e2c600, visible, 1, 1, equipped, looks), visible)
        self.assertEqual(m.read(visible + 60, "I"), (101800,))

    def test_native_character_look_result_keeps_the_selected_profile_register(self):
        from native_1392_machine import CatalogMachine
        from unicorn import UC_HOOK_CODE, x86_const as reg
        m = CatalogMachine(self.image)
        game, profile = m.scratch + 0x10000, m.scratch + 0x18000
        self.assertEqual(self.values["WEAPON_RUN_PROFILE_REGISTER"], 15)
        observed = []
        def observe(cpu, address, size, user):
            observed.append((cpu.reg_read(reg.UC_X86_REG_R15), cpu.reg_read(reg.UC_X86_REG_RAX)))
        m.cpu.hook_add(UC_HOOK_CODE, observe, begin=m.base + 0x5faaa7a, end=m.base + 0x5faaa7a)
        m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
        self.assertEqual(len(observed), 1)
        self.assertEqual(observed[0][0], profile)
        self.assertGreaterEqual(observed[0][1], m.scratch)

    def test_private_field_namespaces_follow_the_actual_native_swap(self):
        from native_1392_machine import ReviewMachine
        m = ReviewMachine(self.image)
        game, actor = m.scratch + 0x20000, m.scratch + 0x24000
        raw = bytes((i * 73 + i // 256) & 255 for i in range(0x1200))
        m.cpu.mem_write(game, raw)
        m.cpu.mem_write(actor, bytes(0xc08))
        m.call(0x5f614f0, game, actor)
        # Shared prefix keys coincide; later private keys have their own
        # namespace. Reusing Game's canonical choice/stat/perk key is unsafe.
        for host, private, size in ((0x398, 0x398, 24), (0x480, 0x480, 8), (0x4fc, 0x4fc, 4),
                                    (0x660, 0x5b8, 24), (0x7ac, 0x6ec, 4), (0x800, 0x728, 24),
                                    (0x830, 0x758, 24), (0xb20, 0x8c0, 24)):
            source = self.field(host)
            target = self.field(private, "ACTOR_RESOLVED_FIELDS")
            self.assertEqual(bytes(m.cpu.mem_read(actor + target, size)), raw[source:source + size])
        header = self.values["ACTOR_PERKS_HEADER"]
        self.assertEqual(header, 0x9b0)
        self.assertEqual(bytes(m.cpu.mem_read(actor + header, 24)), raw[self.field(0x830):self.field(0x830) + 24])

    def test_manifest_spell_and_talent_contacts_keep_native_identity(self):
        import struct
        image = self.image
        def text(at):
            pointer, length = struct.unpack("<2Q", image.pe.get_data(at, 16))
            self.assertTrue(0 < length < 256)
            return image.pe.get_data(pointer - image.pe.OPTIONAL_HEADER.ImageBase, length).decode()
        addresses = dict(self.values["ADDRESSES"])
        contacts = (
            (0x6cc4850, "battle_shout", 6673), (0x6cc4c10, "holy_shock", 25912),
            (0x6cc4df0, "holy_light", 635), (0x6cc5b88, "holy_nova", 15237),
            (0x6cc5c00, "pw_shield", 17), (0x6cc5c78, "renew", 139),
            (0x6cc61a0, "lightning_shield", 324), (0x6cc6380, "healing_wave", 331),
            (0x6cc72f8, "rejuvenation", 774), (0x6cc7370, "regrowth", 8936),
            (0x6cc73e8, "thorns", 467), (0x711ca10, "mortal_strike", 12294),
            (0x711d460, "serpent_sting", 1978), (0x711d7a8, "hunters_mark", 1130),
            (0x711d988, "rupture", 1943), (0x711e360, "earth_shock", 8042),
            (0x711e3d8, "flame_shock", 8050), (0x711edb0, "polymorph", 118),
            (0x711ef90, "curse_of_agony", 980),
        )
        for canonical, key, spell in contacts:
            with self.subTest(key=key):
                definition = addresses[canonical]
                self.assertEqual(text(definition), key)
                self.assertEqual(struct.unpack("<I", image.pe.get_data(definition + 64, 4)), (spell,))
                self.assertTrue(text(definition + 16))
                self.assertTrue(text(definition + 32))
        barrier = dict(self.values["EXTENSION_ADDRESSES"])[0x72a0ca0]
        self.assertEqual(text(barrier), "ice_barrier")
        self.assertEqual(struct.unpack("<I", image.pe.get_data(barrier + 64, 4)), (11426,))
        perk_definitions = ((0x701a618, "rapture", 0, 12), (0x712e628, "hammer_righteous", 3, 15),
                            (0x7136740, "divine_aegis", 1, 15), (0x7146510, "swiftmend", 0, 12),
                            (0x71466b0, "tranquility", 0, 33), (0x7146900, "tree_of_life", 0, 32))
        talents = {key: header for header, key in self.values["COOP_TALENT_KEYS"]}
        for canonical, key, index, tag in perk_definitions:
            pointer, count = struct.unpack("<2Q", image.pe.get_data(talents[key] + 48, 16))
            self.assertLess(index, count)
            definition = pointer - image.pe.OPTIONAL_HEADER.ImageBase + index * 64
            self.assertEqual(addresses[canonical], definition)
            self.assertEqual(struct.unpack("<I", image.pe.get_data(definition, 4)), (tag,))
        self.assertEqual(len(self.values["COOP_TALENT_KEYS"]), 13)
        for header, key in self.values["COOP_TALENT_KEYS"]:
            self.assertEqual(text(header), key)
            self.assertTrue(text(header + 16))

    def test_xp_guard_manifest_preserves_the_remote_receipt_reload(self):
        image = self.image
        guards = self.values["DOWNED_XP_GUARDS"]
        expected = ((0x5f31741, 0x5f3174f, "r13", 4), (0x5e735e2, 0x5e735f0, "r9", 4),
                    (0x5f32a8d, 0x5f32a9b, "r13", 1), (0x5e716f0, 0x5e716fd, "rsi", 1),
                    (0x5fad6c3, 0x5fad6d0, "rsi", 1), (0x5f32004, 0x5f3200e, "r13", 4))
        self.assertEqual(len(guards), 7)
        for row, (entry, resume, actor, phase) in zip(guards, expected):
            self.assertEqual(row[:2], (entry, resume))
            body = {op.address: op for op in image.ins(image.owner(entry))}
            check = body[entry]
            self.assertEqual((check.mnemonic, check.op_str), ("cmp", f"byte ptr [{actor} + 0x10d1], {phase}"))
            branch = body[entry + check.size]
            self.assertEqual(branch.mnemonic, "je" if phase == 4 else "jne")
            self.assertEqual(branch.address + branch.size, resume)
        entry, resume, _, _ = guards[6]
        self.assertEqual((entry, resume), (0x5e707c1, 0x5e707c3))
        body = {op.address: op for op in image.ins(image.owner(entry))}
        self.assertEqual((body[0x5e707bc].mnemonic, body[0x5e707bc].op_str), ("mov", "rdi, qword ptr [rsp + 0x28]"))
        self.assertEqual((body[entry].mnemonic, body[entry].operands[0].imm), ("je", 0x5e707cf))
        self.assertEqual((body[resume].mnemonic, body[resume].op_str), ("mov", "rax, qword ptr [rdi + 8]"))
        for entry, resume, before, after in guards:
            self.assertEqual(image.pe.get_data(entry, len(before)), bytes(before))
            self.assertEqual(image.pe.get_data(resume, len(after)), bytes(after))
        self.assertEqual(len(self.values["DOWNED_XP_PREREQUISITES"]), 1)
        for at, raw in self.values["DOWNED_XP_PREREQUISITES"]:
            self.assertEqual((at, len(raw)), (0x5e707b5, 12))
            self.assertEqual(image.pe.get_data(at, len(raw)), bytes(raw))
        self.assertEqual(self.values["DOWNED_XP_HOST_REWARD_ABI"], 1)
        self.assertEqual(self.values["DOWNED_XP_HOST_REWARD_SLOT"], 0x48)
        self.assertEqual(self.values["DOWNED_XP_PENDING_FIELD"], 0x1030)

    def test_whole_level_producers_keep_native_skip_and_the_admitted_dead_increment(self):
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        for label, start, resume, end in (("host", 0x5f32004, 0x5f3200e, 0x5f3201b),
                                          ("remote", 0x5e707b5, 0x5e707c3, 0x5e707cf)):
            for phase, admitted in ((1, False), (4, False), (4, True)):
                with self.subTest(path=label, phase=phase, admitted=admitted):
                    # Fresh machines keep each fragment's stop boundary separate.
                    m = ReviewMachine(self.image)
                    game, frame, receipt, request = (m.scratch + n for n in (0x10000, 0x18000, 0x20000, 0x22000))
                    m.write(receipt, "I", 3)
                    m.write(request + 8, "Q", receipt)
                    m.write(game + 0x1030, "I", 0)
                    m.write(game + 0x10d1, "B", phase)
                    m.write(frame - 0x48, "Q", receipt)
                    m.write(m.stack + 0x28, "Q", request)
                    for register, value in ((reg.UC_X86_REG_R13, game), (reg.UC_X86_REG_RSI, game),
                                            (reg.UC_X86_REG_RBP, frame), (reg.UC_X86_REG_RDI, 0xbad)):
                        m.cpu.reg_write(register, value)
                    if admitted and label == "remote":
                        m.stop = m.base + 0x5e707c1
                        m.call(start)
                        self.assertEqual(m.cpu.reg_read(reg.UC_X86_REG_RDI), request)
                    m.stop = m.base + end
                    m.call(resume if admitted else start)
                    self.assertEqual(m.read(game + 0x1030, "I"), (3 if phase == 1 or admitted else 0,))
                    self.assertEqual(m.read(game + 0x10d1, "B"), (phase,))

    def test_native_tranquility_uses_its_budget_pair_and_the_selected_throttle(self):
        from native_1392_machine import CatalogMachine
        from unicorn import x86_const as reg
        m = CatalogMachine(self.image)
        game, profile, row = (m.scratch + n for n in (0x10000, 0x18000, 0x22000))
        m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
        m.write(row, "I2f", 33, 3., 12.)
        m.write(game + self.field(0x830), "3Q", 1, row, 1)
        m.write(game + self.field(0xb9c), "f", 10.)
        m.write(game + self.field(0xc46), "B", 1)
        m.write(game + 0x10ce, "B", 1)
        m.write(game + 0x288, "f", 50.)
        m.write(game + self.field(0x1a4), "f", 7.)
        self.assertEqual(self.field(0x1a4), 0x28c)
        self.assertEqual(self.field(0x1ad), 0x2a6)
        m.cpu.reg_write(reg.UC_X86_REG_XMM1, 0x3dcccccd)
        m.call(0x5fb8960, game, 0)
        self.assertEqual(m.read(game + 0x288, "f"), (0.,))
        self.assertEqual(m.read(game + self.field(0x1a4), "f"), (11.,))
        self.assertEqual(m.read(game + self.field(0x1ad), "B"), (0,))

    def test_native_swiftmend_uses_the_relocated_float_cooldown_and_restores_context(self):
        from native_1392_machine import CatalogMachine
        from unicorn import UC_HOOK_MEM_READ, UC_HOOK_MEM_WRITE
        m = CatalogMachine(self.image)
        game, profile, timer, event = (m.scratch + n for n in (0x10000, 0x18000, 0x22000, 0x24000))
        m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
        definitions = dict(self.values["ADDRESSES"])
        m.write(game + self.field(0x830), "3Q", 1, m.base + definitions[0x7146510], 1)
        m.write(game + self.field(0x68), "3Q", 1, timer, 1)
        m.write(timer, "f", 1000.)
        m.write(game + self.field(0x4f8), "f", 35.)
        m.write(game + self.field(0x490), "f", 100.)
        m.write(game + self.field(0xb9c), "f", 10.)
        m.write(game + self.field(0xc46), "B", 1)
        m.write(game + self.field(0x478), "Q", 0x100000001)
        m.write(game + 0x10ce, "B", 1)
        m.write(event + 29, "B", 5)
        reads, writes = [], []
        def read(cpu, access, address, size, value, user):
            reads.append((address - timer, size))
        def context(cpu, access, address, size, value, user):
            writes.append((address - game, size, value))
        m.cpu.hook_add(UC_HOOK_MEM_READ, read, begin=timer, end=timer + 3)
        m.cpu.hook_add(UC_HOOK_MEM_WRITE, context, begin=game + 0x20, end=game + 0x2f)
        m.call(definitions[0x59436a0], game, event, 0)
        self.assertEqual(reads, [(0, 4)])
        self.assertIn((0x20, 8, 1), writes)
        self.assertIn((0x28, 8, 0), writes)
        self.assertEqual(m.read(game + self.field(0x20), "2Q"), (0, 0))
        self.assertEqual(m.read(timer, "f"), (1000.,))
        self.assertEqual(m.read(game + self.field(0xa20) + 16, "Q"), (0,))

    def test_native_hunter_mark_math_and_stores_use_selected_runtime_fields(self):
        import struct
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        m = ReviewMachine(self.image)
        game, ability, enemy = (m.scratch + n for n in (0x10000, 0x18000, 0x20000))
        power, modifier = self.values["MARK_POWER_FIELDS"]
        for base, factor, expected in ((1., 1., .2), (1., 3.0625, .35), (4., 1., .4), (100., 1., .4)):
            m.cpu.mem_write(ability, bytes(self.values["ABILITY_STRIDE"]))
            m.write(ability + power, "f", base)
            m.write(ability + modifier, "f", factor)
            m.write(game + self.field(0xb9c), "f", 20.)
            m.cpu.reg_write(reg.UC_X86_REG_RAX, ability)
            m.cpu.reg_write(reg.UC_X86_REG_R8, 0)
            m.cpu.reg_write(reg.UC_X86_REG_R13, game)
            m.stop = m.base + 0x5fd6a6f
            m.call(0x5fd6a41)
            value = struct.unpack("<f", struct.pack("<I", m.cpu.reg_read(reg.UC_X86_REG_XMM7) & 0xffffffff))[0]
            self.assertAlmostEqual(value, expected)
        m.cpu.reg_write(reg.UC_X86_REG_RAX, enemy)
        m.cpu.reg_write(reg.UC_X86_REG_RCX, self.values["ENEMY_STRIDE"])
        m.cpu.reg_write(reg.UC_X86_REG_R13, game)
        m.cpu.reg_write(reg.UC_X86_REG_RDI, 77)
        m.stop = m.base + 0x5fd6c29
        m.call(0x5fd6c1c)
        self.assertEqual(m.read(game + self.field(0x8f0), "Q"), (77,))
        self.assertAlmostEqual(m.read(enemy + self.values["ENEMY_VULNERABILITY"], "f")[0], .4)

    def test_polymorph_hooks_preserve_native_hit_checks_after_scan_bypass(self):
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        addresses, entries = dict(self.values["ADDRESSES"]), dict(self.values["ENTRIES"])
        expected = ((0x5cfeb6c, 0x5f46572, "lea", "rbx, [r12 + 0xc10]"),
                    (0x5cfed67, 0x5f46759, "movzx", "eax, byte ptr [rbp + 0x340]"),
                    (0x5cfeadd, 0x5f464d4, "mov", "rax, qword ptr [r12 + 0x470]"),
                    (0x5cfeb08, 0x5f46508, "movzx", "edi, byte ptr [r12 + 0x2a6]"))
        for canonical, address, mnemonic, operands in expected:
            self.assertEqual(addresses[canonical], address)
            op = next(op for op in self.image.ins(self.image.owner(address)) if op.address == address)
            self.assertEqual((op.mnemonic, op.op_str), (mnemonic, operands))
            self.assertGreaterEqual(len(entries[canonical]), 5)
            self.assertEqual(self.image.pe.get_data(address, len(entries[canonical])), bytes(entries[canonical]))
        self.assertEqual(self.values["POLYMORPH_HIT_REGISTERS"], [1, 1, 0])
        for blocked_field in (None, 0x40, 0x284, 0xc64):
            m = ReviewMachine(self.image)
            game, frame = m.scratch + 0x10000, m.scratch + 0x18000
            m.cpu.reg_write(reg.UC_X86_REG_R12, game)
            m.cpu.reg_write(reg.UC_X86_REG_RSI, 777)
            m.cpu.reg_write(reg.UC_X86_REG_RBP, frame)
            m.cpu.reg_write(reg.UC_X86_REG_XMM12, 0x3f800000)  # a committed positive hit
            if blocked_field is not None:
                m.write(game + blocked_field, "B", 1)
            m.stop = m.base + addresses[0x5cfeb6c]
            # No synthetic ability is installed. Only the special-kind scan is
            # bypassed; the real engine still decides whether this hit qualifies.
            m.call(addresses[0x5cfeb08])
            self.assertEqual(m.cpu.reg_read(reg.UC_X86_REG_RAX) & 255, int(blocked_field is None))
            self.assertEqual(m.cpu.reg_read(reg.UC_X86_REG_RSI), 777)
            self.assertEqual(m.cpu.reg_read(reg.UC_X86_REG_R12), game)

    def test_native_polymorph_ownership_and_expiry_follow_the_selected_fields(self):
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        m = ReviewMachine(self.image)
        game, actor, owners, enemy = (m.scratch + n for n in (0x10000, 0x14000, 0x18000, 0x19000))
        pointer, count, private_pointer, private_count = self.values["POLYMORPH_OWNERS_FIELDS"]
        m.write(game + pointer, "Q", owners)
        m.cpu.reg_write(reg.UC_X86_REG_R14, 0)
        m.cpu.reg_write(reg.UC_X86_REG_RCX, game)
        m.cpu.reg_write(reg.UC_X86_REG_RDX, 777)
        m.stop = m.base + 0x5fda35c
        m.call(0x5fda347)
        self.assertEqual(m.read(owners, "Q"), (777,))
        self.assertEqual(m.read(game + count, "Q"), (1,))
        m.stop = m.scratch
        m.call(0x5f614f0, game, actor)
        self.assertEqual(m.read(actor + private_pointer, "Q"), (owners,))
        self.assertEqual(m.read(actor + private_count, "Q"), (1,))
        m = ReviewMachine(self.image)
        m.cpu.reg_write(reg.UC_X86_REG_R14, enemy)
        m.cpu.reg_write(reg.UC_X86_REG_XMM6, 0x41200000)  # native clock = 10
        m.cpu.reg_write(reg.UC_X86_REG_XMM0, 0x40800000)  # native duration = 4
        m.stop = m.base + 0x5fda259
        m.call(0x5fda240)
        self.assertEqual(m.read(enemy + self.values["ENEMY_POLYMORPH_EXPIRY"], "f"), (14.,))

    def test_native_modifier_resolver_keeps_spell_scope_and_its_output_extent(self):
        from native_1392_machine import ReviewMachine
        m = ReviewMachine(self.image)
        out, modifiers, row, key_at = (m.scratch + n for n in (0x10000, 0x11000, 0x12000, 0x13000))
        definition = dict(self.values["ADDRESSES"])[0x6cc5c00]
        resolve = dict(self.values["EXTENSION_ADDRESSES"])[0x5ab3360]
        size = self.values["SPELL_MODIFIERS_SIZE"]
        self.assertEqual(size, 40)
        for key, expected in ((b"pw_shield", 1.25), (b"renew", 1.)):
            m.cpu.mem_write(out, bytes([0xa7]) * 96)
            m.cpu.mem_write(key_at, key)
            # The aggregate's second vector owns spell-scoped 32-byte rows.
            m.write(modifiers + 24, "3Q", 1, row, 1)
            m.write(row, "I4x2QIf", 1, key_at, len(key), 0, .25)
            m.call(resolve, out, modifiers, m.base + definition)
            self.assertEqual(m.read(out, "f"), (expected,))
            self.assertEqual(m.read(out + 28, "f"), (1.,))
            self.assertAlmostEqual(m.read(out + 32, "f")[0], .3)
            self.assertEqual(m.bytes_at(out + size, 96 - size), bytes([0xa7]) * (96 - size))

    def test_native_cast_constructor_and_delivery_use_the_relocated_modifiers(self):
        from native_1392_machine import CatalogMachine
        from unicorn import UC_HOOK_MEM_READ, UC_HOOK_MEM_WRITE
        for bonus, expected in ((0., 44.8), (.25, 44.8 * 1.25 ** .5)):
            m = CatalogMachine(self.image)
            game, profile, out, row, key = (m.scratch + n for n in
                                          (0x10000, 0x18000, 0x22000, 0x23000, 0x24000))
            m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
            definition = dict(self.values["ADDRESSES"])[0x6cc5c00]
            size, modifiers = self.values["CAST_ABILITY_SIZE"], self.values["ABILITY_MODIFIERS_OFFSET"]
            self.assertEqual((size, modifiers), (96, 8))
            m.cpu.mem_write(out, bytes([0xa7]) * (size + 16))
            writes, reads = [], []
            def write(cpu, access, address, count, value, user):
                writes.append((address - out, count))
            def read(cpu, access, address, count, value, user):
                reads.append((address - out, count))
            m.cpu.hook_add(UC_HOOK_MEM_WRITE, write, begin=out, end=out + size + 15)
            m.cpu.hook_add(UC_HOOK_MEM_READ, read, begin=out, end=out + size + 15)
            m.call(0x5fc2510, out, game, m.base + definition)
            self.assertEqual(max(at + count for at, count in writes), size)
            self.assertEqual(m.read(out, "Q"), (m.base + definition,))
            self.assertEqual(m.read(out + modifiers, "f"), (1.,))
            m.write(game + self.field(0x4f8), "f", 35.)
            m.write(game + self.field(0x490), "f", 100.)
            m.write(game + self.field(0xb9c), "f", 10.)
            m.write(game + self.field(0xc46), "B", 1)
            m.write(game + self.field(0x478), "Q", 0x100000001)
            m.cpu.mem_write(key, b"pw_shield")
            # Resolve in place, as the native constructor does. A scoped bonus
            # must change the delivered shield, rather than only its display.
            aggregate = game + self.field(0x800)
            m.write(aggregate + 24, "3Q", 1, row, 1)
            m.write(row, "I4x2QIf", 1, key, 9, 0, bonus)
            resolve = dict(self.values["EXTENSION_ADDRESSES"])[0x5ab3360]
            m.call(resolve, out + modifiers, aggregate, m.base + definition)
            m.write(game + self.field(0x480), "If", 0, 0.)
            m.write(game + self.field(0x4fc), "f", 0.)
            before = m.bytes_at(out, size + 16)
            writes.clear()
            self.assertEqual(m.call(0x5f7bb40, game, out) & 255, 1)
            self.assertEqual(m.read(game + self.field(0x480), "I"), (17,))
            self.assertAlmostEqual(m.read(game + self.field(0x480) + 4, "f")[0], expected, places=4)
            self.assertIn((modifiers, 4), reads)
            self.assertIn((self.values["MARK_POWER_FIELDS"][0], 4), reads)
            self.assertEqual(writes, [])
            self.assertEqual(m.bytes_at(out, size + 16), before)
            self.assertEqual(m.bytes_at(out + size, 16), bytes([0xa7]) * 16)

    def test_native_cast_constructor_reuses_the_matching_runtime_row(self):
        from native_1392_machine import CatalogMachine
        m = CatalogMachine(self.image)
        game, profile, rows, out = (m.scratch + n for n in (0x10000, 0x18000, 0x22000, 0x24000))
        m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
        definitions = dict(self.values["ADDRESSES"])
        stride = self.values["ABILITY_STRIDE"]
        self.assertEqual(stride, self.values["CAST_ABILITY_SIZE"])
        for index, canonical in enumerate((0x6cc5c00, 0x6cc6380)):
            m.call(0x5fc2510, rows + index * stride, game, m.base + definitions[canonical])
            # Distinguish stored runtime state from freshly resolved defaults.
            m.write(rows + index * stride + 76, "f", float(11 + index))
            m.write(rows + index * stride + 92, "I", 0x12340000 + index)
        m.write(game + self.field(0x338), "3Q", 2, rows, 2)
        for index, canonical in enumerate((0x6cc5c00, 0x6cc6380)):
            m.cpu.mem_write(out, bytes([0xa7]) * (stride + 16))
            m.call(0x5fc2510, out, game, m.base + definitions[canonical])
            self.assertEqual(m.bytes_at(out, stride), m.bytes_at(rows + index * stride, stride))
            self.assertEqual(m.bytes_at(out + stride, 16), bytes([0xa7]) * 16)

    def test_native_spell_lookup_consumes_the_selected_learned_book(self):
        from native_1392_machine import CatalogMachine
        m = CatalogMachine(self.image)
        game, profile, rows = (m.scratch + n for n in (0x10000, 0x18000, 0x22000))
        m.write(profile + 0x1a4, "B", 1)  # neither test spell is in this class's native list
        m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
        definitions = dict(self.values["ADDRESSES"])
        header, stride = game + self.field(0xb0), self.values["LEARNED_ABILITY_STRIDE"]
        self.assertEqual((header - game, stride), (0xc8, 104))
        for index, canonical in enumerate((0x6cc5c00, 0x6cc6380)):
            m.write(rows + index * stride, "Q", m.base + definitions[canonical])
        m.write(header, "3Q", 2, rows, 2)
        for spell, canonical in ((17, 0x6cc5c00), (331, 0x6cc6380)):
            self.assertEqual(m.call(0x5fb56e0, game, spell), m.base + definitions[canonical])
        self.assertEqual(m.call(0x5fb56e0, game, 0), 0)
        m.write(header + 16, "Q", 1)
        self.assertEqual(m.call(0x5fb56e0, game, 331), 0)
        self.assertEqual(m.call(0x5fb56e0, game, 17), m.base + definitions[0x6cc5c00])

    def test_native_private_set_clone_keeps_its_selected_identity_and_extent(self):
        from native_1392_machine import CatalogMachine
        from unicorn import UC_HOOK_CODE, x86_const as reg
        m = CatalogMachine(self.image)
        game, profile, event, rows = (m.scratch + n for n in (0x10000, 0x18000, 0x22000, 0x24000))
        m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
        raw = b"".join((25).to_bytes(4, "little") + bytes([value]) * 60 for value in (0x31, 0x72))
        m.cpu.mem_write(rows, raw)
        m.write(game + self.field(0x830), "3Q", 2, rows, 2)
        m.write(game + 0x10ce, "B", 1)
        event_slot, count_slot = self.values["SET_CLONE_FRAME_FIELDS"]
        seen = []
        def copied(cpu, address, size, user):
            frame = cpu.reg_read(reg.UC_X86_REG_RBP)
            pointer = cpu.reg_read(reg.UC_X86_REG_R14)
            self.assertEqual(cpu.reg_read(reg.UC_X86_REG_R15), game)
            self.assertEqual(m.read(frame + event_slot, "Q"), (event,))
            self.assertEqual(m.read(frame + count_slot, "Q"), (2,))
            self.assertNotEqual(pointer, rows)
            self.assertEqual(m.bytes_at(pointer, len(raw)), raw)
            seen.append(pointer)
        m.cpu.hook_add(UC_HOOK_CODE, copied, begin=m.base + 0x5fbc8d1, end=m.base + 0x5fbc8d1)
        m.call(dict(self.values["ADDRESSES"])[0x59436a0], game, event, 0)
        self.assertEqual(len(seen), 1)
        self.assertNotIn(seen[0], m.allocations)
        self.assertEqual(m.bytes_at(rows, len(raw)), raw)

    def test_native_set_vulnerability_pair_and_producer_frame_use_selected_fields(self):
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        m = ReviewMachine(self.image)
        enemy = m.scratch + 0x10000
        game_slot, spell_slot, coefficient = self.values["SET_VULNERABILITY_LAYOUT"]
        self.assertEqual((game_slot, spell_slot, coefficient), (0x1e0, 0x1cc, 0x140))
        ops = {op.address: op for op in self.image.ins(self.image.owner(0x5fbc7c0))}
        self.assertEqual(ops[0x5fbca33].op_str, f"qword ptr [rbp + {hex(game_slot)}], r15")
        self.assertEqual(ops[0x5fbc9cb].op_str, f"dword ptr [rbp + {hex(spell_slot)}], r10d")
        self.assertEqual(ops[0x5fbc8e9].op_str, "r10d, dword ptr [r8 + 8]")
        m.write(enemy + self.values["ENEMY_GUID"], "Q", 777)
        field = enemy + coefficient + 8
        self.assertEqual(m.read(field - 0x38, "Q"), (777,))
        m.cpu.reg_write(reg.UC_X86_REG_RCX, field)
        m.cpu.reg_write(reg.UC_X86_REG_R13, 0x3e4ccccd)  # native 20% coefficient
        m.cpu.reg_write(reg.UC_X86_REG_XMM2, 0x41c00000)  # expiry = 24
        m.stop = m.base + 0x5fc05fd
        m.call(0x5fc05f4)
        value, expiry = m.read(enemy + coefficient, "2f")
        self.assertAlmostEqual(value, .2)
        self.assertEqual(expiry, 24.)

    def test_native_dot_append_keeps_the_row_schema_and_selected_source_carriers(self):
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        m = ReviewMachine(self.image)
        rows = m.scratch + 0x10000
        m.cpu.mem_write(rows, bytes([0xa7]) * 112)
        m.cpu.reg_write(reg.UC_X86_REG_RAX, rows)
        m.cpu.reg_write(reg.UC_X86_REG_RDX, 56)  # append the second native row
        m.cpu.reg_write(reg.UC_X86_REG_R13, 1)
        m.cpu.reg_write(reg.UC_X86_REG_RBX, 777)
        m.cpu.reg_write(reg.UC_X86_REG_RDI, 1978)
        m.cpu.reg_write(reg.UC_X86_REG_RSI, 3)
        m.cpu.reg_write(reg.UC_X86_REG_R8, 5)
        m.cpu.reg_write(reg.UC_X86_REG_XMM0, 0x40400000)  # three ticks
        m.cpu.reg_write(reg.UC_X86_REG_XMM6, 0x41400000)  # total damage = 12
        m.cpu.reg_write(reg.UC_X86_REG_XMM7, 0x3f000000)
        m.cpu.reg_write(reg.UC_X86_REG_XMM8, 0x41200000)
        m.cpu.reg_write(reg.UC_X86_REG_XMM9, 0)
        m.stop = m.base + 0x5ee1c09
        m.call(0x5ee1bc9)
        self.assertEqual(m.bytes_at(rows, 56), bytes([0xa7]) * 56)
        self.assertEqual(m.read(rows + 56, "3Q"), (1, 777, 0xffffffffffffffff))
        self.assertEqual(m.read(rows + 56 + 24, "I3f2IfB"),
                         (1978, 4., 0., 10., 3, 3, .5, 5))
        self.assertEqual(m.bytes_at(rows + 109, 3), bytes([0xa7]) * 3)
        self.assertEqual(self.values["SET_DOT_STACK_FIELDS"], [0xd8, 0xa0])
        self.assertEqual(self.values["SET_DOT_ROW_REGISTER"], 2)
        expected = [(0x5f25a5e, 0, 0xb0), (0x5f862a6, 1, 0x1a0), (0x5f86560, 1, 0x1a0),
                    (0x5fb67ef, 0, 0x98), (0x5fbf2ac, 1, 0x1e0)]
        self.assertEqual(self.values["SET_DOT_OWNER_CALLERS"], expected)
        for returned, _, _ in expected:
            # Decode the entire owner to prove these are actual instruction
            # boundaries, not a byte pattern in the middle of another opcode.
            ops = {op.address: op for op in self.image.ins(self.image.owner(returned))}
            call = ops[returned - 5]
            self.assertEqual(call.mnemonic, "call")
            self.assertEqual(call.operands[0].imm, 0x5ee1720)
        carriers = ((0x5f25350, 0x5f253c1, "rsi, rcx"),
                    (0x5f85940, 0x5f85a6f, "qword ptr [rbp + 0x1a0], rsi"),
                    (0x5fb6650, 0x5fb66dc, "rbx, rcx"),
                    (0x5fbc7c0, 0x5fbca33, "qword ptr [rbp + 0x1e0], r15"))
        for owner, at, operands in carriers:
            op = next(op for op in self.image.ins(self.image.owner(owner)) if op.address == at)
            self.assertEqual((op.mnemonic, op.op_str), ("mov", operands))
        entries, addresses = dict(self.values["ENTRIES"]), dict(self.values["ADDRESSES"])
        for canonical, at in ((0x5d3f30e, 0x5fbc8d1), (0x5d426d7, 0x5fc05fd), (0x5b8e99b, 0x5ee1bdb)):
            self.assertEqual(addresses[canonical], at)
            self.assertGreaterEqual(len(entries[canonical]), 5)
            self.assertEqual(self.image.pe.get_data(at, len(entries[canonical])), bytes(entries[canonical]))

    def test_runtime_ability_stride_is_used_by_native_multispell_exports(self):
        from native_1392_machine import CatalogMachine
        m = CatalogMachine(self.image)
        game, profile, rows, out = (m.scratch + n for n in (0x10000, 0x18000, 0x22000, 0x24000))
        m.write(profile + 0x1a4, "2B", 1, 1)
        m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
        definitions = dict(self.values["ADDRESSES"])
        for index, canonical in enumerate((0x6cc5c00, 0x6cc6380)):
            m.write(rows + index * self.values["ABILITY_STRIDE"], "Q", m.base + definitions[canonical])
        m.write(game + self.field(0x338), "3Q", 2, rows, 2)
        m.call(definitions[0x5821710], out, game, 0)
        key, length = m.read(out + 6 * 40, "2Q")
        self.assertEqual(m.bytes_at(key, length), b"AZSSPELLS")
        spells = m.buffer(out + 6 * 40 + 16).decode().split(";")
        self.assertTrue(spells[0].startswith("pw_shield,17,"))
        self.assertTrue(spells[1].startswith("healing_wave,331,"))

    def test_native_ui_receiver_pair_and_event_arguments_keep_their_ownership(self):
        import struct
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        addresses, entries = dict(self.values["ADDRESSES"]), dict(self.values["ENTRIES"])
        vtable = addresses[0x5ede960]
        event, hud = addresses[0x586d550], addresses[0x586d120]
        methods = struct.unpack("<6Q", self.image.pe.get_data(vtable, 48))
        base = self.image.pe.OPTIONAL_HEADER.ImageBase
        self.assertEqual(methods[1:3], (32, 8))
        self.assertEqual(methods[3:5], (base + event, base + hud))
        for canonical, entry in ((0x586d550, event), (0x586d120, hud)):
            self.assertEqual(self.image.pe.get_data(entry, len(entries[canonical])), bytes(entries[canonical]))
        m = ReviewMachine(self.image)
        server, frame, receiver = (m.scratch + n for n in (0x10000, 0x14000, 0x18000))
        sink = self.values["SERVER_UI_SINK"]
        self.assertEqual(sink, 0x1f60)
        m.cpu.mem_write(server + sink - 8, bytes([0xa7]) * 32)
        m.write(frame + 0x958, "Q", receiver)
        m.write(frame + 0x950, "Q", base + vtable)
        m.cpu.reg_write(reg.UC_X86_REG_RSI, server)
        m.cpu.reg_write(reg.UC_X86_REG_RBP, frame)
        m.stop = m.base + 0x5ea8543
        m.call(0x5ea8527)
        self.assertEqual(m.read(server + sink, "2Q"), (receiver, base + vtable))
        self.assertEqual(m.bytes_at(server + sink - 8, 8), bytes([0xa7]) * 8)
        self.assertEqual(m.bytes_at(server + sink + 16, 8), bytes([0xa7]) * 8)
        # Run the actual method prefix through its receiver and borrowed message
        # spills. VM execution/rendering remains a separate live qualification.
        m = ReviewMachine(self.image)
        receiver, message = m.scratch + 0x10000, m.scratch + 0x14000
        payload = b'{"type":"unpaused"}'
        m.cpu.mem_write(message, payload)
        m.stop = m.base + 0x5e99fd8
        m.call(event, receiver, message, len(payload))
        frame = m.cpu.reg_read(reg.UC_X86_REG_RBP)
        self.assertEqual(m.cpu.reg_read(reg.UC_X86_REG_RSI), receiver)
        self.assertEqual(m.read(frame - 0x30, "2Q"), (message, len(payload)))
        self.assertEqual(m.bytes_at(message, len(payload)), payload)
        self.assertEqual(self.values["NATIVE_UI_LOG_FLAG"], 0x18)

    def test_emulated_crt_calls_resolve_to_the_named_imports(self):
        from native_1392_machine import ReviewMachine
        image = self.image
        image.pe.parse_data_directories(directories=[1])
        imports = {f.address: f.name for d in image.pe.DIRECTORY_ENTRY_IMPORT for f in d.imports}
        machine = ReviewMachine(image)
        from compare_official import MD
        expected = ("memcpy", "memmove", "memcmp", "memset", "sinf", "cosf", "ceilf", "roundf", "floorf")
        for address, name in zip(list(machine.services)[:9], expected):
            instruction = next(MD.disasm(image.pe.get_data(address, 6), address))
            self.assertEqual(instruction.mnemonic, "jmp")
            target = image.pe.OPTIONAL_HEADER.ImageBase + address + instruction.size + instruction.operands[0].mem.disp
            self.assertEqual(imports[target], name.encode())

    def test_native_float_field_writer_preserves_raw_bits_and_adjacent_key(self):
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        m = ReviewMachine(self.image)
        fields = m.scratch + 0x10000
        expected = {}
        for key, bits in ((0x498, 10000), (0x499, 17), (0x498, 20000),
                          (0x498, 0x7fc01234), (0x498, 0x80000000), (0x498, 0xffffffff)):
            m.cpu.reg_write(reg.UC_X86_REG_XMM2, bits)
            self.assertEqual(m.call(0x5dd5dd0, fields, key), fields)
            expected[key] = bits
            node, height, count = m.read(fields, "3Q")
            self.assertEqual((height, count), (0, len(expected)))
            self.assertEqual(m.read(node + 0x38, str(count) + "H"), tuple(sorted(expected)))
            self.assertEqual(m.read(node + 8, str(count) + "I"),
                             tuple(expected[k] for k in sorted(expected)))

    def test_scene_unit_packet_keeps_display_hands_and_borrowed_fields(self):
        import struct
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        m = ReviewMachine(self.image)
        out = m.scratch + 0x10000
        block, fields, point = out + 0x100, out + 0x200, out + 0x400
        guid, entry, display = 0xf130a50010000007, 0xa50010, 15596
        m.write(point, "3f", 17, -13.25, 4.5)
        for key, bits in ((0, guid & 0xffffffff), (1, guid >> 32), (2, 9),
                          (3, entry), (4, 0x3f800000), (37, 23173), (38, 23322),
                          (131, display), (164, 1)):
            m.cpu.reg_write(reg.UC_X86_REG_XMM2, bits)
            m.call(0x5dd5dd0, fields, key)
        m.write(block, "4Q", 0, 1, 0, 0)
        m.call(0x5dd48c0, block, guid, 3, 0, point, 0, point + 32, fields)
        unit = m.buffer(block)
        self.assertIn(struct.pack("<3f", 17, -13.25, 4.5), unit)
        self.assertTrue(unit.endswith(struct.pack("<4I", 23173, 23322, display, 1)))
        m.call(0x5dd5320, out, block)
        m.call(0x5e7f6f0, fields)
        self.assertEqual(m.read(out + 24, "H"), (0xa9,))
        self.assertEqual(m.buffer(out), bytes([1, 0, 0, 0, 0]) + unit)
        cap, ptr, _ = m.read(out, "3Q")
        m.call(0x21d0, ptr, cap, 1)
        self.assertFalse(m.allocations)

    def test_native_destructor_confirms_vector_headers_and_element_extents(self):
        from native_1392_machine import ReviewMachine
        vectors = (
            (0x468, self.values["ABILITY_STRIDE"], 8), (0xc8, self.values["LEARNED_ABILITY_STRIDE"], 8),
            (0x6b8, 552, 8), (0x780, 136, 8), (0x798, 64, 8), (0x7b0, 32, 8),
            (0x7f8, 72, 8), (0x810, 32, 8), (0x828, 24, 8), (0x858, 40, 8),
            (0xaa8, 8, 4), (0xb68, 40, 8), (0xc68, 16, 8), (0xcc8, 0xc08, 8),
            (0xce0, 16, 8), (0xcf8, 16, 8), (0xd10, 72, 8), (0xd28, 80, 8),
            (0xd88, 16, 4), (0xda0, 4, 4), (0xdb8, 56, 8), (0xdd0, 40, 8),
            (0xde8, 32, 8), (0xe00, 32, 8), (0xe18, 8, 4), (0xe48, 40, 8),
            (0xe60, 16, 8), (0xe78, 24, 8),
        )
        m = ReviewMachine(self.image)
        game, arc = m.scratch + 0x10000, m.scratch + 0x18000
        for header, size, align in vectors:
            with self.subTest(header=hex(header), element_size=size):
                m.cpu.mem_write(game, bytes(0x1400))
                m.write(game + 0xf58, "Q", arc)
                m.write(arc, "Q", 2)
                pointer = m.allocate(size * 3, align)
                m.write(game + header, "3Q", 3, pointer, 0)
                m.call(0x5e3efc0, game)
                self.assertFalse(m.allocations)
                self.assertEqual(m.read(arc, "Q"), (1,))

    def test_native_tranquility_budget_uses_health_restored_and_absorb_gained(self):
        import struct
        from native_1392_machine import CatalogMachine
        from unicorn import x86_const as reg
        m = CatalogMachine(self.image)
        game, args = m.scratch + 0x10000, m.scratch + 0x18000
        rows, metadata, source = game + 0x10000, game + 0x11000, game + 0x11100
        m.call(0x5faa440, game, args, args + 0x2000, args + 0x4000, args + 0x6000)
        original = bytes(m.cpu.mem_read(game + 0xb50, 24))
        m.write(game + 0x10d1, "B", 1)
        m.write(game + 0xb50, "3Q", 2, rows, 2)
        m.write(metadata, "q5Q", -2, 0, 0, 0, 0, 0)
        m.write(source, "2Q", 0xffffffff, 0)
        cases = (
            (100., 50., 0., 0., 150., 0., 50.),
            (230., 50., 0., 0., 230., 0., 0.),
            (230., 300., .5, 0., 230., 115., 115.),
            (230., 600., .5, 0., 230., 115., 115.),
            (230., 50., .5, 115., 230., 115., 0.),
            (220., 300., .5, 0., 230., 115., 125.),
        )
        try:
            for hp, raw, tree, shield, after_hp, after_shield, budget in cases:
                with self.subTest(hp=hp, raw=raw, tree=tree, existing_shield=shield):
                    m.write(game + 0x5e0, "2f", hp, shield)
                    m.write(game + 0x288, "f", 0.)
                    m.write(rows, "If", 0x20, tree)
                    m.write(rows + 64, "I2f", 0x21, .5, 10.)
                    m.cpu.reg_write(reg.UC_X86_REG_XMM1,
                                   struct.unpack("<I", struct.pack("<f", raw))[0])
                    m.call(0x5f9d0c0, game, 0, 774, metadata, source)
                    self.assertEqual(m.read(game + 0x5e0, "2f"), (after_hp, after_shield))
                    self.assertEqual(m.read(game + 0x288, "f"), (budget,))
        finally:
            m.cpu.mem_write(game + 0xb50, original)

    def test_tranquility_native_critical_roll_and_dispatch_arguments(self):
        import struct
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        m = ReviewMachine(self.image)
        game, frame, targets = m.scratch + 0x10000, m.scratch + 0x12000, m.scratch + 0x13000
        guid = 0xf130000100000007
        m.write(frame + 0xe8, "Q", targets)
        m.write(targets, "Q", guid)
        seed = (0xffffffffffffffff, 0x0123456789abcdef, 0xfedcba9876543210, 0x123456789abcdef0)
        after = (0xece8ece0ece8ece0, 0, 0x8bec56301275cdef, 0xe263e262e263e262)
        uniform = 0.2355555295944214
        # Execute the original per-target roll up to its real Damage call.
        # Equality is not a critical hit; the three stack arguments stay distinct.
        for chance, critical in ((0., False), (uniform, False), (.25, True), (1., True)):
            with self.subTest(chance=chance):
                m.write(game + 0xf60, "4Q", *seed)
                m.write(game + 0x64c, "2f", chance, 1.75)
                m.write(game + 0x2a4, "3B", 0, 91, 7)
                m.cpu.reg_write(reg.UC_X86_REG_RSI, game)
                m.cpu.reg_write(reg.UC_X86_REG_RBP, frame)
                m.cpu.reg_write(reg.UC_X86_REG_RSP, m.stack)
                m.cpu.reg_write(reg.UC_X86_REG_XMM7, struct.unpack("<I", struct.pack("<f", 10.))[0])
                m.cpu.emu_start(m.base + 0x5fb9245, m.base + 0x5fb931a, count=1000)
                self.assertEqual(m.cpu.reg_read(reg.UC_X86_REG_RIP), m.base + 0x5fb931a)
                self.assertEqual(m.read(game + 0xf60, "4Q"), after)
                bits = m.cpu.reg_read(reg.UC_X86_REG_XMM0) & 0xffffffff
                self.assertEqual(struct.unpack("<f", struct.pack("<I", bits))[0], uniform)
                self.assertEqual(m.cpu.reg_read(reg.UC_X86_REG_RCX), game)
                self.assertEqual(m.cpu.reg_read(reg.UC_X86_REG_RDX), guid)
                self.assertEqual(m.cpu.reg_read(reg.UC_X86_REG_R9), 740)
                self.assertEqual(m.read(m.stack + 0x20, "B"), (3,))
                self.assertEqual(m.read(m.stack + 0x28, "B"), (int(critical),))
                self.assertEqual(m.read(m.stack + 0x30, "B"), (0,))
                amount = struct.unpack("<f", struct.pack("<I", m.cpu.reg_read(reg.UC_X86_REG_XMM2) & 0xffffffff))[0]
                self.assertEqual(amount, 17.5 if critical else 10.)
                self.assertEqual(m.read(game + 0x2a4, "3B"), (1, 91, 3))
                m.cpu.emu_start(m.base + 0x5fb9320, m.base + 0x5fb932d, count=1000)
                self.assertEqual(m.read(game + 0x2a4, "3B"), (0, 91, 7))

    def test_native_lethal_damage_retires_a_corpse_only_in_its_own_actor_context(self):
        import struct
        from native_1392_machine import CatalogMachine
        from unicorn import x86_const as reg
        m = CatalogMachine(self.image)
        game, profile, point = [m.scratch + n for n in (0x10000, 0x18000, 0x22000)]
        m.write(profile + 0x1a4, "3B", 1, 1, 0)
        m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
        m.write(game + 0x10d1, "B", 1)
        m.write(game + 0x10cb, "B", 1)
        m.write(game + 0xfc0, "Q", 0)
        m.write(point, "I3f", 1, 3., -2., 1.)
        m.call(self.values["GOBLIN_SPAWN"][0], game, 0, point)
        _, row, count = m.read(game + 0x6b8, "3Q")
        self.assertEqual(count, 1)
        guid, = m.read(row + 0x110, "Q")
        m.write(row + 0x118, "Q", 1)
        m.write(game + 0x1010, "f", 50.)
        m.cpu.reg_write(reg.UC_X86_REG_XMM2, struct.unpack("<I", struct.pack("<f", 1e9))[0])
        m.call(0x5f43210, game, guid, 0, 23922, 3, 0, 0)
        self.assertEqual(m.read(row + 0xd0, "If"), (1, 50.))
        self.assertLessEqual(m.read(row + 0x198, "f")[0], 0.)
        self.assertEqual(m.read(row + 0x118, "Q"), (1,))
        m.write(game + 0x1010, "f", 100.)
        for slot, expected_count in ((0, 1), (1, 0)):
            m.write(game + 0xfc0, "Q", slot)
            m.cpu.reg_write(reg.UC_X86_REG_XMM1, struct.unpack("<I", struct.pack("<f", .02))[0])
            m.call(0x5f4c680, game)
            self.assertEqual(m.read(game + 0x6c8, "Q"), (expected_count,))
        _, packets, count = m.read(game + 0x810, "3Q")
        destroyed = []
        for i in range(count):
            _, pointer, size, opcode = m.read(packets + i * 32, "3QH")
            if opcode == 0xaa:
                self.assertEqual(size, 8)
                destroyed.append(m.read(pointer, "Q")[0])
        self.assertEqual(destroyed.count(guid), 1)

    def test_native_damage_call_retains_all_seven_arguments(self):
        import struct
        from native_1392_machine import CatalogMachine
        from unicorn import x86_const as reg
        m = CatalogMachine(self.image)
        game, args, enemy = m.scratch + 0x10000, m.scratch + 0x18000, m.scratch + 0x20000
        m.call(0x5faa440, game, args, args + 0x2000, args + 0x4000, args + 0x6000)
        original = bytes(m.cpu.mem_read(game + 0x6b8, 24))
        guid = 0xf130000100000007
        try:
            m.write(game + 0x10d1, "B", 1)
            m.write(game + 0x6b8, "3Q", 1, enemy, 1)
            m.write(enemy + 0x110, "Q", guid)
            m.write(enemy + 0x30, "Q", 0xffffffffffffffff)
            m.write(enemy + 0x198, "2f", 1000., 1000.)
            m.cpu.reg_write(reg.UC_X86_REG_XMM2, struct.unpack("<I", struct.pack("<f", 10.))[0])
            m.call(0x5f43210, game, guid, 0, 740, 3, 0, 0)
            self.assertEqual(m.read(enemy + 0x198, "2f"), (990., 1000.))
        finally:
            m.cpu.mem_write(game + 0x6b8, original)

    def test_native_perk_noop_rejoins_the_next_row_without_interpreting_payload(self):
        import struct
        body = {i.address: i for i in self.image.ins(0x5fbc7c0)}
        instruction = body[0x5fbcb01]
        table = instruction.address + instruction.size + instruction.operands[1].mem.disp
        relative, = struct.unpack("<i", self.image.pe.get_data(table + (25 - 12) * 4, 4))
        self.assertEqual(table + relative, 0x5fbca60)
        # Termination is a full u32 sentinel now, not a single byte.
        self.assertEqual(body[0x5fbca75].operands[1].size, 4)
        self.assertEqual(body[0x5fbca77].operands[1].imm, -1)
        self.assertEqual(body[0x5fbca80].operands[1].mem.disp, 64)

    def test_support_talent_definitions_keep_distinct_reordered_payloads(self):
        import struct
        image = self.image
        def row(header, key, index):
            pointer, length = struct.unpack("<2Q", image.pe.get_data(header, 16))
            self.assertEqual(image.pe.get_data(pointer - image.pe.OPTIONAL_HEADER.ImageBase, length), key.encode())
            pointer, count = struct.unpack("<2Q", image.pe.get_data(header + 48, 16))
            self.assertLess(index, count)
            return image.pe.get_data(pointer - image.pe.OPTIONAL_HEADER.ImageBase + index * 64, 64)
        for header, key, index, coefficient in ((0x73d57d0, "divine_aegis", 1, .1),
                                                (0x73cda58, "hammer_righteous", 3, .5)):
            raw = row(header, key, index)
            self.assertEqual(struct.unpack_from("<I", raw)[0], 15)
            self.assertEqual(struct.unpack_from("<I", raw, 8)[0], 10)
            self.assertAlmostEqual(struct.unpack_from("<f", raw, 40)[0], coefficient)
            self.assertEqual(struct.unpack_from("<f2I", raw, 44), (10., 15237, 1))
        raw = row(0x73e56b8, "swiftmend", 0)
        self.assertEqual(struct.unpack_from("<I", raw)[0], 12)
        self.assertEqual(struct.unpack_from("<I", raw, 8)[0], 9)
        self.assertAlmostEqual(struct.unpack_from("<f", raw, 12)[0], .4)
        self.assertEqual(struct.unpack_from("<2f", raw, 56), (1., 8.))
        raw = row(0x73e5c38, "tree_of_life", 0)
        self.assertEqual(struct.unpack_from("<I", raw)[0], 32)
        self.assertAlmostEqual(struct.unpack_from("<f", raw, 4)[0], .2)
        self.assertEqual(struct.unpack_from("<I2f", row(0x73e5978, "tranquility", 0)), (33, 3., 12.))
        raw = row(0x73cb058, "fortified", 0)
        self.assertEqual(struct.unpack_from("<I", raw)[0], 30)
        self.assertEqual(struct.unpack_from("<I", raw, 8)[0], 7)
        self.assertAlmostEqual(struct.unpack_from("<f", raw, 40)[0], .1)
        self.assertAlmostEqual(struct.unpack_from("<f", raw, 44)[0], .3)

    def test_power_word_shield_keeps_the_strongest_pool_but_refreshes_expiry(self):
        from native_1392_machine import CatalogMachine
        m = CatalogMachine(self.image)
        game, profile, row = (m.scratch + n for n in (0x10000, 0x18000, 0x22000))
        m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
        m.write(game + self.field(0x490), "f", 100.)
        m.write(game + self.field(0x4f8), "f", 80.)
        m.write(game + self.field(0xc41), "B", 1)
        m.write(game + self.field(0xb40), "Q", 1)
        m.write(game + self.field(0x478), "Q", 0x100000002)
        m.write(row, "2Q3fI2fI", 0, 0x100000001, 0., 0., 0., 11426, .4, 10., 15)
        m.call(0x5f39070, game, row)
        self.assertEqual(m.read(game + self.field(0x480), "If"), (11426, 40.))
        # PW:S is an auxiliary pool here. A main-pool-only observation would
        # silently transfer its proc ownership on the second caster's weak cast.
        for producer, fraction, clock, expected in ((0, .5, 1., 50.), (1, .25, 2., 50.),
                                                   (1, .5, 3., 50.), (1, .75, 4., 75.),
                                                   (0, 3., 5., 100.)):
            with self.subTest(producer=producer, fraction=fraction):
                m.write(game + self.field(0xb9c), "f", clock)
                m.write(row, "Q", producer)
                m.write(row + 28, "I", 17)
                m.write(row + 32, "f", fraction)
                m.call(0x5f39070, game, row)
                _, pointer, count = m.read(game + self.values["STRONGEST_SHIELD_POOLS"], "3Q")
                self.assertEqual(count, 1)
                self.assertEqual(m.read(pointer, "If"), (17, expected))
                self.assertEqual(m.read(game + self.field(0x4fc), "f"), (40. + expected,))
                _, pointer, count = m.read(game + self.field(0x398), "3Q")
                shield = [pointer + i * 32 for i in range(count)
                          if m.read(pointer + i * 32 + 20, "I") == (17,)]
                self.assertEqual(len(shield), 1)
                self.assertEqual(m.read(shield[0] + 24, "f"), (clock + 10.,))

    def test_learned_leader_of_the_pack_builds_the_active_bear_form(self):
        from native_1392_machine import CatalogMachine
        m = CatalogMachine(self.image)
        game, profile, learned = (m.scratch + n for n in (0x10000, 0x18000, 0x22000))
        header = 0x73e4c08
        # The saved learned row points to the full definition, 64 bytes before
        # its key/name header; this is the production constructor/rebuild path.
        m.write(learned, "QB7x", m.base + header - 64, 1)
        m.write(profile + 0x18, "3Q", 1, learned, 1)
        m.write(profile + 0x1a4, "2B", 11, 1)
        m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
        m.call(0x5f988f0, game)
        self.assertEqual(m.read(game + 0x44c, "2B"), (11, 1))
        capacity, pointer, count = m.read(game + 0xb50, "3Q")
        self.assertLessEqual(count, capacity)
        self.assertEqual(count, 4)
        forms = [m.read(pointer + i * 64, "2I") for i in range(count)
                 if m.read(pointer + i * 64, "I") == (38,)]
        self.assertEqual(forms, [(38, 9634)])
        self.assertFalse(any(m.read(pointer + i * 64, "I") == (37,) for i in range(count)))

    def test_native_necklace_builder_keeps_beneficiary_effects_and_owned_allocations(self):
        from native_1392_machine import CatalogMachine
        m = CatalogMachine(self.image)
        out = m.scratch + 0x10000
        cases = (("renew", 0x73bac58, 27, 10.), ("rejuvenation", 0x73bc350, 27, 12.),
                 ("regrowth", 0x73bc3c8, 30, .4), ("holy_light", 0x73b9d58, 30, .4),
                 ("pw_shield", 0x73babe0, 15, .5), ("rupture", 0x73ba640, 23, .2))
        cached = None
        for key, definition, tag, value in cases:
            with self.subTest(spell=key):
                m.call(0x5deab20, out, m.base + definition)
                cap, pointer, count = m.read(out + 48, "3Q")
                self.assertEqual(count, 3)
                self.assertLessEqual(count, cap)
                row = pointer + 128
                self.assertEqual(m.read(row, "I"), (tag,))
                if tag in (27, 30):
                    self.assertEqual(m.read(row + 8, "I"), (7,))
                    self.assertEqual(m.read(row + 16, "Q"), (1,))
                    ptr, length = m.read(row + 24, "2Q")
                    self.assertEqual(m.bytes_at(ptr, length), key.encode())
                    self.assertAlmostEqual(m.read(row + 40, "f")[0], .05 if tag == 27 else value)
                    if tag == 27:
                        self.assertEqual(m.read(row + 48, "fI"), (value, 14))
                    else:
                        self.assertAlmostEqual(m.read(row + 44, "f")[0], .4)
                elif tag == 15:
                    self.assertEqual(m.read(row + 8, "I"), (10,))
                    self.assertEqual(m.read(row + 40, "2f2I"), (value, 10., 15237, 1))
                else:
                    self.assertEqual(m.read(row + 8, "Q"), (1,))
                    ptr, length = m.read(row + 16, "2Q")
                    self.assertEqual(m.bytes_at(ptr, length), key.encode())
                    self.assertAlmostEqual(m.read(row + 32, "f")[0], value)
                    self.assertEqual(m.read(row + 36, "f3I"), (7., 1943, 2, 1))
                for offset, stride, alignment in ((0, 1, 1), (24, 1, 1), (48, 64, 8)):
                    capacity, ptr, _ = m.read(out + offset, "3Q")
                    if capacity:
                        self.assertEqual(m.allocations[ptr], (capacity * stride, alignment))
                        m.call(0x21d0, ptr, capacity * stride, alignment)
                        self.assertNotIn(ptr, m.allocations)
                # The first call initializes the native translation/catalog cache.
                # All subsequent constructors must release their own outputs.
                if cached is None:
                    cached = dict(m.allocations)
                else:
                    self.assertEqual(m.allocations, cached)

    def test_native_set_builder_distinguishes_tier_zero_point_five_and_reserved_slot(self):
        import struct
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        m = ReviewMachine(self.image)
        out = m.scratch + 0x10000
        latest = {1: [73, 73, 71], 2: [60], 3: [42, 64], 4: [65], 5: [],
                  7: [66], 8: [67], 9: [69], 11: [70]}
        for class_id, tags in latest.items():
            for tier in range(8):
                with self.subTest(class_id=class_id, tier=tier):
                    m.cpu.reg_write(reg.UC_X86_REG_XMM3, struct.unpack("<I", struct.pack("<f", 1.))[0])
                    m.call(0x5e32350, out, class_id, tier)
                    capacity, pointer, count = m.read(out, "3Q")
                    if tier == 6:
                        self.assertEqual(capacity, 0xffffffffffffffff)
                        # None does not initialize or own the remaining words.
                        continue
                    self.assertLessEqual(count, capacity)
                    self.assertLessEqual(capacity, 32)
                    actual = [m.read(pointer + index * 64, "I")[0] for index in range(count)]
                    if tier == 7:
                        self.assertEqual(actual, tags)
                    if capacity:
                        self.assertEqual(m.allocations[pointer], (capacity * 64, 8))
                        m.call(0x21d0, pointer, capacity * 64, 8)
                    if m.allocations:
                        # Tier 0.5 initializes a native nine-class catalog once.
                        # Its process-global Vec is not owned by the returned perks.
                        cap, cache, length = m.read(m.base + 0x8ac8dd8, "3Q")
                        self.assertEqual((cap, length), (9, 9))
                        self.assertEqual(m.allocations, {cache: (9 * 72, 8)})

    def native_stats(self, machine, game, out):
        machine.call(0x5f89950, out, game)
        line = machine.buffer(out)
        capacity, pointer, _ = machine.read(out, "3Q")
        machine.call(0x21d0, pointer, capacity, 1)
        return dict((key.strip().decode(), value.decode()) for key, value in
                    (part.split(b"=", 1) for part in line.split(b";")))

    def test_native_stat_fields_are_observed_by_named_hud_values(self):
        from native_1392_machine import CatalogMachine
        m = CatalogMachine(self.image)
        game, settings, out = m.scratch + 0x10000, m.scratch + 0x18000, m.scratch + 0x2a000
        m.call(0x5faa440, game, settings, settings + 0x2000, settings + 0x4000, settings + 0x6000)
        baseline = self.native_stats(m, game, out)
        fields = ((0x640, "hp", "1"), (0x644, "dmg", "1.375"), (0x648, "cd", "1.375"),
                  (0x64c, "crit", "1.375"), (0x650, "cm", "1.38"), (0x654, "spd", "1.375"),
                  (0x658, "pick", "1.4"), (0x65c, "xp", "1.38"), (0x660, "arm", "1.375"),
                  (0x668, "reg", "1.38"), (0x66c, "area", "1.375"), (0x670, "luck", "1.38"),
                  (0x674, "gold", "1.38"), (0x67c, "echo", "1.375"), (0x680, "heal", "1.4"),
                  (0x684, "bp", "1.375"), (0x68c, "doth", "1.375"))
        for offset, name, value in fields:
            with self.subTest(stat=name):
                original = bytes(m.cpu.mem_read(game + offset, 4))
                try:
                    m.write(game + offset, "f", 1.375)
                    actual = self.native_stats(m, game, out)
                    expected = dict(baseline, **{name: value})
                    self.assertEqual(actual, expected)
                finally:
                    m.cpu.mem_write(game + offset, original)

    def test_native_stat_inputs_rebuild_repeat_and_restore_without_accumulation(self):
        from native_1392_machine import CatalogMachine
        m = CatalogMachine(self.image)
        game, settings, out, row = (m.scratch + offset for offset in (0x10000, 0x18000, 0x2a000, 0x2b000))
        m.call(0x5faa440, game, settings, settings + 0x2000, settings + 0x4000, settings + 0x6000)
        original = bytes(m.cpu.mem_read(game + self.field(0x800), 24))
        m.call(0x5f988f0, game)
        baseline = self.native_stats(m, game, out)
        expected = (("dmg", "1.125"), ("cd", "0.889"), ("hp", "259"), ("crit", "0.175"),
                    ("cm", "2.12"), ("arm", "0.345"), ("reg", "0.93"), ("spd", "1.125"), ("area", "1.125"))
        for kind, (stat, value) in enumerate(expected):
            with self.subTest(input_kind=kind):
                m.write(row, "If", kind, .125)
                m.write(game + self.field(0x800), "3Q", 1, row, 1)
                try:
                    for _ in range(2):
                        m.call(0x5f988f0, game)
                        self.assertEqual(self.native_stats(m, game, out), dict(baseline, **{stat: value}))
                finally:
                    m.cpu.mem_write(game + self.field(0x800), original)
                m.call(0x5f988f0, game)
                self.assertEqual(self.native_stats(m, game, out), baseline)

    def test_native_healing_meter_credits_the_selected_actor_and_spell(self):
        import struct
        from native_1392_machine import CatalogMachine
        from unicorn import x86_const as reg
        m = CatalogMachine(self.image)
        game, profile, metadata, source = (m.scratch + n for n in (0x10000, 0x18000, 0x22000, 0x23000))
        m.write(profile + 0x1a4, "2B", 5, 1)
        m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
        m.write(game + self.field(0x478), "Q", 0x100000001)
        m.write(game + self.field(0xc46), "B", 1)
        m.write(metadata, "qQ", -2, 0)
        m.write(source, "II", 0xffffffff, 0)
        entry = dict(self.values["ADDRESSES"])[0x590bab0]
        header = game + self.field(0x6d8)
        self.assertEqual(header - game, 0x910)
        for amount, expected in ((0., 0.), (12.5, 12.5), (7.25, 19.75), (-1., 19.75)):
            m.cpu.reg_write(reg.UC_X86_REG_XMM1, struct.unpack("<I", struct.pack("<f", amount))[0])
            m.call(entry, game, 0, 139, metadata, source)
            capacity, pointer, count = m.read(header, "3Q")
            self.assertLessEqual(count, capacity)
            self.assertEqual(count, int(expected > 0.))
            if count:
                self.assertEqual(m.read(pointer + 64, "d"), (expected,))
                # The retained native row identifies Renew with its icon.
                icon, length = m.read(pointer + 48, "2Q")
                self.assertEqual(m.bytes_at(icon, length), b"Spell_Holy_Renew")

    def test_native_gold_rounding_coinage_and_cooperative_receipts(self):
        from native_1392_machine import ReviewMachine
        m = ReviewMachine(self.image)
        game = m.scratch + 0x10000
        cases = ((0, 1, 11, 1.25, 14), (1, 1, 11, 1.25, 14), (1, 2, 11, 1.25, 11),
                 (1, 5, 11, 1.25, 7), (2, 2, 11, 1.25, 14), (0, 1, 3, .5, 2))
        for role, players, amount, multiplier, scaled in cases:
            with self.subTest(role=role, players=players, amount=amount):
                m.cpu.mem_write(game, bytes(0x1200))
                m.write(game + 0x10cb, "B", role)
                m.write(game + 0xfc0, "Q", 4)
                m.write(game + 0xfd0, "Q", players)
                m.write(game + 0x674, "f", multiplier)
                m.write(game + 0x5f0, "I", 20)
                m.write(game + self.field(0x7ac), "I", 40)
                m.call(0x5fb5280, game, amount)
                self.assertEqual(m.read(game + 0x5f0, "I"), (20 + scaled,))
                self.assertEqual(m.read(game + self.field(0x7ac), "I"), (40 + scaled,))
                root, height, count = m.read(game + self.field(0xb20), "3Q")
                self.assertEqual((height, count), (0, 1))
                self.assertEqual(m.read(root + 0x38, "H"), (0x498,))
                self.assertEqual(m.read(root + 8, "I"), ((20 + scaled) * 10000,))
                capacity, pointer, count = m.read(game + 0xcf8, "3Q")
                self.assertEqual(count, int(role != 0))
                if count:
                    self.assertEqual(m.read(pointer, "QI"), (4, amount))
                if capacity:
                    m.call(0x21d0, pointer, capacity * 16, 8)
                m.call(0x5e7f6f0, game + self.field(0xb20))
                self.assertFalse(m.allocations)

    def test_native_conversion_keeps_eligibility_credit_and_choice_ownership(self):
        from native_1392_machine import CatalogMachine
        from unicorn import UC_HOOK_CODE, x86_const as reg
        addresses = dict(self.values["ADDRESSES"])
        for phase, enabled, kind, price, expected in ((1, 1, 3, 80, 0), (2, 0, 3, 80, 0),
                                                    (2, 1, 255, 80, 0), (2, 1, 3, 80, 20),
                                                    (2, 1, 3, 20, 10)):
            with self.subTest(phase=phase, enabled=enabled, kind=kind, price=price):
                m = CatalogMachine(self.image)
                game, profile = m.scratch + 0x10000, m.scratch + 0x18000
                m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
                m.write(game + self.field(0xc46), "B", phase)
                m.write(game + self.field(0x2a6), "B", enabled)
                m.write(game + self.field(0xc44), "B", kind)
                m.write(game + 0x1070, "I", price)
                m.write(game + self.field(0x500), "I", 0)
                m.write(game + self.field(0x4c0), "f", 1.)
                picked = []
                # The reward acceptance operation is a separate porting seam.
                # Observe the native tail dispatch, without claiming it passed.
                def accept(cpu, address, size, user):
                    picked.append((cpu.reg_read(reg.UC_X86_REG_RCX), cpu.reg_read(reg.UC_X86_REG_RDX)))
                    stack = cpu.reg_read(reg.UC_X86_REG_RSP)
                    cpu.reg_write(reg.UC_X86_REG_RIP, m.read(stack, "Q")[0])
                    cpu.reg_write(reg.UC_X86_REG_RSP, stack + 8)
                m.cpu.hook_add(UC_HOOK_CODE, accept, begin=m.base + 0x5fe7640, end=m.base + 0x5fe7640)
                m.call(addresses[0x597e580], game)
                self.assertEqual(m.read(game + self.field(0x500), "I"), (expected,))
                self.assertEqual(picked, [(game, kind)] if expected else [])
                self.assertEqual(m.read(game + self.field(0xc46), "B"), (phase,))
                self.assertEqual(m.read(game + self.field(0xc44), "B"), (kind,))

    def test_native_cache_purchase_debits_once_and_banks_one_item(self):
        from native_1392_machine import ReviewMachine
        m = ReviewMachine(self.image)
        game = m.scratch + 0x10000
        for gold, price, next_price in ((19, 20, 20), (20, 20, 31), (100, 40, 57)):
            with self.subTest(gold=gold, price=price):
                m.cpu.mem_write(game, bytes(0x1200))
                m.write(game + 0x5f0, "I", gold)
                m.write(game + 0x1070, "I", price)
                m.write(game + 0xa98, "I", 42)
                result = m.call(0x5f60ac0, game) & 255
                self.assertEqual(result, int(gold >= price))
                self.assertEqual(m.read(game + 0x5f0, "I"), (gold - price if result else gold,))
                self.assertEqual(m.read(game + 0xa98, "I"), (42,))
                self.assertEqual(m.read(game + 0x1070, "I"), (next_price,))
                self.assertEqual(m.read(game + 0xa9c, "I"), (result,))
                self.assertEqual(m.read(game + 0x1028, "I"), (result,))
                if result:
                    m.call(0x5e7f6f0, game + 0xfa0)
                self.assertFalse(m.allocations)

    def test_native_amulet_pool_hook_keeps_three_real_choices_and_native_phase(self):
        import struct
        from native_1392_machine import CatalogMachine
        from unicorn import UC_HOOK_CODE, x86_const as reg
        m = CatalogMachine(self.image)
        game, settings = m.scratch + 0x10000, m.scratch + 0x18000
        m.call(0x5faa440, game, settings, settings + 0x2000, settings + 0x4000, settings + 0x6000)
        m.write(game + 0x10d1, "B", 1)
        m.write(game + 0x5b0, "Q", 0x100000001)
        m.write(game + 0x1028, "I", 1)
        calls = []
        def pool(cpu, address, size, user):
            frame = cpu.reg_read(reg.UC_X86_REG_RBP)
            self.assertEqual(m.read(frame + 0x198, "Q"), (game,))
            self.assertEqual(cpu.reg_read(reg.UC_X86_REG_RSI), game)
            capacity, pointer, count = m.read(frame + 0x1a0, "3Q")
            self.assertGreaterEqual(capacity, 3)
            self.assertLessEqual(count, capacity)
            self.assertLessEqual(capacity, 256)
            before = bytes(cpu.mem_read(pointer + 72, (capacity - 3) * 24))
            cpu.mem_write(pointer, b"".join(struct.pack("<fIQ8x", 1., 0, 1) for _ in range(3)))
            m.write(frame + 0x1b0, "Q", 3)
            cpu.reg_write(reg.UC_X86_REG_R15, 3)
            self.assertEqual(bytes(cpu.mem_read(pointer + 72, (capacity - 3) * 24)), before)
            calls.append(capacity)
        hook = m.cpu.hook_add(UC_HOOK_CODE, pool, begin=m.base + 0x5fedf37, end=m.base + 0x5fedf37)
        try:
            m.call(0x5fe59e0, game, 1, 1)
        finally:
            m.cpu.hook_del(hook)
        self.assertEqual(len(calls), 1)
        capacity, pointer, count = m.read(game + 0x840, "3Q")
        self.assertGreaterEqual(capacity, count)
        self.assertEqual(count, 3)
        keys = []
        for index in range(count):
            tag, owned = m.read(pointer + index * 160 + 0x78, "2Q")
            self.assertEqual(tag, 0x8000000000000003)
            key, length = m.read(owned + 8, "2Q")
            self.assertTrue(0 < length <= 80)
            keys.append(m.bytes_at(key, length))
        self.assertEqual(len(set(keys)), 3)
        self.assertEqual(m.read(game + 0x10d1, "B"), (2,))
        self.assertEqual(m.read(game + 0x1028, "I"), (1,))

    def test_native_range_query_does_not_apply_area_a_second_time(self):
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        import struct
        m = ReviewMachine(self.image)
        game, enemies, point, out = (m.scratch + offset for offset in (0x10000, 0x14000, 0x18000, 0x18100))
        m.write(game + 0x6b8, "3Q", 3, enemies, 3)
        for index, distance in enumerate((5., 15., 25.)):
            row = enemies + index * 552
            m.write(row + 0x110, "Q", 0x100000001 + index)
            m.write(row + 0x30, "Q", 0xffffffffffffffff)
            m.write(row + 0x198, "2f", 100., 100.)
            m.write(row + 0x1b4, "8f", distance, 0., 0., distance, 0., 0., 0., 0.)
        for area, radius, expected in ((1., 10., 1), (2., 10., 1), (2., 20., 2)):
            m.write(game + 0x66c, "f", area)
            m.cpu.reg_write(reg.UC_X86_REG_XMM3, struct.unpack("<I", struct.pack("<f", radius))[0])
            m.call(0x5f872c0, out, game, point, 0)
            capacity, pointer, count = m.read(out, "3Q")
            self.assertEqual(count, expected)
            self.assertEqual(m.read(pointer, str(count) + "Q"), tuple(0x100000001 + i for i in range(expected)))
            m.call(0x21d0, pointer, capacity * 8, 8)
            self.assertFalse(m.allocations)

    def test_native_goblin_spawn_retains_the_borrowed_template_and_event_identity(self):
        import struct
        from native_1392_machine import CatalogMachine
        from unicorn import UC_HOOK_CODE, x86_const as reg
        image = self.image
        for map_index, elite in ((0, 0), (10, 0), (12, 0), (10, 1)):
            with self.subTest(map_index=map_index, elite=elite):
                m = CatalogMachine(image)
                game, profile, point, definition = (m.scratch + n for n in (0x10000, 0x18000, 0x22000, 0x23000))
                m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
                m.write(game + self.field(0xc46), "B", 1)
                m.write(game + self.field(0x478), "Q", 0x100000001)
                m.write(point, "I3f", 1, 3., -2., 1.)
                map_row = self.values["GOBLIN_MAP_TABLE"] + map_index * 240
                name, length = struct.unpack("<2Q", image.pe.get_data(map_row + 8, 16))
                m.write(game + 0x3c8, "2Q", name, length)
                pointer, count = struct.unpack("<2Q", image.pe.get_data(map_row + 0x78, 16))
                raw = image.pe.get_data(pointer - m.base, count * 80)
                data = bytearray(next(raw[at:at + 80] for at in range(0, len(raw), 80) if raw[at + 76] == 1))
                text = b"Venture Co. Treasurer"
                m.cpu.mem_write(definition + 128, text)
                data[:16] = bytes(16)
                struct.pack_into("<QQII", data, 16, definition + 128, len(text),
                                 self.values["GOBLIN_ENTRY"], self.values["GOBLIN_DISPLAY"])
                struct.pack_into("<f", data, 40, 2.)
                struct.pack_into("<f", data, 48, struct.unpack_from("<f", data, 48)[0] * 1.5)
                struct.pack_into("<3f", data, 52, 0., 5., 5.2)
                data[64:] = bytes(16)
                data[76] = elite
                m.cpu.mem_write(definition, bytes(data))
                seen = []
                def replace(cpu, address, size, user):
                    self.assertEqual(cpu.reg_read(reg.UC_X86_REG_RSI), game)
                    seen.append(cpu.reg_read(reg.UC_X86_REG_R15))
                    cpu.reg_write(reg.UC_X86_REG_R15, definition)
                seam, guard = self.values["GOBLIN_DEFINITION"]
                m.cpu.hook_add(UC_HOOK_CODE, replace, begin=m.base + seam, end=m.base + seam)
                spawn, spawn_guard = self.values["GOBLIN_SPAWN"]
                self.assertEqual(image.pe.get_data(seam, len(guard)), bytes(guard))
                self.assertEqual(image.pe.get_data(spawn, len(spawn_guard)), bytes(spawn_guard))
                m.call(spawn, game, 0, point)
                self.assertEqual(len(seen), 1)
                capacity, rows, count = m.read(game + self.field(0x570), "3Q")
                self.assertEqual(count, 1)
                self.assertGreaterEqual(capacity, count)
                guid, = m.read(rows + self.values["ENEMY_GUID"], "Q")
                self.assertEqual((guid >> 24) & 0xffffff, self.values["GOBLIN_ENTRY"])
                self.assertEqual(m.read(rows + self.values["ENEMY_DEAD"], "I"), (0,))
                hp, maximum = m.read(rows + 0x198, "2f")
                self.assertGreater(hp, 0.)
                self.assertEqual(hp, maximum)
                self.assertEqual(self.values["ENEMY_HP"], 0x198)
                self.assertEqual(self.values["ENEMY_DEFINITION"], 0x120)
                self.assertEqual(m.read(rows + 0x120, "Q"), (definition,))
                self.assertEqual(m.read(rows + self.values["ENEMY_ELITE"], "B"), (elite,))
                self.assertEqual(m.read(rows + self.values["SOLO_ENEMY_ENCOUNTER"], "Q"), ((1 << 64) - 1,))
                self.assertEqual(m.bytes_at(definition, 80), bytes(data))

    def test_native_spawn_position_preserves_float_arguments_rng_and_output_extent(self):
        from native_1392_machine import CatalogMachine
        from unicorn import x86_const as reg
        m = CatalogMachine(self.image)
        game, profile, out = (m.scratch + n for n in (0x10000, 0x18000, 0x22000))
        m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
        m.write(game + self.field(0x4e0), "3f", 10., -20., 5.)
        m.write(game + self.field(0xae8), "4Q", 1, 2, 3, 4)
        m.cpu.mem_write(out, bytes([0xa7]) * 32)
        m.cpu.reg_write(reg.UC_X86_REG_XMM2, 0x40c00000)
        m.cpu.reg_write(reg.UC_X86_REG_XMM3, 0x41100000)
        entry, guard = self.values["GOBLIN_POSITION"]
        self.assertEqual(self.image.pe.get_data(entry, len(guard)), bytes(guard))
        m.call(entry, out, game, 0, 0, 0x3f000000)
        actual = m.read(out, "3f")
        # Independently reproduced by the pinned 0.139.2 native function with
        # the same inputs. This checks placement, not terrain/map readiness.
        for value, expected in zip(actual, (2.704246997833252, -19.419612884521484, 0.)):
            self.assertAlmostEqual(value, expected, places=5)
        self.assertEqual(m.bytes_at(out + 12, 20), bytes([0xa7]) * 20)
        self.assertEqual(m.read(game + self.field(0xae8), "4Q"),
                         (7711200288555919426, 16593524375674973210, 836542563739834208, 4952411123052889824))

    def test_all_selected_maps_keep_native_guardian_templates_for_event_creation(self):
        import math
        import struct
        image, base = self.image, self.image.pe.OPTIONAL_HEADER.ImageBase
        table, count = self.values["GOBLIN_MAP_TABLE"], self.values["BLESSING_MAP_COUNT"]
        self.assertEqual(count, 13)
        names = []
        for index in range(count):
            row = table + index * 240
            key, length = struct.unpack("<2Q", image.pe.get_data(row + 8, 16))
            names.append(image.pe.get_data(key - base, length).decode())
            pointer, size = struct.unpack("<2Q", image.pe.get_data(row + 0x78, 16))
            self.assertTrue(1 <= size <= 64)
            self.assertTrue(base <= pointer < base + image.pe.OPTIONAL_HEADER.SizeOfImage)
            raw = image.pe.get_data(pointer - base, size * 80)
            guardians = [raw[at:at + 80] for at in range(0, len(raw), 80) if raw[at + 76] == 1]
            self.assertTrue(guardians, names[-1])
            for guardian in guardians:
                hp, = struct.unpack_from("<f", guardian, 48)
                self.assertTrue(math.isfinite(hp) and 1. <= hp <= 1e9)
        self.assertEqual(names[-3:], ["scholomance", "aq", "naxx"])

    def test_native_boss_mark_prefix_caps_all_thirteen_maps_at_six_rows(self):
        import struct
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        m = ReviewMachine(self.image)
        game = m.scratch + 0x10000
        factors = [.5, 1., 1.25, 1.5, 2., 2.5]
        names = []
        for index in range(13):
            row = 0x73b5ac0 + index * 240
            key, length = struct.unpack("<2Q", self.image.pe.get_data(row + 8, 16))
            boss, = struct.unpack("<Q", self.image.pe.get_data(row + 0x88, 8))
            names.append(self.image.pe.get_data(key - m.base, length).decode())
            for heroic in range(6):
                for endless, repeats in ((False, 0), (True, 0), (True, 3)):
                    with self.subTest(map=names[-1], heroic=heroic, endless=endless, repeats=repeats):
                        m.cpu.mem_write(game, bytes(0x1200))
                        m.write(game + 0x3c8, "2Q", key, length)
                        m.write(game + 0x467, "B", heroic)
                        m.write(game + 0xf08, "B", int(endless))
                        m.write(game + 0x1078, "I", repeats)
                        # Stop immediately before the original native mark writes.
                        # No boss kill, loot, progression or pity is fabricated.
                        m.stop = m.base + 0x5f3fff7
                        m.call(0x5f3fea0, game, boss)
                        bits = m.cpu.reg_read(reg.UC_X86_REG_XMM6) & 0xffffffff
                        actual, = struct.unpack("<f", struct.pack("<I", bits))
                        expected = (min(index // 2, 5) + 1) * factors[heroic]
                        if endless:
                            expected *= max(repeats, 1)
                        self.assertEqual(actual, expected)
                        for offset in (0x1038, 0xa7c, 0xa80, 0xa84):
                            self.assertEqual(m.read(game + offset, "I"), (0,))
        self.assertEqual(names[-3:], ["scholomance", "aq", "naxx"])

    def test_native_actor_compaction_uses_the_new_extent(self):
        body = {i.address: i for i in self.image.ins(0x5f20090)}
        for site in (0x5f20136, 0x5f2016a, 0x5f20367):
            self.assertEqual(body[site].mnemonic, "imul")
            self.assertEqual(body[site].operands[-1].imm, 0xc08)
        # Copies the row after its leading eight-byte discriminant.
        self.assertEqual(body[0x5f20154].operands[-1].imm, 0xc00)

    def test_grenade_appends_one_projectile_with_native_delay_and_token(self):
        from native_1392_machine import ReviewMachine
        m = ReviewMachine(self.image)
        game = m.scratch + 0x20000
        effect, origin, enemy = game + 0x4000, game + 0x4080, game + 0x5000
        guid = 0x100000042
        token = 0x415a535800000000 | 1
        vector = game + 0x7c8
        m.write(game + 0x1010, "f", 75.)
        m.write(game + 0x6c0, "QQ", enemy, 1)
        m.write(enemy + 0x110, "Q", guid)
        m.write(effect, "IQ", 8, token)
        m.write(vector, "3Q", 0, 8, 0)
        m.call(0x5f88990, game, 13237, 2, effect, guid, 0, 0x3f000000, origin)
        cap, row, count = m.read(vector, "3Q")
        self.assertEqual(count, 1)
        self.assertEqual(m.allocations[row], (cap * 144, 8))
        self.assertEqual(m.read(row + 112, "IQfI"), (8, token, 75.5, 13237))
        body = {i.address: i for i in self.image.ins(0x5f88990)}
        self.assertEqual(body[0x5f889f9].operands[-1].imm, 0x228)
        self.assertEqual(body[0x5f88a20].operands[0].mem.disp, 0x110)

    def test_native_raid_marker_command_sets_and_moves_one_guid(self):
        from native_1392_machine import ReviewMachine
        m = ReviewMachine(self.image)
        server = m.scratch + 0x10000
        packet = server + 0x9000
        guid = 0xf00100abc
        m.write(server + 0x10e8, "I", 3)
        for icon in (3, 7):
            m.write(packet, "BQ", icon, guid)
            # Selected native command tag 0x321, with the same Win64 payload ABI.
            m.call(0x5ea4350, server, 0, 0, 0x321, packet, 9)
            self.assertEqual(m.read(server + 0x1fc0, "8Q"),
                             tuple(guid if index == icon else 0 for index in range(8)))

    def test_ui_aliases_distinguish_self_party_pets_and_retired_slots(self):
        from native_1392_machine import ReviewMachine
        body = {i.address: i for i in self.image.ins(0x2854a20)}
        self.assertEqual(body[0x2854df6].op_str, "qword ptr [r15 + 0x3308], rax")
        self.assertEqual(body[0x2854e22].op_str, "xmmword ptr [r15 + 0x3330], xmm0")
        self.assertEqual(body[0x2854e2a].op_str, "xmmword ptr [r15 + 0x3340], xmm1")
        m = ReviewMachine(self.image)
        root = m.scratch + 0x10000
        aliases, out = root + 0x18 + 0x3290, root + 0x5000
        own = 0x100000010
        m.write(aliases + 0x60, "Q", own)
        for index in range(4):
            for field, prefix in ((0x88, "party"), (0xa8, "partypet")):
                guid = own + 8 * field + index
                m.write(aliases + field + 8 * index, "Q", guid)
                m.call(0x29e0f60, out, aliases, guid)
                self.assertEqual(m.buffer(out), f"{prefix}{index + 1}".encode())
        m.write(aliases + 0x88, "Q", 0)
        for absent in (0, own, own + 8 * 0x88):
            m.call(0x29e0f60, out, aliases, absent)
            self.assertEqual(m.read(out, "q"), (-1,))

    def test_native_event_creation_and_per_slot_delivery(self):
        import struct
        from native_1392_machine import ReviewMachine
        machine = ReviewMachine(self.image)
        game = machine.scratch + 0x10000
        server = game - 16
        descriptor, position, metadata, name = (game + n for n in (0x2000, 0x2100, 0x2200, 0x2300))
        peer, stream = game + 0x30000, game + 0x40000
        queue, outbound = machine.allocate(32, 8), game + 0x60000
        machine.write(game + 0x1044, "I", 7)
        machine.write(game + 0xf70, "4Q", 133, 211, 343, 555)
        machine.write(game + 0x810, "3Q", 1, queue, 1)
        prior = bytes(range(32))
        machine.cpu.mem_write(queue, prior)
        machine.write(server + 0x10e8, "I", 3)
        machine.write(server + 0x1120, "2Q", peer, 1)
        machine.write(peer, "I", 1)
        machine.write(peer + 8, "Q", 1)
        machine.write(peer + 0x86d, "B", 1)
        machine.write(peer + 0x390, "3Q", 8, outbound, 0)
        machine.write(server + 0x15b0, "2Q", stream, 1)
        machine.write(stream + 0xa8, "B", 1)
        machine.write(stream + 0x90, "Q", stream + 0x800)

        guest = ReviewMachine(self.image)
        guest_server = guest.scratch + 0x10000
        message = guest_server + 0x8000
        guest_stream = guest_server + 0x10000
        guest.write(guest_server + 0x1575, "B", 1)
        guest.write(guest_server + 0x15b0, "2Q", guest_stream, 1)
        guest.write(guest_stream + 0xa8, "B", 1)
        guest.write(guest_stream + 0x90, "Q", guest_stream + 0x800)
        guest.write(guest_server + 0x11a8, "Q", message + 0x400)
        guest.cpu.mem_write(message + 0x400, bytes([255]) * 32)

        def deliver(packet, opcode, slots=(0, 1, 2)):
            self.assertEqual(machine.read(packet + 24, "H"), (opcode,))
            payload = machine.buffer(packet)
            count = machine.read(peer + 0x3a0, "Q")[0]
            local_count = len(machine.local_packets)
            for slot in slots:
                machine.call(0x5ed0a60, server, slot, packet)
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
            guest.call(0x5ed2450, guest_server, message)
            self.assertNotIn(owned, guest.allocations)
            self.assertEqual(guest.local_packets[-1], (guest_stream + 0x810,
                struct.pack(">H", len(payload) + 2) + struct.pack("<H", opcode) + payload))

        for serial, (entry, display, scale, title) in enumerate((
            (0xa50001, 36, 1.0, b"Lost Supplies"),
            (0xa50007, 2972, 0.75, b"Green Crystal"),
        ), 8):
            machine.cpu.mem_write(name, title)
            machine.call(0x5f04830, metadata, entry, 5, display, name, len(title))
            self.assertTrue(machine.buffer(metadata).startswith(struct.pack("<3I", entry, 5, display) + title + b"\0"))
            deliver(metadata, 0x5f)
            machine.write(descriptor, "IIf", entry, display, scale)
            xyz = (17.0, -13.25, 4.5)
            machine.write(position, "3f", *xyz)
            guid = machine.call(0x5f5fe60, game, descriptor, position, 5, 0)
            self.assertEqual(guid, (0xf110 << 48) | (entry << 24) | serial)
            capacity, current_queue, length = machine.read(game + 0x810, "3Q")
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
            borrowed = game + 0x81000
            machine.cpu.mem_write(borrowed + 32, body)
            machine.write(borrowed, "4Q", len(body), borrowed + 32, len(body), 0xa9)
            machine.write(game + 0x820, "Q", 1)
            capacity, buffer, _ = machine.read(row, "3Q")
            machine.call(0x21d0, buffer, capacity, 1)
            deliver(borrowed, 0xa9)
            # Personal hide goes to the guest only. A scene-wide removal reaches
            # both transports. Both carry the full GUID from native creation.
            machine.cpu.mem_write(borrowed + 32, guid_bytes)
            machine.write(borrowed, "4Q", 8, borrowed + 32, 8, 0xaa)
            deliver(borrowed, 0xaa, (1,) if serial == 8 else (0, 1))
        self.assertEqual(len(machine.local_packets), 5)

    def test_context_swap_exchanges_each_owned_byte_and_preserves_padding(self):
        from native_1392_machine import ReviewMachine
        m = ReviewMachine(self.image)
        game, actor = m.scratch + 0x20000, m.scratch + 0x24000
        size = 0xc08
        samples = []
        for shift in (0, 8):
            m.cpu.mem_write(game, bytes((i >> shift) & 255 for i in range(0x1200)))
            m.cpu.mem_write(actor, bytes(((0x4000 + i) >> shift) & 255 for i in range(size)))
            m.call(0x5f614f0, game, actor)
            samples.append((bytes(m.cpu.mem_read(game, 0x1200)), bytes(m.cpu.mem_read(actor, size))))
        host, private = ([lo | hi << 8 for lo, hi in zip(samples[0][k], samples[1][k])] for k in (0, 1))
        padding = {2889, 2890, 2891, 3073, 3074, 3075, 3076, 3077, 3078, 3079}
        exchanged = 0
        for index, source in enumerate(private):
            if index in padding:
                self.assertEqual(source, 0x4000 + index)
            else:
                self.assertLess(source, len(host))
                self.assertEqual(host[source], 0x4000 + index)
                exchanged += 1
        self.assertEqual(exchanged, 3070)

    def native_reward_machine(self):
        from native_1392_machine import CatalogMachine
        m = CatalogMachine(self.image)
        game, profile = m.scratch + 0x10000, m.scratch + 0x18000
        m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
        return m, game

    def native_reward_entry(self, name):
        return dict(self.values["EXTENSION_ADDRESSES"])[self.values[name + "_RVA"]]

    def force_native_amulet_pool(self, machine, game):
        import struct
        from unicorn import UC_HOOK_CODE, x86_const as reg
        calls = []
        entry, guard = self.values["GOBLIN_AMULET_POOL"]
        self.assertEqual(self.image.pe.get_data(entry, len(guard)), bytes(guard))
        def force(cpu, address, size, user):
            frame = cpu.reg_read(reg.UC_X86_REG_RBP)
            self.assertEqual(machine.read(frame + self.values["GOBLIN_REWARD_GAME_FRAME"], "Q"), (game,))
            header = frame + self.values["GOBLIN_REWARD_POOL_FRAME"]
            capacity, pointer, count = machine.read(header, "3Q")
            self.assertGreaterEqual(capacity, 3)
            self.assertLessEqual(count, capacity)
            machine.cpu.mem_write(pointer, b"".join(struct.pack("<fIQ8x", 1.0, 0, 1) for _ in range(3)))
            machine.write(header + 16, "Q", 3)
            cpu.reg_write(reg.UC_X86_REG_R15, 3)
            calls.append(game)
        token = machine.cpu.hook_add(UC_HOOK_CODE, force, begin=machine.base + entry, end=machine.base + entry)
        return token, calls

    def test_native_amulet_offer_accepts_once_and_uses_the_selected_pointer_aliases(self):
        m, game = self.native_reward_machine()
        phase, bank = self.field(0xc46), 0x1028
        m.write(game + phase, "B", 1)
        m.write(game + bank, "I", 1)
        token, calls = self.force_native_amulet_pool(m, game)
        m.call(self.values["ITEM_CHOICE_RVA"], game, 1, 1)
        m.cpu.hook_del(token)
        self.assertEqual(calls, [game])
        self.assertEqual(m.read(game + phase, "B"), (2,))
        self.assertEqual(m.read(game + self.field(0xc44), "B"), (1,))
        capacity, pointer, count = m.read(game + self.field(0x660), "3Q")
        self.assertGreaterEqual(capacity, count)
        self.assertEqual(count, 3)
        # Production goblin/owner readers borrow the pointer, never capacity.
        self.assertEqual(m.read(game + self.field(0x668), "Q"), (pointer,))
        self.assertEqual(m.read(game + self.field(0x678, "FIELD1201_RESOLVED_FIELDS"), "Q"), (pointer,))
        self.assertEqual(m.read(game + self.field(0x670), "Q"), (count,))
        self.assertEqual(m.read(game + self.field(0x680, "FIELD1201_RESOLVED_FIELDS"), "Q"), (count,))
        self.assertEqual(m.read(pointer + 0x78, "Q"), (self.values["AMULET_CHOICE_TAG"],))
        definition = m.read(pointer + 0x80, "Q")[0]
        key, length = m.read(definition + 8, "2Q")
        selected = m.bytes_at(key, length)
        self.assertIn(selected.decode(), self.values["AMULET_KEYS"])
        accept = self.native_reward_entry("REWARD_ACCEPT")
        before = bytes(m.cpu.mem_read(game, 0x1200))
        m.call(accept, game, count)
        self.assertEqual(bytes(m.cpu.mem_read(game, 0x1200)), before)
        m.call(accept, game, 0)
        owned = game + self.field(self.values["AMULET_OWNED"], "FIELD1201_RESOLVED_FIELDS")
        self.assertEqual(m.buffer(owned), selected)
        self.assertEqual(m.read(game + phase, "B"), (1,))
        self.assertEqual(m.read(game + bank, "I"), (0,))
        self.assertEqual(m.read(game + self.field(0x670), "Q"), (0,))
        after = bytes(m.cpu.mem_read(game, 0x1200))
        m.call(accept, game, 0)
        self.assertEqual(bytes(m.cpu.mem_read(game, 0x1200)), after)

    def test_native_ring_card_grants_then_upgrades_and_closes_both_choices(self):
        m, game = self.native_reward_machine()
        descriptor = m.scratch + 0x22000
        ring = m.base + self.values["RING_TABLE"]
        m.write(descriptor, "3Q", 0x8000000000000002, ring, 0)
        inventory_pointer, inventory_count = self.values["RING_INVENTORY"]
        for rank, expected_power in ((1, 0.255), (2, 0.4124074)):
            with self.subTest(rank=rank):
                output = m.allocate(160, 8)
                self.assertEqual(m.call(self.native_reward_entry("REWARD_CARD"), output, game, descriptor, 0), output)
                self.assertEqual(m.read(output + 0x78, "2Q"), (0x8000000000000002, ring))
                m.write(game + self.field(0x660), "3Q", 1, output, 1)
                m.write(game + self.field(0xc46), "B", 2)
                m.write(game + self.field(0xc44), "B", 1)
                m.write(game + 0x1028, "I", 1)
                m.call(self.native_reward_entry("REWARD_ACCEPT"), game, 0)
                pointer = m.read(game + self.field(inventory_pointer, "FIELD1201_RESOLVED_FIELDS"), "Q")[0]
                self.assertEqual(m.read(game + self.field(inventory_count, "FIELD1201_RESOLVED_FIELDS"), "Q"), (1,))
                definition, _, power, actual_rank = m.read(pointer, "2QfI")
                self.assertEqual((definition, actual_rank), (ring, rank))
                self.assertAlmostEqual(power, expected_power, places=6)
                self.assertEqual(m.read(game + self.field(0xc46), "B"), (1,))
                self.assertEqual(m.read(game + 0x1028, "I"), (0,))
                self.assertEqual(m.read(game + self.field(0x670), "Q"), (0,))

    def test_native_rerolls_keep_free_paid_and_rejected_transactions(self):
        m, game = self.native_reward_machine()
        phase = self.field(0xc46)
        free = self.field(self.values["REWARD_REROLLS"], "FIELD117_RESOLVED_FIELDS")
        paid = free + 4
        gold = self.field(0x500)
        m.write(game + phase, "B", 1)
        m.write(game + 0x1028, "I", 1)
        token, calls = self.force_native_amulet_pool(m, game)
        m.call(self.values["ITEM_CHOICE_RVA"], game, 1, 1)
        for stage, available, spent, balance, expected in (
            (2, 1, 0, 500, (0, 0, 500, 1)),
            (2, 0, 0, 500, (0, 1, 400, 1)),
            (2, 0, 1, 500, (0, 2, 200, 1)),
            (2, 0, 0, 99, (0, 0, 99, 0)),
            (1, 1, 0, 500, (1, 0, 500, 0)),
        ):
            with self.subTest(phase=stage, free=available, paid=spent, gold=balance):
                m.write(game + phase, "B", stage)
                m.write(game + free, "I", available)
                m.write(game + paid, "I", spent)
                m.write(game + gold, "I", balance)
                before = len(calls)
                m.call(self.native_reward_entry("REWARD_REROLL"), game)
                actual = (m.read(game + free, "I")[0], m.read(game + paid, "I")[0],
                          m.read(game + gold, "I")[0], len(calls) - before)
                self.assertEqual(actual, expected)
        m.cpu.hook_del(token)

    def test_native_amulet_exclusion_preserves_order_and_drops_only_removed_owners(self):
        from native_1392_machine import ReviewMachine
        entry, guard = self.values["GOBLIN_AMULET_EXCLUDE"]
        self.assertEqual(self.image.pe.get_data(entry, len(guard)), bytes(guard))
        keys = ["alpha", "beta", "gamma", "beta", "delta"]
        for owned, excluded in ((None, []), ("alpha", []), (None, ["beta"]),
                                ("alpha", ["beta", "delta"]), ("gamma", ["alpha", "beta", "delta"]),
                                ("missing", ["unknown"])):
            with self.subTest(owned=owned, excluded=excluded):
                m = ReviewMachine(self.image)
                pool, owner, denied = (m.scratch + offset for offset in (0x10000, 0x11000, 0x12000))
                rows = m.allocate(len(keys) * 112 + 16, 8)
                resources = []
                for index, key in enumerate(keys):
                    key_data = m.allocate(len(key), 1)
                    m.cpu.mem_write(key_data, key.encode())
                    title, perks = m.allocate(4, 1), m.allocate(64, 8)
                    m.cpu.mem_write(title, b"name")
                    resources.append({key_data, title, perks})
                    m.write(rows + index * 112, "3Q", len(key), key_data, len(key))
                    m.write(rows + index * 112 + 24, "3Q", 4, title, 4)
                    m.write(rows + index * 112 + 48, "3Q", 1, perks, 1)
                    m.write(rows + index * 112 + 104, "Q", index)
                m.cpu.mem_write(rows + len(keys) * 112, b"BOUNDARY-CANARY!")
                m.write(pool, "3Q", len(keys), rows, len(keys))
                def borrowed_string(header, key):
                    data = m.scratch + 0x20000 + (header - m.scratch) * 2
                    m.cpu.mem_write(data, key.encode())
                    m.write(header, "3Q", len(key), data, len(key))
                if owned is None:
                    m.write(owner, "3Q", (1 << 64) - 1, 0, 0)
                else:
                    borrowed_string(owner, owned)
                deny_rows = m.scratch + 0x18000
                m.write(denied, "3Q", len(excluded), deny_rows, len(excluded))
                for index, key in enumerate(excluded):
                    borrowed_string(deny_rows + index * 24, key)
                borrowed = (bytes(m.cpu.mem_read(owner, 24)), bytes(m.cpu.mem_read(denied, 24)),
                            bytes(m.cpu.mem_read(deny_rows, len(excluded) * 24)))
                before = set(m.allocations)
                m.call(entry, pool, owner, denied)
                capacity, pointer, count = m.read(pool, "3Q")
                expected = [index for index, key in enumerate(keys) if key != owned and key not in excluded]
                self.assertEqual((capacity, pointer, count), (len(keys), rows, len(expected)))
                self.assertEqual([m.read(pointer + i * 112 + 104, "Q")[0] for i in range(count)], expected)
                self.assertEqual([m.buffer(pointer + i * 112).decode() for i in range(count)], [keys[i] for i in expected])
                freed = set().union(*(resources[i] for i in range(len(keys)) if i not in expected))
                self.assertEqual(before - set(m.allocations), freed)
                self.assertEqual(bytes(m.cpu.mem_read(rows + len(keys) * 112, 16)), b"BOUNDARY-CANARY!")
                self.assertEqual(borrowed, (bytes(m.cpu.mem_read(owner, 24)), bytes(m.cpu.mem_read(denied, 24)),
                                            bytes(m.cpu.mem_read(deny_rows, len(excluded) * 24))))


    def test_retained_entry_guards_cannot_disappear_when_addresses_are_translated(self):
        from binding_registry import parse_literal
        baseline = json.loads((ROOT / "compat/clients/0.139.2.json").read_text())
        previous = {r["name"]: parse_literal(r["value"]) for r in baseline["constants"]}
        for table in ("ENTRIES", "EXTENSION_ENTRIES"):
            self.assertFalse(set(dict(previous[table])) - set(dict(self.values[table])), table)

    def test_reviewed_entry_guards_match_complete_instructions_in_the_pinned_image(self):
        from capstone import Cs, CS_ARCH_X86, CS_MODE_64
        addresses = dict(self.values["ADDRESSES"])
        extensions = dict(self.values["EXTENSION_ADDRESSES"])
        disassembler = Cs(CS_ARCH_X86, CS_MODE_64)
        named = [(name, value) for name, value in self.values.items()
                 if isinstance(value, tuple) and len(value) == 2
                 and isinstance(value[0], int) and isinstance(value[1], list)]
        named += [("tooltip", row) for row in self.values["KEPT_TOOLTIP_BINDINGS"]]
        for name, (entry, guard) in named:
            with self.subTest(named=name, entry=hex(entry)):
                raw = bytes(guard)
                self.assertGreaterEqual(len(raw), 5)
                self.assertEqual(self.image.pe.get_data(entry, len(raw)), raw)
                self.assertEqual(sum(op.size for op in disassembler.disasm(raw, entry)), len(raw))
        for table, locations in (("ENTRIES", addresses), ("EXTENSION_ENTRIES", extensions)):
            for canonical, guard in self.values[table]:
                with self.subTest(table=table, entry=hex(canonical)):
                    entry = locations[canonical]
                    raw = bytes(guard)
                    self.assertGreaterEqual(len(raw), 5)
                    self.assertEqual(self.image.pe.get_data(entry, len(raw)), raw)
                    self.assertEqual(sum(op.size for op in disassembler.disasm(raw, entry)), len(raw))

    def test_native_hud_export_reads_selected_health_gold_and_run_phase(self):
        m, game = self.native_reward_machine()
        output = m.scratch + 0x30000
        entry = dict(self.values["ADDRESSES"])[0x590f0f0]
        def read_hud():
            self.assertEqual(m.call(entry, output, game), output)
            value = json.loads(m.buffer(output))
            capacity, pointer, length = m.read(output, "3Q")
            self.assertLessEqual(length, capacity)
            m.call(0x21d0, pointer, capacity, 1)
            return value
        initial = read_hud()
        self.assertEqual((initial["type"], initial["phase"], initial["level"], initial["hp"]),
                         ("hud", "ready", 1, 230))
        m.write(game + self.field(0xc46), "B", 1)
        m.write(game + self.field(0x4f8), "f", 92.0)
        m.write(game + self.field(0x500), "I", 42)
        m.write(game + self.field(0xb9c), "f", 1.25)
        active = read_hud()
        self.assertEqual((active["phase"], active["hp"], active["hpMax"], active["gold"], active["time"]),
                         ("playing", 92, 230, 42, 1.25))


    def test_native_hurt_borrows_source_metadata_and_keeps_health_and_shield_mitigation(self):
        import struct
        from unicorn import x86_const as reg
        m, game = self.native_reward_machine()
        source, row = m.scratch + 0x35000, m.scratch + 0x30000
        m.write(source, "II2Q", 1, 17, 0, 0)
        borrowed = bytes(m.cpu.mem_read(source, 24))
        for phase, shield, expected_hp, expected_absorb in (
            (0, False, 230.0, 0.0), (1, False, 222.2, 0.0), (1, True, 230.0, 107.2)
        ):
            with self.subTest(phase=phase, shield=shield):
                m.write(game + self.field(0xc46), "B", phase)
                m.write(game + self.field(0x4f8), "f", 230.0)
                if shield:
                    m.write(row, "2Q3fI2fI", 0, 0, 0.0, 0.0, 0.0, 17, 0.5, 10.0, 15)
                    m.call(dict(self.values["ADDRESSES"])[0x58ffac0], game, row)
                m.cpu.reg_write(reg.UC_X86_REG_XMM1, struct.unpack("<I", struct.pack("<f", 10.0))[0])
                m.call(dict(self.values["ADDRESSES"])[0x5908cc0], game, 0, 0, 0, source)
                self.assertAlmostEqual(m.read(game + self.field(0x4f8), "f")[0], expected_hp, places=4)
                self.assertAlmostEqual(m.read(game + self.field(0x4fc), "f")[0], expected_absorb, places=4)
                self.assertEqual(bytes(m.cpu.mem_read(source, 24)), borrowed)
        self.assertEqual(self.values["HURT_SOURCE_LAYOUT"], 1)
        body = self.image.ins(self.image.owner(dict(self.values["ADDRESSES"])[0x5908cc0]))
        self.assertTrue(any(op.op_str == "rdi, qword ptr [rbp + 0x1c0]" for op in body))
        self.assertFalse(any("[rbp + 0x1c8]" in op.op_str for op in body))


    def test_native_session_callbacks_keep_their_distinct_float_arguments(self):
        import struct
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        m = ReviewMachine(self.image)
        server = m.scratch + 0x10000
        m.write(server, "Qd", 0, 1.0)
        timestamp = 3.25
        m.cpu.reg_write(reg.UC_X86_REG_XMM1, struct.unpack("<Q", struct.pack("<d", timestamp))[0])
        m.stop = m.base + 0x5ea8c99
        m.call(dict(self.values["ADDRESSES"])[0x5877810], server)
        self.assertEqual(m.read(server, "Qd"), (1, timestamp))
        # Stop at the native guest update tail; this establishes forwarding,
        # not whole-world simulation or live transport behavior.
        m = ReviewMachine(self.image)
        game = m.scratch + 0x10000
        m.write(game + self.field(0xc41), "B", 1)
        delta_bits = struct.unpack("<I", struct.pack("<f", 0.125))[0]
        m.cpu.reg_write(reg.UC_X86_REG_XMM1, delta_bits)
        m.stop = m.base + 0x5f2e510
        m.call(dict(self.values["ADDRESSES"])[0x5919240], game)
        self.assertEqual(m.cpu.reg_read(reg.UC_X86_REG_RCX), game)
        self.assertEqual(m.cpu.reg_read(reg.UC_X86_REG_XMM1) & 0xffffffff, delta_bits)
        self.assertEqual(m.cpu.reg_read(reg.UC_X86_REG_RSP), m.stack)

    def test_native_shared_xp_wrapper_keeps_request_ownership_and_self_exclusion(self):
        from unicorn import UC_HOOK_CODE, x86_const as reg
        m, game = self.native_reward_machine()
        request, header, receipts, language = (m.scratch + offset for offset in (0x30000, 0x31000, 0x32000, 0x34000))
        m.write(game + self.field(0xc46), "B", 1)
        m.write(game + self.field(0xb40), "Q", 0)
        m.write(game + 0xfd0, "Q", 2)
        m.write(game + self.field(0xc41), "B", 1)
        m.write(request, "Q", header)
        m.cpu.mem_write(language, b"fr")
        awards = []
        award = dict(self.values["ADDRESSES"])[0x591db30]
        def capture(cpu, address, size, user):
            stack = cpu.reg_read(reg.UC_X86_REG_RSP)
            awards.append((cpu.reg_read(reg.UC_X86_REG_RCX), cpu.reg_read(reg.UC_X86_REG_RDX) & 0xffffffff,
                           m.read(stack, "Q")[0] - m.base))
            cpu.reg_write(reg.UC_X86_REG_RIP, m.read(stack, "Q")[0])
            cpu.reg_write(reg.UC_X86_REG_RSP, stack + 8)
        token = m.cpu.hook_add(UC_HOOK_CODE, capture, begin=m.base + award, end=m.base + award)
        for rows, expected in (([], []), ([(0, 100)], []),
                               ([(0, 100), (1, 2), (2, 1)], [(game, 3, 0x5e736bf)]),
                               ([(2, 1), (1, 2), (0, 100)], [(game, 3, 0x5e736bf)])):
            with self.subTest(receipts=rows):
                for index, (owner, amount) in enumerate(rows):
                    m.write(receipts + index * 16, "QII", owner, amount, 0)
                m.write(header, "3Q", len(rows), receipts, len(rows))
                before = (bytes(m.cpu.mem_read(request, 8)), bytes(m.cpu.mem_read(header, 24)),
                          bytes(m.cpu.mem_read(receipts, len(rows) * 16)))
                count = len(awards)
                m.call(dict(self.values["ADDRESSES"])[0x58daa10], language, 2, request, game)
                self.assertEqual(awards[count:], expected)
                self.assertEqual(before, (bytes(m.cpu.mem_read(request, 8)), bytes(m.cpu.mem_read(header, 24)),
                                          bytes(m.cpu.mem_read(receipts, len(rows) * 16))))
        m.cpu.hook_del(token)


    def test_native_class_icon_rectangles_cover_every_byte_input_without_overwrite(self):
        from native_1392_machine import ReviewMachine
        m = ReviewMachine(self.image)
        output, definition = m.scratch + 0x10000, m.scratch + 0x11000
        expected = {1: (0., 0., 64., 64.), 2: (0., 128., 64., 192.),
                    3: (0., 64., 64., 128.), 4: (127., 0., 190., 64.),
                    5: (127., 64., 190., 128.), 7: (64., 64., 127., 128.),
                    8: (64., 0., 127., 64.), 9: (190., 64., 253., 128.)}
        for class_id in range(256):
            m.write(definition + 0x8c, "B", class_id)
            m.cpu.mem_write(output, b"\xcc" * 32)
            self.assertEqual(m.call(dict(self.values["ADDRESSES"])[0xfffa0], output, definition), output)
            self.assertEqual(m.read(output, "4f"), expected.get(class_id, (190., 0., 253., 64.)))
            self.assertEqual(bytes(m.cpu.mem_read(output + 16, 16)), b"\xcc" * 16)

    def test_native_separator_constructor_keeps_the_selected_node_extent_and_owned_gradients(self):
        from native_1392_machine import ReviewMachine
        m = ReviewMachine(self.image)
        output, node, color = (m.scratch + offset for offset in (0x10000, 0x11000, 0x12000))
        size = self.values["NODE_SIZE"]
        self.assertEqual(size, 384)
        for index in range(4):
            m.write(node + index * 24, "3Q", 0, 8, 0)
        m.write(node + self.values["BONUS_UI_HEADER_WIDTH"], "If", 1, 208.0)
        m.write(node + self.values["BONUS_UI_HEADER_HEIGHT"], "If", 1, 40.0)
        m.write(node + self.values["NODE_COLUMN_GAP"], "If", 1, 6.0)
        m.write(node + self.values["BONUS_UI_HEADER_DIRECTION"], "B", 1)
        m.write(color, "I4f", 0, .25, .5, .75, 1.0)
        original = bytes(m.cpu.mem_read(node, size)), bytes(m.cpu.mem_read(color, 20))
        m.cpu.mem_write(output, b"\xcc" * (size + 48))
        entry, guard = self.values["BONUS_SEPARATOR_BUILD"]
        self.assertEqual(self.image.pe.get_data(entry, len(guard)), bytes(guard))
        self.assertEqual(m.call(entry, output, node, color), output)
        self.assertEqual(m.read(output + self.values["BONUS_UI_HEADER_WIDTH"], "If"), (1, 208.0))
        self.assertEqual(m.read(output + self.values["BONUS_UI_HEADER_HEIGHT"], "If"), (1, 1.0))
        self.assertEqual(m.read(output + self.values["NODE_COLUMN_GAP"], "If"), (1, 6.0))
        self.assertEqual(m.read(output + size, "Q")[0], 1)
        self.assertEqual(m.read(output + size + 16, "Q")[0], 1)
        self.assertEqual(bytes(m.cpu.mem_read(output + size + 24, 24)), b"\xcc" * 24)
        self.assertEqual(original, (bytes(m.cpu.mem_read(node, size)), bytes(m.cpu.mem_read(color, 20))))
        self.assertEqual(sorted(m.allocations.values()), [(72, 8), (96, 4)])
        for pointer, (capacity, alignment) in list(m.allocations.items()):
            m.call(0x21d0, pointer, capacity, alignment)
        self.assertFalse(m.allocations)

    def test_native_card_prefix_retains_eleven_arguments_and_the_header_node_tuple(self):
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        m = ReviewMachine(self.image)
        parent, image, rectangle, text, color, action = (m.scratch + offset for offset in
                                                        (0x10000, 0x11000, 0x12000, 0x13000, 0x14000, 0x15000))
        identity = 0x1122334455667788
        font = 0x1234567887654321
        m.write(parent, "6Q", identity, 0, 0, 0, 0, 0)
        m.write(image, "3Q", 1, 0, 0)
        m.write(rectangle, "4f", 0., 0., 64., 64.)
        m.cpu.mem_write(text, b"Note")
        m.write(color, "I4f", 0, .25, .5, .75, 1.)
        spawn, guard = self.values["BONUS_COLUMN_SPAWN"]
        self.assertEqual(self.image.pe.get_data(spawn, len(guard)), bytes(guard))
        m.stop = m.base + spawn
        m.call(dict(self.values["ADDRESSES"])[0x100a20], parent, font, image, rectangle,
               0x42000000, text, 4, 0, 1, color, action)
        frame = m.cpu.reg_read(reg.UC_X86_REG_RBP)
        bundle = m.cpu.reg_read(reg.UC_X86_REG_R8)
        self.assertEqual(m.read(bundle, "Q"), (identity,))
        self.assertEqual(m.cpu.reg_read(reg.UC_X86_REG_RDX), parent + 8)
        self.assertEqual(m.cpu.reg_read(reg.UC_X86_REG_RSI), font)
        self.assertEqual(m.cpu.reg_read(reg.UC_X86_REG_RDI), rectangle)
        self.assertEqual(m.read(frame + 0x2b5, "2B"), (1, 0))
        node = bundle + self.values["BONUS_UI_HEADER_NODE"]
        self.assertEqual(self.values["BONUS_UI_HEADER_NODE"], 8)
        self.assertEqual(m.read(node + self.values["BONUS_UI_HEADER_DIRECTION"], "B"), (1,))
        for index in range(4):
            self.assertEqual(m.read(node + index * 24, "3Q"), (0, (8, 8, 4, 4)[index], 0))
        self.assertEqual(m.read(m.cpu.reg_read(reg.UC_X86_REG_RSP), "Q")[0] - m.base,
                         self.values["BONUS_HEADER_CALLER"])
        self.assertEqual(dict(self.values["EXTENSION_ADDRESSES"])[self.values["BONUS_UI_HEADER_CALLER"]],
                         self.values["BONUS_HEADER_CALLER"])

    def test_native_event_height_keeps_option_registers_grid_bounds_and_nan_rejection(self):
        import struct
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        m = ReviewMachine(self.image)
        terrain, heights = m.scratch + 0x10000, m.scratch + 0x11000
        entry, guard = self.values["EVENT_GROUND_HEIGHT"]
        self.assertEqual(self.image.pe.get_data(entry, len(guard)), bytes(guard))
        def sample(x, y):
            for register, value in ((reg.UC_X86_REG_XMM1, x), (reg.UC_X86_REG_XMM2, y)):
                m.cpu.reg_write(register, struct.unpack("<I", struct.pack("<f", value))[0])
            present = m.call(entry, terrain) & 1
            bits = m.cpu.reg_read(reg.UC_X86_REG_XMM0) & 0xffffffff
            return struct.unpack("<f", struct.pack("<I", bits))[0] if present else None
        m.write(terrain + 0xc4, "B", 1)
        m.write(terrain + 0xc0, "f", 12.5)
        self.assertEqual(sample(10., -20.), 12.5)
        m.write(terrain + 0xc4, "B", 0)
        m.write(terrain + 0x58, "Q", 0)
        m.write(terrain + 0x70, "4Q", 16, heights, 16, 4)
        m.write(terrain + 0x90, "2f", 0., 0.)
        m.write(heights, "16f", *[float(i + 1) for i in range(16)])
        self.assertEqual(sample(.75, .25), 2.0)
        self.assertEqual(sample(1.75, 1.75), 16.0)
        self.assertIsNone(sample(-.25, .25))
        self.assertIsNone(sample(2., .25))
        m.write(heights + 4, "f", float("nan"))
        self.assertIsNone(sample(.75, .25))
        # Native placement obtains this exact Rc field before borrowing +16.
        placement = self.image.ins(self.image.owner(self.values["GOBLIN_POSITION"][0]))
        self.assertTrue(any(op.op_str == f"rbx, qword ptr [rdi + {hex(self.values['EVENT_TERRAIN_OFFSET'])}]"
                            for op in placement))
        self.assertTrue(any(op.mnemonic == "call" and op.operands[0].imm == entry for op in placement))

    def test_native_navigation_clip_preserves_goal_or_stops_before_a_blocked_cell(self):
        from native_1392_machine import ReviewMachine
        m = ReviewMachine(self.image)
        grid, data, output, origin, goal = (m.scratch + offset for offset in
                                           (0x10000, 0x11000, 0x12000, 0x13000, 0x14000))
        m.write(grid, "4Q2f", 64, data, 64, 8, -4., -4.)
        m.write(origin, "3f", 0., 0., 3.)
        m.write(goal, "3f", 2., 0., 8.)
        entry, guard = self.values["GOBLIN_CLIP"]
        self.assertEqual(self.image.pe.get_data(entry, len(guard)), bytes(guard))
        for blocked in [False, True]:
            m.cpu.mem_write(data, bytes(64))
            if blocked:
                for y in range(8):
                    m.write(data + y * 8 + 5, "B", 1)
            m.cpu.mem_write(output, b"\xcc" * 28)
            m.call(entry, output, grid, origin, goal)
            x, y, z = m.read(output, "3f")
            if blocked:
                self.assertAlmostEqual(x, 0.3571428657, places=6)
                self.assertEqual(y, 0.)
                self.assertAlmostEqual(z, 3.8928570747, places=6)
            else:
                self.assertEqual((x, y, z), (2., 0., 8.))
            self.assertEqual(bytes(m.cpu.mem_read(output + 12, 16)), b"\xcc" * 16)

    def test_native_event_wire_builders_keep_spline_stack_values_and_announcement_lengths(self):
        from native_1392_machine import ReviewMachine
        m = ReviewMachine(self.image)
        output, origin, goal, name, text = (m.scratch + offset for offset in
                                           (0x10000, 0x11000, 0x12000, 0x13000, 0x14000))
        m.write(origin, "3f", 1., 2., 3.)
        m.write(goal, "3f", 4., 6., 8.)
        spline, guard = self.values["SOLO_SCENE_SPLINE"]
        self.assertEqual(self.image.pe.get_data(spline, len(guard)), bytes(guard))
        self.assertEqual(m.call(spline, output, 0xf130123400000010, origin, goal, 0x11223344, 1500), output)
        self.assertEqual(m.buffer(output).hex(),
                         "f110341230f10000803f0000004000004040443322110000010000dc05000001000000000080400000c04000000041")
        capacity, pointer, _ = m.read(output, "3Q")
        m.call(0x21d0, pointer, capacity, 1)
        sender, message = b"Lost Supplies", b"A cache rests among the ruins."
        m.cpu.mem_write(name, sender)
        m.cpu.mem_write(text, message)
        notice, guard = self.values["GOBLIN_EVENT"]
        self.assertEqual(self.image.pe.get_data(notice, len(guard)), bytes(guard))
        self.assertEqual(m.call(notice, output, name, len(sender), text, len(message)), output)
        self.assertEqual(m.buffer(output).hex(),
                         "5a000000000e0000004c6f737420537570706c6965730000000000000000001f0000004120636163686520726573747320616d6f6e6720746865207275696e732e0000")
        self.assertEqual(bytes(m.cpu.mem_read(name, len(sender))), sender)
        self.assertEqual(bytes(m.cpu.mem_read(text, len(message))), message)
        capacity, pointer, _ = m.read(output, "3Q")
        m.call(0x21d0, pointer, capacity, 1)
        self.assertFalse(m.allocations)

    def test_native_exhausted_shield_exposes_the_consumed_spell_and_absorbed_budget(self):
        import struct
        from unicorn import UC_HOOK_CODE, x86_const as reg
        m, game = self.native_reward_machine()
        row, source = m.scratch + 0x30000, m.scratch + 0x35000
        m.write(game + self.field(0xc46), "B", 1)
        m.write(row, "2Q3fI2fI", 0, 0, 0., 0., 0., 17, .5, 10., 15)
        m.call(dict(self.values["ADDRESSES"])[0x58ffac0], game, row)
        m.write(source, "II2Q", 1, 17, 0, 0)
        captured = []
        seam = dict(self.values["ADDRESSES"])[0x59090c5]
        registers = (reg.UC_X86_REG_RAX, reg.UC_X86_REG_RCX, reg.UC_X86_REG_RDX, reg.UC_X86_REG_R8,
                     reg.UC_X86_REG_R9, reg.UC_X86_REG_R10, reg.UC_X86_REG_R11, reg.UC_X86_REG_RBX,
                     reg.UC_X86_REG_RBP, reg.UC_X86_REG_RSI, reg.UC_X86_REG_RDI, reg.UC_X86_REG_R12,
                     reg.UC_X86_REG_R13, reg.UC_X86_REG_R14, reg.UC_X86_REG_R15)
        def observe(cpu, address, size, user):
            values = [struct.unpack("<f", struct.pack("<I", cpu.reg_read(register) & 0xffffffff))[0]
                      for register in (reg.UC_X86_REG_XMM0, reg.UC_X86_REG_XMM1)]
            captured.append((cpu.reg_read(registers[self.values["SHIELD_BREAK_GAME_REGISTER"]]),
                             cpu.reg_read(registers[self.values["SHIELD_BREAK_SPELL_REGISTER"]]) & 0xffffffff,
                             values))
        token = m.cpu.hook_add(UC_HOOK_CODE, observe, begin=m.base + seam, end=m.base + seam)
        m.cpu.reg_write(reg.UC_X86_REG_XMM1, struct.unpack("<I", struct.pack("<f", 300.))[0])
        m.call(dict(self.values["ADDRESSES"])[0x5908cc0], game, 0, 0, 0, source)
        m.cpu.hook_del(token)
        self.assertEqual(captured, [(game, 17, [115., 115.])])
        self.assertAlmostEqual(m.read(game + self.field(0x4f8), "f")[0], 111., places=4)


    def test_ui_context_lookup_keeps_owned_result_and_balanced_borrows(self):
        from native_1392_machine import ReviewMachine
        from unicorn import UC_HOOK_CODE
        from unicorn import x86_const as reg
        # HashMap's key and Any's inner type differ. Only std::any::type_id is
        # supplied by the fixture vtable; native lookup/downcast/refcounts run.
        for case in ("empty", "missing", "valid", "wrong-type"):
            with self.subTest(case=case):
                m = ReviewMachine(self.image)
                app, ctrl, any_rc, vtable = (m.scratch + n for n in (0x10000, 0x12000, 0x14000, 0x16000))
                key = self.image.pe.get_data(0x6a12d70, 16)
                expected = self.image.pe.get_data(0x6a12d50, 16)
                if case != "empty":
                    m.write(app + 0x28, "2Q", ctrl, 15)
                    m.write(app + 0x40, "Q", 1)
                    m.cpu.mem_write(ctrl, bytes([255]) * 32)
                    if case != "missing":
                        bucket = ctrl - 12 * 32
                        m.cpu.mem_write(ctrl + 11, bytes([0x2d]))
                        m.cpu.mem_write(bucket, key)
                        m.write(bucket + 16, "2Q", any_rc, vtable)
                        m.write(any_rc, "2Q", 1, 1)
                        m.write(any_rc + 24, "Q", 0x1234567890abcdef)
                service = m.scratch + 0x2000
                m.cpu.mem_write(service, bytes([0xc3]))
                m.write(vtable + 16, "2Q", 8, service)
                def type_id(cpu, address, size, user):
                    self.assertEqual(cpu.reg_read(reg.UC_X86_REG_RDX), any_rc + 16)
                    cpu.mem_write(cpu.reg_read(reg.UC_X86_REG_RCX), expected if case == "valid" else key)
                m.cpu.hook_add(UC_HOOK_CODE, type_id, begin=service, end=service)
                result = m.call(self.values["UI_CONTEXT_LOOKUP"][0], app)
                self.assertEqual(result, any_rc if case == "valid" else 0)
                self.assertEqual(m.read(app + 0x20, "Q"), (0,))
                if case in ("valid", "wrong-type"):
                    self.assertEqual(m.read(any_rc, "Q"), (2 if case == "valid" else 1,))
                if result:
                    self.assertEqual(m.read(result + 24, "Q"), (0x1234567890abcdef,))

    def test_native_spline_sequence_is_incremented_and_passed_to_the_packet(self):
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        import struct
        for previous in (0, 0xfffffffe, 0xffffffff):
            with self.subTest(previous=previous):
                m = ReviewMachine(self.image)
                game, frame, out, origin, goal = (m.scratch + n for n in (0x10000, 0x14000, 0x18000, 0x1a000, 0x1c000))
                m.write(game + self.values["SOLO_SCENE_SEQUENCE"], "I", previous)
                m.write(origin, "3f", 1., 2., 3.)
                m.write(goal, "3f", 4., 6., 8.)
                # Start at the actual native load, stop immediately after its
                # actual packet-builder call. The caller and callee both run.
                m.stop = m.base + 0x5f5fbf1
                for register, value in ((reg.UC_X86_REG_RSI, game), (reg.UC_X86_REG_RBP, frame),
                                        (reg.UC_X86_REG_RBX, out), (reg.UC_X86_REG_R14, origin),
                                        (reg.UC_X86_REG_R15, goal), (reg.UC_X86_REG_RDX, 0xf130123400000010)):
                    m.cpu.reg_write(register, value)
                m.call(0x5f5fb9c)
                sequence = (previous + 1) & 0xffffffff
                self.assertEqual(m.read(game + self.values["SOLO_SCENE_SEQUENCE"], "I"), (sequence,))
                self.assertEqual(m.buffer(out)[18:22], struct.pack("<I", sequence))
                self.assertEqual(m.read(m.stack + 0x20, "I"), (sequence,))
                cap, pointer, _ = m.read(out, "3Q")
                m.stop = m.scratch
                m.call(0x21d0, pointer, cap, 1)


    def test_native_heal_packet_and_queue_growth_preserve_every_owned_row(self):
        from native_1392_machine import ReviewMachine
        m = ReviewMachine(self.image)
        out, queue = m.scratch + 0x10000, m.scratch + 0x12000
        m.write(queue, "3Q", 0, 8, 0)
        m.call(self.values["HEAL_LOG_RESERVE"][0], queue)
        cap, pointer, length = m.read(queue, "3Q")
        self.assertEqual((cap, length), (4, 0))
        for critical in (0, 1):
            m.cpu.mem_write(out + 32, b"CANARY!!")
            result = m.call(self.values["HEAL_LOG_PACKET"][0], out,
                            0xf130123400000010, 0xf130123400000020, 331, 123, critical)
            self.assertEqual(result, out)
            expected = bytes.fromhex("f110341230f1f120341230f14b0100007b000000") + bytes([critical])
            self.assertEqual(m.buffer(out), expected)
            self.assertEqual(bytes(m.cpu.mem_read(out + 32, 8)), b"CANARY!!")
            capacity, owned, _ = m.read(out, "3Q")
            m.call(0x21d0, owned, capacity, 1)
        original = bytes(range(128))
        m.cpu.mem_write(pointer, original)
        m.write(queue + 16, "Q", 4)
        m.call(self.values["HEAL_LOG_RESERVE"][0], queue)
        cap, pointer, length = m.read(queue, "3Q")
        self.assertEqual((cap, length), (8, 4))
        self.assertEqual(bytes(m.cpu.mem_read(pointer, 128)), original)
        m.call(0x21d0, pointer, cap * 32, 8)
        self.assertFalse(m.allocations)
        # The actual healing producer appends to this queue after its packet call.
        body = {op.address: op for op in self.image.ins(0x5f9d0c0)}
        self.assertEqual(body[0x5f9d2b3].op_str,
                         f"rcx, [rsi + {hex(self.values['HEAL_LOG_QUEUE'])}]")
        self.assertEqual(body[0x5f9d2ba].operands[0].imm, self.values["HEAL_LOG_RESERVE"][0])

    def test_native_emitters_keep_the_real_effect_and_healing_rows(self):
        from native_1392_machine import CatalogMachine
        from unicorn import x86_const as reg
        import struct
        for canonical, rich in ((0x58fec60, False), (0x58fed50, True)):
            with self.subTest(rich=rich):
                m = CatalogMachine(self.image)
                game, profile, effect = (m.scratch + n for n in (0x10000, 0x18000, 0x22000))
                m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
                m.write(game + self.field(0xc41), "B", 1)
                m.write(game + self.field(0xb40), "Q", 3)
                m.write(game + self.field(0x478), "Q", 0xf130123400000010)
                m.write(game + self.field(0x4e0), "3f", 1., 2., 3.)
                m.write(effect, "IfI", 13, .25, 324)
                m.cpu.reg_write(reg.UC_X86_REG_XMM3, struct.unpack("<I", struct.pack("<f", 12.))[0])
                m.cpu.reg_write(reg.UC_X86_REG_XMM2, struct.unpack("<I", struct.pack("<f", .25))[0])
                entry = dict(self.values["ADDRESSES"])[canonical]
                m.call(entry, game, 331 if rich else 324, *(() if rich else (effect,)))
                _, pointer, count = m.read(game + 0xdb8, "3Q")
                self.assertEqual(count, 1)
                row = bytes(m.cpu.mem_read(pointer, 56))
                self.assertEqual(struct.unpack_from("<QQ3fI", row),
                                 (3, 0xf130123400000010, 1., 2., 3., 331 if rich else 324))
                if rich:
                    self.assertEqual(struct.unpack_from("<2fI", row, 32), (.25, 12., 14))
                else:
                    self.assertEqual(struct.unpack_from("<IfIfI", row, 32), (0, 12., 13, .25, 324))

    def test_native_goblin_motion_publishes_selected_full_xyz_and_origin(self):
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        import struct
        m = ReviewMachine(self.image)
        enemy, frame = m.scratch + 0x14000, m.scratch + 0x18000
        m.write(frame + self.values["BASIC_POSITION"], "3f", 1., 2., 3.)
        m.cpu.reg_write(reg.UC_X86_REG_RBP, frame)
        m.cpu.reg_write(reg.UC_X86_REG_RSI, enemy)
        xy = getattr(reg, f"UC_X86_REG_XMM{self.values['GOBLIN_MOTION_XY_XMM']}")
        m.cpu.reg_write(xy, int.from_bytes(struct.pack("<4f", 4., 6., 123., 456.), "little"))
        m.cpu.reg_write(reg.UC_X86_REG_XMM14, int.from_bytes(struct.pack("<4f", 8., 12., 13., 14.), "little"))
        m.cpu.reg_write(reg.UC_X86_REG_XMM9, int.from_bytes(struct.pack("<4f", 1., 2., 0., 0.), "little"))
        m.cpu.reg_write(reg.UC_X86_REG_XMM6, struct.unpack("<I", struct.pack("<f", 10.))[0])
        m.stop = m.base + 0x5f5034d
        m.call(self.values["GOBLIN_FINAL_MOTION"][0])
        position = enemy + self.values["ENEMY_POSITION"]
        self.assertEqual(m.read(position, "3f"), (4., 6., 8.))
        self.assertEqual(m.read(position - 12, "3f"), (1., 2., 3.))
        self.assertEqual(m.read(position + 12, "f"), (10.,))
        body = {op.address: op for op in self.image.ins(0x5f4c680)}
        self.assertEqual(body[0x5f4e067].op_str,
                         f"xmmword ptr [rbp + {hex(self.values['BASIC_DISTANCE_FRAME'])}], xmm0")
        self.assertEqual(body[0x5f4fdd5].op_str,
                         f"xmm13, dword ptr [rbp + {hex(self.values['BASIC_DISTANCE_FRAME'])}]")


    def test_native_spell_formatter_reads_selected_rank_and_retains_owned_output(self):
        from native_1392_machine import CatalogMachine
        m = CatalogMachine(self.image)
        game, profile, ability, out = (m.scratch + n for n in (0x10000, 0x18000, 0x22000, 0x26000))
        m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
        addresses = dict(self.values["ADDRESSES"])
        rank_at = self.values["SPELL_PRESENTATION_RANK_OFFSET"]
        for key, name in ((0x6cc5c78, "Renew"), (0x6cc6380, "Healing Wave"),
                          (0x6cc5c00, "Power Word: Shield"), (0x6cc7370, "Regrowth")):
            m.call(0x5fc2510, ability, game, m.base + addresses[key])
            self.assertEqual(m.read(ability + rank_at, "I"), (1,))
            for rank in (1, 3):
                m.write(ability + rank_at, "I", rank)
                m.cpu.mem_write(out, bytes([0xa7]) * 192)
                self.assertEqual(m.call(addresses[0x58629b0], out, ability,
                                        game + self.field(0x490), 0, ability + 0x2000), out)
                self.assertEqual(m.buffer(out).decode(), name)
                self.assertEqual(m.buffer(out + 72).decode(), f"Rank {rank}")
                self.assertTrue(m.buffer(out + 24))
                self.assertEqual(m.bytes_at(out + 176, 16), bytes([0xa7]) * 16)
                # Native strings, including optional strings, remain owned.
                for offset in range(0, 168, 24):
                    cap, pointer, length = m.read(out + offset, "3Q")
                    if 0 < cap < (1 << 63):
                        m.call(0x21d0, pointer, cap, 1)

    def test_native_support_query_preserves_radius_exclusions_order_and_ownership(self):
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        import struct
        m = ReviewMachine(self.image)
        game, rows, point, out, excluded = (m.scratch + n for n in (0x10000, 0x14000, 0x18000, 0x18100, 0x19000))
        m.write(game + 0x6b8, "3Q", 3, rows, 3)
        for index, distance in enumerate((5., 15., 25.)):
            row = rows + index * self.values["ENEMY_STRIDE"]
            m.write(row + self.values["ENEMY_GUID"], "Q", 0x100000001 + index)
            m.write(row + 0x30, "Q", (1 << 64) - 1)
            m.write(row + self.values["ENEMY_HP"], "2f", 100., 100.)
            m.write(row + self.values["ENEMY_POSITION"] - 12, "8f", distance, 0., 0., distance, 0., 0., 0., 0.)
        for radius, exclude, expected in ((10., False, (0x100000001,)),
                                         (20., False, (0x100000001, 0x100000002)),
                                         (20., True, (0x100000002,))):
            m.write(excluded, "Q", 0x100000001)
            m.cpu.reg_write(reg.UC_X86_REG_XMM3, struct.unpack("<I", struct.pack("<f", radius))[0])
            m.call(dict(self.values["ADDRESSES"])[0x5938db0], out, game, point, 0,
                   excluded if exclude else 8, int(exclude))
            cap, pointer, count = m.read(out, "3Q")
            self.assertEqual(m.read(pointer, f"{count}Q"), expected)
            m.call(0x21d0, pointer, cap * 8, 8)
        self.assertFalse(m.allocations)

    def test_native_aegis_and_swiftmend_dispatch_conditions_and_exact_query_caller(self):
        from native_1392_machine import CatalogMachine
        from unicorn import UC_HOOK_CODE, x86_const as reg
        import struct
        m = CatalogMachine(self.image)
        game, profile, row, perk, source, timer, event = (m.scratch + n for n in
            (0x10000, 0x18000, 0x22000, 0x24000, 0x25000, 0x26000, 0x27000))
        m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
        addresses = dict(self.values["ADDRESSES"])
        m.write(game + self.field(0xc46), "B", 1)
        m.write(game + self.field(0x478), "Q", 0x100000001)
        m.cpu.mem_write(perk, self.image.pe.get_data(addresses[0x7136740], 64))
        m.write(game + self.field(0x830), "3Q", 1, perk, 1)
        m.write(row, "2Q3fI2fI", 0, 0, 0., 0., 0., 17, .5, 10., 15)
        m.call(addresses[0x58ffac0], game, row)
        conditions, queries, casts = [], [], []
        def condition(cpu, address, size, user):
            self.assertEqual(cpu.reg_read(reg.UC_X86_REG_RCX), game)
            kind = m.read(cpu.reg_read(reg.UC_X86_REG_RDX), "I")[0]
            event_kind = m.read(cpu.reg_read(reg.UC_X86_REG_R8) + 29, "B")[0]
            conditions.append((kind, event_kind))
        def query(cpu, address, size, user):
            stack = cpu.reg_read(reg.UC_X86_REG_RSP)
            radius = struct.unpack("<f", struct.pack("<I", cpu.reg_read(reg.UC_X86_REG_XMM3) & 0xffffffff))[0]
            queries.append((m.read(stack, "Q")[0] - m.base, radius))
        def cast(cpu, address, size, user):
            definition = m.read(cpu.reg_read(reg.UC_X86_REG_RDX), "Q")[0]
            casts.append(m.read(definition + 64, "I")[0])
        m.cpu.hook_add(UC_HOOK_CODE, condition, begin=m.base + addresses[0x5d3ef10], end=m.base + addresses[0x5d3ef10])
        m.cpu.hook_add(UC_HOOK_CODE, query, begin=m.base + addresses[0x5938db0], end=m.base + addresses[0x5938db0])
        m.cpu.hook_add(UC_HOOK_CODE, cast, begin=m.base + addresses[0x59317f0], end=m.base + addresses[0x59317f0])
        m.write(source, "II2Q", 1, 17, 0, 0)
        m.cpu.reg_write(reg.UC_X86_REG_XMM1, struct.unpack("<I", struct.pack("<f", 300.))[0])
        m.call(addresses[0x5908cc0], game, 0, 0, 0, source)
        self.assertIn((10, 6), conditions)
        self.assertEqual(queries, [(addresses[0x5944de9], 10.)])
        m.write(game + self.field(0x830), "3Q", 1, m.base + addresses[0x7146510], 1)
        m.write(game + self.field(0x68), "3Q", 1, timer, 1)
        m.write(game + self.field(0x4f8), "f", 35.)
        m.write(game + self.field(0x490), "f", 100.)
        m.write(game + self.field(0xb9c), "f", 10.)
        m.write(game + 0x10ce, "B", 1)
        m.write(event + 29, "B", 5)
        m.call(addresses[0x59436a0], game, event, 0)
        self.assertIn((9, 5), conditions)
        self.assertEqual(casts, [8936])
        self.assertGreater(m.read(game + self.field(0x4f8), "f")[0], 35.)
        self.assertEqual(m.read(timer, "f"), (18.,))

    def test_native_aura_export_keeps_spell_expiry_and_selected_frame(self):
        from native_1392_machine import CatalogMachine
        from unicorn import UC_HOOK_CODE, x86_const as reg
        m = CatalogMachine(self.image)
        game, profile, row = (m.scratch + n for n in (0x10000, 0x18000, 0x22000))
        m.call(0x5faa440, game, profile, profile + 0x2000, profile + 0x4000, profile + 0x6000)
        m.write(game + self.field(0xb9c), "f", 10.)
        m.write(game + self.field(0x478), "Q", 0x100000001)
        m.write(row, "QIfIIfI", 0, 13, .25, 0, 324, 20., 0)
        m.write(game + self.field(0x398), "3Q", 1, row, 1)
        observed = []
        def observe(cpu, address, size, user):
            self.assertEqual(cpu.reg_read(reg.UC_X86_REG_RAX), game)
            frame = cpu.reg_read(reg.UC_X86_REG_RBP)
            pointer = m.read(frame + self.values["AURA_ROWS_FRAME_SLOT"], "Q")[0]
            observed.append(m.read(pointer, "IIfI"))
        seam = dict(self.values["EXTENSION_ADDRESSES"])[self.values["FATIGUE_AURA_RVA"]]
        m.cpu.hook_add(UC_HOOK_CODE, observe, begin=m.base + seam, end=m.base + seam)
        m.call(dict(self.values["ADDRESSES"])[0x5904950], game)
        self.assertEqual(observed, [(1, 324, 20., 1)])


    def test_effect_xp_preserves_its_raw_carrier_before_native_rounding(self):
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        m = ReviewMachine(self.image)
        game = m.scratch + 0x10000
        m.write(game + self.field(0x4ac), "f", 1.25)
        for register, value in ((reg.UC_X86_REG_RSI, game), (reg.UC_X86_REG_R15, 37),
                                (reg.UC_X86_REG_R12, 999), (reg.UC_X86_REG_XMM1, 0x3f000000),
                                (reg.UC_X86_REG_XMM12, 0x3f800000), (reg.UC_X86_REG_XMM15, 0)):
            m.cpu.reg_write(register, value)
        m.stop = m.base + dict(self.values["ADDRESSES"])[0x591db30]
        m.call(0x5fd3049)
        self.assertEqual(m.cpu.reg_read(reg.UC_X86_REG_RCX), game)
        self.assertEqual(m.cpu.reg_read(reg.UC_X86_REG_RDX), 23)
        self.assertEqual(m.cpu.reg_read(getattr(reg, f"UC_X86_REG_R{self.values['XP_EFFECT_REGISTER']}")), 37)
        self.assertEqual(m.read(m.cpu.reg_read(reg.UC_X86_REG_RSP), "Q")[0] - m.base,
                         dict(self.values["ADDRESSES"])[0x5949b3b])

    def test_native_codex_insert_preserves_order_highest_rarity_and_duplicate_noop(self):
        from native_1392_machine import ReviewMachine
        m = ReviewMachine(self.image)
        collection, key = m.scratch + 0x10000, m.scratch + 0x14000
        m.write(collection + 0x90, "3Q", 0, 8, 0)
        entry = dict(self.values["ADDRESSES"])[0x5c4d7f0]
        for text, rarity, added in ((b"insignia", 0, 1), (b"eldre_thalas", 0, 1), (b"chosen", 0, 1),
                                   (b"insignia", 0, 0), (b"chosen", 3, 1), (b"chosen", 0, 0)):
            m.cpu.mem_write(key, text)
            self.assertEqual(m.call(entry, collection, key, len(text), rarity) & 255, added)
        cap, pointer, count = m.read(collection + 0x90, "3Q")
        self.assertEqual(count, 3)
        self.assertEqual([(m.buffer(pointer + i * 32), m.read(pointer + i * 32 + 24, "B")[0])
                          for i in range(count)], [(b"chosen", 3), (b"eldre_thalas", 0), (b"insignia", 0)])
        for i in range(count):
            owned_cap, owned, _ = m.read(pointer + i * 32, "3Q")
            m.call(0x21d0, owned, owned_cap, 1)
        m.call(0x21d0, pointer, cap * 32, 8)
        self.assertFalse(m.allocations)

    def test_native_necklace_leech_stacks_expire_without_replacing_the_baseline(self):
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        import struct
        m = ReviewMachine(self.image)
        game, state, perk = (m.scratch + n for n in (0x10000, 0x14000, 0x16000))
        m.write(game + 0x68, "3Q", 1, state, 1)
        m.write(game + self.field(0x830), "3Q", 1, perk, 1)
        m.write(game + self.field(0xb9c), "f", 10.)
        m.write(state, "QIf", 0, 3, 20.)
        m.write(perk, "I", 27)
        m.write(perk + 40, "f", .05)
        m.write(perk + 52, "B", 14)
        values = []
        for expiry in (20., 10., 9.):
            m.write(state + 12, "f", expiry)
            m.call(dict(self.values["ADDRESSES"])[0x5cf1c70], game)
            values.append(struct.unpack("<f", struct.pack("<I", m.cpu.reg_read(reg.UC_X86_REG_XMM0) & 0xffffffff))[0])
        self.assertAlmostEqual(values[0] - values[1], .15)
        self.assertAlmostEqual(values[1], .015)
        self.assertEqual(values[1], values[2])

    def test_native_actor_picker_stores_the_selected_slot_in_the_enemy_target(self):
        from native_1392_machine import ReviewMachine
        from unicorn import x86_const as reg
        m = ReviewMachine(self.image)
        rows, iterator, point, enemy, frame = (m.scratch + n for n in (0x10000, 0x14000, 0x16000, 0x18000, 0x20000))
        m.write(rows, "3fI", 10., 0., 0., 1)
        m.write(rows + 16, "3fI", 3., 0., 0., 0)
        m.write(rows + 32, "3fI", 4., 0., 0., 1)
        m.write(iterator, "4Q", rows, rows + 48, 0, point)
        m.write(point, "3f", 5., 0., 0.)
        self.assertEqual(m.call(0x5f24ac0, iterator, point), 2)
        self.assertEqual(m.cpu.reg_read(reg.UC_X86_REG_RDX), rows + 32)
        for register, value in ((reg.UC_X86_REG_R14, enemy), (reg.UC_X86_REG_RBP, frame),
                                (reg.UC_X86_REG_XMM6, 0x41200000)):
            m.cpu.reg_write(register, value)
        m.stop = m.base + 0x5f4e915
        m.call(0x5f4e8f2)
        self.assertEqual(m.read(enemy + self.values["ENEMY_TARGET"], "Q"), (2,))

    def test_native_drop_catalog_covers_all_zones_classes_and_owned_item_names(self):
        from native_1392_machine import CatalogMachine
        m = CatalogMachine(self.image)
        out, item = m.scratch + 0x10000, m.scratch + 0x12000
        addresses = dict(self.values["ADDRESSES"])
        # The first item lookup initializes the native process-wide catalog.
        # Its borrowed icon/name data stay alive; only the returned name and
        # bonus vector are caller-owned. Compare against that warmed baseline.
        m.call(addresses[0x5bffe00], item, 101800)
        owned_cap, owned, _ = m.read(item, "3Q")
        bonuses_cap, bonuses, _ = m.read(item + 24, "3Q")
        if owned_cap:
            m.call(0x21d0, owned, owned_cap, 1)
        if bonuses_cap:
            m.call(0x21d0, bonuses, bonuses_cap * 8, 4)
        retained = len(m.allocations)
        count_by_zone = []
        for zone in range(self.values["BLESSING_MAP_COUNT"]):
            total = 0
            for boss in range(5):
                for class_id in (1, 2, 3, 4, 5, 7, 8, 9, 11):
                    m.cpu.mem_write(out, bytes([0xa7]) * 40)
                    m.call(addresses[0x5bfd310], out, class_id, zone, boss)
                    cap, pointer, count = m.read(out, "3Q")
                    self.assertLessEqual(count, min(64, cap))
                    self.assertEqual(m.bytes_at(out + 24, 16), bytes([0xa7]) * 16)
                    total += count
                    for i in range(count):
                        identity, chance = m.read(pointer + i * 8, "If")
                        self.assertTrue(0. < chance <= 1.)
                        m.cpu.mem_write(item, bytes([0xa7]) * 160)
                        m.call(addresses[0x5bffe00], item, identity)
                        self.assertNotEqual(m.read(item, "Q")[0], (1 << 64) - 1)
                        self.assertTrue(m.buffer(item).decode())
                        self.assertEqual(m.bytes_at(item + 144, 16), bytes([0xa7]) * 16)
                        owned_cap, owned, _ = m.read(item, "3Q")
                        bonuses_cap, bonuses, _ = m.read(item + 24, "3Q")
                        if owned_cap:
                            m.call(0x21d0, owned, owned_cap, 1)
                        if bonuses_cap:
                            m.call(0x21d0, bonuses, bonuses_cap * 8, 4)
                    if cap:
                        m.call(0x21d0, pointer, cap * 8, 4)
            count_by_zone.append(total)
        self.assertEqual(count_by_zone, [47] * 10 + [83, 47, 47])
        self.assertEqual(len(m.allocations), retained)


if __name__ == "__main__":
    unittest.main()
