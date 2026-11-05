//! Exact {{CLIENT_VERSION}} binding. duplicate rewards use native marks.
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
// Only reviewed AZSX-consumed field spans are translated; padding gaps are excluded.
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
{{BASIC_DISTANCE_FRAME}}
{{BASIC_ENEMY}}
{{BASIC_POSITION}}
{{BASIC_ORIGIN_XY}}
{{BASIC_ORIGIN_Z}}
{{ENCOUNTER_GAME_FRAME}}
{{ENCOUNTER_CONTEXT}}
{{ENEMY_STRIDE}}
{{ENEMY_DEAD}}
{{ENEMY_GUID}}
{{ENEMY_TARGET}}
{{ENEMY_AURAS_START}}
{{ENEMY_AURAS_END}}
{{ENEMY_HP}}
{{ENEMY_POSITION}}
{{ENEMY_VULNERABILITY}}
{{ENEMY_ELITE}}
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
{{MENU_INPUT_APPLY}}
{{MENU_TITLE_RESULT}}
{{MENU_UI_QUEUE}}
{{MENU_ESCAPE_QUEUE_RETURN}}
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


{{UI_ALIASES}}
{{UI_SELF_GUID}}
{{UI_PARTY_GUIDS}}

// Shared reward bonus: native icon-row title spawn and Node/ChildOf container.
{{BONUS_TITLE_SPAWN}}
{{BONUS_COLUMN_SPAWN}}
{{BONUS_HEADER_CALLER}}
// Reviewed Node deserializer 0x28758e0 (Node starts at RBP+0x50),
// AlignItems::Start decoder 0x2876550 returns 1.
{{BONUS_NODE_ROW_GAP}}
{{BONUS_NODE_ALIGN_ITEMS}}
{{BONUS_ALIGN_START}}

// Optional Banished Demon contacts, exact selected image.
{{DEMON_BANISH_OFFSET}}
{{DEMON_BANISH_READ}}
{{DEMON_BANISH_CONSUME}}
{{DEMON_BANISH_SERIALIZE}}
{{DEMON_BANISH_ACTION}}
{{DEMON_DEFINITIONS}}
{{DEMON_ENTRIES}}
{{DEMON_DISPLAYS}}
{{DEMON_EVENT_ENTRY}}


// Optional Treasure Goblin contacts; checked separately from required AZSX activation.
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
{{GOBLIN_MOTION_XY_XMM}}

{{GOBLIN_AMULET_POOL}}
{{GOBLIN_REWARD_GAME_FRAME}}
{{GOBLIN_REWARD_POOL_FRAME}}

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
{{MINIMAP_STATE_FIELDS}}

// Versioned optional event ABI/data: never infer these from another client.
{{GOBLIN_OUTFIT_ATLAS}}
{{GOBLIN_OUTFIT_APPEARANCE}}
// Frame offset of the full entity handle, or None when both callbacks retain it in R12.
{{GOBLIN_OUTFIT_ENTITY}}
{{GOBLIN_OUTFIT_ARMOR_KEY}}
{{GOBLIN_OUTFIT_SKIN_CALLER}}
{{GOBLIN_OUTFIT_MODELS}}
{{BOSS_MARKS_TABLE}}
{{NORMAL_MARKS_FACTOR_RVA}}
{{HEROIC_REWARD_TABLE}}

// Indices into the saved register contract supplied by the support bridge.
{{SHIELD_BREAK_GAME_REGISTER}} // RBX
{{SHIELD_BREAK_SPELL_REGISTER}} // R15

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
{{REWARD_ACCEPT_BYTES}}
{{REWARD_ACCEPT_RVA}}
{{REWARD_CARD_BYTES}}
{{REWARD_CARD_RVA}}
{{BONUS_ICON_BYTES}}
{{BONUS_ICON_RVA}}
{{BONUS_LINE_BYTES}}
{{BONUS_LINE_RVA}}
{{SPELL_MODIFIERS_BYTES}}
{{SPELL_MODIFIERS_RVA}}
{{SPELL_ON_HIT_BYTES}}
{{SPELL_ON_HIT_RVA}}
{{SPELL_PERKS}}
{{REWARD_REROLLS}}
{{REWARD_REROLL_BYTES}}
{{REWARD_REROLL_RVA}}
{{BONUS_STATUS_COLOR}}
{{BONUS_STATUS_FONT_SIZE}}
{{BONUS_TEXT_NODE_BYTES}}
{{BONUS_TEXT_NODE_RVA}}
{{BONUS_UI_ACTION_BYTES}}
{{BONUS_UI_ACTION_RVA}}
{{BONUS_UI_ASSETS}}
{{BONUS_UI_BYTES}}
{{BONUS_UI_CENTER}}
{{BONUS_UI_COLOR}}
{{BONUS_UI_HEADER_CALLER}}
{{BONUS_UI_HEADER_DIRECTION}}
{{BONUS_UI_HEADER_HEIGHT}}
{{BONUS_UI_HEADER_JUSTIFY}}
{{BONUS_UI_HEADER_MARGIN_TOP}}
{{BONUS_UI_HEADER_NODE}}
{{BONUS_UI_HEADER_WIDTH}}
{{BONUS_UI_RVA}}
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
{{SOLO_SCENE_SEQUENCE}}
{{SOLO_SCENE_SPLINE}}
{{SOLO_CHICKEN_ENTRY}}
{{EVENT_CLOCK_OFFSET}}
{{GOBLIN_ENDLESS_KILLS_OFFSET}}
{{GOBLIN_MARK_FIELDS}}

