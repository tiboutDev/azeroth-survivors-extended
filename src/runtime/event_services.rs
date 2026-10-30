//! Native event operations; no runtime, memory accessor or adapter escapes.
use super::{io, Runtime};
use asx_game::official::{
    Entry, EventObject, GoblinDefinition, GoblinHandle, GoblinSnapshot, PartyObservation,
    RitualScene, StatInputLoan,
};
use asx_game::Result;

pub(super) trait Services {
    fn events_goblin_notice(&self, game: u64, text: &str) -> Result<()>;
    fn events_finish_goblin_rewards(&self, game: u64, actors: &[(u32, u64)]) -> Result<()>;
    #[cfg(feature = "test-hooks")]
    fn events_fixture_goblin(
        &self,
        game: u64,
        now: f32,
        handle: GoblinHandle,
        party: &PartyObservation,
    );

    fn events_info(&self, message: &str);
    fn events_warn(&self, message: &str);
    fn events_solo_fortune(
        &self,
        game: u64,
        owner: u64,
        blessing: asx_gameplay::blessings::Blessing,
    ) -> Result<()>;
    fn events_group_fortunes(
        &self,
        game: u64,
        rewards: &[((u32, u64), asx_gameplay::blessings::Blessing)],
    ) -> Result<()>;
    fn events_group_sale(&self, game: u64, id: (u32, u64), amount: u32) -> Result<u32>;

    fn events_verify_supplies(&self) -> Result<()>;
    fn events_verify_ritual(&self) -> Result<()>;
    fn events_verify_fixture_boss(&self) -> Result<()>;
    fn events_refresh_stats(&self, game: u64) -> Result<()>;
    fn events_damage_fixture_target(&self, game: u64, target: u64, damage: f32) -> Result<()>;
    fn events_credit_supply(&self, game: u64, owner: u64, amount: u32) -> Result<u32>;

    fn events_blessing_fixture_definition(&self, g: u64, i: u32) -> Result<u64>;
    fn events_blessing_fixture_targets(&self, g: u64) -> Result<Vec<u64>>;
    fn events_clamp_event_health(&self, game: u64, owner: u64, before: f32) -> Result<()>;
    fn events_consumable_stats(
        &self,
        game: u64,
        buffs: &[asx_gameplay::supplies::Consumable],
    ) -> Result<Option<StatInputLoan>>;
    unsafe fn events_despawn_goblin(&self, g: u64, h: &mut GoblinHandle) -> Result<()>;
    fn events_area(&self, game: u64) -> Result<asx_gameplay::world_events::Area>;
    fn events_endless(&self, game: u64) -> Result<bool>;
    fn events_enemy_count(&self, game: u64) -> Result<u64>;
    unsafe fn events_ground(
        &self,
        game: u64,
        p: [f32; 2],
        footprint: f32,
        max_slope: f32,
    ) -> Result<Option<[f32; 3]>>;
    unsafe fn events_ground_sample(
        &self,
        game: u64,
        p: [f32; 2],
        footprint: f32,
    ) -> Result<Option<asx_gameplay::world_events::GroundSite>>;
    unsafe fn events_notice(&self, game: u64, sender: &str, text: &str) -> Result<()>;
    unsafe fn events_object(
        &self,
        game: u64,
        visual: u8,
        position: [f32; 3],
    ) -> Result<EventObject>;
    unsafe fn events_ritual_scene(&self, game: u64, point: [f32; 3]) -> Result<RitualScene>;
    fn events_test_stat_input(&self, game: u64) -> Result<Vec<u8>>;
    fn events_fixture_event_damage(
        &self,
        game: u64,
        slot: u32,
        guid: u64,
        target: u64,
    ) -> Result<()>;
    fn events_fixture_event_pose(
        &self,
        game: u64,
        slot: u32,
        guid: u64,
        point: [f32; 3],
    ) -> Result<()>;
    unsafe fn events_fixture_spawn_encounter(&self, g: u64, d: u64);
    fn events_goblin_arena(&self, g: u64) -> Result<asx_gameplay::treasure_goblins::Arena>;
    unsafe fn events_goblin_clip(&self, g: u64, o: [f32; 3], d: [f32; 3]) -> Result<[f32; 3]>;
    fn events_goblin_elite(&self, g: u64, h: &mut GoblinHandle) -> Result<()>;
    fn events_goblin_marks_amount(&self, g: u64) -> Result<f32>;
    unsafe fn events_goblin_name(&self, g: u64) -> Result<()>;
    fn events_goblin_run(&self, game: u64) -> Result<Option<PartyObservation>>;
    fn events_goblin_snapshot(
        &self,
        g: u64,
        h: &mut GoblinHandle,
    ) -> Result<Option<GoblinSnapshot>>;
    fn events_goblin_template(&self, g: u64) -> Result<(usize, GoblinDefinition)>;
    unsafe fn events_hide_event_object(
        &self,
        game: u64,
        slot: u32,
        guid: u64,
        object: EventObject,
    ) -> Result<()>;
    fn events_refresh_blessings(&self, g: u64, ids: &[(u32, u64)]) -> Result<()>;
    unsafe fn events_remove_event_object(&self, game: u64, object: EventObject) -> Result<()>;
    unsafe fn events_remove_ritual_scene(&self, game: u64, scene: &RitualScene) -> Result<()>;
    fn events_restore_stat_input(&self, loan: StatInputLoan) -> Result<()>;
    fn events_ring_stat_values(&self, g: u64) -> Result<[f32; 8]>;
    fn events_run_source(&self, game: u64) -> Result<Option<(u32, u64)>>;
    fn events_solo_run(&self, game: u64) -> Result<Option<PartyObservation>>;
    unsafe fn events_spawn_goblin(&self, g: u64) -> Result<Option<GoblinHandle>>;
    fn events_world_party(&self, game: u64) -> Result<Option<PartyObservation>>;
}
impl Services for Runtime {
    fn events_goblin_notice(&self, game: u64, text: &str) -> Result<()> {
        unsafe {
            self.adapter.goblin_notice(
                &io::Access {
                    local: &self.local,
                    adapter: self.adapter,
                },
                self.local.base as u64,
                game,
                text,
            )
        }
    }
    fn events_finish_goblin_rewards(&self, game: u64, actors: &[(u32, u64)]) -> Result<()> {
        super::treasure_goblins::deliver_rewards(self, game, actors)
    }
    #[cfg(feature = "test-hooks")]
    fn events_fixture_goblin(
        &self,
        game: u64,
        now: f32,
        handle: GoblinHandle,
        party: &PartyObservation,
    ) {
        super::treasure_goblins::fixture_active(self, game, now, handle, party)
    }

