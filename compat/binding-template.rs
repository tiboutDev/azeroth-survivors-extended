//! Exact {{CLIENT_VERSION}} binding. Glyphs disabled; duplicate rewards use native marks.
{{SHA256}}
{{VERSION}}
{{ADDRESSES}}
{{EXTENSION_ADDRESSES}}
{{ENTRIES}}
{{EXTENSION_ENTRIES}}
pub(crate) fn rva(old: u64) -> u64 {
    ADDRESSES
        .iter()
        .find(|(a, _)| *a == old)
        .map(|(_, b)| *b)
        .expect("Unreviewed {{CLIENT_VERSION}} address")
}
pub(crate) fn baseline(native: u64) -> u64 {
    ADDRESSES
        .iter()
        .find(|(_, b)| *b == native)
        .map(|(a, _)| *a)
        .unwrap_or(0)
}
pub(crate) fn expected(old: u64) -> &'static [u8] {
    ENTRIES
        .iter()
        .find(|(a, _)| *a == old)
        .map(|(_, b)| *b)
        .expect("Unreviewed {{CLIENT_VERSION}} bytes")
}
pub(crate) fn extension(old117: u64) -> u64 {
    EXTENSION_ADDRESSES
        .iter()
        .find(|(a, _)| *a == old117)
        .map(|(_, b)| *b)
        .expect("Unreviewed {{CLIENT_VERSION}} extension")
}
pub(crate) fn extension_bytes(old117: u64) -> &'static [u8] {
    EXTENSION_ENTRIES
        .iter()
        .find(|(a, _)| *a == old117)
        .map(|(_, b)| *b)
        .expect("Unreviewed {{CLIENT_VERSION}} extension bytes")
}
// Only reviewed ASX-consumed field spans are translated; padding gaps are excluded.
// Resolved against the reviewed current image; no historical descriptor calls.
fn translated(spans: &[(u64, u64, u64)], offset: u64) -> u64 {
    let &(start, _, native) = spans
        .iter()
        .find(|(start, end, _)| (*start..=*end).contains(&offset))
        .expect("Unreviewed {{CLIENT_VERSION}} field");
    native + offset - start
}
{{GAME_RESOLVED_FIELDS}}
pub(crate) fn game_field(offset:u64)->u64{translated(GAME_RESOLVED_FIELDS,offset)}

{{ACTOR_RESOLVED_FIELDS}}
pub(crate) fn actor_field(offset:u64)->u64{translated(ACTOR_RESOLVED_FIELDS,offset)}

{{FIELD117_RESOLVED_FIELDS}}
pub(crate) fn field117(offset:u64)->u64{translated(FIELD117_RESOLVED_FIELDS,offset)}

{{FIELD1201_RESOLVED_FIELDS}}
pub(crate) fn field1201(offset:u64)->u64{translated(FIELD1201_RESOLVED_FIELDS,offset)}

{{CONTEXT_SIZE}}
{{SERVER_ROLE}}
{{SERVER_PEERS}}
{{SERVER_GAME_STREAM}}
{{SERVER_UI_SINK}}
{{MENU_ATLAS_FRAME}}
{{MENU_COLOR_FRAME}}
{{KEPT_TOOLTIP_ROW}}
{{BASIC_GAME}}
{{BASIC_ENEMY}}
{{BASIC_POSITION}}
{{BASIC_ORIGIN_XY}}
{{BASIC_ORIGIN_Z}}
{{ENCOUNTER_GAME_FRAME}}
{{ENEMY_STRIDE}}
{{ENEMY_DEAD}}
{{ENEMY_GUID}}
{{ENEMY_TARGET}}
{{ENEMY_AURAS_START}}
{{ENEMY_AURAS_END}}
{{ENEMY_HP}}
{{ENEMY_POSITION}}
{{ENEMY_VULNERABILITY}}
{{ITEM_CHOICE_RVA}}
{{RING_TABLE}}
{{AMULET_TABLE}}
{{ITEM_CHOICE_BYTES}}
{{BLESSING_SPAWN_BYTES}}
{{KEPT_TOOLTIP_BINDINGS}}
{{KEPT_TOOLTIP_FOCUS}}

// Diagnostic-only native KEY_AT producer; Bevy KeyCode anchors B=21/C=22/N=33/P=35/W=42.

// Native UI Input::Pause(bool) and the existing reliable input stream.
{{MENU_INPUT}}
{{MENU_SEND}}
{{MENU_OPCODE}}
{{SERVER_PEER_PTR}}
{{SERVER_PEER_STRIDE}}

// Escape local-menu branch -> native Input::Pause(true), same native frame.
{{MENU_KEY}}
{{MENU_KEY_NATIVE_PAUSE}}

// Native reward-card horizontal gradient separator and the selected Node layout.
{{BONUS_SEPARATOR_BUILD}}
{{BONUS_SEPARATOR_SPAWN}}
{{BONUS_SEPARATOR_CALLER}}
{{NODE_SIZE}}
// Selected Node deserializer: base RBP+0x50, column_gap at RBP+0x198.
{{NODE_COLUMN_GAP}}

