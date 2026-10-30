//! Exact-build selection for the official multiplayer engine. This registry is
//! independent of legacy custom-engine profiles and never installs their hooks.
pub use crate::official_native::pause::PauseLayout;
pub use crate::official_native::NATIVE_PLAYERS;
use crate::{memory::Mem, official_native, Result};
pub trait MutableMemory: Mem {
    fn write(&self, address: u64, bytes: &[u8]) -> Result<()>;
    fn allocate_native(&self, bytes: &[u8]) -> Result<u64>;
    fn free_native(&self, address: u64, capacity: usize) -> Result<()>;
}
pub trait CurrencyMemory: MutableMemory {
    fn coinage(&self, map: u64, gold: u32) -> Result<()>;
}
pub trait SupportCalls: CurrencyMemory {
    fn loot_send(
        &self,
        _server: u64,
        _peer: Option<u64>,
        _opcode: u16,
        _bytes: &[u8],
    ) -> Result<()> {
        Err("Native loot transport unavailable".into())
    }
    fn loot_ui(&self, _server: u64, _value: &serde_json::Value) -> Result<()> {
        Err("Native loot UI unavailable".into())
    }

    fn open_item_choice(&self, _game: u64) -> Result<()> {
        Err("Native item presentation unavailable".into())
    }
    fn resolve_glyph_modifiers(&self, _output: u64, _perks: u64, _definition: u64) -> Result<()> {
        Err("Glyph calls unavailable".into())
    }
    fn native_boss_health(&self, game: u64, index: u64) -> Result<f32>;
    fn award_experience(&self, game: u64, amount: u32) -> Result<()>;
    fn emit_support_effect(
        &self,
        game: u64,
        spell: u32,
        data: &[u8; 12],
        duration: f32,
    ) -> Result<()>;
    fn emit_support_heal(&self, game: u64, spell: u32, fraction: f32) -> Result<()>;
    fn refresh_support(&self, game: u64) -> Result<()>;
    fn support_hit(&self, game: u64, guid: u64, amount: f32, spell: u32) -> Result<()>;
    fn support_hurt(&self, game: u64, amount: f32, enemy: u64, melee: bool) -> Result<()>;
    fn support_damage(&self, game: u64, guid: u64, amount: f32) -> Result<()>;
    fn shield_proc(
        &self,
        game: u64,
        event: &[u8; 40],
        origin: [f32; 3],
        recipient: u64,
    ) -> Result<()>;
    fn cast_actor(&self, game: u64, ability: u64) -> Result<bool>;
    fn swap_actor(&self, game: u64, row: u64) -> Result<()>;
    fn flush_actor(&self, game: u64) -> Result<()>;
    fn healing_meter(&self, game: u64, spell: u32, amount: f32) -> Result<()>;
}
pub use crate::official_native::support::{
    AuraState, BreakFrame, NativeEffect, OwnedEffect, PeriodicTick, PerkLoan, RetaliationTick,
    SupportCaster, SupportDelivery, SupportReceipt, SupportTalents,
};
pub use official_native::{
    ActorObservation, BasicFrame, CombatActor, Observation, PartyObservation, Role,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExperienceAward {
    pub source: &'static str,
    pub raw: u32,
    pub amount: u32,
}
#[derive(Clone, Copy, Debug)]
pub struct Entry {
    pub rva: u64,
    pub bytes: &'static [u8],
}
#[derive(Clone, Copy, Debug)]
pub struct SelectedAdapter {
    native: crate::official_native::Adapter,
}
impl SelectedAdapter {
    pub fn loot_fixture_entry(self) -> Option<Entry> {
        self.native.loot_fixture_entry()
    }
    pub unsafe fn loot_fixture_dispatch(
        self,
        m: &dyn MutableMemory,
        frame: u64,
        base: u64,
        peer: u64,
        item: u32,
    ) -> Result<u64> {
        self.native
            .loot_fixture_dispatch(m, frame, base, peer, item)
    }
    pub fn loot_victory(self, m: &dyn Mem, game: u64) -> Result<bool> {
        self.native.loot_victory(m, game)
    }
    pub fn loot_actor_gold(
        self,
        m: &dyn Mem,
        game: u64,
        slot: u32,
        guid: u64,
    ) -> Result<(f32, u8)> {
        self.native.loot_actor_gold(m, game, slot, guid)
    }
    pub unsafe fn loot_fixture_append(
        self,
        m: &dyn MutableMemory,
        game: u64,
        base: u64,
        slot: u32,
        item: u32,
    ) -> Result<bool> {
        self.native.loot_fixture_append(m, game, base, slot, item)
    }
    pub fn loot_entries(self) -> Vec<Entry> {
        self.native.loot_entries()
    }
    pub fn loot_send_entry(self, peer: bool) -> Option<Entry> {
        self.native.loot_send_entry(peer)
    }
    pub fn loot_offer(self, m: &dyn Mem, frame: u64, peer: u64) -> Result<(u64, u32, Vec<u32>)> {
        self.native.loot_offer(m, frame, peer)
    }
    pub fn loot_known(self, m: &dyn Mem, frame: u64) -> Result<bool> {
        self.native.loot_known(m, frame)
    }
    pub fn loot_packet(self, m: &dyn Mem, packet: u64) -> Result<Option<(u16, Vec<u8>)>> {
        self.native.loot_packet(m, packet)
    }
    pub fn loot_bytes(self, m: &dyn Mem, ptr: u64, len: u64) -> Result<Vec<u8>> {
        self.native.loot_bytes(m, ptr, len)
    }
    pub fn loot_hide_drop(
        self,
        m: &dyn MutableMemory,
        ptr: u64,
        len: u64,
    ) -> Result<serde_json::Value> {
        self.native.loot_hide_drop(m, ptr, len)
    }
    pub fn loot_ui_sink(self, m: &dyn Mem, server: u64) -> Result<(u64, u64)> {
        self.native.loot_ui_sink(m, server)
    }

    pub fn select(hash: &str) -> Result<Self> {
        Ok(Self {
            native: crate::official_native::Adapter::select(hash)?,
        })
    }

    pub fn name(self) -> &'static str {
        self.native.name()
    }
    pub fn pause_layout(self) -> PauseLayout {
        PauseLayout::new(self.native.pause_addresses())
    }
    pub fn version(self) -> &'static str {
        self.native.version()
    }
    pub fn server_update(self) -> Entry {
        self.native.server_update()
    }
    pub fn game_address(self, server: u64) -> Result<u64> {
        self.native.game_address(server)
    }
    pub fn fixture_cast_resolved(
        self,
        io: &dyn SupportCalls,
        game: u64,
        slot: u32,
        ability: u64,
    ) -> Result<bool> {
        self.native.fixture_cast_resolved(io, game, slot, ability)
    }
    pub fn fixture_support_definition(self, spell: u32) -> Result<u64> {
        self.native.fixture_support_definition(spell)
    }
    pub fn fixture_support_wounds(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        caster: u32,
        target: u32,
    ) -> Result<()> {
        self.native
            .fixture_support_wounds(mem, game, caster, target)
    }
    pub fn fixture_cast_actor(
        self,
        io: &dyn SupportCalls,
        game: u64,
        slot: u32,
        ability: u64,
    ) -> Result<bool> {
        self.native.fixture_cast_actor(io, game, slot, ability)
    }
    pub fn support_buff_slots(self, mem: &dyn Mem, game: u64, spell: u32) -> Result<Vec<u32>> {
        self.native.support_buff_slots(mem, game, spell)
    }
    pub fn fixture_perk_loan(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        base: u64,
        rapture: bool,
        next_allowed: f32,
    ) -> Result<PerkLoan> {
        self.native
            .fixture_perk_loan(mem, game, base, rapture, next_allowed)
    }
    pub fn fixture_restore_perks(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        loan: PerkLoan,
    ) -> Result<Option<f32>> {
        self.native.fixture_restore_perks(mem, game, loan)
    }
    pub fn fixture_break_shield(self, io: &dyn SupportCalls, game: u64, slot: u32) -> Result<bool> {
        self.native.fixture_break_shield(io, game, slot)
    }
    pub fn hurt_entry(self) -> Entry {
        self.native.hurt_entry()
    }
    pub fn elite_entry(self) -> Entry {
        self.native.elite_entry()
    }
    pub fn elite_game(self, mem: &dyn Mem, r: [u64; 3]) -> Result<Option<u64>> {
        self.native.elite_game(mem, r)
    }
    pub fn deliver_elite_item(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        slot: u32,
        guid: u64,
    ) -> Result<bool> {
        self.native.deliver_elite_item(mem, game, slot, guid)
    }
    pub fn fixture_guardian(self, io: &dyn SupportCalls, game: u64) -> Result<Option<u64>> {
        self.native.fixture_guardian(io, game)
    }
    pub fn fixture_guardian_target(
        self,
        io: &dyn SupportCalls,
        game: u64,
    ) -> Result<Option<(u64, f32)>> {
        self.native.fixture_guardian_target(io, game)
    }
    pub fn support_query_origin(
        self,
        mem: &dyn MutableMemory,
        p: u64,
        stack: u64,
        return_rva: u64,
        origin: [f32; 3],
    ) -> Result<bool> {
        self.native
            .support_query_origin(mem, p, stack, return_rva, origin)
    }
    pub fn support_emit_entries(self) -> [Entry; 4] {
        self.native.support_emit_entries()
    }
    pub fn fixture_retaliation_hit(
        self,
        io: &dyn SupportCalls,
        game: u64,
        slot: u32,
    ) -> Result<Option<u64>> {
        self.native.fixture_retaliation_hit(io, game, slot)
    }
    pub fn native_effect(self, mem: &dyn Mem, game: u64, spell: u32) -> Result<NativeEffect> {
        self.native.native_effect(mem, game, spell)
    }
    pub fn emit_hybrid(
        self,
        io: &dyn SupportCalls,
        game: u64,
        base: u64,
        spell: u32,
        before: u64,
        maximum: f32,
    ) -> Result<()> {
        self.native
            .emit_hybrid(io, game, base, spell, before, maximum)
    }
    pub fn hide_retaliation(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        owners: &[OwnedEffect],
    ) -> Result<Vec<RetaliationTick>> {
        self.native.hide_retaliation(mem, game, owners)
    }
    pub fn restore_retaliation(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        t: &RetaliationTick,
    ) -> Result<bool> {
        self.native.restore_retaliation(mem, game, t)
    }
    pub fn retaliate(
        self,
        io: &dyn SupportCalls,
        game: u64,
        t: &RetaliationTick,
        enemy: u64,
    ) -> Result<(f32, f32)> {
        self.native.retaliate(io, game, t, enemy)
    }
    pub fn menu_bindings(self, base: usize) -> crate::official_native::MenuBindings {
        self.native.menu_bindings(base)
    }
    pub fn menu_cache_entries(self) -> [Entry; 2] {
        self.native.menu_cache_entries()
    }
    pub fn menu_entries(self) -> [Entry; 7] {
        self.native.menu_entries()
    }
    pub fn spell_presentation_entry(self) -> Entry {
        self.native.spell_presentation_entry()
    }
    pub fn spell_presentation_owner(
        self,
        mem: &dyn Mem,
        ability: u64,
        stats: u64,
        kind: u8,
    ) -> Result<Option<(u32, u32, u32)>> {
        self.native
            .spell_presentation_owner(mem, ability, stats, kind)
    }
    pub fn spell_presentation_strings(
        self,
        mem: &dyn Mem,
        output: u64,
    ) -> Result<Vec<(u32, String)>> {
        self.native.spell_presentation_strings(mem, output)
    }
    pub fn party_addon_entry(self) -> Entry {
        self.native.party_addon_entry()
    }
    pub fn party_build(self, value: &serde_json::Value) -> Result<PartyBuild> {
        self.native.party_build(value)
    }
    pub fn decorate_party_addon(
        self,
        mem: &dyn MutableMemory,
        output: u64,
        wire: &str,
    ) -> Result<()> {
        self.native.decorate_party_addon(mem, output, wire)
    }
    pub fn ui_mailbox_writer(self) -> Option<Entry> {
        self.native.ui_mailbox_writer()
    }
    pub fn scaling_probe_entry(self) -> Entry {
        self.native.scaling_probe_entry()
    }
    pub fn fixture_scaling_probe(self, io: &dyn SupportCalls, game: u64) -> Result<[f32; 4]> {
        self.native.fixture_scaling_probe(io, game)
    }
    /// Original native instructions, retained only for read-only integrity checks.
    pub fn native_difficulty_entries(self) -> Vec<Entry> {
        self.native.native_difficulty_entries()
    }
    pub fn verify_native_difficulty(self, mem: &dyn Mem, base: u64) -> Result<()> {
        for entry in self.native_difficulty_entries() {
            if mem.read(base + entry.rva, entry.bytes.len())? != entry.bytes {
                return Err("Native difficulty instructions were modified".into());
            }
        }
        Ok(())
    }
    pub fn deferred_xp_entry(self) -> Entry {
        self.native.deferred_xp_entry()
    }
    pub fn deferred_xp_actor(self, mem: &dyn Mem, game: u64) -> Result<Option<(u32, u64)>> {
        self.native.deferred_xp_actor(mem, game)
    }
    pub fn replay_experience(
        self,
        io: &dyn SupportCalls,
        game: u64,
        slot: u32,
        guid: u64,
        amount: u32,
    ) -> Result<bool> {
        self.native.replay_experience(io, game, slot, guid, amount)
    }
    pub fn combat_descriptions(self, m: &dyn Mem, b: u64) -> Result<Vec<(u64, String)>> {
        self.native.combat_descriptions(m, b)
    }
    pub fn combat_entries(self) -> [Entry; 5] {
        self.native.combat_entries()
    }
    pub fn combat_actor(self, m: &dyn Mem, g: u64, b: u64) -> Result<CombatActor> {
        self.native.combat_actor(m, g, b)
    }
    pub fn combat_enemies(self, m: &dyn Mem, g: u64) -> Result<Vec<asx_gameplay::Enemy>> {
        self.native.combat_enemies(m, g)
    }
    pub fn combat_hp(self, m: &dyn Mem, g: u64, e: u64) -> Result<Option<f32>> {
        self.native.combat_hp(m, g, e)
    }
    pub fn write_combat_mark(self, m: &dyn MutableMemory, g: u64, e: u64, s: f32) -> Result<()> {
        self.native.write_combat_mark(m, g, e, s)
    }
    pub fn basic_frame(self, m: &dyn Mem, f: u64) -> Result<BasicFrame> {
        self.native.basic_frame(m, f)
    }
    pub fn basic_resume_entry(self) -> (usize, &'static std::sync::atomic::AtomicUsize) {
        self.native.basic_resume_entry()
    }
    pub fn restore_basic_origin(
        self,
        m: &dyn MutableMemory,
        f: u64,
        g: u64,
        p: [f32; 3],
    ) -> Result<()> {
        self.native.restore_basic_origin(m, f, g, p)
    }
    pub fn prepare_basic_target(
        self,
        m: &dyn MutableMemory,
        f: &BasicFrame,
        p: [f32; 3],
    ) -> Result<f32> {
        self.native.prepare_basic_target(m, f, p)
    }
    pub fn hurt_actor(
        self,
        io: &dyn SupportCalls,
        g: u64,
        s: u32,
        id: u64,
        a: f32,
        e: u64,
        melee: bool,
    ) -> Result<Option<f32>> {
        self.native.hurt_actor(io, g, s, id, a, e, melee)
    }
    pub fn fixture_combat_enemy(
        self,
        m: &dyn MutableMemory,
        g: u64,
        s: u32,
    ) -> Result<Option<u64>> {
        self.native.fixture_combat_enemy(m, g, s)
    }
    pub fn attack_actor(
        self,
        io: &dyn SupportCalls,
        g: u64,
        s: u32,
        e: u64,
        sp: u32,
        a: f32,
    ) -> Result<()> {
        self.native.attack_actor(io, g, s, e, sp, a)
    }
    pub fn support_proc_entries(self) -> [Entry; 4] {
        self.native.support_proc_entries()
    }
    pub fn support_break_frame(self, registers: [u64; 15], values: [f32; 2]) -> Result<BreakFrame> {
        self.native.support_break_frame(registers, values)
    }
    pub fn support_overheal(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        t: SupportTalents,
        raw: f32,
        effective: f32,
    ) -> Result<()> {
        self.native.support_overheal(mem, game, t, raw, effective)
    }
    pub fn support_receipt(
        self,
        io: &dyn SupportCalls,
        game: u64,
        r: &SupportReceipt,
    ) -> Result<Option<usize>> {
        self.native.support_receipt(io, game, r)
    }
    pub fn support_shield_break(
        self,
        io: &dyn SupportCalls,
        game: u64,
        o: &OwnedEffect,
        origin: [f32; 3],
        amount: f32,
    ) -> Result<()> {
        self.native
            .support_shield_break(io, game, o, origin, amount)
    }
    pub fn support_enemies(
        self,
        mem: &dyn Mem,
        game: u64,
    ) -> Result<Vec<crate::enemies::EnemyExact>> {
        self.native.support_enemies(mem, game)
    }
    pub fn periodic_entry(self) -> Entry {
        self.native.periodic_entry()
    }
    pub fn support_active(self, mem: &dyn Mem, game: u64) -> Result<asx_gameplay::support::Active> {
        self.native.support_active(mem, game)
    }
    pub fn capture_aura(self, mem: &dyn Mem, game: u64, spell: u32) -> Result<AuraState> {
        self.native.capture_aura(mem, game, spell)
    }
    pub fn prepare_foreign_shield(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        state: &AuraState,
    ) -> Result<()> {
        self.native.prepare_foreign_shield(mem, game, state)
    }
    pub fn restore_aura(self, mem: &dyn MutableMemory, game: u64, state: &AuraState) -> Result<()> {
        self.native.restore_aura(mem, game, state)
    }
    pub fn effect_owner(
        self,
        mem: &dyn Mem,
        game: u64,
        caster: u32,
        guid: u64,
        spell: u32,
        shield: bool,
    ) -> Result<OwnedEffect> {
        self.native
            .effect_owner(mem, game, caster, guid, spell, shield)
    }
    pub fn hide_foreign_periodic(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        dt: f32,
        owners: &[OwnedEffect],
    ) -> Result<Vec<PeriodicTick>> {
        self.native.hide_foreign_periodic(mem, game, dt, owners)
    }
    pub fn restore_periodic(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        tick: &PeriodicTick,
    ) -> Result<bool> {
        self.native.restore_periodic(mem, game, tick)
    }
    pub fn support_entries(self) -> [Entry; 4] {
        self.native.support_entries()
    }
    pub fn flush_entry(self) -> Entry {
        self.native.flush_entry()
    }
    pub fn support_spell(self, mem: &dyn Mem, base: u64, ability: u64) -> Result<Option<u32>> {
        self.native.support_spell(mem, base, ability)
    }
    pub fn support_caster(self, mem: &dyn Mem, game: u64) -> Result<SupportCaster> {
        self.native.support_caster(mem, game)
    }
    pub fn support_queue_count(self, mem: &dyn Mem, game: u64) -> Result<u64> {
        self.native.support_queue_count(mem, game)
    }
    pub fn route_support(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        before: u64,
        spell: u32,
        target: Option<u32>,
        maximum: f32,
    ) -> Result<()> {
        self.native
            .route_support(mem, game, before, spell, target, maximum)
    }
    pub fn support_delivery(
        self,
        mem: &dyn Mem,
        game: u64,
        descriptor: u64,
    ) -> Result<Option<SupportDelivery>> {
        self.native.support_delivery(mem, game, descriptor)
    }
    pub fn credit_support(
        self,
        io: &dyn SupportCalls,
        game: u64,
        slot: u32,
        spell: u32,
        amount: f32,
    ) -> Result<(f64, f64)> {
        self.native.credit_support(io, game, slot, spell, amount)
    }
    pub fn chest_entry(self) -> Entry {
        self.native.chest_entry()
    }
    pub fn convert_entry(self) -> Entry {
        self.native.convert_entry()
    }
    pub fn fixture_conversion_flag(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        enabled: bool,
    ) -> Result<bool> {
        self.native.fixture_conversion_flag(mem, game, enabled)
    }
    pub fn conversion_ready(self, mem: &dyn Mem, game: u64) -> Result<bool> {
        self.native.conversion_ready(mem, game)
    }
    pub fn chest_state(self, mem: &dyn Mem, game: u64) -> Result<(usize, u32, u32)> {
        self.native.chest_state(mem, game)
    }
    pub fn project_current_chest(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        cost: u32,
        opened: u32,
    ) -> Result<()> {
        self.native.project_current_chest(mem, game, cost, opened)
    }
    pub fn deliver_chests(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        slot: u32,
        amount: u32,
        cost: u32,
        opened: u32,
    ) -> Result<bool> {
        self.native
            .deliver_chests(mem, game, slot, amount, cost, opened)
    }
    pub fn wallet_seed(
        self,
        mem: &dyn Mem,
        game: u64,
    ) -> Result<asx_gameplay::official::SharedWallet> {
        self.native.wallet_seed(mem, game)
    }
    /// Selected native gold invocation; preserves the existing hooked award path.
    pub unsafe fn event_credit_gold(self,base:u64,game:u64,amount:u32)->usize {
        unsafe{self.native.event_credit_gold(base,game,amount)}
    }
    pub fn gold_entry(self) -> Entry {
        self.native.gold_entry()
    }
    pub fn swap_entry(self) -> Entry {
        self.native.swap_entry()
    }
    pub fn currency_entry(self) -> Entry {
        self.native.currency_entry()
    }
    pub fn gold_source(self, site: u64) -> asx_gameplay::official::Award {
        self.native.gold_source(site)
    }
    pub fn racial_gold_bonus(self, race: u8) -> f32 {
        self.native.racial_gold_bonus(race)
    }
    pub fn current_gold(self, mem: &dyn Mem, game: u64) -> Result<(u32, f32, u8)> {
        self.native.current_gold(mem, game)
    }
    pub fn is_currency(self, game: u64, map: u64, field: u16) -> bool {
        self.native.is_currency(game, map, field)
    }
    pub fn project_current_wallet(
        self,
        mem: &dyn CurrencyMemory,
        game: u64,
        wallet: asx_gameplay::official::SharedWallet,
    ) -> Result<()> {
        self.native.project_current_wallet(mem, game, wallet)
    }
    pub fn project_wallet(
        self,
        mem: &dyn CurrencyMemory,
        game: u64,
        wallet: asx_gameplay::official::SharedWallet,
    ) -> Result<()> {
        self.native.project_wallet(mem, game, wallet)
    }
    pub fn shared_xp(self) -> Entry {
        self.native.shared_xp()
    }
    pub fn shared_xp_input(self, mem: &dyn Mem, request: u64, game: u64) -> Result<u32> {
        self.native.shared_xp_input(mem, request, game)
    }
    pub fn award_xp(self) -> Entry {
        self.native.award_xp()
    }
    pub fn racial_xp_bonus(self, race_id: u8) -> f32 {
        self.native.racial_xp_bonus(race_id)
    }
    pub fn normalize_xp(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        return_rva: u64,
        registers: [u64; 3],
        multiplier: f32,
        shared_guest: Option<u32>,
    ) -> Result<Option<ExperienceAward>> {
        self.native
            .normalize_xp(mem, game, return_rva, registers, multiplier, shared_guest)
    }
    pub fn hud(self) -> Entry {
        self.native.hud()
    }
    pub fn world_update(self) -> Entry {
        self.native.world_update()
    }
    pub fn command(self) -> Entry {
        self.native.command()
    }
    pub fn menu_key_entry(self) -> Option<Entry> {
        self.native.menu_key_entry()
    }
    pub fn menu_key_target(self) -> Option<u64> {
        self.native.menu_key_target()
    }
    pub fn menu_entry(self) -> Option<Entry> {
        self.native.menu_entry()
    }
    pub fn pause_notice(self, a: bool) -> Option<&'static str> {
        self.native.pause_notice(a)
    }
    pub fn menu_request(self, m: &dyn Mem, p: u64) -> Result<Option<bool>> {
        self.native.menu_request(m, p)
    }
    pub fn menu_peer(self, m: &dyn Mem, s: u64, p: u64) -> Result<u32> {
        self.native.menu_peer(m, s, p)
    }
    pub unsafe fn menu_send(self, m: &dyn Mem, b: u64, s: u64, v: &[u8]) -> Result<()> {
        self.native.menu_send(m, b, s, v)
    }
    pub fn menu_wire(self, m: &dyn Mem, s: u64) -> Result<(u64, bool)> {
        self.native.menu_wire(m, s)
    }
    pub fn menu_opcode(self) -> Option<u16> {
        self.native.menu_opcode()
    }
    pub fn fixture_pause_position(self, mem: &dyn Mem, server: u64) -> Result<[f32; 3]> {
        self.native.fixture_pause_position(mem, server)
    }
    pub fn command_is_movement(self, opcode: u16) -> bool {
        self.native.command_is_movement(opcode)
    }
    pub fn world_party(self, mem: &dyn Mem, game: u64) -> Result<Option<PartyObservation>> {
        self.native.world_party(mem, game)
    }
    pub fn hud_value(self, mem: &dyn Mem, output: u64) -> Result<serde_json::Value> {
        self.native.hud_value(mem, output)
    }
    pub fn ui_hud(self) -> Entry {
        self.native.ui_hud()
    }
    pub fn ui_event(self) -> Entry {
        self.native.ui_event()
    }
    pub fn heap(self) -> [Entry; 2] {
        self.native.heap()
    }
    pub fn pause_addresses(self) -> [usize; 13] {
        self.native.pause_addresses()
    }
    pub fn hud_phase(self, value: &serde_json::Value) -> Result<asx_gameplay::Phase> {
        self.native.hud_phase(value)
    }
    pub fn read_ui(self, mem: &dyn Mem, pointer: u64, length: u64) -> Result<serde_json::Value> {
        self.native.read_ui(mem, pointer, length)
    }
    pub fn decorate_pause(
        self,
        value: &mut serde_json::Value,
        view: &asx_gameplay::official::PausePresentation,
    ) {
        self.native.decorate_pause(value, view)
    }
    pub fn decode_pause(
        self,
        value: &serde_json::Value,
    ) -> Result<Option<asx_gameplay::official::PausePresentation>> {
        self.native.decode_pause(value)
    }
    pub fn replace_hud(
        self,
        mem: &dyn MutableMemory,
        output: u64,
        value: &serde_json::Value,
    ) -> Result<()> {
        self.native.replace_hud(mem, output, value)
    }
    pub fn ui_receiver(self, mem: &dyn Mem, server: u64, base: u64) -> Result<u64> {
        self.native.ui_receiver(mem, server, base)
    }
    pub fn set_ui_autopick(
        self,
        mem: &dyn MutableMemory,
        receiver: u64,
        enabled: bool,
    ) -> Result<bool> {
        self.native.set_ui_autopick(mem, receiver, enabled)
    }
    pub fn verify(self, mem: &dyn Mem, base: u64) -> Result<()> {
        let entry = self.scaling_probe_entry();
        if mem.read(base + entry.rva, entry.bytes.len())? != entry.bytes {
            return Err("Native scaling helper mismatch".into());
        }
        self.verify_native_difficulty(mem, base)?;
        for entry in [
            self.deferred_xp_entry(),
            self.combat_entries()[0],
            self.combat_entries()[1],
            self.combat_entries()[2],
            self.combat_entries()[3],
            self.combat_entries()[4],
            self.server_update(),
            self.hud(),
            self.world_update(),
            self.command(),
            self.award_xp(),
            self.shared_xp(),
            self.gold_entry(),
            self.swap_entry(),
            self.currency_entry(),
            self.chest_entry(),
            self.convert_entry(),
            self.support_entries()[0],
            self.support_entries()[1],
            self.support_entries()[2],
            self.support_entries()[3],
            self.flush_entry(),
            self.periodic_entry(),
            self.hurt_entry(),
            self.support_emit_entries()[0],
            self.support_emit_entries()[1],
            self.support_emit_entries()[2],
            self.support_emit_entries()[3],
            self.support_proc_entries()[0],
            self.support_proc_entries()[1],
            self.support_proc_entries()[2],
            self.support_proc_entries()[3],
            self.ui_hud(),
            self.ui_event(),
            self.heap()[0],
            self.heap()[1],
        ] {
            if mem.read(
                base.checked_add(entry.rva)
                    .ok_or("Official image overflow")?,
                entry.bytes.len(),
            )? != entry.bytes
            {
                return Err(format!("{} entry changed at {:x}", self.name(), entry.rva));
            }
        }
        if self.native.needs_guardian_item_supplement() {
            let e = self.elite_entry();
            if mem.read(base + e.rva, e.bytes.len())? != e.bytes {
                return Err("Native guardian boundary changed".into());
            }
        }
        Ok(())
    }
    pub fn observe(self, mem: &dyn Mem, server: u64) -> Result<Observation> {
        self.native.observe(mem, server)
    }
    pub fn party(self, mem: &dyn Mem, server: u64) -> Result<Option<PartyObservation>> {
        self.native.party(mem, server)
    }
    pub fn scale_direct_heal_delivery(
        self,
        delivery: &mut SupportDelivery,
        factor: f32,
    ) -> Result<bool> {
        self.native.scale_direct_heal_delivery(delivery, factor)
    }
    pub fn hide_fatigued_periodic(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        dt: f32,
        owners: &[OwnedEffect],
    ) -> Result<Vec<PeriodicTick>> {
        self.native.hide_fatigued_periodic(mem, game, dt, owners)
    }
    pub fn run_source(self, mem: &dyn Mem, game: u64) -> Result<Option<(u32, u64)>> {
        self.native.run_source(mem, game)
    }
    pub fn solo_run(self, mem: &dyn Mem, game: u64) -> Result<Option<PartyObservation>> {
        self.native.solo_run(mem, game)
    }
    pub fn effect_source(self, mem: &dyn Mem, game: u64) -> Result<Option<(u32, u64)>> {
        self.native.effect_source(mem, game)
    }
    pub fn hud_actor(self, mem: &dyn Mem, game: u64) -> Result<Option<u32>> {
        self.native.hud_actor(mem, game)
    }
}