    fn events_info(&self, message: &str) {
        self.log.info(message)
    }
    fn events_warn(&self, message: &str) {
        self.log.warn(message)
    }
    fn events_solo_fortune(
        &self,
        game: u64,
        owner: u64,
        blessing: asx_gameplay::blessings::Blessing,
    ) -> Result<()> {
        super::blessings::ritual_reward(self, game, owner, blessing)
    }
    fn events_group_fortunes(
        &self,
        game: u64,
        rewards: &[((u32, u64), asx_gameplay::blessings::Blessing)],
    ) -> Result<()> {
        super::blessings::ritual_rewards(self, game, rewards)
    }
    fn events_group_sale(&self, game: u64, id: (u32, u64), amount: u32) -> Result<u32> {
        super::wallet::reward_credit(self, game, id, amount, 1, "Supplies sale")?
            .into_iter()
            .next()
            .ok_or_else(|| "Missing supplies sale receipt".into())
    }

    fn events_verify_supplies(&self) -> Result<()> {
        verify(
            self,
            self.adapter
                .world_event_entries()
                .into_iter()
                .chain(self.adapter.event_placement_entries()),
        )
    }
    fn events_verify_ritual(&self) -> Result<()> {
        verify(
            self,
            self.adapter
                .world_event_entries()
                .into_iter()
                .chain(self.adapter.ritual_scene_entries())
                .chain(self.adapter.event_placement_entries()),
        )
    }
    fn events_verify_fixture_boss(&self) -> Result<()> {
        verify(self, [self.adapter.blessing_spawn_entry()])
    }
    fn events_refresh_stats(&self, game: u64) -> Result<()> {
        use asx_game::official::SupportCalls;
        io::Access {
            local: &self.local,
            adapter: self.adapter,
        }
        .refresh_support(game)
    }
    fn events_damage_fixture_target(&self, game: u64, target: u64, damage: f32) -> Result<()> {
        use asx_game::official::SupportCalls;
        io::Access {
            local: &self.local,
            adapter: self.adapter,
        }
        .support_damage(game, target, damage)
    }
    fn events_credit_supply(&self, game: u64, owner: u64, amount: u32) -> Result<u32> {
        let before = self
            .events_solo_run(game)?
            .and_then(|p| p.actors.first().filter(|a| a.guid == owner).map(|a| a.gold))
            .ok_or("Supplies sale source changed")?;
        unsafe {
            self.adapter
                .event_credit_gold(self.local.base as u64, game, amount)
        };
        let after = self
            .events_solo_run(game)?
            .and_then(|p| p.actors.first().filter(|a| a.guid == owner).map(|a| a.gold))
            .ok_or("Supplies sale recipient changed")?;
        after
            .checked_sub(before)
            .ok_or_else(|| "Invalid supplies sale receipt".into())
    }

