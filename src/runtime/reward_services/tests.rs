use super::Services;
use crate::runtime::{enchantments, gems};
use crate::test_support::TEST_LOCK;
use asx_game::official::{AmuletChoice, RingChoice, RingStatLoan};
use asx_gameplay::{enchantments::RankedEnchantment, gems::Gem};
use std::sync::{atomic::{AtomicUsize, Ordering}, Mutex};

const GAME: u64 = 0x1000;
const GUID: u64 = 0x2000;

struct Fake {
    amulet: AmuletChoice,
    ring: RingChoice,
    reenter_on_refresh: std::sync::atomic::AtomicUsize,
    gem_bonuses: Mutex<Vec<Gem>>,
    ring_bonuses: Mutex<Vec<Vec<RankedEnchantment>>>,
    native_calls: AtomicUsize,
    events: Mutex<Vec<&'static str>>,
}

impl Fake {
    fn new() -> Self {
        Self {
            amulet: AmuletChoice { game: GAME, slot: 0, guid: GUID, amulet: 3, previous: None, rank: 0, fingerprint: 1 },
            ring: RingChoice { game: GAME, slot: 0, guid: GUID, ring: 0, rank: 1, rarity: 0, value: 1 },
            reenter_on_refresh: std::sync::atomic::AtomicUsize::new(0),
            gem_bonuses: Mutex::new(Vec::new()), ring_bonuses: Mutex::new(Vec::new()),
            native_calls: AtomicUsize::new(0), events: Mutex::new(Vec::new()),
        }
    }
    fn event(&self, name: &'static str) { self.events.lock().unwrap().push(name); }
    fn accept_gem(&self) {
        gems::card(self, 0x3000, GAME);
        let selected = gems::selecting(self, GAME, 0);
        gems::accepted(self, &selected);
    }
    fn accept_enchantment(&self) {
        enchantments::card(self, 0x3001, GAME);
        let selected = enchantments::selecting(self, GAME, 0);
        enchantments::accepted(self, &selected);
    }
}

impl Services for Fake {
    fn amulet_card(&self, _: u64, _: u64) -> Result<Option<AmuletChoice>, String> { Ok(Some(self.amulet)) }
    fn amulet_selected(&self, _: u64, _: u64) -> Result<Option<AmuletChoice>, String> { Ok(Some(self.amulet)) }
    fn amulet_owned(&self, _: u64) -> Result<Option<u64>, String> { Ok(Some(self.amulet.amulet)) }
    fn amulet_rank(&self, _: u64) -> Result<u32, String> { Ok(1) }
    fn ring_card(&self, _: u64, _: u64) -> Result<Option<RingChoice>, String> { Ok(Some(self.ring)) }
    fn ring_selected(&self, _: u64, _: u64) -> Result<Option<RingChoice>, String> { Ok(Some(self.ring)) }
    fn ring_owned(&self, _: u64, _: u8) -> Result<Option<(u32, u32)>, String> { Ok(Some((2, 0))) }
    fn run_source(&self, _: u64) -> Result<Option<(u32, u64)>, String> { Ok(Some((0, GUID))) }
    fn reward_card_note(&self, _: u64, _: &str) -> Result<(), String> { self.event("card-note"); Ok(()) }
    fn refresh_stats(&self, _: u64) -> Result<(), String> {
        self.event("refresh");
        match self.reenter_on_refresh.swap(0, Ordering::SeqCst) {
            1 => gems::native_stats(self, GAME, || { self.native_calls.fetch_add(1, Ordering::SeqCst); }),
            2 => enchantments::native_stats(self, GAME, || { self.native_calls.fetch_add(1, Ordering::SeqCst); }),
            _ => {}
        }
        Ok(())
    }
    fn gem_stats(&self, _: u64, gem: Gem) -> Result<Option<RingStatLoan>, String> {
        self.gem_bonuses.lock().unwrap().push(gem); self.event("gem-stats"); Ok(None)
    }
    fn ring_stats(&self, _: u64, bonuses: &[RankedEnchantment]) -> Result<Option<RingStatLoan>, String> {
        self.ring_bonuses.lock().unwrap().push(bonuses.to_vec()); self.event("ring-stats"); Ok(None)
    }
    fn restore_stats(&self, _: RingStatLoan) -> Result<(), String> { panic!("fake never issues a stat loan") }
    fn warn(&self, _: &str) { self.event("warn"); }
    fn info(&self, _: &str) { self.event("info"); }
}

fn reset_state() { gems::reset(); enchantments::reset(); }

#[test]
fn accepted_choices_commit_before_reentrant_native_stat_rebuild() {
    let _guard = TEST_LOCK.lock().unwrap(); reset_state();
    let fake = Fake::new(); fake.reenter_on_refresh.store(1, Ordering::SeqCst);
    fake.accept_gem();
    assert_eq!(fake.gem_bonuses.lock().unwrap().len(), 1, "accepted gem is visible to refresh-triggered rebuild");
    assert_eq!(fake.native_calls.load(Ordering::SeqCst), 1, "reentrant refresh reaches the native callback");
    let events = fake.events.lock().unwrap().clone();
    assert!(events.iter().position(|e| *e == "info").unwrap() > events.iter().position(|e| *e == "refresh").unwrap());
    // The selection guard has dropped here; persistence proves acceptance committed
    // the gem instead of relying on its temporary pending-choice entry.
    gems::native_stats(&fake, GAME, || {});
    assert_eq!(fake.gem_bonuses.lock().unwrap().len(), 2);

    fake.native_calls.store(0, Ordering::SeqCst);
    fake.reenter_on_refresh.store(2, Ordering::SeqCst); fake.accept_enchantment();
    assert_eq!(fake.ring_bonuses.lock().unwrap().len(), 1, "accepted enchantment is visible to refresh-triggered rebuild");
    assert_eq!(fake.ring_bonuses.lock().unwrap()[0].len(), 1);
    assert_eq!(fake.native_calls.load(Ordering::SeqCst), 1, "reentrant refresh reaches the native callback");
    // The selection guard has dropped; the accepted ring bonus remains in state.
    enchantments::native_stats(&fake, GAME, || {});
    assert_eq!(fake.ring_bonuses.lock().unwrap().len(), 2);
    reset_state();
}

#[test]
fn nested_stat_reentry_and_panics_release_gem_and_ring_guards() {
    let _guard = TEST_LOCK.lock().unwrap(); reset_state();
    let fake = Fake::new(); fake.accept_gem(); fake.accept_enchantment();

    fake.native_calls.store(0, Ordering::SeqCst);
    gems::native_stats(&fake, GAME, || {
        fake.native_calls.fetch_add(1, Ordering::SeqCst);
        gems::native_stats(&fake, GAME, || { fake.native_calls.fetch_add(1, Ordering::SeqCst); });
    });
    assert_eq!(fake.gem_bonuses.lock().unwrap().len(), 1, "nested callback skips another gem application");
    assert_eq!(fake.native_calls.load(Ordering::SeqCst), 2);

    fake.native_calls.store(0, Ordering::SeqCst);
    enchantments::native_stats(&fake, GAME, || {
        fake.native_calls.fetch_add(1, Ordering::SeqCst);
        enchantments::native_stats(&fake, GAME, || { fake.native_calls.fetch_add(1, Ordering::SeqCst); });
    });
    assert_eq!(fake.ring_bonuses.lock().unwrap().len(), 1, "nested callback skips another ring application");
    assert_eq!(fake.native_calls.load(Ordering::SeqCst), 2);

    let gem_panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        gems::native_stats(&fake, GAME, || panic!("controlled gem callback panic"));
    }));
    assert!(gem_panic.is_err());
    let gem_count = fake.gem_bonuses.lock().unwrap().len(); gems::native_stats(&fake, GAME, || {});
    assert_eq!(fake.gem_bonuses.lock().unwrap().len(), gem_count + 1, "panic clears gem reentry marker");

    let ring_panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        enchantments::native_stats(&fake, GAME, || panic!("controlled ring callback panic"));
    }));
    assert!(ring_panic.is_err());
    let ring_count = fake.ring_bonuses.lock().unwrap().len(); enchantments::native_stats(&fake, GAME, || {});
    assert_eq!(fake.ring_bonuses.lock().unwrap().len(), ring_count + 1, "panic clears ring reentry marker");
    reset_state();
}
