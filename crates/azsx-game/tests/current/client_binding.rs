use azsx_game::{
    current, memory::Mem, official::SelectedAdapter,
    official_native,
};
struct Blocks(std::collections::BTreeMap<u64, Vec<u8>>);
impl Mem for Blocks {
    fn read(&self, a: u64, n: usize) -> Result<Vec<u8>, String> {
        let (base, b) = self.0.range(..=a).next_back().ok_or("missing block")?;
        b.get((a - base) as usize..(a - base) as usize + n)
            .map(|v| v.to_vec())
            .ok_or("read outside exact context".into())
    }
}
fn actor(size: usize, class: u8, race: u8, guid: u64) -> Vec<u8> {
    let mut b = vec![0; size];
    b[0x464] = class;
    b[0x465] = race;
    for (at, v) in [(0x658, 100f32), (0x674, 1.1), (0x68c, 1.2), (0x5f8, 80.)] {
        b[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }
    for (at, v) in [(0x5f0, 7u32), (0x5f4, 18), (0x608, 23)] {
        b[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }
    b[0x5c8..0x5d0].copy_from_slice(&guid.to_le_bytes());
    b
}
#[test]
fn revision_capabilities_do_not_disable_shared_jewelry_ui() {
    let a = SelectedAdapter::select(current::SHA256).unwrap();
    assert_eq!(a.version(), "0.141.0");
    assert!(!a.duplicate_loot_enabled());
    assert!(a.native_marks_enabled());
    assert!(a.loot_entries().is_empty());
    assert!(a.supports_enchantments());
    assert!(a.supports_bonus_tooltips());
    assert!(!a.needs_guardian_item_supplement());
    assert_eq!(a.bonus_ui_entries().len(), 5);
    assert_eq!(a.blessing_entries().len(), 3);
    assert!(a.native_difficulty_entries().is_empty());
    assert!(SelectedAdapter::select("0.141.0").is_err());
    for retired in [
        "eb5fd1c1b96e6180fcf5f3d685d90a61471af28d22e5f060c02b7b7663fbf79e",
        "e8ff033544095e81d74087ff400d50286b26af02291e649f4c5e4d76cbcf8863",
        "8829953418a4d996cc5287f0e9fad25407ca30ceb64d105a4f57833caef78016",
        "b6dbd788835ad80929fc6113f99c0fe4867dda2fe36ac3afb5810127b0a8b709",
        "ec3a5b15fa39fed63ce4516c4fc159bdac92691ca44f88befc849010338d14f7",
        official_native::SHA256,
        "4a31214ae3d0bbf44f8cd51a4a0c8f01ea7e9debbe8af30cb9ce4f8f2cc6f172",
        "5dd6819d836748ccf809ab76fe6509c9ee793e087939b5d036c430d835a14e63",
        "d57dbb9f6b325e502204940ac7e3b82bab2ea7db6022c68a9b2bebb91c556bc9",
        "507a791fad3aec59afb2f90d2737cdab4dbbe79b98c5e74d7699fe44bed16293",
        "291c281bd0a4255fdb3d04b2e442121e2df144f388f16de086b430e875aaad08",
        "d3ff38943ce81df46c4c2562632d3bf868ed871540d732b235a4c68323c3ab7d",
        "22f7938eb4f99f2857dc091b1c57c147bd848eaabfeaa81e444c0a3393ca8150",
        "08f8c58d6967b94598c56b822a1e6b323a52924a299a6df70ef6055ca839b8cd",
    ] {
        assert!(
            SelectedAdapter::select(retired).is_err(),
            "retired native hash must fail closed: {retired}"
        );
    }
}
#[test]
fn independent_host_and_private_layouts_keep_identity_phase_and_resources() {
    let a = SelectedAdapter::select(current::SHA256).unwrap();
    let mut root = actor(0x1200, 1, 1, 111);
    root[0x10e3] = 1;
    root[0x10e9] = 1;
    root[0xce8..0xcf0].copy_from_slice(&0x20000u64.to_le_bytes());
    root[0xcf0..0xcf8].copy_from_slice(&1u64.to_le_bytes());
    root[0x1028..0x102c].copy_from_slice(&3.5f32.to_le_bytes());
    let mut server = vec![0; 0x2100];
    server[0x10..0x10 + root.len()].copy_from_slice(&root);
    server[0x1100..0x1104].copy_from_slice(&3u32.to_le_bytes());
    server[0x1140..0x1148].copy_from_slice(&1u64.to_le_bytes());
    let mut guest = actor(0xc20, 3, 3, 222);
    guest[0xc18] = 2;
    let m = Blocks([(0x10000, server), (0x20000, guest)].into());
    let p = a.party(&m, 0x10000).unwrap().unwrap();
    assert_eq!(p.clock, 3.5);
    assert_eq!(p.actors.len(), 2);
    assert_eq!((p.actors[0].class_id, p.actors[1].class_id), (1, 3));
    assert_eq!((p.actors[0].guid, p.actors[1].guid), (111, 222));
    assert_eq!(p.actors[1].phase, azsx_gameplay::Phase::Choosing);
    for actor in p.actors {
        assert_eq!((actor.level, actor.xp, actor.gold), (7, 18, 23));
        assert_eq!(actor.hp, 80.);
        assert_eq!(actor.maximum, 100.);
        assert!((actor.gold_multiplier - 1.2).abs() < 0.0001);
    }
}

#[test]
fn wallet_reads_independently_shifted_scalars() {
    let adapter = SelectedAdapter::select(current::SHA256).unwrap();
    let memory = Blocks([(0x10000, actor(0x1200, 1, 1, 111))].into());
    let (gold, multiplier, race) = adapter.current_gold(&memory, 0x10000).unwrap();
    assert_eq!((gold, race), (23, 1));
    assert!((multiplier - 1.2).abs() < 0.0001);
}

#[test]
fn native_tooltip_frame_and_diagnostic_focus_are_version_selected() {
    let a = SelectedAdapter::select(current::SHA256).unwrap();
    assert_eq!(a.kept_tooltip_fixture_focus().unwrap().rva, 0x33e6c15);
    let native = official_native::Adapter::select(current::SHA256).unwrap();
    assert_eq!(native.kept_tooltip_row(0x10000), Some(0xffa0));
}

#[test]
fn basic_pursuit_uses_selected_stack_fields_and_restores_them() {
    use azsx_game::official::MutableMemory;
    struct M(std::cell::RefCell<Vec<u8>>);
    impl Mem for M {
        fn read(&self, a: u64, n: usize) -> Result<Vec<u8>, String> {
            self.0
                .borrow()
                .get(a as usize..a as usize + n)
                .map(|b| b.to_vec())
                .ok_or("outside fixture".into())
        }
    }
    impl MutableMemory for M {
        fn write(&self, a: u64, b: &[u8]) -> Result<(), String> {
            let mut data = self.0.borrow_mut();
            data.get_mut(a as usize..a as usize + b.len())
                .ok_or("outside fixture")?
                .copy_from_slice(b);
            Ok(())
        }
        fn allocate_native(&self, _: &[u8]) -> Result<u64, String> {
            Err("no allocations".into())
        }
        fn free_native(&self, _: u64, _: usize) -> Result<(), String> {
            Err("no allocations".into())
        }
    }
    let m = M(std::cell::RefCell::new(vec![0; 0x24000]));
    let a = SelectedAdapter::select(current::SHA256).unwrap();
    let f = 0x10000;
    m.write(f + 0x80, &0xfeedfacecafebeefu64.to_le_bytes())
        .unwrap();
    m.write(f + 0x3e0, &0x18000u64.to_le_bytes()).unwrap();
    m.write(f + 0x3d0, &0x20000u64.to_le_bytes()).unwrap();
    m.write(0x20110, &77u64.to_le_bytes()).unwrap();
    for (at, v) in [(0x390, 3f32), (0x394, 4.), (0x398, 0.)] {
        m.write(f + at, &v.to_le_bytes()).unwrap();
    }
    let frame = a.basic_frame(&m, f).unwrap();
    assert_eq!((frame.game, frame.enemy), (0x18000, 77));
    assert_eq!(
        a.prepare_basic_target(&m, &frame, [0., 0., 8.]).unwrap(),
        5.
    );
    assert_eq!(m.read(f + 0x294, 4).unwrap(), 8f32.to_le_bytes());
    assert_eq!(m.read(f + 0x70, 8).unwrap(), [0; 8]);
    assert!(a
        .restore_basic_origin(&m, f, 0x18001, [1., 2., 3.])
        .is_err());
    a.restore_basic_origin(&m, f, 0x18000, [1., 2., 3.])
        .unwrap();
    assert_eq!(
        m.read(f + 0x70, 8).unwrap(),
        [1f32.to_le_bytes(), 2f32.to_le_bytes()].concat()
    );
    assert_eq!(
        m.read(f + 0x80, 8).unwrap(),
        0xfeedfacecafebeefu64.to_le_bytes()
    );
}

#[test]
fn multiple_private_actors_use_the_selected_native_stride() {
    let a = SelectedAdapter::select(current::SHA256).unwrap();
    let mut host = actor(0x1200, 1, 1, 111);
    host[0x10e3] = 1;
    host[0x10e9] = 1;
    host[0xce8..0xcf0].copy_from_slice(&0x20000u64.to_le_bytes());
    host[0xcf0..0xcf8].copy_from_slice(&2u64.to_le_bytes());
    let mut server = vec![0; 0x2100];
    server[0x10..0x10 + host.len()].copy_from_slice(&host);
    server[0x1100..0x1104].copy_from_slice(&3u32.to_le_bytes());
    server[0x1140..0x1148].copy_from_slice(&2u64.to_le_bytes());
    let mut first = actor(0xc20, 3, 3, 222);
    first[0xc18] = 1;
    let mut second = actor(0xc20, 5, 1, 333);
    second[0xc18] = 2;
    first.extend(second);
    let p = a
        .party(
            &Blocks([(0x10000, server), (0x20000, first)].into()),
            0x10000,
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        p.actors.iter().map(|a| a.guid).collect::<Vec<_>>(),
        [111, 222, 333]
    );
    assert_eq!(p.actors[2].class_id, 5);
}

#[test]
fn native_pause_keeps_actual_hud_phases_and_reward_content() {
    let a = SelectedAdapter::select(current::SHA256).unwrap();
    let view = azsx_gameplay::official::PausePresentation {
        active: true,
        visible: true,
        local_menu: false,
        title: "Waiting for players".into(),
        detail: "Ally is choosing a reward.".into(),
    };
    for phase in ["playing", "choosing", "dead", "victory", "paused"] {
        let original = serde_json::json!({"type":"hud","phase":phase,"choices":[{"title":"Native reward"}],"gold":42});
        let mut hud = original.clone();
        a.decorate_pause(&mut hud, &view);
        assert_eq!(hud["phase"], phase);
        assert_eq!(hud["choices"], original["choices"]);
        assert_eq!(hud["gold"], 42);
        let mut resumed = original.clone();
        a.decorate_pause(&mut resumed, &Default::default());
        assert_eq!(resumed["phase"], phase);
    }
}

#[test]
fn native_pause_uses_the_distinct_pause_and_unpause_notifications() {
    let a = SelectedAdapter::select(current::SHA256).unwrap();
    assert_eq!(a.pause_notice(true), Some("{\"type\":\"paused\"}"));
    assert_eq!(a.pause_notice(false), Some("{\"type\":\"unpaused\"}"));
    let mut action = vec![7, 0, 0, 0, 1];
    let mut m = Blocks([(0x50000, action.clone())].into());
    assert_eq!(a.menu_request(&m, 0x50000).unwrap(), Some(true));
    action[4] = 0;
    m.0.insert(0x50000, action.clone());
    assert_eq!(a.menu_request(&m, 0x50000).unwrap(), Some(false));
    action[4] = 2;
    m.0.insert(0x50000, action);
    assert!(a.menu_request(&m, 0x50000).is_err());
}
#[test]
fn menu_wire_binds_peer_and_rejects_bad_extents() {
    let a = SelectedAdapter::select(current::SHA256).unwrap();
    let mut server = vec![0; 0x2100];
    server[0x1100..0x1104].copy_from_slice(&3u32.to_le_bytes());
    server[0x1138..0x1140].copy_from_slice(&0x20000u64.to_le_bytes());
    server[0x1140..0x1148].copy_from_slice(&1u64.to_le_bytes());
    let mut peer = vec![0; 0x870];
    peer[..4].copy_from_slice(&1u32.to_le_bytes());
    peer[8..16].copy_from_slice(&3u64.to_le_bytes());
    let mut body = 12u64.to_le_bytes().to_vec();
    body.push(1);
    let mut stack = vec![0; 56];
    stack[40..48].copy_from_slice(&0x40000u64.to_le_bytes());
    stack[48..56].copy_from_slice(&9u64.to_le_bytes());
    let m = Blocks(
        [
            (0x10000, server),
            (0x20000, peer),
            (0x30000, stack),
            (0x40000, body),
        ]
        .into(),
    );
    assert_eq!(a.menu_peer(&m, 0x10000, 0).unwrap(), 3);
    assert!(a.menu_peer(&m, 0x10000, 1).is_err());
    assert_eq!(a.menu_wire(&m, 0x30000).unwrap(), (12, true));
    let mut bad = m;
    bad.0.get_mut(&0x20000).unwrap()[0] = 0;
    assert!(a.menu_peer(&bad, 0x10000, 0).is_err());
    bad.0.get_mut(&0x10000).unwrap()[0x1120..0x1128].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(a.menu_peer(&bad, 0x10000, 0).is_err());
    bad.0.get_mut(&0x40000).unwrap()[8] = 2;
    assert!(a.menu_wire(&bad, 0x30000).is_err());
    bad.0.get_mut(&0x30000).unwrap()[48] = 10;
    assert!(a.menu_wire(&bad, 0x30000).is_err());
}

#[test]
fn owner_spell_formatter_uses_the_selected_game_context() {
    let a = SelectedAdapter::select(current::SHA256).unwrap();
    let mut game = vec![0; 0x1200];
    game[0x10cb] = 1;
    game[0xfc0..0xfc8].copy_from_slice(&2u64.to_le_bytes());
    let mut ability = vec![0; 52];
    ability[..8].copy_from_slice(&0x50000u64.to_le_bytes());
    ability[48..].copy_from_slice(&2u32.to_le_bytes());
    let m = Blocks(
        [
            (0x40000, ability),
            (0x50040, 845u32.to_le_bytes().to_vec()),
            (0x60000, game),
        ]
        .into(),
    );
    assert_eq!(
        a.spell_presentation_owner(&m, 0x40000, 0x60640, 0).unwrap(),
        Some((2, 845, 2))
    );
    assert_eq!(
        a.spell_presentation_owner(&m, 0x40000, 0x60640, 1).unwrap(),
        None
    );
}

#[test]
fn owner_spell_text_is_copied_from_native_owned_string_rows() {
    let a = SelectedAdapter::select(current::SHA256).unwrap();
    let title = b"Cleave";
    let body = b"Native damage.\nYour bonuses: +58% damage";
    let mut row = vec![0; 0xb0];
    for (offset, address, text) in [
        (0, 0x70000u64, title.as_slice()),
        (24, 0x71000, body.as_slice()),
    ] {
        for (n, value) in [text.len() as u64, address, text.len() as u64]
            .iter()
            .enumerate()
        {
            row[offset + n * 8..offset + n * 8 + 8].copy_from_slice(&value.to_le_bytes());
        }
    }
    let m = Blocks(
        [
            (0x70000, title.to_vec()),
            (0x71000, body.to_vec()),
            (0x80000, row),
        ]
        .into(),
    );
    assert_eq!(
        a.spell_presentation_strings(&m, 0x80000).unwrap(),
        vec![
            (0, "Cleave".into()),
            (24, String::from_utf8(body.to_vec()).unwrap())
        ]
    );
}

#[test]
fn private_actor_native_extent_is_checked_before_context_io() {
    let adapter = SelectedAdapter::select(current::SHA256).unwrap();
    let mut host = actor(0x1200, 1, 1, 111);
    host[0x10e3] = 1;
    host[0x10e9] = 1;
    // One normalized row fits here; one actual native0xc20 row overflows.
    host[0xce8..0xcf0].copy_from_slice(&(u64::MAX - 0x990).to_le_bytes());
    host[0xcf0..0xcf8].copy_from_slice(&1u64.to_le_bytes());
    let mut server = vec![0; 0x2100];
    server[0x10..0x10 + host.len()].copy_from_slice(&host);
    server[0x1100..0x1104].copy_from_slice(&3u32.to_le_bytes());
    server[0x1140..0x1148].copy_from_slice(&1u64.to_le_bytes());
    let memory = Blocks([(0x10000, server)].into());
    assert_eq!(
        adapter.party(&memory, 0x10000).unwrap_err(),
        "Invalid official context pointer"
    );
}

#[test]
fn shield_break_reads_game_from_rbx_and_aura_spell_from_r15() {
    let a = SelectedAdapter::select(azsx_game::current::SHA256).unwrap();
    let mut registers = [0u64; 15];
    registers[7] = 0x10000;
    registers[12] = 0xbad;
    registers[13] = 0xdead;
    registers[11] = 2; // R12 is the incoming damage kind.
    registers[14] = 17; // R15 is the consumed shield spell.
    let frame = a.support_break_frame(registers, [23., 7.]).unwrap();
    assert_eq!((frame.game, frame.spell, frame.amount), (0x10000, 17, 7.));
}

#[test]
fn event_gold_call_preserves_game_amount_and_native_return() {
    unsafe extern "system" fn native(game:u64, amount:u32)->usize {
        assert_eq!(game,0x12345678);
        assert_eq!(amount,37);
        0x5678
    }
    let adapter=SelectedAdapter::select(current::SHA256).unwrap();
    let base=(native as *const () as usize).checked_sub(adapter.gold_entry().rva as usize).unwrap();
    assert_eq!(unsafe{adapter.event_credit_gold(base as u64,0x12345678,37)},0x5678);
}

#[test]
fn completed_encounter_uses_selected_carriers_and_rejects_changed_game_context() {
    let a = SelectedAdapter::select(current::SHA256).unwrap();
    let (game, frame, base, defs, name) = (0x10000u64, 0x20000u64, 0x10000000u64, 0x40000u64, 0x50000u64);
    let mut root = actor(0x1200, 1, 1, 111);
    root[0x10e9] = 1;
    root[0x3e0..0x3e8].copy_from_slice(&name.to_le_bytes());
    root[0x3e8..0x3f0].copy_from_slice(&4u64.to_le_bytes());
    let mut stack = vec![0; 0x430];
    stack[0x338..0x340].copy_from_slice(&game.to_le_bytes());
    stack[0x310..0x318].copy_from_slice(&(defs + 104).to_le_bytes());
    let mut map = vec![0; 240];
    map[8..16].copy_from_slice(&name.to_le_bytes());
    map[16..24].copy_from_slice(&4u64.to_le_bytes());
    map[0x88..0x90].copy_from_slice(&defs.to_le_bytes());
    map[0x90..0x98].copy_from_slice(&5u64.to_le_bytes());
    let mut m = Blocks([(game, root), (frame, stack), (name, b"test".to_vec()), (base + 0x73a06b0, map)].into());
    // Actual 0.141.0 carriers: R15 Game, definition at frame+0x310.
    assert_eq!(a.native_encounter_context(&m, frame, [999, 998, game], base).unwrap(), Some((game, 5)));
    assert_eq!(a.blessing_event_context(&m, [frame, defs + 104, game], base).unwrap(), Some((game, 5)));
    m.0.get_mut(&game).unwrap()[0xa8c..0xa90].copy_from_slice(&1u32.to_le_bytes());
    assert_eq!(a.blessing_event_context(&m, [frame, defs + 104, game], base).unwrap(), None);
    m.0.get_mut(&game).unwrap()[0xa8c..0xa90].fill(0);
    assert!(a.blessing_event_context(&m, [frame, defs + 1, game], base).is_err());
    assert!(a.blessing_event_context(&m, [frame, defs + 104, 999], base).is_err());
    assert!(a.blessing_event_context(&m, [u64::MAX, 0, 0], base).is_err());
}

#[test]
fn dead_shared_xp_uses_the_current_native_actor_without_reading_the_borrowed_vector() {
    let a=SelectedAdapter::select(current::SHA256).unwrap();
    let game=0x10010u64;
    let fixture=|role:u32,coop:u8,phase:u8,slot:u64,guid:u64| {
        let mut server=vec![0;0x2100];
        server[0x1100..0x1104].copy_from_slice(&role.to_le_bytes());
        server[0x1140..0x1148].copy_from_slice(&1u64.to_le_bytes());
        let root=&mut server[0x10..];root[0x10e3]=coop;root[0x10e9]=phase;
        root[0xfd8..0xfe0].copy_from_slice(&slot.to_le_bytes());
        root[0x5c8..0x5d0].copy_from_slice(&guid.to_le_bytes());
        root[0x1048..0x104c].copy_from_slice(&17u32.to_le_bytes());
        // No actor-vector memory is supplied: its ownership is lent out here.
        let mut stack=vec![0;0x1800];stack[0xb8..0xc0].copy_from_slice(&0x50000u64.to_le_bytes());
        let mut request=vec![0;16];request[8..16].copy_from_slice(&0x50000u64.to_le_bytes());
        Blocks([(0x10000,server),(0x2ff00,stack),(0x40000,request),(0x50000,17u32.to_le_bytes().to_vec())].into())
    };
    for kind in 0..7 {
        for slot in 0..5 {
            let m=fixture(3,1,4,slot,111+slot);
            let (game_index,amount_index)=match kind{0|2|5=>(0,2),1=>(1,2),3=>(2,3),4|6=>(2,4),_=>unreachable!()};
            let mut regs=[17;6];regs[game_index]=game;regs[5]=0x30000;if kind==6{regs[4]=0x40000;}
            assert_eq!(a.downed_xp_continuation(&m,kind,regs).unwrap(),Some(current::DOWNED_XP_GUARDS[kind].1));
            if kind<5{regs[amount_index]=0;assert_eq!(a.downed_xp_continuation(&m,kind,regs).unwrap(),None); }
        }
        for (role,coop,phase) in [(0,1,4),(1,1,4),(2,1,4),(3,0,4),(3,1,1),(3,1,2),(3,1,5)] {
            let m=fixture(role,coop,phase,0,111);let mut regs=[game;6];regs[4]=0x40000;regs[5]=0x30000;
            assert_eq!(a.downed_xp_continuation(&m,kind,regs).unwrap(),None);
        }
        for (slot,guid) in [(5,111),(0,0)] {
            assert!(a.downed_xp_continuation(&fixture(3,1,4,slot,guid),kind,[game;6]).is_err());
        }
    }
    assert!(a.downed_xp_continuation(&fixture(3,1,4,0,111),7,[game;6]).is_err());
}

#[test]
fn learned_bear_form_drives_production_taunt_eligibility() {
    let adapter = SelectedAdapter::select(current::SHA256).unwrap();
    for (tag, spell, expected) in [(38u32, 9634u32, true), (37, 9634, false), (38, 768, false)] {
        let mut root = actor(0x1200, 11, 1, 111);
        root[0x10e3] = 1;
        root[0x10e9] = 1;
        root[0xb70..0xb78].copy_from_slice(&0x20000u64.to_le_bytes());
        root[0xb78..0xb80].copy_from_slice(&1u64.to_le_bytes());
        let mut perk = vec![0; 64];
        perk[..4].copy_from_slice(&tag.to_le_bytes());
        perk[4..8].copy_from_slice(&spell.to_le_bytes());
        let m = Blocks([(0x10000, root), (0x20000, perk)].into());
        let actor = adapter.combat_actor(&m, 0x10000, 0x140000000).unwrap();
        assert_eq!(actor.talents.dire_bear_form, expected, "tag={tag}, spell={spell}");
        assert!(!actor.talents.fortified);
        assert!(!actor.talents.ardent_defender);
    }
}

#[test]
fn a_guests_native_form_is_restored_from_the_private_perk_header() {
    use azsx_gameplay::supplies::Costume;
    let adapter = SelectedAdapter::select(current::SHA256).unwrap();
    let mut root = actor(0x1200, 1, 1, 111);
    root[0x10e3] = 1;
    root[0xce8..0xcf0].copy_from_slice(&0x20000u64.to_le_bytes());
    root[0xcf0..0xcf8].copy_from_slice(&1u64.to_le_bytes());
    let mut guest = actor(0xc20, 11, 1, 222);
    guest[0x9c8..0x9d0].copy_from_slice(&1u64.to_le_bytes());
    guest[0x9d0..0x9d8].copy_from_slice(&0x30000u64.to_le_bytes());
    guest[0x9d8..0x9e0].copy_from_slice(&1u64.to_le_bytes());
    let mut form = vec![0; 64];
    form[..4].copy_from_slice(&38u32.to_le_bytes());
    form[4..8].copy_from_slice(&9634u32.to_le_bytes());
    let m = Blocks([(0x10000, root), (0x20000, guest), (0x30000, form)].into());
    assert_eq!(adapter.supply_display(&m, 0x10000, (1, 222), Some(Costume::Skeleton)).unwrap(), 7550);
    assert_eq!(adapter.supply_display(&m, 0x10000, (1, 222), None).unwrap(), 2281);
    assert!(adapter.supply_display(&m, 0x10000, (1, 223), None).is_err());
}

#[test]
fn hunter_mark_observation_reaches_a_nonfirst_runtime_ability() {
    let adapter = SelectedAdapter::select(current::SHA256).unwrap();
    let base = 0x140000000u64;
    let mut root = actor(0x1200, 3, 1, 111);
    root[0x10e3] = 1;
    root[0x10e9] = 1;
    root[0x488..0x490].copy_from_slice(&0x20000u64.to_le_bytes());
    root[0x490..0x498].copy_from_slice(&2u64.to_le_bytes());
    root[0xc40..0xc48].copy_from_slice(&77u64.to_le_bytes());
    let mut rows = vec![0; 2 * 96];
    rows[..8].copy_from_slice(&(base + 0x73babe0).to_le_bytes());
    rows[96..104].copy_from_slice(&(base + 0x73ba460).to_le_bytes());
    rows[96 + 56..96 + 60].copy_from_slice(&4f32.to_le_bytes());
    rows[96 + 8..96 + 12].copy_from_slice(&1f32.to_le_bytes());
    let m = Blocks([(0x10000, root), (0x20000, rows),
        (base + 0x73babe0 + 64, 17u32.to_le_bytes().to_vec()),
        (base + 0x73ba460 + 64, 1130u32.to_le_bytes().to_vec())].into());
    let mark = adapter.combat_actor(&m, 0x10000, base).unwrap().mark;
    assert_eq!(mark.enemy, Some(77));
    assert_eq!(mark.strength, 0.4);
}
