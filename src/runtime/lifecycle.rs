//! Feature lifetime boundaries shared by native UI teardown and solo capture.
use super::{blessings, combat, duplicate_loot, elites, enchantments, experience,
    fatigue, gems, glyphs, menu_pause, party_ui, support, wallet};
use services::Services;
mod services;

#[cfg(test)]
mod tests;

pub(super) fn end(runtime: &dyn Services) {
    super::world_events::coop::reset();
    super::rituals::reset();
    super::supplies::reset();
    super::treasure_goblins::reset();
    super::raid_markers::reset();
    super::ui_context::reset();
    super::combat_text::reset();
    menu_pause::reset();
    duplicate_loot::reset();
    wallet::reset();
    support::reset();
    elites::reset();
    combat::reset();
    experience::reset();
    party_ui::reset();
    fatigue::reset();
    blessings::reset();
    glyphs::reset();
    enchantments::reset();gems::reset();
    runtime.store_solo(None);
}

pub(super) fn refresh_solo(r:&dyn Services,game:u64){
    let snapshot=r.solo_run(game).ok().flatten();
    if snapshot.is_some(){
        let left_coop=r.detach_coop();
        if left_coop{super::raid_markers::reset();wallet::reset();support::reset();combat::reset();experience::reset();fatigue::reset();}
        blessings::observe(game,snapshot.as_ref());glyphs::observe(game,snapshot.as_ref());enchantments::observe(game,snapshot.as_ref());gems::observe(game,snapshot.as_ref());
    }
    if snapshot.is_none(){super::supplies::reset();super::rituals::reset();}
    r.store_solo(snapshot);
}
