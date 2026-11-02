//! Portable escort and single-tier engineering policy. Native IO is adapter-owned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    BanishedDemon,
    ChickenEscort,
    TurretConstruction,
    HeroAssistance,
}
impl Kind {
    pub const ALL: [Self; 4] = [
        Self::BanishedDemon,
        Self::ChickenEscort,
        Self::TurretConstruction,
        Self::HeroAssistance,
    ];
    /// Demon and escort are the two reviewed native encounter families.
    pub fn active(escort_allowed: bool) -> Vec<Self> {
        if escort_allowed { vec![Self::BanishedDemon, Self::ChickenEscort] }
        else { vec![Self::BanishedDemon] }
    }
}
/// A one-minute upper bound on large maps; smaller arenas get shorter routes.
pub const ESCORT_SEGMENTS: usize = 120;
pub const ESCORT_SEGMENT_LENGTH: f32 = 1.5;
/// Keep the route proportional to the native playable radius. The minimum is
/// bounded for unusually small future maps; native ground checks still decide
/// whether any candidate is viable.
pub fn escort_segments(radius: f32) -> Option<usize> {
    if !radius.is_finite() || radius < 10. {
        return None;
    }
    Some(((radius * 1.35 / ESCORT_SEGMENT_LENGTH).floor() as usize).clamp(8, ESCORT_SEGMENTS))
}
/// Every active participant must be alive, playing and within six yards.
pub fn event_party_present(
    members: impl IntoIterator<Item = (bool, [f32; 2])>, focus: [f32; 2], radius: f32,
) -> bool {
    if !radius.is_finite() || radius <= 0. { return false }
    let mut count = 0;
    for (ready, position) in members {
        count += 1;
        if !ready || position.iter().any(|v| !v.is_finite())
            || (position[0] - focus[0]).hypot(position[1] - focus[1]) > radius {
            return false;
        }
    }
    count > 0
}
pub fn escort_party_present(
    members: impl IntoIterator<Item = (bool, [f32; 2])>, focus: [f32; 2],
) -> bool { event_party_present(members,focus,6.) }
/// Small scenery points behind the starting chicken, clear of its forward route.
pub fn escort_crash_sites(route: &[[f32; 3]]) -> Option<[[f32; 2]; 4]> {
    let [start, next, ..] = route else { return None };
    let dx = next[0] - start[0];
    let dy = next[1] - start[1];
    let length = dx.hypot(dy);
    if !length.is_finite() || length < 0.5 { return None }
    let forward = [dx / length, dy / length];
    let side = [-forward[1], forward[0]];
    Some([(-1.5, 1.3), (-1.7, -1.2), (-0.8, 2.2), (-2.4, 0.)]
        .map(|(along, across)| {
            [start[0] + along * forward[0] + across * side[0],
             start[1] + along * forward[1] + across * side[1]]
        }))
}
/// Survives scene cleanup/death; only a new run identity or clock rollback
/// admits another escort. Reserve before native creation, never after reward.
#[derive(Default)]
pub struct RunOpportunities {
    run: Option<(u64, u64)>,
    last: f32,
    escort_used: bool,
    first_boss_at: Option<f32>,
}
impl RunOpportunities {
    pub const fn new() -> Self {
        Self {
            run: None,
            last: 0.,
            escort_used: false,
            first_boss_at: None,
        }
    }
    pub fn observe(&mut self, run: u64, owner: u64, now: f32) -> bool {
        if run == 0 || owner == 0 || !now.is_finite() || now < 0. {
            return false;
        }
        if self.run != Some((run, owner)) || now < self.last {
            self.run = Some((run, owner));
            self.escort_used = false;
            self.first_boss_at = None;
        }
        self.last = now;
        true
    }
    /// Only the selected native completed-encounter callback opens this gate.
    pub fn encounter(&mut self, run: u64, owner: u64, now: f32) -> bool {
        if !self.observe(run, owner, now) || self.first_boss_at.is_some() {
            return false;
        }
        self.first_boss_at = Some(now);
        true
    }
    pub fn first_boss_at(&self, run: u64, owner: u64) -> Option<f32> {
        (self.run == Some((run, owner))).then_some(self.first_boss_at).flatten()
    }
    pub fn allows(&self, kind: Kind) -> bool {
        self.run.is_some() && (kind != Kind::ChickenEscort || self.first_boss_at.is_some() && !self.escort_used)
    }
    pub fn reserve_escort(&mut self, run: u64, owner: u64) -> bool {
        if self.run != Some((run, owner)) || self.first_boss_at.is_none() || self.escort_used {
            return false;
        }
        self.escort_used = true;
        true
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Proofs {
    pub assets: bool,
    pub cleanup: bool,
    pub hostile_combat: bool,
    pub banish: bool,
    pub ally_combat: bool,

    pub navigation: bool,
    pub attribution: bool,
    pub hero_abilities: bool,
    pub gadget: bool,
}
impl Proofs {
    pub fn permits(self, kind: Kind, solo: bool) -> bool {
        self.assets && self.cleanup && match kind {
            Kind::ChickenEscort => self.navigation && self.gadget && self.attribution,
            Kind::BanishedDemon => self.hostile_combat && self.banish && self.attribution,
            Kind::TurretConstruction => solo && self.ally_combat && self.attribution,
            Kind::HeroAssistance => solo && self.ally_combat && self.navigation
                && self.attribution && self.hero_abilities,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Identity {
    pub run: u64,
    pub owner: u64,
    pub event: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Waiting,
    Preparing,
    Engaged,
    Succeeded,
    Failed,
    Retired,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reward {
    BanishCharge,
    EngineeringGadget,
}
#[derive(Clone, Debug)]
pub struct Scene {
    pub id: Identity,
    pub kind: Kind,
    stage: Stage,
    held: f32,
    last: f32,
    was_present: bool,
    service_until: Option<f32>,
    reward_taken: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct Observation {
    pub id: Identity,
    pub now: f32,
    pub playing: bool,
    pub solo: bool,
    pub player_alive: bool,
    pub present: bool,
    pub unit_alive: bool,
    pub arrived: bool,
    pub enemy_defeated: bool,
}
impl Scene {
    pub fn new(id: Identity, kind: Kind, now: f32, proofs: Proofs, solo: bool) -> Option<Self> {
        (id.run != 0
            && id.owner != 0
            && id.event != 0
            && now.is_finite()
            && now >= 0.
            && proofs.permits(kind, solo))
        .then_some(Self {
            id,
            kind,
            stage: Stage::Waiting,
            held: 0.,
            last: now,
            was_present: false,
            service_until: None,
            reward_taken: false,
        })
    }
    pub fn stage(&self) -> Stage {
        self.stage
    }
    pub fn await_service(&mut self) {
        if self.stage == Stage::Engaged
            && matches!(self.kind, Kind::TurretConstruction | Kind::HeroAssistance)
        {
            self.service_until = None;
        }
    }
    pub fn remaining(&self, now: f32) -> f32 {
        self.service_until.map_or(0., |end| (end - now).max(0.))
    }
    pub fn begin_service(&mut self, now: f32) {
        if self.stage == Stage::Engaged {
            self.service_until = match self.kind {
                Kind::TurretConstruction => Some(now + 45.),
                Kind::HeroAssistance => Some(now + 35.),
                _ => None,
            };
        }
    }
    pub fn held(&self) -> f32 {
        self.held
    }
    pub fn observe(&mut self, o: Observation) -> Stage {
        if o.id != self.id || (!o.solo && !matches!(self.kind, Kind::ChickenEscort | Kind::BanishedDemon))
            || !o.now.is_finite() || o.now < self.last {
            self.retire();
            return self.stage;
        }
        let dt = (o.now - self.last).min(0.5);
        self.last = o.now;
        if matches!(
            self.stage,
            Stage::Succeeded | Stage::Failed | Stage::Retired
        ) {
            return self.stage;
        }
        if !o.player_alive {
            self.stage = Stage::Failed;
            return self.stage;
        }
        if !o.playing {
            self.was_present = false;
            return self.stage;
        }
        if self.stage == Stage::Waiting && o.present {
            self.stage = Stage::Preparing;
        }
        if self.stage == Stage::Preparing {
            if self.was_present && o.present {
                self.held += dt;
            }
            let duration = match self.kind {
                Kind::BanishedDemon => 2.,
                Kind::ChickenEscort => 2.,
                Kind::TurretConstruction => 8.,
                Kind::HeroAssistance => 3.,
            };
            if self.held >= duration {
                self.stage = Stage::Engaged;
                self.service_until = match self.kind {
                    Kind::TurretConstruction => Some(o.now + 45.),
                    Kind::HeroAssistance => Some(o.now + 35.),
                    _ => None,
                };
            }
        }
        self.was_present = o.present;
        if self.stage == Stage::Engaged {
            self.stage = match self.kind {
                Kind::BanishedDemon if o.enemy_defeated => Stage::Succeeded,
                Kind::ChickenEscort if o.arrived => Stage::Succeeded,
                Kind::TurretConstruction | Kind::HeroAssistance
                    if self.service_until.is_some_and(|t| o.now >= t) =>
                {
                    Stage::Succeeded
                }
                Kind::TurretConstruction | Kind::HeroAssistance if !o.unit_alive => Stage::Failed,
                _ => self.stage,
            };
        }
        self.stage
    }
    /// Reserve before invoking native code. An uncertain native commit is never retried.
    pub fn take_reward(&mut self) -> Option<Reward> {
        if self.stage != Stage::Succeeded || self.reward_taken {
            return None;
        }
        self.reward_taken = true;
        match self.kind {
            Kind::BanishedDemon => Some(Reward::BanishCharge),
            Kind::ChickenEscort => Some(Reward::EngineeringGadget),
            _ => None,
        }
    }
    pub fn retire(&mut self) {
        self.stage = Stage::Retired;
        self.reward_taken = true;
        self.was_present = false;
    }
}
/// Owns engagement only; waiting scenes and existing boss scheduling stay native.
#[derive(Default)]
pub struct Engagement {
    active: Option<Identity>,
}
impl Engagement {
    pub fn acquire(&mut self, id: Identity, solo: bool) -> bool {
        if !solo || id.run == 0 || id.owner == 0 || id.event == 0 {
            return false;
        }
        match self.active {
            None => {
                self.active = Some(id);
                true
            }
            Some(old) => old == id,
        }
    }
    pub fn release(&mut self, id: Identity) {
        if self.active == Some(id) {
            self.active = None;
        }
    }
    pub fn reset(&mut self) {
        self.active = None;
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Gadget {
    Net = 0,
    RocketBoots = 1,
    Grenade = 3,
    ShrinkRay = 4,
}
impl Gadget {
    pub const ALL: [Self; 4] = [Self::Net, Self::RocketBoots, Self::Grenade, Self::ShrinkRay];
    pub fn interval(self) -> (u32, u32) {
        match self {
            Self::Net => (12, 20),
            Self::RocketBoots => (16, 25),
            Self::Grenade => (25, 35),
            Self::ShrinkRay => (10, 18),
        }
    }
    pub fn duration(self) -> f32 {
        match self {
            Self::Net => 2.,
            Self::RocketBoots => 2.,
            Self::Grenade => 0.,
            Self::ShrinkRay => 6.,
        }
    }
    pub fn ordinary_only(self) -> bool {
        matches!(self, Self::Net | Self::Grenade)
    }
}
#[derive(Clone, Debug)]
pub struct Equipment {
    pub kind: Gadget,
    next: f32,
    expires: f32,
    last: f32,
}
impl Equipment {
    pub fn new(kind: Gadget, now: f32, draw: u32) -> Option<Self> {
        if !now.is_finite() || now < 0. {
            return None;
        }
        let mut value = Self {
            kind,
            next: now,
            expires: now,
            last: now,
        };
        value.schedule(now, draw);
        Some(value)
    }
    fn schedule(&mut self, now: f32, draw: u32) {
        let (low, high) = self.kind.interval();
        self.next = now + (low + draw % (high - low + 1)) as f32;
    }
    pub fn due(&self, now: f32) -> bool {
        now.is_finite() && now >= self.last && now >= self.next
    }
    pub fn next_trigger(&self) -> f32 {
        self.next
    }
    #[cfg(feature = "test-hooks")]
    pub fn diagnostic_ready(&mut self, now: f32) {
        if now.is_finite() && now >= self.last {
            self.next = now;
        }
    }
    /// Caller draws only after a trigger or a new reward, never for idle frames.
    pub fn trigger(&mut self, now: f32, playing: bool, alive: bool, draw: u32) -> bool {
        if !now.is_finite() || now < self.last {
            return false;
        }
        self.last = now;
        if !playing || !alive || now < self.next {
            return false;
        }
        self.expires = now + self.kind.duration();
        self.schedule(now, draw);
        true
    }
    pub fn effect_until(&self) -> f32 {
        self.expires
    }
    /// One fixed-power gadget; diagnostic replacement never upgrades it.
    /// The native caller must remove an old effect before applying a replacement.
    pub fn replace(&mut self, kind: Gadget, now: f32, draw: u32) -> bool {
        if !now.is_finite() || now < self.last {
            return false;
        }
        self.kind = kind;
        self.expires = now;
        self.last = now;
        self.schedule(now, draw);
        true
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn proofs() -> Proofs {
        Proofs {
            assets: true,
            cleanup: true,
            hostile_combat: true,
            banish: true,
            ally_combat: true,
            navigation: true,
            attribution: true,
            hero_abilities: true,
            gadget: true,
        }
    }
    fn id() -> Identity {
        Identity {
            run: 1,
            owner: 11,
            event: 2,
        }
    }
    fn obs(now: f32) -> Observation {
        Observation {
            id: id(),
            now,
            playing: true,
            solo: true,
            player_alive: true,
            present: true,
            unit_alive: true,
            arrived: false,
            enemy_defeated: false,
        }
    }
    fn engaged(kind: Kind) -> Scene {
        let mut s = Scene::new(id(), kind, 0., proofs(), true).unwrap();
        for i in 0..=20 {
            s.observe(obs(i as f32 / 2.));
        }
        assert_eq!(s.stage(), Stage::Engaged);
        s
    }
    #[test]
    fn only_reviewed_events_enter_normal_draws() {
        assert_eq!(Kind::active(true), vec![Kind::BanishedDemon, Kind::ChickenEscort]);
        assert_eq!(Kind::active(false), vec![Kind::BanishedDemon]);
        for allowed in [true, false] {
            assert!(!Kind::active(allowed).contains(&Kind::TurretConstruction));
            assert!(!Kind::active(allowed).contains(&Kind::HeroAssistance));
        }
    }
    #[test]
    fn one_escort_per_run_even_after_scene_failure_or_death() {
        let mut budget = RunOpportunities::default();
        assert!(!budget.allows(Kind::ChickenEscort));
        assert!(!budget.observe(0, 11, 0.));
        assert!(budget.observe(1, 11, 10.));
        assert!(!budget.allows(Kind::ChickenEscort));
        assert!(!budget.reserve_escort(1, 11)); // elapsed time alone cannot open the gate
        assert!(!budget.encounter(1, 22, f32::NAN));
        assert!(budget.encounter(1, 11, 180.));
        assert!(!budget.encounter(1, 11, 181.));
        assert_eq!(budget.first_boss_at(1, 11), Some(180.));
        assert!(!budget.reserve_escort(1, 22));
        assert!(budget.reserve_escort(1, 11));
        for now in [182., 200., 600.] {
            assert!(budget.observe(1, 11, now));
            assert!(!budget.allows(Kind::ChickenEscort));
            assert!(!budget.reserve_escort(1, 11));
            assert!(budget.allows(Kind::BanishedDemon));
        }
        assert!(budget.observe(1, 11, 0.)); // next run reuses native addresses
        assert_eq!(budget.first_boss_at(1, 11), None);
        assert!(!budget.reserve_escort(1, 11));
        assert!(budget.encounter(1, 11, 190.));
        assert!(budget.reserve_escort(1, 11));
        assert!(budget.observe(2, 11, 0.));
        assert!(!budget.allows(Kind::ChickenEscort));
        assert!(!budget.observe(2, 11, f32::NAN));
    }
    #[test]
    fn no_unproved_activation_or_cooperative_prototype_fallback() {
        for k in Kind::ALL {
            assert!(Scene::new(id(), k, 0., Proofs::default(), true).is_none());
            assert_eq!(Scene::new(id(), k, 0., proofs(), false).is_some(),
                matches!(k, Kind::ChickenEscort | Kind::BanishedDemon));
            let mut p = proofs();
            p.cleanup = false;
            assert!(!p.permits(k, true));
        }
    }
    #[test]
    fn every_indispensable_specific_proof_is_required() {
        let mut p = proofs();
        p.banish = false;
        assert!(!p.permits(Kind::BanishedDemon, true));
        p = proofs();
        p.navigation = false;
        assert!(!p.permits(Kind::ChickenEscort, true) && !p.permits(Kind::HeroAssistance, true));
        p = proofs();
        p.ally_combat = false;
        assert!(!p.permits(Kind::TurretConstruction, true));
    }
    #[test]
    fn preparation_preserves_partial_work_and_does_not_catch_up_pause() {
        let mut s = Scene::new(id(), Kind::TurretConstruction, 0., proofs(), true).unwrap();
        s.observe(obs(0.));
        s.observe(obs(0.5));
        let held = s.held();
        s.observe(Observation {
            present: false,
            ..obs(1.)
        });
        s.observe(obs(1.5));
        assert_eq!(s.held(), held);
        s.observe(Observation {
            playing: false,
            ..obs(100.)
        });
        s.observe(obs(100.5));
        assert_eq!(s.held(), held);
        s.observe(obs(101.));
        assert_eq!(s.held(), held + 0.5);
    }
    #[test]
    fn demon_reward_only_after_native_defeat_and_at_most_once() {
        let mut s = engaged(Kind::BanishedDemon);
        assert_eq!(s.take_reward(), None);
        s.observe(Observation {
            enemy_defeated: true,
            ..obs(11.)
        });
        assert_eq!(s.take_reward(), Some(Reward::BanishCharge));
        assert_eq!(s.take_reward(), None);
        s.observe(obs(12.));
        assert_eq!(s.take_reward(), None);
    }
    #[test]
    fn invulnerable_chicken_needs_navigation_not_a_native_health_pool() {
        let p = proofs();
        assert!(p.permits(Kind::ChickenEscort, true));
        assert!(p.permits(Kind::TurretConstruction, true));
        let mut s = engaged(Kind::ChickenEscort);
        assert_eq!(
            s.observe(Observation {
                unit_alive: false,
                ..obs(11.)
            }),
            Stage::Engaged
        );
        assert_eq!(s.take_reward(), None);
        s.observe(Observation {
            arrived: true,
            ..obs(11.)
        });
        assert_eq!(s.take_reward(), Some(Reward::EngineeringGadget));
    }
    #[test]
    fn identity_death_coop_and_clock_rollback_retire_without_rewards() {
        for o in [
            Observation {
                id: Identity { owner: 22, ..id() },
                ..obs(11.)
            },
            obs(1.),
            Observation {
                now: f32::NAN,
                ..obs(11.)
            },
        ] {
            let mut s = engaged(Kind::HeroAssistance);
            assert_eq!(s.observe(o), Stage::Retired);
        }
        for k in Kind::ALL {
            let mut s = engaged(k);
            assert_eq!(
                s.observe(Observation {
                    player_alive: false,
                    ..obs(11.)
                }),
                Stage::Failed
            );
            assert_eq!(s.take_reward(), None);
        }
    }
    #[test]
    fn cooperative_escort_waits_for_every_active_participant() {
        let focus = [10., 20.];
        assert!(!escort_party_present([], focus));
        assert!(escort_party_present([(true, [10., 20.]), (true, [15., 20.])], focus));
        assert!(!escort_party_present([(true, [10., 20.]), (true, [16.1, 20.])], focus));
        assert!(!escort_party_present([(true, [10., 20.]), (false, [10., 20.])], focus));
        assert!(!escort_party_present([(true, [f32::NAN, 20.])], focus));
        let mut scene = Scene::new(id(), Kind::ChickenEscort, 0., proofs(), false).unwrap();
        assert_eq!(scene.observe(Observation { solo:false, present:true, ..obs(0.) }), Stage::Preparing);
        scene.observe(Observation { solo:false, present:true, ..obs(1.) });
        let held = scene.held();
        scene.observe(Observation { solo:false, present:false, ..obs(1.5) });
        scene.observe(Observation { solo:false, present:false, ..obs(2.) });
        assert_eq!(scene.held(), held);
        scene.observe(Observation { solo:false, present:true, ..obs(2.5) });
        scene.observe(Observation { solo:false, present:true, ..obs(3.) });
        assert!(scene.held() > held);
    }
    #[test]
    fn cooperative_demon_needs_the_whole_group_to_break_the_binding() {
        let focus=[10.,20.];
        assert!(event_party_present([(true,[10.,20.]),(true,[13.9,20.])],focus,4.));
        assert!(!event_party_present([(true,[10.,20.]),(true,[14.1,20.])],focus,4.));
        assert!(!event_party_present([(true,[10.,20.]),(false,[10.,20.])],focus,4.));
        let mut scene=Scene::new(id(),Kind::BanishedDemon,0.,proofs(),false).unwrap();
        scene.observe(Observation{solo:false,present:true,..obs(0.)});
        scene.observe(Observation{solo:false,present:true,..obs(1.)});
        let held=scene.held();
        scene.observe(Observation{solo:false,present:false,..obs(1.5)});
        scene.observe(Observation{solo:false,present:false,..obs(2.)});
        assert_eq!(scene.held(),held);
        scene.observe(Observation{solo:false,present:true,..obs(2.5)});
        scene.observe(Observation{solo:false,present:true,..obs(3.)});
        scene.observe(Observation{solo:false,present:true,..obs(3.5)});
        scene.observe(Observation{solo:false,present:true,..obs(4.)});
        assert_eq!(scene.stage(),Stage::Engaged);
        assert_eq!(scene.take_reward(),None);
        scene.observe(Observation{solo:false,enemy_defeated:true,..obs(4.5)});
        assert_eq!(scene.take_reward(),Some(Reward::BanishCharge));
        assert_eq!(scene.take_reward(),None);
    }
    #[test]
    fn escort_length_scales_with_native_map_radius() {
        assert_eq!(escort_segments(32.), Some(28));
        assert_eq!(escort_segments(44.), Some(39));
        assert_eq!(escort_segments(120.), Some(108));
        assert_eq!(escort_segments(150.), Some(120));
        assert_eq!(escort_segments(f32::NAN), None);
        assert_eq!(escort_segments(9.), None);
    }
    #[test]
    fn services_expire_and_missing_native_turret_retires() {
        let mut s = engaged(Kind::TurretConstruction);
        assert_eq!(
            s.observe(Observation {
                unit_alive: false,
                ..obs(11.)
            }),
            Stage::Failed
        );
        let mut s = engaged(Kind::HeroAssistance);
        assert_eq!(s.observe(obs(60.)), Stage::Succeeded);
        assert_eq!(s.take_reward(), None);
    }
    #[test]
    fn service_requires_a_real_native_start_and_expiry_wins_over_removed_row() {
        let mut turret = engaged(Kind::TurretConstruction);
        turret.await_service();
        assert_eq!(turret.remaining(11.), 0.);
        assert_eq!(turret.observe(obs(60.)), Stage::Engaged);
        turret.begin_service(60.);
        assert_eq!(turret.remaining(60.), 45.);
        assert_eq!(
            turret.observe(Observation {
                unit_alive: false,
                ..obs(105.)
            }),
            Stage::Succeeded
        );
        let mut hero = engaged(Kind::HeroAssistance);
        assert_eq!(
            hero.observe(Observation {
                unit_alive: false,
                ..obs(11.)
            }),
            Stage::Failed
        );
    }
    #[test]
    fn engagement_does_not_release_another_scene() {
        let mut e = Engagement::default();
        assert!(e.acquire(id(), true));
        let other = Identity { event: 3, ..id() };
        assert!(!e.acquire(other, true));
        e.release(other);
        assert!(!e.acquire(other, true));
        e.release(id());
        assert!(e.acquire(other, true));
        e.reset();
        assert!(!e.acquire(id(), false));
    }
    #[test]
    fn all_active_gadgets_have_bounded_delays_and_one_effect() {
        for kind in Gadget::ALL {
            let mut g = Equipment::new(kind, 0., u32::MAX).unwrap();
            assert!(!g.trigger(1., true, true, 0));
            assert!(!g.trigger(100., false, true, 0));
            assert!(g.trigger(100., true, true, 0));
            assert!(!g.trigger(100., true, true, 0));
            assert!(g.effect_until() <= 106.);
            assert!(!g.trigger(101., true, false, 0));
            for _ in 0..9 {
                assert!(g.replace(kind, 101., u32::MAX));
            }
            assert_eq!(g.kind, kind);
            assert_eq!(g.effect_until(), 101.);
        }
    }
    #[test]
    fn boots_are_two_seconds_and_replacement_cannot_stack() {
        let mut g = Equipment::new(Gadget::RocketBoots, 0., 0).unwrap();
        assert!(g.trigger(16., true, true, 0));
        assert_eq!(g.effect_until(), 18.);
        assert!(g.replace(Gadget::Net, 17., 0));
        assert_eq!(g.effect_until(), 17.);
        assert!(!g.trigger(18., true, true, 0));
    }
}


#[cfg(test)]
mod crash_scene_tests {
    use super::escort_crash_sites;
    #[test]
    fn debris_remains_behind_and_away_from_the_chicken_on_rotated_routes() {
        for end in [[1.5,0.,0.],[0.,1.5,0.],[-1.5,0.,0.],[0.,-1.5,0.]] {
            let sites=escort_crash_sites(&[[0.;3],end]).unwrap();
            for p in sites {
                let forward=p[0]*end[0]+p[1]*end[1];
                assert!(forward<0.,"debris must not cover the route");
                assert!((1.0..=3.0).contains(&p[0].hypot(p[1])));
            }
        }
        assert!(escort_crash_sites(&[[0.;3],[0.;3]]).is_none());
    }
}