// CMSG_RAID_TARGET_UPDATE decoder and native eight-icon table in Server.
{{SERVER_RAID_MARKS}}
{{RAID_MARKER_COMMAND}}

// World combat text: after the native interpolation/clamp, before projection.
{{DAMAGE_TEXT_SIZE}}
{{DIRECT_HEAL_TEXT}}
{{PERIODIC_HEAL_TEXT}}
{{TEXT_ENTITY}}
{{TEXT_RESERVE}}
{{HEAL_TEXT_COLOR}}

// Native vplates slices and layout application, before their first rendered frame.
{{PLATE_UPDATE}}
{{PLATE_STYLE}}
{{PLATE_STYLE_CALLERS}}
{{PLATE_ROW_SIZE}}
{{PLATE_CACHE_PTR}}
{{PLATE_CACHE_LEN}}
{{PLATE_CACHE_ROW}}

{{UI_UNIT_HASH}}
{{UI_UNIT_MAP}}
{{UI_UNIT_STRIDE}}
{{UI_UNIT_RANK}}
{{UI_ALIASES}}
{{UI_SELF_GUID}}
{{UI_PARTY_GUIDS}}
{{NAMEPLATE_SCALE}}

// Shared reward bonus: native icon-row title spawn and Node/ChildOf container.
{{BONUS_TITLE_SPAWN}}
{{BONUS_COLUMN_SPAWN}}
{{BONUS_HEADER_CALLER}}
// Reviewed Node deserializer 0x28758e0 (Node starts at RBP+0x50),
// AlignItems::Start decoder 0x2876550 returns 1.
{{BONUS_NODE_ROW_GAP}}
{{BONUS_NODE_ALIGN_ITEMS}}
{{BONUS_ALIGN_START}}

// Optional Treasure Goblin contacts; checked separately from required ASX activation.
{{GOBLIN_SPAWN}}
{{GOBLIN_DEFINITION}}
{{GOBLIN_PACKET_SEND}}
{{GOBLIN_QUERY_BUILD}}
{{GOBLIN_CLIP}}
{{GOBLIN_MAP_TABLE}}
{{GOBLIN_ENTRY}}
{{GOBLIN_DISPLAY}}
{{GOBLIN_POSITION}}
{{GOBLIN_PACKET_PROBE}}

// Native baked NPC texture path builder; optional, exact selected skin only.
{{GOBLIN_SKIN}}

// Exact builder used by native wave announcements (A pack hunts you!).
{{GOBLIN_EVENT}}

{{GOBLIN_RECYCLE}}

{{GOBLIN_OUTFIT_METADATA}}

{{GOBLIN_OUTFIT_KEY}}

// Final navigation-adjusted basic movement, before native packet construction.
{{GOBLIN_FINAL_MOTION}}

{{GOBLIN_AMULET_POOL}}

{{GOBLIN_AMULET_EXCLUDE}}

{{EVENT_OBJECT_CREATE}}

{{EVENT_OBJECT_INFO}}

{{EVENT_UNIT_FIELDS_SET}}

{{EVENT_UNIT_FIELDS_DROP}}

{{EVENT_UNIT_CREATE}}

{{EVENT_UPDATE_PACKET}}

{{EVENT_GROUND_HEIGHT}}

/// Native app-data lookup returning the retained Rc<RefCell<UiState>>.
{{UI_CONTEXT_LOOKUP}}

// Versioned optional event ABI/data: never infer these from another client.
{{GOBLIN_OUTFIT_ATLAS}}
{{GOBLIN_OUTFIT_APPEARANCE}}
{{GOBLIN_OUTFIT_ENTITY}}
{{GOBLIN_OUTFIT_ARMOR_KEY}}
{{GOBLIN_OUTFIT_SKIN_CALLER}}
{{GOBLIN_OUTFIT_MODELS}}
{{BOSS_MARKS_TABLE}}
{{NORMAL_MARKS_FACTOR_RVA}}
{{HEROIC_REWARD_TABLE}}

// Indices into the saved register contract supplied by the support bridge.
{{SHIELD_BREAK_GAME_REGISTER}} // R13
{{SHIELD_BREAK_SPELL_REGISTER}} // R12

// Direct/periodic healing callbacks borrow their own native ECS frames.
{{HEAL_TEXT_PAYLOAD}}
{{HEAL_TEXT_OBJECTS}}
{{HEAL_TEXT_QUERY}}
{{HEAL_TEXT_PARAMS}}

{{AURA_ROWS_FRAME_SLOT}}

// The compositor carries its variant tag and appearance in live registers.
// This frame owns only the 52-byte untagged armor payload.
{{GOBLIN_OUTFIT_KEY_SIZE}}
{{GOBLIN_OUTFIT_KEY_PREFIX}}
{{GOBLIN_OUTFIT_KEY_RACE_OFFSET}}