// Selected world-number register, resolver composition, event tags and run difficulty.
{{DAMAGE_TEXT_XMM}}
{{DAMAGE_TEXT_ROW_REGISTER}}
{{HEAL_TEXT_RESOLVER_ABI}}
{{TEXT_QUERY}}
{{HEAL_TEXT_TAG}}
// x86 register numbers for the slot and full entity handle at the visible-item return.
{{WEAPON_EQUIPMENT_REGISTERS}}
{{WEAPON_RUN_PROFILE_REGISTER}}
// Warrior, Paladin and Shaman talent tree bases and lengths in specialization order.
{{WEAPON_CATALOG_PROBE}}
{{WEAPON_CATALOG_FRAME}}
{{WEAPON_TALENT_TREES}}
{{COOP_TALENT_KEYS}}
{{SET_DOT_OWNER_CALLERS}}
{{WEAPON_PREVIEW_COLLECTION_SLOT}}
{{HEAL_TEXT_DISCRIMINANTS}}
{{HEROIC_LEVEL}}

// Native sink console logging, independent of the Game autostart flag.
{{NATIVE_UI_LOG_FLAG}}

// Native dead actors retain XP/levels without opening choice cards.
{{DOWNED_XP_GUARDS}}

{{DOWNED_XP_PENDING_FIELD}}

{{DOWNED_XP_HOST_REWARD_SLOT}}

{{ADDON_API_READER}}
{{ADDON_API_READER_BYTES}}

{{LOBBY_SUBMIT}}
{{LOBBY_SUBMIT_BYTES}}
{{LOBBY_NAME}}

{{SOLO_ENEMY_ELITE}}
{{SOLO_ENEMY_ENCOUNTER}}
{{ENEMY_GUARDIAN_INDEX}}
{{ENEMY_DEFINITION}}
{{SOLO_ENEMY_ROOT_UNTIL}}
{{SOLO_NATIVE_SHIELD}}
{{SOLO_MORTAR_SPELL}}
{{SOLO_SAPPER_SPELL}}
{{SOLO_MORTAR_TOKEN}}
{{SOLO_PROJECTILE_VECTOR}}
{{SOLO_PROJECTILE_STRIDE}}
{{SOLO_HURT_REDUCTION}}
{{SOLO_PLAYER_SCALE_BASE}}
{{SOLO_PROJECTILE_CREATE}}
{{SOLO_PROJECTILE_DELAY_READ}}
{{SOLO_PROJECTILE_ORIGIN_READ}}

{{UNIT_FIELD_VALUE_ABI}}

{{PERK_ORIGINS_HEADER}}

// Perk observation layout; copies are never native dispatcher input.
{{PERK_OBSERVATION_LAYOUT}}

// Healing budget: retained raw-heal contract or health restored plus absorb gained.
{{NATURE_HEALING_BUDGET}}

// Optional reviewed RNG, chance, multiplier and two proc-guard field offsets.
{{NATURE_CRITICAL_FIELDS}}

// Native map-pair marks table extent; later zones retain its last reward row.
{{BOSS_MARKS_ROW_COUNT}}

// Zero retains the historical additive support attribution policy.
{{STRONGEST_SHIELD_POOLS}}

{{DOWNED_XP_HOST_REWARD_ABI}}

{{DOWNED_XP_PREREQUISITES}}

{{ACTOR_PERKS_HEADER}}

{{ABILITY_STRIDE}}
{{LEARNED_ABILITY_STRIDE}}

{{ABILITY_MODIFIERS_OFFSET}}
{{SPELL_MODIFIERS_SIZE}}

{{MARK_POWER_FIELDS}}

{{CAST_ABILITY_SIZE}}

// Selected native sheep ownership headers and enemy expiry.
{{POLYMORPH_OWNERS_FIELDS}}
{{ENEMY_POLYMORPH_EXPIRY}}

// Indexes into RSI/R12/R13: scan Game, rupture Game, enemy GUID.
{{POLYMORPH_HIT_REGISTERS}}

// Native set-proc frame Game/spell and enemy coefficient/expiry pair.
{{SET_VULNERABILITY_LAYOUT}}
// Zero retains direct event/count registers at the private clone seam.
{{SET_CLONE_FRAME_FIELDS}}

// Native DoT append return/saved-frame slots and row-offset register in RAX/RCX/RDX/RDI.
{{SET_DOT_STACK_FIELDS}}
{{SET_DOT_ROW_REGISTER}}

{{HURT_SOURCE_LAYOUT}}

{{SPELL_PRESENTATION_RANK_OFFSET}}

{{XP_EFFECT_REGISTER}}