    fn events_blessing_fixture_definition(&self, g: u64, i: u32) -> Result<u64> {
        self.adapter
            .blessing_fixture_definition(&self.local, g, self.local.base as u64, i)
    }
    fn events_blessing_fixture_targets(&self, g: u64) -> Result<Vec<u64>> {
        self.adapter.blessing_fixture_targets(&self.local, g)
    }
    fn events_clamp_event_health(&self, game: u64, owner: u64, before: f32) -> Result<()> {
        self.adapter.clamp_event_health(
            &io::Access {
                local: &self.local,
                adapter: self.adapter,
            },
            game,
            owner,
            before,
        )
    }
    fn events_consumable_stats(
        &self,
        game: u64,
        buffs: &[asx_gameplay::supplies::Consumable],
    ) -> Result<Option<StatInputLoan>> {
        self.adapter.consumable_stats(
            &io::Access {
                local: &self.local,
                adapter: self.adapter,
            },
            game,
            buffs,
        )
    }
    unsafe fn events_despawn_goblin(&self, g: u64, h: &mut GoblinHandle) -> Result<()> {
        unsafe {
            self.adapter.despawn_goblin(
                &io::Access {
                    local: &self.local,
                    adapter: self.adapter,
                },
                self.local.base as u64,
                g,
                h,
            )
        }
    }
    fn events_area(&self, game: u64) -> Result<asx_gameplay::world_events::Area> {
        self.adapter
            .event_area(&self.local, self.local.base as u64, game)
    }
    fn events_endless(&self, game: u64) -> Result<bool> {
        self.adapter.event_endless(&self.local, game)
    }
    fn events_enemy_count(&self, game: u64) -> Result<u64> {
        self.adapter.event_enemy_count(&self.local, game)
    }
    unsafe fn events_ground(
        &self,
        game: u64,
        p: [f32; 2],
        footprint: f32,
        max_slope: f32,
    ) -> Result<Option<[f32; 3]>> {
        unsafe {
            self.adapter.event_ground(
                &self.local,
                self.local.base as u64,
                game,
                p,
                footprint,
                max_slope,
            )
        }
    }
    unsafe fn events_ground_sample(
        &self,
        game: u64,
        p: [f32; 2],
        footprint: f32,
    ) -> Result<Option<asx_gameplay::world_events::GroundSite>> {
        unsafe {
            self.adapter.event_ground_sample(
                &self.local,
                self.local.base as u64,
                game,
                p,
                footprint,
            )
        }
    }
    unsafe fn events_notice(&self, game: u64, sender: &str, text: &str) -> Result<()> {
        unsafe {
            self.adapter.event_notice(
                &io::Access {
                    local: &self.local,
                    adapter: self.adapter,
                },
                self.local.base as u64,
                game,
                sender,
                text,
            )
        }
    }
    unsafe fn events_object(
        &self,
        game: u64,
        visual: u8,
        position: [f32; 3],
    ) -> Result<EventObject> {
        unsafe {
            self.adapter.event_object(
                &io::Access {
                    local: &self.local,
                    adapter: self.adapter,
                },
                self.local.base as u64,
                game,
                visual,
                position,
            )
        }
    }
    unsafe fn events_ritual_scene(&self, game: u64, point: [f32; 3]) -> Result<RitualScene> {
        unsafe {
            self.adapter.event_ritual_scene(
                &io::Access {
                    local: &self.local,
                    adapter: self.adapter,
                },
                self.local.base as u64,
                game,
                point,
            )
        }
    }
    fn events_test_stat_input(&self, game: u64) -> Result<Vec<u8>> {
        self.adapter.event_test_stat_input(&self.local, game)
    }
    fn events_fixture_event_damage(
        &self,
        game: u64,
        slot: u32,
        guid: u64,
        target: u64,
    ) -> Result<()> {
        self.adapter.fixture_event_damage(
            &io::Access {
                local: &self.local,
                adapter: self.adapter,
            },
            game,
            slot,
            guid,
            target,
        )
    }
    fn events_fixture_event_pose(
        &self,
        game: u64,
        slot: u32,
        guid: u64,
        point: [f32; 3],
    ) -> Result<()> {
        self.adapter.fixture_event_pose(
            &io::Access {
                local: &self.local,
                adapter: self.adapter,
            },
            game,
            slot,
            guid,
            point,
        )
    }
    unsafe fn events_fixture_spawn_encounter(&self, g: u64, d: u64) {
        unsafe { self.adapter.fixture_spawn_encounter(self.local.base, g, d) }
    }
    fn events_goblin_arena(&self, g: u64) -> Result<asx_gameplay::treasure_goblins::Arena> {
        self.adapter
            .goblin_arena(&self.local, g, self.local.base as u64)
    }
    unsafe fn events_goblin_clip(&self, g: u64, o: [f32; 3], d: [f32; 3]) -> Result<[f32; 3]> {
        unsafe {
            self.adapter
                .goblin_clip(&self.local, self.local.base as u64, g, o, d)
        }
    }
    fn events_goblin_elite(&self, g: u64, h: &mut GoblinHandle) -> Result<()> {
        self.adapter.goblin_elite(
            &io::Access {
                local: &self.local,
                adapter: self.adapter,
            },
            g,
            h,
        )
    }
    fn events_goblin_marks_amount(&self, g: u64) -> Result<f32> {
        self.adapter
            .goblin_marks_amount(&self.local, self.local.base as u64, g)
    }
    unsafe fn events_goblin_name(&self, g: u64) -> Result<()> {
        unsafe {
            self.adapter.goblin_name(
                &io::Access {
                    local: &self.local,
                    adapter: self.adapter,
                },
                self.local.base as u64,
                g,
            )
        }
    }
    fn events_goblin_run(&self, game: u64) -> Result<Option<PartyObservation>> {
        self.adapter.goblin_run(&self.local, game)
    }
    fn events_goblin_snapshot(
        &self,
        g: u64,
        h: &mut GoblinHandle,
    ) -> Result<Option<GoblinSnapshot>> {
        self.adapter.goblin_snapshot(&self.local, g, h)
    }
    fn events_goblin_template(&self, g: u64) -> Result<(usize, GoblinDefinition)> {
        self.adapter
            .goblin_template(&self.local, g, self.local.base as u64)
    }
    unsafe fn events_hide_event_object(
        &self,
        game: u64,
        slot: u32,
        guid: u64,
        object: EventObject,
    ) -> Result<()> {
        unsafe {
            self.adapter.hide_event_object(
                &self.local,
                self.local.base as u64,
                game,
                slot,
                guid,
                object,
            )
        }
    }
    fn events_refresh_blessings(&self, g: u64, ids: &[(u32, u64)]) -> Result<()> {
        self.adapter.refresh_blessings(
            &io::Access {
                local: &self.local,
                adapter: self.adapter,
            },
            g,
            ids,
        )
    }
    unsafe fn events_remove_event_object(&self, game: u64, object: EventObject) -> Result<()> {
        unsafe {
            self.adapter
                .remove_event_object(&self.local, self.local.base as u64, game, object)
        }
    }
    unsafe fn events_remove_ritual_scene(&self, game: u64, scene: &RitualScene) -> Result<()> {
        unsafe {
            self.adapter
                .remove_ritual_scene(&self.local, self.local.base as u64, game, scene)
        }
    }
    fn events_restore_stat_input(&self, loan: StatInputLoan) -> Result<()> {
        self.adapter.restore_stat_input(
            &io::Access {
                local: &self.local,
                adapter: self.adapter,
            },
            loan,
        )
    }
    fn events_ring_stat_values(&self, g: u64) -> Result<[f32; 8]> {
        self.adapter.ring_stat_values(&self.local, g)
    }
    fn events_run_source(&self, game: u64) -> Result<Option<(u32, u64)>> {
        self.adapter.run_source(&self.local, game)
    }
    fn events_solo_run(&self, game: u64) -> Result<Option<PartyObservation>> {
        self.adapter.solo_run(&self.local, game)
    }
    unsafe fn events_spawn_goblin(&self, g: u64) -> Result<Option<GoblinHandle>> {
        unsafe {
            self.adapter
                .spawn_goblin(&self.local, self.local.base as u64, g)
        }
    }
    fn events_world_party(&self, game: u64) -> Result<Option<PartyObservation>> {
        self.adapter.world_party(&self.local, game)
    }
}

fn verify(r: &Runtime, entries: impl IntoIterator<Item = Entry>) -> Result<()> {
    for e in entries {
        if r.local.read(r.local.base + e.rva as usize, e.bytes.len())? != e.bytes {
            return Err(format!(
                "Optional event native contact changed at {:#x}",
                e.rva
            ));
        }
    }
    Ok(())
}