/// Owned presentation data; never a native pointer or a second gameplay authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartyIcon {
    pub id: u32,
    pub rank: u32,
    pub name: String,
    pub icon: String,
    pub text: String,
    pub spell_tip: Option<SpellTip>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PartyBuild {
    pub spells: Vec<PartyIcon>,
    pub items: Vec<PartyIcon>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpellTip {
    pub title: String,
    pub body: String,
    pub rank: String,
    pub range: String,
    pub cast: String,
    pub cooldown: String,
}

pub use crate::official_native::FatigueAura;
impl SelectedAdapter {
    pub fn fatigue_aura_entry(self) -> Option<Entry> {
        self.native.fatigue_aura_entry()
    }
    pub fn fatigue_aura_frame(self, m: &dyn Mem, r: [u64; 15]) -> Result<Option<FatigueAura>> {
        self.native.fatigue_aura_frame(m, r)
    }
    pub fn project_fatigue_aura(
        self,
        m: &dyn MutableMemory,
        f: &FatigueAura,
        s: u8,
        t: f32,
    ) -> Result<bool> {
        self.native.project_fatigue_aura(m, f, s, t)
    }
}

impl SelectedAdapter {
    pub fn blessing_entries(self) -> Vec<Entry> {
        self.native.blessing_entries()
    }
    pub fn blessing_endless_entry(self) -> Entry {
        self.native.blessing_endless_entry()
    }
    pub fn blessing_fixture_victory(self, m: &dyn Mem, g: u64) -> Result<bool> {
        self.native.blessing_fixture_victory(m, g)
    }
    pub fn blessing_event(self, m: &dyn Mem, f: u64, d: u64, b: u64) -> Result<Option<(u64, u64)>> {
        self.native.blessing_event(m, f, d, b)
    }
    pub fn blessing_map(self, m: &dyn Mem, g: u64, b: u64) -> Result<(u64, u64)> {
        self.native.blessing_map(m, g, b)
    }
    pub fn blessing_pool_index(self, m: &dyn Mem, f: u64, i: u64) -> Result<u64> {
        self.native.blessing_pool_index(m, f, i)
    }
    pub fn blessing_stats(
        self,
        m: &dyn MutableMemory,
        g: u64,
        b: Option<asx_gameplay::blessings::Blessing>,
    ) -> Result<()> {
        self.native.blessing_stats(m, g, b)
    }
    pub fn refresh_blessings(self, m: &dyn SupportCalls, g: u64, ids: &[(u32, u64)]) -> Result<()> {
        self.native.refresh_blessings(m, g, ids)
    }
    pub fn project_blessing_aura(self, m: &dyn MutableMemory, f: &FatigueAura) -> Result<bool> {
        self.native.project_blessing_aura(m, f)
    }
}

impl SelectedAdapter {
    pub fn blessing_fixture_targets(self, m: &dyn Mem, g: u64) -> Result<Vec<u64>> {
        self.native.blessing_fixture_targets(m, g)
    }
    pub fn blessing_stat_values(self, m: &dyn Mem, g: u64) -> Result<[f32; 5]> {
        self.native.blessing_stat_values(m, g)
    }
    pub fn blessing_spawn_entry(self) -> Entry {
        self.native.blessing_spawn_entry()
    }
    pub fn blessing_fixture_definition(self, m: &dyn Mem, g: u64, b: u64, i: u32) -> Result<u64> {
        self.native.blessing_fixture_definition(m, g, b, i)
    }
}

pub use crate::official_native::{BonusUi, GlyphChoice, GlyphOwner};
impl SelectedAdapter {
    pub fn glyph_entries(self) -> Vec<Entry> {
        self.native.glyph_entries()
    }
    pub fn glyph_owner(self, m: &dyn Mem, b: u64, p: u64, d: u64) -> Result<Option<GlyphOwner>> {
        self.native.glyph_owner(m, b, p, d)
    }
    pub fn glyph_card(self, m: &dyn Mem, b: u64, g: u64, o: u64) -> Result<Option<GlyphChoice>> {
        self.native.glyph_card(m, b, g, o)
    }
    pub fn glyph_selected(
        self,
        m: &dyn Mem,
        b: u64,
        g: u64,
        i: u64,
    ) -> Result<Option<GlyphChoice>> {
        self.native.glyph_selected(m, b, g, i)
    }
    pub fn glyph_rank(self, m: &dyn Mem, b: u64, c: &GlyphChoice) -> Result<Option<u32>> {
        self.native.glyph_rank(m, b, c)
    }
    pub fn refresh_glyph(self, m: &dyn SupportCalls, b: u64, c: &GlyphChoice) -> Result<()> {
        self.native.refresh_glyph(m, b, c)
    }
    pub fn apply_glyph(
        self,
        m: &dyn MutableMemory,
        o: u64,
        g: asx_gameplay::glyphs::Glyph,
    ) -> Result<()> {
        self.native.apply_glyph(m, o, g)
    }
    pub fn reward_card_note(self, m: &dyn MutableMemory, o: u64, n: &str) -> Result<()> {
        self.native.reward_card_note(m, o, n)
    }
    pub fn glyph_text(self, m: &dyn MutableMemory, h: u64, n: &str) -> Result<()> {
        self.native.append_reward_note(m, h, n)
    }
}

impl SelectedAdapter {
    pub fn glyph_fixture_pick(
        self,
        m: &dyn Mem,
        b: u64,
        g: u64,
        s: &[String],
        n: &[u32],
        p: u32,
    ) -> Result<Option<(u64, bool)>> {
        self.native.glyph_fixture_pick(m, b, g, s, n, p)
    }
    pub fn glyph_fixture_ability(
        self,
        m: &dyn Mem,
        b: u64,
        g: u64,
        a: u64,
    ) -> Result<Option<(GlyphOwner, u32, Vec<f32>)>> {
        self.native.glyph_fixture_ability(m, b, g, a)
    }
    pub fn glyph_fixture_modifiers(self, m: &dyn Mem, o: u64) -> Result<Vec<f32>> {
        self.native.glyph_fixture_modifiers(m, o)
    }
}

impl SelectedAdapter {
    pub fn glyph_fixture_card_probe(self, m: &dyn Mem, b: u64, g: u64, o: u64) -> Result<String> {
        self.native.glyph_fixture_card_probe(m, b, g, o)
    }
}

impl SelectedAdapter {
    pub fn bonus_ui_entries(self) -> Vec<Entry> {
        self.native.bonus_ui_entries()
    }
    pub fn bonus_ui_context(self, m: &dyn Mem, c: u64) -> Result<BonusUi> {
        self.native.bonus_ui_context(m, c)
    }
    pub fn bonus_ui_size(self, m: &dyn MutableMemory, c: u64, b: u64) -> Result<()> {
        self.native.bonus_ui_size(m, c, b)
    }
    pub fn scaled_heal_effect(
        self,
        m: &dyn Mem,
        e: u64,
        f: f32,
    ) -> Result<Option<([u8; 12], f32, f32)>> {
        self.native.scaled_heal_effect(m, e, f)
    }
    pub fn glyph_extra_healing(self, g: asx_gameplay::glyphs::Glyph) -> f32 {
        self.native.glyph_extra_healing(g)
    }
    pub unsafe fn glyph_special_presentation(
        self,
        m: &dyn Mem,
        o: usize,
        a: [u64; 5],
        f: f32,
    ) -> Result<u64> {
        self.native.glyph_special_presentation(m, o, a, f)
    }
    pub fn glyph_fixture_resolve(
        self,
        m: &dyn SupportCalls,
        b: u64,
        o: GlyphOwner,
    ) -> Result<Vec<f32>> {
        self.native.glyph_fixture_resolve(m, b, o)
    }
    pub fn glyph_fixture_support_clear(
        self,
        m: &dyn MutableMemory,
        g: u64,
        s: u32,
    ) -> Result<AuraState> {
        self.native.glyph_fixture_support_clear(m, g, s)
    }
    pub fn glyph_fixture_support_value(self, m: &dyn Mem, g: u64, s: u32) -> Result<(f32, f32)> {
        self.native.glyph_fixture_support_value(m, g, s)
    }
    pub fn glyph_fixture_buff(self, m: &dyn Mem, g: u64, s: u32) -> Result<NativeEffect> {
        self.native.glyph_fixture_buff(m, g, s)
    }
    pub fn glyph_fixture_cast_choice(
        self,
        m: &dyn SupportCalls,
        b: u64,
        c: &GlyphChoice,
    ) -> Result<bool> {
        self.native.glyph_fixture_cast_choice(m, b, c)
    }
    pub fn glyph_fixture_mark_target(self, m: &dyn Mem, g: u64, s: u32) -> Result<Option<u64>> {
        self.native.glyph_fixture_mark_target(m, g, s)
    }
    pub fn glyph_fixture_refresh_ability(
        self,
        m: &dyn SupportCalls,
        b: u64,
        g: u64,
        a: u64,
    ) -> Result<()> {
        self.native.glyph_fixture_refresh_ability(m, b, g, a)
    }
    pub fn glyph_fixture_current(self, m: &dyn Mem, b: u64, c: &GlyphChoice) -> Result<Vec<f32>> {
        self.native.glyph_fixture_current(m, b, c)
    }
    pub fn bonus_ui_drawback(self, m: &dyn MutableMemory, b: u64) -> Result<()> {
        self.native.bonus_ui_drawback(m, b)
    }
    pub fn bonus_ui_status(self, m: &dyn MutableMemory, b: u64, k: bool) -> Result<()> {
        self.native.bonus_ui_status(m, b, k)
    }
    pub unsafe fn glyph_ui_draw(
        self,
        b: usize,
        p: u64,
        f: u64,
        u: BonusUi,
        g: Option<asx_gameplay::glyphs::Glyph>,
    ) -> Result<()> {
        self.native.glyph_ui_draw(b, p, f, u, g)
    }
}

impl SelectedAdapter {
    pub unsafe fn glyph_fixture_reroll(
        self,
        m: &dyn MutableMemory,
        b: usize,
        g: u64,
    ) -> Result<()> {
        self.native.glyph_fixture_reroll(m, b, g)
    }
}

impl SelectedAdapter {
    pub fn bonus_ui_no_action(self, f: u64) -> u64 {
        self.native.bonus_ui_no_action(f)
    }
}

impl SelectedAdapter {
    pub fn glyph_sheep_count(self, m: &dyn Mem, g: u64) -> Result<u64> {
        self.native.glyph_sheep_count(m, g)
    }
    pub fn glyph_sheep_flags(self, n: u64, l: u32, f: u64) -> u64 {
        self.native.glyph_sheep_flags(n, l, f)
    }
    pub fn glyph_polymorph_description(self, t: &str, f: f32, l: u32) -> Result<String> {
        self.native.glyph_polymorph_description(t, f, l)
    }
    pub fn glyph_polymorph_text(self, m: &dyn MutableMemory, h: u64, f: f32, l: u32) -> Result<()> {
        self.native.glyph_polymorph_text(m, h, f, l)
    }
}

impl SelectedAdapter {
    pub fn glyph_fixture_presentation(
        self,
        m: &dyn MutableMemory,
        b: u64,
        o: GlyphOwner,
    ) -> Result<Vec<(u32, String)>> {
        self.native.glyph_fixture_presentation(m, b, o)
    }
}

impl SelectedAdapter {
    pub fn glyph_fear_effect(
        self,
        m: &dyn Mem,
        e: u64,
        f: f32,
    ) -> Result<Option<([u8; 12], f32, f32)>> {
        self.native.glyph_fear_effect(m, e, f)
    }
}

impl SelectedAdapter {
    pub fn glyph_fixture_leech_fraction(self, m: &dyn Mem, e: u64) -> Result<Option<f32>> {
        self.native.glyph_fixture_leech_fraction(m, e)
    }
}

impl SelectedAdapter {
    pub fn glyph_faerie_text(self, m: &dyn MutableMemory, h: u64, f: f32) -> Result<()> {
        self.native.glyph_faerie_text(m, h, f)
    }
    pub fn glyph_faerie_description(self, t: &str, f: f32) -> Result<String> {
        self.native.glyph_faerie_description(t, f)
    }
}

impl SelectedAdapter {
    pub fn rejuvenation_source_slots(
        self,
        m: &dyn Mem,
        g: u64,
        c: u64,
    ) -> Result<Option<Vec<u32>>> {
        self.native.rejuvenation_source_slots(m, g, c)
    }
}

impl SelectedAdapter {
    pub fn needs_guardian_item_supplement(self) -> bool {
        self.native.needs_guardian_item_supplement()
    }
}

impl SelectedAdapter {
    pub fn item_choice_entry(self) -> Option<Entry> {
        self.native.item_choice_entry()
    }
    pub fn present_pending_item(
        self,
        io: &dyn SupportCalls,
        g: u64,
        s: u32,
        id: u64,
    ) -> Result<bool> {
        self.native.present_pending_item(io, g, s, id)
    }
}

impl SelectedAdapter {
    pub fn armory_gold_bonus(self, m: &dyn Mem, g: u64, s: u32, id: u64) -> Result<Option<f32>> {
        self.native.armory_gold_bonus(m, g, s, id)
    }
}

impl SelectedAdapter {
    pub fn fixture_armory_gold(
        self,
        m: &dyn SupportCalls,
        g: u64,
        s: u32,
        id: u64,
        v: f32,
    ) -> Result<()> {
        self.native.fixture_armory_gold(m, g, s, id, v)
    }
}

pub use crate::official_native::{RingChoice, RingStatLoan};
impl SelectedAdapter {
    pub fn supports_enchantments(self) -> bool {
        self.native.supports_enchantments()
    }
    pub fn ring_card(self, m: &dyn Mem, b: u64, g: u64, o: u64) -> Result<Option<RingChoice>> {
        self.native.ring_card(m, b, g, o)
    }
    pub fn reward_choice_source(self,m:&dyn Mem,game:u64,index:u64)->Result<Option<(u32,u64)>>{self.native.reward_choice_source(m,game,index)}
    pub fn ring_selected(self, m: &dyn Mem, b: u64, g: u64, i: u64) -> Result<Option<RingChoice>> {
        self.native.ring_selected(m, b, g, i)
    }
    pub fn ring_owned(self, m: &dyn Mem, g: u64, r: u8, b: u64) -> Result<Option<(u32, u32)>> {
        self.native.ring_owned(m, g, r, b)
    }
    pub fn ring_stats(
        self,
        m: &dyn MutableMemory,
        g: u64,
        e: &[asx_gameplay::enchantments::RankedEnchantment],
    ) -> Result<Option<RingStatLoan>> {
        self.native.ring_stats(m, g, e)
    }
    pub fn restore_ring_stats(self, m: &dyn MutableMemory, l: RingStatLoan) -> Result<()> {
        self.native.restore_ring_stats(m, l)
    }
}

impl SelectedAdapter {
    pub unsafe fn enchantment_ui_draw(
        self,
        b: usize,
        p: u64,
        f: u64,
        u: BonusUi,
        e: asx_gameplay::enchantments::RankedEnchantment,
    ) -> Result<()> {
        self.native.enchantment_ui_draw(b, p, f, u, e)
    }
}

impl SelectedAdapter {
    pub fn ring_stat_values(self, m: &dyn Mem, g: u64) -> Result<[f32; 8]> {
        self.native.ring_stat_values(m, g)
    }
    pub fn ring_input_header(self, m: &dyn Mem, g: u64) -> Result<Vec<u8>> {
        self.native.ring_input_header(m, g)
    }
    pub fn ring_fixture_pick(self, m: &dyn Mem, b: u64, g: u64) -> Result<Option<u64>> {
        self.native.ring_fixture_pick(m, b, g)
    }
}

impl SelectedAdapter {
    pub fn ring_actor_owned(
        self,
        m: &dyn Mem,
        g: u64,
        s: u32,
        id: u64,
        r: u8,
        b: u64,
    ) -> Result<Option<(u32, u32)>> {
        self.native.ring_actor_owned(m, g, s, id, r, b)
    }
}

impl SelectedAdapter {
    pub fn reward_choice_kind(self, m: &dyn Mem, g: u64) -> Result<Option<u8>> {
        self.native.reward_choice_kind(m, g)
    }
    pub unsafe fn fixture_reward_action(
        self,
        m: &dyn MutableMemory,
        b: usize,
        g: u64,
        a: u8,
    ) -> Result<()> {
        self.native.fixture_reward_action(m, b, g, a)
    }
}

pub use crate::official_native::{AggroEnemy, AggroLoan};
impl SelectedAdapter {
    pub fn supports_immunity_aggro(self) -> bool {
        self.native.supports_immunity_aggro()
    }
    pub fn combat_immune(self, m: &dyn Mem, g: u64, s: u32, id: u64) -> Result<bool> {
        self.native.combat_immune(m, g, s, id)
    }
    pub fn aggro_snapshot(
        self,
        m: &dyn Mem,
        g: u64,
    ) -> Result<(
        Vec<asx_gameplay::immunity_aggro::Candidate>,
        Vec<AggroEnemy>,
    )> {
        self.native.aggro_snapshot(m, g)
    }
    pub fn aggro_prepare(
        self,
        m: &dyn MutableMemory,
        g: u64,
        b: &[u32],
        c: &[(u64, Option<u32>)],
    ) -> Result<AggroLoan> {
        self.native.aggro_prepare(m, g, b, c)
    }
    pub fn aggro_restore(self, m: &dyn MutableMemory, l: AggroLoan) -> Result<()> {
        self.native.aggro_restore(m, l)
    }
}

impl SelectedAdapter {
    pub fn fixture_immunity(
        self,
        m: &dyn SupportCalls,
        g: u64,
        s: u32,
        spell: u32,
        d: f32,
    ) -> Result<()> {
        self.native.fixture_immunity(m, g, s, spell, d)
    }
}

pub use crate::official_native::AmuletChoice;
impl SelectedAdapter {
    pub fn amulet_card(self, m: &dyn Mem, b: u64, g: u64, o: u64) -> Result<Option<AmuletChoice>> {
        self.native.amulet_card(m, b, g, o)
    }
    pub fn amulet_selected(
        self,
        m: &dyn Mem,
        b: u64,
        g: u64,
        i: u64,
    ) -> Result<Option<AmuletChoice>> {
        self.native.amulet_selected(m, b, g, i)
    }
    pub fn amulet_owned(self, m: &dyn Mem, g: u64) -> Result<Option<u64>> {
        self.native.amulet_owned(m, g)
    }
    pub fn amulet_rank(self, m: &dyn Mem, g: u64) -> Result<u32> {
        self.native.amulet_rank(m, g)
    }
    pub fn gem_stats(
        self,
        m: &dyn MutableMemory,
        g: u64,
        e: asx_gameplay::gems::Gem,
    ) -> Result<Option<RingStatLoan>> {
        self.native.gem_stats(m, g, e)
    }
    pub fn amulet_fixture_pick(self, m: &dyn Mem, b: u64, g: u64) -> Result<Option<u64>> {
        self.native.amulet_fixture_pick(m, b, g)
    }
    pub unsafe fn gem_ui_draw(
        self,
        b: usize,
        p: u64,
        f: u64,
        u: BonusUi,
        e: asx_gameplay::gems::Gem,
    ) -> Result<()> {
        self.native.gem_ui_draw(b, p, f, u, e)
    }
}

impl SelectedAdapter {
    pub fn supports_bonus_tooltips(self) -> bool {
        self.native.supports_bonus_tooltips()
    }
    pub fn bonus_tooltip_entries(self) -> Vec<Entry> {
        self.native.bonus_tooltip_entries()
    }
    pub unsafe fn kept_tooltip_attach(
        self,
        m: &dyn MutableMemory,
        b: usize,
        f: u64,
        font: u64,
        u: BonusUi,
        k: asx_gameplay::kept_bonus::KeptBonus,
    ) -> Result<u64> {
        self.native.kept_tooltip_attach(m, b, f, font, u, k)
    }
    pub fn kept_tooltip_fixture_focus(self) -> Option<Entry> {
        self.native.kept_tooltip_fixture_focus()
    }
    pub fn kept_tooltip_body(
        self,
        m: &dyn Mem,
        c: u64,
    ) -> Result<Option<(u64, asx_gameplay::kept_bonus::KeptBonus)>> {
        self.native.kept_tooltip_body(m, c)
    }
    pub fn kept_tooltip_descriptor(
        self,
        k: asx_gameplay::kept_bonus::KeptBonus,
    ) -> Option<[u64; 11]> {
        self.native.kept_tooltip_descriptor(k)
    }
    pub unsafe fn kept_tooltip_target(self, m: &dyn MutableMemory, b: usize, r: u64) -> Result<()> {
        self.native.kept_tooltip_target(m, b, r)
    }
    pub fn kept_tooltip_title(self, m: &dyn MutableMemory, b: u64) -> Result<()> {
        self.native.kept_tooltip_title(m, b)
    }
    pub fn bonus_compact_size(self, m: &dyn MutableMemory, c: u64, b: u64) -> Result<()> {
        self.native.bonus_compact_size(m, c, b)
    }
    pub fn bonus_compact_style(self, m: &dyn MutableMemory, b: u64) -> Result<()> {
        self.native.bonus_compact_style(m, b)
    }
    pub unsafe fn kept_bonus_ui_draw(
        self,
        b: usize,
        p: u64,
        f: u64,
        u: BonusUi,
        g: asx_gameplay::kept_bonus::KeptBonus,
    ) -> Result<()> {
        self.native.kept_bonus_ui_draw(b, p, f, u, g)
    }
}

impl SelectedAdapter {
    pub fn glyph_retention_pick(self, m: &dyn Mem, b: u64, g: u64) -> Result<Option<u64>> {
        self.native.glyph_retention_pick(m, b, g)
    }
}

impl SelectedAdapter {
    pub fn glyphs_enabled(self) -> bool {
        self.native.glyphs_enabled()
    }
    pub fn native_marks_enabled(self) -> bool {
        self.native.native_marks_enabled()
    }
    pub fn duplicate_loot_enabled(self) -> bool {
        self.native.duplicate_loot_enabled()
    }
}

impl SelectedAdapter {
    pub unsafe fn bonus_header_draw(
        self,
        b: usize,
        p: u64,
        f: u64,
        u: BonusUi,
        title: &str,
        icon: &str,
    ) -> Result<()> {
        self.native.bonus_header_draw(b, p, f, u, title, icon)
    }
}

impl SelectedAdapter {
    pub fn blessing_event_context(
        self,
        m: &dyn Mem,
        registers: [u64; 3],
        base: u64,
    ) -> Result<Option<(u64, u64)>> {
        self.native.blessing_event_context(m, registers, base)
    }
}

impl SelectedAdapter {
    pub fn reward_entries(self) -> [Entry; 2] {
        self.native.reward_entries()
    }
    pub fn native_hit_effect_entry(self) -> Entry {
        self.native.native_hit_effect_entry()
    }
}

impl SelectedAdapter {
    pub unsafe fn fixture_spawn_encounter(self, b: usize, g: u64, d: u64) {
        self.native.fixture_spawn_encounter(b, g, d)
    }
}

pub use crate::official_native::{BonusHeaderColumn, BonusSeparator};
impl SelectedAdapter {
    pub fn bonus_separator_entries(self) -> [Entry; 2] {
        self.native.bonus_separator_entries()
    }
    pub fn bonus_separator_capture(
        self,
        m: &dyn Mem,
        caller: u64,
        node: u64,
        color: u64,
    ) -> Result<Option<BonusSeparator>> {
        self.native.bonus_separator_capture(m, caller, node, color)
    }
    pub unsafe fn bonus_separator_draw(
        self,
        m: &dyn Mem,
        base: usize,
        parent: u64,
        style: BonusSeparator,
    ) -> Result<()> {
        self.native.bonus_separator_draw(m, base, parent, style)
    }
}

impl SelectedAdapter {
    pub unsafe fn pause_remote_motion(self,m:&dyn Mem,base:u64,game:u64)->Result<usize>{unsafe{self.native.pause_remote_motion(m,base,game)}}
    pub fn raid_markers(self, m: &dyn Mem, server: u64) -> Result<[Option<u64>; 8]> {
        self.native.raid_markers(m, server)
    }
    pub unsafe fn set_raid_marker(
        self,
        m: &dyn Mem,
        base: u64,
        server: u64,
        slot: u32,
        guid: u64,
        icon: u8,
    ) -> Result<bool> {
        self.native
            .set_raid_marker(m, base, server, slot, guid, icon)
    }
}

impl SelectedAdapter {
    pub fn combat_text_entries(self) -> [Entry; 8] {
        self.native.combat_text_entries()
    }
    pub fn native_heal_texts(
        self,
        m: &dyn Mem,
        bp: u64,
        periodic: bool,
        players: &[u64],
    ) -> crate::Result<Vec<crate::official_native::HealText>> {
        self.native.native_heal_texts(m, bp, periodic, players)
    }
    pub unsafe fn draw_native_heal_text(
        self,
        m: &dyn MutableMemory,
        base: usize,
        bp: u64,
        periodic: bool,
        heal: crate::official_native::HealText,
    ) -> crate::Result<bool> {
        self.native
            .draw_native_heal_text(m, base, bp, periodic, heal)
    }
}

impl SelectedAdapter {
    pub fn native_ui_participants(self, m: &dyn Mem, script: u64) -> Result<Vec<u64>> {
        self.native.native_ui_participants(m, script)
    }
}

impl SelectedAdapter {
    pub fn bonus_column_entries(self) -> [Entry; 2] {
        self.native.bonus_column_entries()
    }
    pub fn bonus_column_capture(
        self,
        m: &dyn Mem,
        caller: u64,
        bundle: u64,
        meta: u64,
    ) -> Result<Option<BonusHeaderColumn>> {
        self.native.bonus_column_capture(m, caller, bundle, meta)
    }
    pub unsafe fn bonus_column_draw(
        self,
        m: &dyn MutableMemory,
        base: usize,
        commands: u64,
        title: u64,
        style: BonusHeaderColumn,
    ) -> Result<[u64; 6]> {
        self.native
            .bonus_column_draw(m, base, commands, title, style)
    }
}
impl SelectedAdapter {
    pub fn bonus_column_spawner(self, m: &dyn Mem, parent: u64, column: u64) -> Result<[u64; 6]> {
        self.native.bonus_column_spawner(m, parent, column)
    }
}

pub use crate::official_native::{GoblinDefinition, GoblinHandle, GoblinSnapshot};
impl SelectedAdapter {
    pub fn goblin_arena(
        self,
        m: &dyn Mem,
        g: u64,
        b: u64,
    ) -> Result<asx_gameplay::treasure_goblins::Arena> {
        self.native.goblin_arena(m, g, b)
    }
    pub fn goblin_final_motion_entry(self) -> Entry {
        self.native.goblin_final_motion_entry()
    }
    pub fn goblin_final_motion(
        self,
        m: &dyn Mem,
        f: u64,
        s: u64,
        g: u64,
        id: u64,
    ) -> Result<Option<([f32; 3], [f32; 3])>> {
        self.native.goblin_final_motion(m, f, s, g, id)
    }
    pub fn goblin_hold_motion(self, m: &dyn MutableMemory, s: u64, o: [f32; 3]) -> Result<()> {
        self.native.goblin_hold_motion(m, s, o)
    }
    pub fn goblin_run(self, m: &dyn Mem, game: u64) -> Result<Option<PartyObservation>> {
        self.native.goblin_run(m, game)
    }
    pub fn goblin_recycle_entry(self) -> Entry {
        self.native.goblin_recycle_entry()
    }
    pub fn goblin_filter_recycle(
        self,
        m: &dyn MutableMemory,
        i: u64,
        o: u64,
        g: u64,
        id: u64,
    ) -> Result<bool> {
        self.native.goblin_filter_recycle(m, i, o, g, id)
    }
    pub fn goblin_outfit_entries(self) -> [Entry; 2] {
        self.native.goblin_outfit_entries()
    }
    pub fn goblin_outfit_metadata(self, m: &dyn MutableMemory, f: u64) -> Result<Option<u64>> {
        self.native.goblin_outfit_metadata(m, f)
    }
    pub fn goblin_outfit_key(self, m: &dyn MutableMemory, f: u64, e: u64) -> Result<()> {
        self.native.goblin_outfit_key(m, f, e)
    }
    pub fn goblin_entries(self) -> Vec<Entry> {
        self.native.goblin_entries()
    }
    pub fn goblin_skin_entry(self) -> Entry {
        self.native.goblin_skin_entry()
    }
    pub fn goblin_skin_probe(
        self,
        m: &dyn Mem,
        n: u64,
        l: usize,
        f: u64,
        c: u64,
    ) -> Result<Option<String>> {
        self.native.goblin_skin_probe(m, n, l, f, c)
    }
    pub fn goblin_skin_path(
        self,
        m: &dyn MutableMemory,
        output: u64,
        name: u64,
        length: usize,
    ) -> Result<bool> {
        self.native.goblin_skin_path(m, output, name, length)
    }
    pub fn goblin_definition_entry(self) -> Entry {
        self.native.goblin_definition_entry()
    }
    pub fn goblin_template(self, m: &dyn Mem, g: u64, b: u64) -> Result<(usize, GoblinDefinition)> {
        self.native.goblin_template(m, g, b)
    }
    pub unsafe fn spawn_goblin(self, m: &dyn Mem, b: u64, g: u64) -> Result<Option<GoblinHandle>> {
        self.native.spawn_goblin(m, b, g)
    }
    pub fn goblin_elite(self, m: &dyn MutableMemory, g: u64, h: &mut GoblinHandle) -> Result<()> {
        self.native.goblin_elite(m, g, h)
    }
    pub fn goblin_snapshot(
        self,
        m: &dyn Mem,
        g: u64,
        h: &mut GoblinHandle,
    ) -> Result<Option<GoblinSnapshot>> {
        self.native.goblin_snapshot(m, g, h)
    }
    pub unsafe fn goblin_clip(
        self,
        m: &dyn Mem,
        b: u64,
        g: u64,
        o: [f32; 3],
        d: [f32; 3],
    ) -> Result<[f32; 3]> {
        self.native.goblin_clip(m, b, g, o, d)
    }
    pub unsafe fn goblin_name(self, m: &dyn MutableMemory, b: u64, g: u64) -> Result<()> {
        self.native.goblin_name(m, b, g)
    }
    pub unsafe fn goblin_notice(
        self,
        m: &dyn MutableMemory,
        b: u64,
        g: u64,
        t: &str,
    ) -> Result<()> {
        self.native.goblin_notice(m, b, g, t)
    }
    pub unsafe fn despawn_goblin(
        self,
        m: &dyn MutableMemory,
        b: u64,
        g: u64,
        h: &mut GoblinHandle,
    ) -> Result<()> {
        self.native.despawn_goblin(m, b, g, h)
    }
}

impl SelectedAdapter {
    pub fn goblin_packet_probe_entry(self) -> Entry {
        self.native.goblin_packet_probe_entry()
    }
    pub fn goblin_packet_probe(
        self,
        m: &dyn Mem,
        p: u64,
    ) -> Result<Option<(u16, u64, Option<[f32; 3]>)>> {
        self.native.goblin_packet_probe(m, p)
    }
}

impl SelectedAdapter {
    pub fn goblin_reward_entries(self) -> [Entry; 3] {
        self.native.goblin_reward_entries()
    }
    pub fn goblin_amulet_pool(self, m: &dyn MutableMemory, f: u64, g: u64) -> Result<()> {
        self.native.goblin_amulet_pool(m, f, g)
    }
    pub fn goblin_amulet_key(self, m: &dyn Mem, g: u64, o: u64) -> Result<Option<String>> {
        self.native.goblin_amulet_key(m, g, o)
    }
    pub fn goblin_exclusion_keys(self, m: &dyn Mem, h: u64) -> Result<Vec<[u64; 3]>> {
        self.native.goblin_exclusion_keys(m, h)
    }
    pub fn goblin_reward_actor(self, m: &dyn Mem, g: u64) -> Result<Option<(u32, u64)>> {
        self.native.goblin_reward_actor(m, g)
    }
    pub fn goblin_reward_ready(self, m: &dyn Mem, g: u64, s: u32, id: u64) -> Result<bool> {
        self.native.goblin_reward_ready(m, g, s, id)
    }
    pub fn goblin_credit_marks(
        self,
        io: &dyn SupportCalls,
        g: u64,
        s: u32,
        id: u64,
        a: f32,
    ) -> Result<()> {
        self.native.goblin_credit_marks(io, g, s, id, a)
    }
    pub fn goblin_marks_amount(self, m: &dyn Mem, b: u64, g: u64) -> Result<f32> {
        self.native.goblin_marks_amount(m, b, g)
    }
    pub unsafe fn goblin_open_amulets(
        self,
        io: &dyn SupportCalls,
        b: u64,
        g: u64,
        s: u32,
        id: u64,
        a: f32,
    ) -> Result<(Vec<String>, f32)> {
        self.native.goblin_open_amulets(io, b, g, s, id, a)
    }
}

pub use crate::official_native::{EventObject, RitualScene};
impl SelectedAdapter {
    pub fn world_event_entries(self) -> Vec<Entry> {
        self.native.world_event_entries()
    }
    pub unsafe fn event_object(
        self,
        m: &dyn MutableMemory,
        base: u64,
        game: u64,
        visual: u8,
        position: [f32; 3],
    ) -> Result<EventObject> {
        self.native.event_object(m, base, game, visual, position)
    }
    pub unsafe fn remove_event_object(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
        object: EventObject,
    ) -> Result<()> {
        self.native.remove_event_object(m, base, game, object)
    }
    pub unsafe fn event_notice(
        self,
        m: &dyn MutableMemory,
        base: u64,
        game: u64,
        sender: &str,
        text: &str,
    ) -> Result<()> {
        self.native.event_notice(m, base, game, sender, text)
    }
}

pub use crate::official_native::StatInputLoan;
impl SelectedAdapter {
    pub fn apply_consumable_speed(self,m:&dyn MutableMemory,game:u64,value:f32)->Result<()>{self.native.apply_consumable_speed(m,game,value)}
    pub fn consumable_stats(
        self,
        m: &dyn MutableMemory,
        game: u64,
        buffs: &[asx_gameplay::supplies::Consumable],
    ) -> Result<Option<StatInputLoan>> {
        self.native.consumable_stats(m, game, buffs)
    }
    pub fn restore_stat_input(self, m: &dyn MutableMemory, loan: StatInputLoan) -> Result<()> {
        self.native.restore_stat_input(m, loan)
    }
}

impl SelectedAdapter {
    pub fn project_consumable_auras(
        self,
        m: &dyn MutableMemory,
        f: &FatigueAura,
        buffs: &[(asx_gameplay::supplies::Family, f32)],
    ) -> Result<Vec<(asx_gameplay::supplies::Family, u32)>> {
        self.native.project_consumable_auras(m, f, buffs)
    }
    pub fn event_test_stat_input(self, m: &dyn Mem, game: u64) -> Result<Vec<u8>> {
        self.native.event_test_stat_input(m, game)
    }
}

impl SelectedAdapter {
    pub unsafe fn event_position(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
        near: f32,
        far: f32,
    ) -> Result<Option<[f32; 3]>> {
        self.native.event_position(m, base, game, near, far)
    }
}

impl SelectedAdapter {
    pub fn event_endless(self, m: &dyn Mem, game: u64) -> Result<bool> {
        self.native.event_endless(m, game)
    }
}

impl SelectedAdapter {
    pub fn ritual_scene_entries(self) -> Vec<Entry> {
        self.native.ritual_scene_entries()
    }
    pub unsafe fn event_sayge(
        self,
        m: &dyn MutableMemory,
        base: u64,
        game: u64,
        position: [f32; 3],
    ) -> Result<EventObject> {
        self.native.event_sayge(m, base, game, position)
    }
    pub fn event_enemy_count(self, m: &dyn Mem, game: u64) -> Result<u64> {
        self.native.event_enemy_count(m, game)
    }
}

impl SelectedAdapter {
    pub unsafe fn event_ritual_scene(
        self,
        m: &dyn MutableMemory,
        base: u64,
        game: u64,
        point: [f32; 3],
    ) -> Result<RitualScene> {
        self.native.event_ritual_scene(m, base, game, point)
    }
    pub unsafe fn remove_ritual_scene(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
        scene: &RitualScene,
    ) -> Result<()> {
        self.native.remove_ritual_scene(m, base, game, scene)
    }
}

pub use crate::official_native::EventMinimap;
impl SelectedAdapter {
    pub fn event_placement_entries(self) -> Vec<Entry> {
        self.native.event_placement_entries()
    }
    pub fn event_area(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
    ) -> Result<asx_gameplay::world_events::Area> {
        self.native.event_area(m, base, game)
    }
    pub unsafe fn event_ground(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
        p: [f32; 2],
        footprint: f32,
        max_slope: f32,
    ) -> Result<Option<[f32; 3]>> {
        self.native
            .event_ground(m, base, game, p, footprint, max_slope)
    }
    pub fn event_minimap(
        self,
        m: &dyn Mem,
        script: u64,
        owner: u64,
    ) -> Result<Option<EventMinimap>> {
        self.native.event_minimap(m, script, owner)
    }
    pub fn clamp_event_health(
        self,
        m: &dyn MutableMemory,
        game: u64,
        owner: u64,
        before: f32,
    ) -> Result<()> {
        self.native.clamp_event_health(m, game, owner, before)
    }
}

impl SelectedAdapter {
    pub fn ui_context_entry(self) -> Entry {
        self.native.ui_context_entry()
    }
    pub fn ui_context_result(self, root: u64) -> Option<u64> {
        self.native.ui_context_result(root)
    }
}

impl SelectedAdapter {
    pub fn event_aura_readback(self, m: &dyn Mem, f: &FatigueAura) -> Result<Vec<(u32, u32)>> {
        self.native.event_aura_readback(m, f)
    }
}

impl SelectedAdapter {
    pub unsafe fn event_ground_sample(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
        p: [f32; 2],
        footprint: f32,
    ) -> Result<Option<asx_gameplay::world_events::GroundSite>> {
        self.native.event_ground_sample(m, base, game, p, footprint)
    }
}

impl SelectedAdapter {
    pub unsafe fn hide_event_object(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
        slot: u32,
        guid: u64,
        object: EventObject,
    ) -> Result<()> {
        self.native
            .hide_event_object(m, base, game, slot, guid, object)
    }
}

impl SelectedAdapter {
    pub fn event_party(self, m: &dyn Mem, game: u64) -> Result<Option<PartyObservation>> {
        self.native.event_party(m, game)
    }
}

impl SelectedAdapter {
    pub fn fixture_event_pose(
        self,
        m: &dyn MutableMemory,
        game: u64,
        slot: u32,
        guid: u64,
        point: [f32; 3],
    ) -> Result<()> {
        self.native.fixture_event_pose(m, game, slot, guid, point)
    }
}

impl SelectedAdapter {
    pub fn fixture_event_damage(
        self,
        io: &dyn SupportCalls,
        game: u64,
        slot: u32,
        guid: u64,
        target: u64,
    ) -> Result<()> {
        self.native
            .fixture_event_damage(io, game, slot, guid, target)
    }
}

impl SelectedAdapter{
 pub fn diagnostic_writer_entry(self)->Entry{self.native.diagnostic_writer_entry()}
}

#[cfg(feature="test-hooks")]
impl SelectedAdapter {
 pub unsafe fn fixture_world_ui_npcs(self,m:&dyn MutableMemory,base:u64,game:u64)->Result<Vec<(String,u64)>>{self.native.fixture_world_ui_npcs(m,base,game)}
}

#[cfg(feature="test-hooks")]
impl SelectedAdapter {
 pub fn fixture_world_ui_wound(self,m:&dyn MutableMemory,game:u64,slot:u32)->Result<()>{self.native.fixture_world_ui_wound(m,game,slot)}
}

#[cfg(feature="test-hooks")]
impl SelectedAdapter {
 pub fn fixture_heal_packet(self,m:&dyn Mem,bp:u64,periodic:bool)->Result<Vec<u8>>{self.native.fixture_heal_packet(m,bp,periodic)}
}

impl SelectedAdapter {
 pub fn fractional_heal_clock(self,m:&dyn Mem,game:u64)->Result<f32>{self.native.fractional_heal_clock(m,game)}
 pub fn fractional_heal_context(self,m:&dyn Mem,game:u64)->Result<(f32,SupportCaster)>{self.native.fractional_heal_context(m,game)}
 pub unsafe fn emit_fractional_heal_text(self,m:&dyn MutableMemory,base:usize,game:u64,heal:official_native::HealText,spell:u32)->Result<()>{self.native.emit_fractional_heal_text(m,base,game,heal,spell)}
}

#[cfg(feature="test-hooks")]
impl SelectedAdapter {
 pub fn fixture_world_ui_hold(self,m:&dyn MutableMemory,game:u64,ids:&[(String,u64)])->Result<()>{self.native.fixture_world_ui_hold(m,game,ids)}
}
