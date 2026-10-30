//! Client-independent Darkmoon rewards. Amounts are whole percentages, stored as basis points.
use std::collections::BTreeMap;
/// ASX progression stops after the fifth completed encounter, on every map.
pub const PROGRESSION_BOSS_CAP:u32=5;
pub const CATEGORY_COUNT:u32=3;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Blessing {
    pub category: u8,
    pub rarity: u8,
    pub amount: u32,
    pub boss: u32,
}
impl Blessing {
    pub fn roll(boss: u32, category_roll: u32, rarity_roll: u32) -> Self {
        let category = (category_roll % CATEGORY_COUNT) as u8;
        let rarity = match rarity_roll % 100 {
            0..=59 => 0,
            60..=84 => 1,
            85..=94 => 2,
            95..=98 => 3,
            _ => 4,
        };
        let scale = ([10, 14, 18, 24, 32][rarity as usize]
            + 2 * boss.min(PROGRESSION_BOSS_CAP).saturating_sub(1))
        .min(40);
        Self {
            category,
            rarity,
            // Round the final award once, so applied stats and UI agree.
            amount: (([1500, 1200, 2500][category as usize] * scale / 10 + 50) / 100) * 100,
            boss,
        }
    }
    pub fn factor(self) -> f32 {
        self.amount as f32 / 10000.0
    }
}
#[derive(Default)]
pub struct Blessings {
    last_event: Option<u64>,
    awards: u32,
    pub actors: BTreeMap<(u32, u64), Blessing>,
}
impl Blessings {
    pub fn retain(&mut self, ids: &[(u32, u64)]) {
        self.actors.retain(|id, _| ids.contains(id));
    }
    pub fn complete(
        &mut self,
        event: u64,
        ids: &[(u32, u64)],
        mut roll: impl FnMut(u32) -> u32,
    ) -> bool {
        if self.last_event.is_some_and(|last| event <= last) {
            return false;
        }
        self.last_event = Some(event);
        self.awards = self.awards.saturating_add(1);
        let boss = self.awards;
        for id in ids {
            self.actors
                .insert(*id, Blessing::roll(boss, roll(CATEGORY_COUNT), roll(100)));
        }
        true
    }
    pub fn strongest(&self, category: u8) -> f32 {
        self.actors
            .values()
            .filter(|b| b.category == category)
            .map(|b| b.amount)
            .max()
            .unwrap_or(0) as f32
            / 10000.0
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_weights_progression_and_caps() {
        let mut counts = [0; 5];
        for n in 0..100 {
            counts[Blessing::roll(1, 0, n).rarity as usize] += 1;
        }
        assert_eq!(counts, [60, 25, 10, 4, 1]);
        assert_eq!(Blessing::roll(1, 2, 99).amount, 8000);
        assert_eq!(Blessing::roll(5, 1, 85).amount, 3100);
        for c in 0..CATEGORY_COUNT {
            assert_eq!(
                Blessing::roll(u32::MAX, c, 99).amount,
                [6000, 4800, 10000][c as usize]
            );
        }
    }
    #[test]
    fn whole_percent_awards_round_only_after_rarity_and_progression() {
        for boss in 1..=6 {
            for category in 0..CATEGORY_COUNT {
                for rarity in 0..100 {
                    assert_eq!(Blessing::roll(boss, category, rarity).amount % 100, 0);
                }
            }
        }
        // Common XP: 12, 14.4, 16.8, 19.2, 21.6 -> 12, 14, 17, 19, 22.
        assert_eq!((1..=5).map(|boss| Blessing::roll(boss, 1, 0).amount).collect::<Vec<_>>(),
                   [1200, 1400, 1700, 1900, 2200]);
        assert_eq!(Blessing::roll(1, 1, 99).factor(), 0.38);
    }
    #[test]
    fn three_equal_categories_and_fixed_progression() {
        let mut counts=[0;3];for draw in 0..300{counts[Blessing::roll(1,draw,0).category as usize]+=1;}assert_eq!(counts,[100;3]);
        for category in 0..CATEGORY_COUNT{for rarity in 0..100{let cap=Blessing::roll(5,category,rarity);for boss in [6,12,u32::MAX]{assert_eq!(Blessing::roll(boss,category,rarity).amount,cap.amount);}}}
    }
    #[test]
    fn replacement_clears_personal_luck_and_does_not_add_party_economy() {
        let mut s=Blessings::default();let ids=[(0,10),(1,20)];
        assert!(s.complete(1,&ids,|n|if n==CATEGORY_COUNT{0}else{99}));assert_eq!(s.actors[&ids[0]].category,0);assert_eq!(s.strongest(1),0.);assert_eq!(s.strongest(2),0.);
        assert!(s.complete(2,&ids,|n|if n==CATEGORY_COUNT{1}else{0}));assert!(s.actors.values().all(|b|b.category==1));assert_eq!(s.strongest(0),0.);assert_eq!(s.strongest(1),0.14);
    }
    #[test]
    fn endless_rewards_continue_with_fixed_map_cap() {
        let mut s = Blessings::default();
        let ids = [(0, 10)];
        for event in 1..=300 {
            assert!(s.complete(event, &ids, |_| 0));
        }
        let b = s.actors[&ids[0]];
        assert_eq!(b.boss, 300);
        assert_eq!(b.amount, 2700);
        assert_eq!(b.amount, Blessing::roll(5, 0, 0).amount);
        assert!(!s.complete(300, &ids, |_| panic!("duplicate callback")));
        assert!(s.complete(301, &ids, |n| if n == CATEGORY_COUNT { 2 } else { 99 }));
        assert_eq!(s.actors[&ids[0]].amount, 10000);
    }
    #[test]
    fn duplicate_event_replacement_disconnect_and_group_max() {
        let mut s = Blessings::default();
        let ids = [(0, 10), (1, 20)];
        assert!(s.complete(7, &ids, |n| if n == CATEGORY_COUNT { 1 } else { 99 }));
        assert!(!s.complete(7, &ids, |_| panic!("duplicate must not roll")));
        assert_eq!(s.strongest(1), 0.38);
        assert!(s.complete(8, &ids, |_| 0));
        assert_eq!(s.strongest(1), 0.0);
        assert_eq!(s.actors[&ids[0]].boss, 2);
        s.retain(&[(0, 11)]);
        assert!(s.actors.is_empty());
    }
}