// Diagnostic-only sink entry. Never an activation or settings-persistence channel.
{{DIAGNOSTIC_WRITER}}

// Selected integration keys and constructor layouts; resolve keys through selected maps.
{{BLESSING_BOSS_STRIDE}}
{{BLESSING_ENCOUNTER_PHASE}}
{{BLESSING_ENDLESS_BYTES}}
{{BLESSING_ENDLESS_RVA}}
{{BLESSING_EVENT_BYTES}}
{{BLESSING_EVENT_RVA}}
{{BLESSING_GAME_MAP}}
{{BLESSING_GROUP_PHASE}}
{{BLESSING_MAP_COUNT}}
{{BLESSING_MAP_STRIDE}}
{{BLESSING_MAP_TABLE}}
{{BLESSING_POOL_BYTES}}
{{BLESSING_POOL_FRAME}}
{{BLESSING_POOL_RVA}}
{{BLESSING_SPAWN_RVA}}
{{BLESSING_STATS_BYTES}}
{{BLESSING_STATS_RVA}}
{{BLESSING_STAT_FIELDS}}
{{FATIGUE_AURA_BYTES}}
{{FATIGUE_AURA_RVA}}
{{GLYPH_ABILITIES}}
{{GLYPH_ABILITY_STRIDE}}
{{GLYPH_ACCEPT_BYTES}}
{{GLYPH_ACCEPT_RVA}}
{{GLYPH_CARD_BYTES}}
{{GLYPH_CARD_RVA}}
{{GLYPH_CHOICES}}
{{GLYPH_CHOICE_KIND}}
{{GLYPH_CHOICE_PAYLOAD}}
{{GLYPH_CHOICE_RARITY}}
{{GLYPH_CHOICE_STRIDE}}
{{GLYPH_CHOOSING}}
{{GLYPH_DEFINITIONS}}
{{GLYPH_DERIVED}}
{{GLYPH_ICON_BYTES}}
{{GLYPH_ICON_RVA}}
{{GLYPH_LINE_BYTES}}
{{GLYPH_LINE_RVA}}
{{GLYPH_MODIFIERS_BYTES}}
{{GLYPH_MODIFIERS_RVA}}
{{GLYPH_ON_HIT_BYTES}}
{{GLYPH_ON_HIT_RVA}}
{{GLYPH_PERKS}}
{{GLYPH_REROLLS}}
{{GLYPH_REROLL_BYTES}}
{{GLYPH_REROLL_RVA}}
{{GLYPH_SHEEP_COUNT}}
{{GLYPH_SHEEP_GATE_BYTES}}
{{GLYPH_SHEEP_GATE_RVA}}
{{GLYPH_STATUS_COLOR}}
{{GLYPH_STATUS_FONT_SIZE}}
{{GLYPH_TEXT_NODE_BYTES}}
{{GLYPH_TEXT_NODE_RVA}}
{{GLYPH_UI_ACTION_BYTES}}
{{GLYPH_UI_ACTION_RVA}}
{{GLYPH_UI_ASSETS}}
{{GLYPH_UI_BYTES}}
{{GLYPH_UI_CENTER}}
{{GLYPH_UI_COLOR}}
{{GLYPH_UI_HEADER_CALLER}}
{{GLYPH_UI_HEADER_DIRECTION}}
{{GLYPH_UI_HEADER_HEIGHT}}
{{GLYPH_UI_HEADER_JUSTIFY}}
{{GLYPH_UI_HEADER_MARGIN_TOP}}
{{GLYPH_UI_HEADER_NODE}}
{{GLYPH_UI_HEADER_WIDTH}}
{{GLYPH_UI_RVA}}
{{AGGRO_CANDIDATES}}
{{AMULET_CHOICE_TAG}}
{{AMULET_KEYS}}
{{AMULET_OWNED}}
{{AMULET_RANK}}
{{AMULET_STRIDE}}
{{KEPT_TOOLTIP_MARKER}}
{{RING_INVENTORY}}
{{RING_KEYS}}
{{RING_STAT_INPUT}}
{{RING_STAT_KINDS}}
{{RING_STAT_OUTPUTS}}
{{RING_STRIDE}}
{{ARMORY_GOLD_STAT}}
{{ARMORY_STATS_HEADER}}

{{FRACTIONAL_HEAL_POINT}}
{{HEAL_LOG_PACKET}}
{{HEAL_LOG_RESERVE}}
{{HEAL_LOG_QUEUE}}

{{PERIODIC_TEXT_POINT}}

// Reviewed event fields; symbolic offsets stay outside event orchestration.
{{EVENT_ENDLESS_OFFSET}}
{{EVENT_TERRAIN_OFFSET}}
{{EVENT_SCENE_COUNTER_OFFSET}}
{{EVENT_CLOCK_OFFSET}}
{{GOBLIN_ENDLESS_KILLS_OFFSET}}
{{GOBLIN_MARK_FIELDS}}
