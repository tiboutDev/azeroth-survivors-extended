use asx_game::{
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
    b[0x354] = class;
    b[0x355] = race;
    for (at, v) in [(0x4c8, 100f32), (0x4e4, 1.1), (0x4f8, 1.2), (0x538, 80.)] {
        b[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }
    for (at, v) in [(0x530, 7u32), (0x534, 18), (0x544, 23)] {
        b[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }
    b[0x4b0..0x4b8].copy_from_slice(&guid.to_le_bytes());
    b
}
#[test]
fn revision_capabilities_do_not_disable_shared_jewelry_ui() {
    let a = SelectedAdapter::select(current::SHA256).unwrap();
    assert_eq!(a.version(), "0.130.0");
    assert!(!a.glyphs_enabled());
    assert!(!a.duplicate_loot_enabled());
    assert!(a.native_marks_enabled());
    assert!(a.loot_entries().is_empty());
    assert!(a.supports_enchantments());
    assert!(a.supports_bonus_tooltips());
    assert!(!a.needs_guardian_item_supplement());
    assert_eq!(a.bonus_ui_entries().len(), 5);
    assert_eq!(a.blessing_entries().len(), 3);
    assert!(a.native_difficulty_entries().is_empty());
    assert!(SelectedAdapter::select("0.130.0").is_err());
    for retired in [
        "b6dbd788835ad80929fc6113f99c0fe4867dda2fe36ac3afb5810127b0a8b709",
        "ec3a5b15fa39fed63ce4516c4fc159bdac92691ca44f88befc849010338d14f7",
        official_native::SHA256,
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
    let mut root = actor(0xe00, 1, 1, 111);
    root[0xd9e] = 1;
    root[0xda3] = 1;
    root[0xa70..0xa78].copy_from_slice(&0x20000u64.to_le_bytes());
    root[0xa78..0xa80].copy_from_slice(&1u64.to_le_bytes());
    root[0xce0..0xce4].copy_from_slice(&3.5f32.to_le_bytes());
    let mut server = vec![0; 0x2100];
    server[0x10..0x10 + root.len()].copy_from_slice(&root);
    server[0xdb8..0xdbc].copy_from_slice(&3u32.to_le_bytes());
    server[0xdf8..0xe00].copy_from_slice(&1u64.to_le_bytes());
    let mut guest = actor(0xa38, 3, 3, 222);
    guest[0xa34] = 2;
    let m = Blocks([(0x10000, server), (0x20000, guest)].into());
    let p = a.party(&m, 0x10000).unwrap().unwrap();
    assert_eq!(p.clock, 3.5);
    assert_eq!(p.actors.len(), 2);
    assert_eq!((p.actors[0].class_id, p.actors[1].class_id), (1, 3));
    assert_eq!((p.actors[0].guid, p.actors[1].guid), (111, 222));
    assert_eq!(p.actors[1].phase, asx_gameplay::Phase::Choosing);
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
    let memory = Blocks([(0x10000, actor(0xe00, 1, 1, 111))].into());
    let (gold, multiplier, race) = adapter.current_gold(&memory, 0x10000).unwrap();
    assert_eq!((gold, race), (23, 1));
    assert!((multiplier - 1.2).abs() < 0.0001);
}

#[test]
fn native_tooltip_frame_and_diagnostic_focus_are_version_selected() {
    let a = SelectedAdapter::select(current::SHA256).unwrap();
    assert_eq!(a.kept_tooltip_fixture_focus().unwrap().rva, 0x28b8ec5);
    let native = official_native::Adapter::select(current::SHA256).unwrap();
    assert_eq!(native.kept_tooltip_row(0x10000), Some(0xffa0));
}

#[test]
fn basic_pursuit_uses_130_stack_fields_and_restores_them() {
    use asx_game::official::MutableMemory;
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
    m.write(f + 0x3a0, &0x18000u64.to_le_bytes()).unwrap();
    m.write(f + 0x398, &0x20000u64.to_le_bytes()).unwrap();
    m.write(0x200f0, &77u64.to_le_bytes()).unwrap();
    for (at, v) in [(0x370, 3f32), (0x374, 4.), (0x378, 0.)] {
        m.write(f + at, &v.to_le_bytes()).unwrap();
    }
    let frame = a.basic_frame(&m, f).unwrap();
    assert_eq!((frame.game, frame.enemy), (0x18000, 77));
    assert_eq!(
        a.prepare_basic_target(&m, &frame, [0., 0., 8.]).unwrap(),
        5.
    );
    assert_eq!(m.read(f + 0x27c, 4).unwrap(), 8f32.to_le_bytes());
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
    let mut host = actor(0xe00, 1, 1, 111);
    host[0xd9e] = 1;
    host[0xda3] = 1;
    host[0xa70..0xa78].copy_from_slice(&0x20000u64.to_le_bytes());
    host[0xa78..0xa80].copy_from_slice(&2u64.to_le_bytes());
    let mut server = vec![0; 0x2100];
    server[0x10..0x10 + host.len()].copy_from_slice(&host);
    server[0xdb8..0xdbc].copy_from_slice(&3u32.to_le_bytes());
    server[0xdf8..0xe00].copy_from_slice(&2u64.to_le_bytes());
    let mut first = actor(0xa38, 3, 3, 222);
    first[0xa34] = 1;
    let mut second = actor(0xa38, 5, 1, 333);
    second[0xa34] = 2;
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
    let view = asx_gameplay::official::PausePresentation {
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
    server[0xdb8..0xdbc].copy_from_slice(&3u32.to_le_bytes());
    server[0xdf0..0xdf8].copy_from_slice(&0x20000u64.to_le_bytes());
    server[0xdf8..0xe00].copy_from_slice(&1u64.to_le_bytes());
    let mut peer = vec![0; 0x758];
    peer[..4].copy_from_slice(&1u32.to_le_bytes());
    peer[8..16].copy_from_slice(&1u64.to_le_bytes());
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
    assert_eq!(a.menu_peer(&m, 0x10000, 0).unwrap(), 1);
    assert!(a.menu_peer(&m, 0x10000, 1).is_err());
    assert_eq!(a.menu_wire(&m, 0x30000).unwrap(), (12, true));
    let mut bad = m;
    bad.0.get_mut(&0x40000).unwrap()[8] = 2;
    assert!(a.menu_wire(&bad, 0x30000).is_err());
    bad.0.get_mut(&0x30000).unwrap()[48] = 10;
    assert!(a.menu_wire(&bad, 0x30000).is_err());
}

#[test]
fn owner_spell_formatter_uses_the_selected_130_game_context() {
    let a = SelectedAdapter::select(current::SHA256).unwrap();
    let mut game = vec![0; 0xe00];
    game[0xd9e] = 1;
    game[0xc98..0xca0].copy_from_slice(&2u64.to_le_bytes());
    let mut ability = vec![0; 12];
    ability[..8].copy_from_slice(&0x50000u64.to_le_bytes());
    ability[8..].copy_from_slice(&2u32.to_le_bytes());
    let m = Blocks(
        [
            (0x40000, ability),
            (0x50040, 845u32.to_le_bytes().to_vec()),
            (0x60000, game),
        ]
        .into(),
    );
    assert_eq!(
        a.spell_presentation_owner(&m, 0x40000, 0x604c8, 0).unwrap(),
        Some((2, 845, 2))
    );
    assert_eq!(
        a.spell_presentation_owner(&m, 0x40000, 0x604c8, 1).unwrap(),
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
    let mut host = actor(0xda8, 1, 1, 111);
    host[0xd9e] = 1;
    host[0xda3] = 1;
    // One normalized row fits here; one actual native0xa38 row overflows.
    host[0xa70..0xa78].copy_from_slice(&(u64::MAX - 0x990).to_le_bytes());
    host[0xa78..0xa80].copy_from_slice(&1u64.to_le_bytes());
    let mut server = vec![0; 0x2100];
    server[0x10..0x10 + host.len()].copy_from_slice(&host);
    server[0xdb8..0xdbc].copy_from_slice(&3u32.to_le_bytes());
    server[0xdf8..0xe00].copy_from_slice(&1u64.to_le_bytes());
    let memory = Blocks([(0x10000, server)].into());
    assert_eq!(
        adapter.party(&memory, 0x10000).unwrap_err(),
        "Invalid official context pointer"
    );
}

#[test]
fn shield_break_130_reads_game_from_rbx_and_spell_from_r12() {
    let a = SelectedAdapter::select(asx_game::current::SHA256).unwrap();
    let mut registers = [0u64; 15];
    registers[7] = 0x10000;
    registers[12] = 0xbad;
    registers[13] = 0xdead;
    registers[11] = 17;
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
