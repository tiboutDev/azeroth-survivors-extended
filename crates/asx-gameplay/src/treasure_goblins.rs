//! Optional run events. No native identities, transport or wall-clock timers.
pub const ARRIVAL_SECONDS: f32 = 1.5;
/// Native ordinary-boss amount, awarded only as whole marks. Never round up
/// or manufacture a minimum reward on a map/difficulty yielding less than one.
pub fn whole_boss_marks(amount: f32) -> Option<u32> {
    (amount.is_finite() && (0.0..=10_000_000.0).contains(&amount))
        .then(|| amount.floor() as u32)
}

/// The native circular arena, with a small clearance for the enlarged creature.
#[derive(Clone, Copy, Debug)]
pub struct Arena {
    pub center: [f32; 2],
    pub radius: f32,
}
impl Arena {
    pub fn contains(self, p: [f32; 3]) -> bool {
        let radius = self.radius - 2.;
        self.center.iter().chain(p.iter()).all(|v| v.is_finite())
            && radius.is_finite()
            && radius > 0.
            && (p[0] - self.center[0]).hypot(p[1] - self.center[1]) <= radius
    }
    /// Clip the straight segment at the arena rim, never teleport back inside.
    pub fn clip(self, origin: [f32; 3], goal: [f32; 3]) -> Option<[f32; 3]> {
        if !self.contains(origin) || goal.iter().any(|v| !v.is_finite()) {
            return None;
        }
        if self.contains(goal) {
            return Some(goal);
        }
        let d = [goal[0] - origin[0], goal[1] - origin[1]];
        let o = [origin[0] - self.center[0], origin[1] - self.center[1]];
        let a = d[0] * d[0] + d[1] * d[1];
        if a < 0.0001 {
            return None;
        }
        let b = o[0] * d[0] + o[1] * d[1];
        let c = o[0] * o[0] + o[1] * o[1] - (self.radius - 2.01).powi(2);
        let discriminant = b * b - a * c;
        if discriminant < 0. {
            return None;
        }
        let t = ((-b + discriminant.sqrt()) / a).clamp(0., 1.);
        let clipped = std::array::from_fn(|i| origin[i] + t * (goal[i] - origin[i]));
        self.contains(clipped).then_some(clipped)
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Active {
    pub id: u64,
    pub expires: f32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resolution {
    Killed { id: u64 },
    Escaped { id: u64 },
}
#[derive(Clone, Debug)]
pub struct Director {
    next: Option<f32>,
    active: Option<Active>,
    rng: u64,
    appeared: u32,
    encounters: u32,
}
impl Director {
    pub fn new(seed: u64) -> Self {
        Self {
            next: None,
            active: None,
            rng: seed,
            appeared: 0,
            encounters: 0,
        }
    }
    fn roll(&mut self, n: u32) -> u32 {
        loop {
            self.rng = self.rng.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = self.rng;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            z ^= z >> 31;
            if z < u64::MAX - u64::MAX % u64::from(n) {
                return (z % u64::from(n)) as u32;
            }
        }
    }
    pub fn chance(&self) -> u32 {
        match self.appeared {
            0 => 12,
            1 => 6,
            _ => 3,
        }
    }
    /// Called once by the native completed-encounter transition, never per entity.
    pub fn encounter(&mut self, now: f32) -> bool {
        if !now.is_finite() {
            return false;
        }
        self.encounters = self.encounters.saturating_add(1);
        if self.active.is_some() || self.next.is_some() {
            return false;
        }
        if self.roll(100) >= self.chance() {
            return false;
        }
        self.next = Some(now + 20. + self.roll(21) as f32);
        true
    }
    pub fn encounters(&self) -> u32 {
        self.encounters
    }
    pub fn abandon(&mut self) {
        self.active = None;
    }
    pub fn active(&self) -> Option<Active> {
        self.active
    }
    pub fn due(&self, now: f32) -> bool {
        now.is_finite() && self.next.is_some_and(|t| now >= t) && self.active.is_none()
    }
    /// Consume the opportunity before native placement; never retry an uncertain commit.
    pub fn reserve(&mut self, _now: f32) {
        self.next = None;
    }
    pub fn spawned(&mut self, id: u64, now: f32) -> bool {
        if id == 0 || !now.is_finite() || self.active.is_some() {
            return false;
        }
        self.appeared = self.appeared.saturating_add(1);
        self.active = Some(Active {
            id,
            expires: now + 25.,
        });
        true
    }
    pub fn resolve(&mut self, now: f32, alive: bool) -> Option<Resolution> {
        let a = self.active?;
        if !now.is_finite() || alive && now < a.expires {
            return None;
        }
        self.active = None;
        Some(if alive {
            Resolution::Escaped { id: a.id }
        } else {
            Resolution::Killed { id: a.id }
        })
    }
}
/// Candidate destinations ordered away from nearby living pursuers. Native terrain
/// validation selects a reachable candidate; lateral alternatives include tangential movement around walls.
pub fn flee_candidates(
    origin: [f32; 3],
    pursuers: &[[f32; 3]],
    heading: [f32; 2],
) -> Vec<[f32; 3]> {
    if origin.iter().any(|v| !v.is_finite()) {
        return vec![];
    }
    let mut away = [0f32; 2];
    for p in pursuers.iter().filter(|p| p.iter().all(|v| v.is_finite())) {
        let dx = origin[0] - p[0];
        let dy = origin[1] - p[1];
        let d2 = dx * dx + dy * dy;
        let weight = 1. / d2.max(1.);
        away[0] += dx * weight;
        away[1] += dy * weight;
    }
    let length = away[0].hypot(away[1]);
    let direction = if length > 0.001 {
        [away[0] / length, away[1] / length]
    } else if heading.iter().all(|v| v.is_finite()) && heading[0].hypot(heading[1]) > 0.1 {
        let n = heading[0].hypot(heading[1]);
        [heading[0] / n, heading[1] / n]
    } else {
        [1., 0.]
    };
    [
        0f32,
        0.52,
        -0.52,
        1.05,
        -1.05,
        std::f32::consts::FRAC_PI_2,
        -std::f32::consts::FRAC_PI_2,
    ]
    .into_iter()
    .map(|angle| {
        let (s, c) = angle.sin_cos();
        [
            origin[0] + 18. * (direction[0] * c - direction[1] * s),
            origin[1] + 18. * (direction[0] * s + direction[1] * c),
            origin[2],
        ]
    })
    .collect()
}
/// Short-lived memory of a direction that produced no actual displacement.
/// Avoids repeatedly selecting a nominally clear destination behind an obstacle.
#[derive(Clone, Copy, Debug, Default)]
pub struct FleeNavigation {
    probe: Option<(f32, [f32; 3])>,
    blocked: Option<([f32; 2], f32)>,
}
impl FleeNavigation {
    pub fn observe(&mut self, now: f32, position: [f32; 3], goal: Option<[f32; 3]>) {
        if !now.is_finite() || position.iter().any(|v| !v.is_finite()) {
            return;
        }
        if self.blocked.is_some_and(|(_, until)| now >= until) {
            self.blocked = None;
        }
        match self.probe {
            Some((since, _)) if now >= since && now - since < 0.8 => {}
            Some((_, before)) => {
                if (position[0] - before[0]).hypot(position[1] - before[1]) < 0.5 {
                    if let Some(goal) = goal {
                        let delta = [goal[0] - position[0], goal[1] - position[1]];
                        let length = delta[0].hypot(delta[1]);
                        if length.is_finite() && length > 1. {
                            self.blocked =
                                Some(([delta[0] / length, delta[1] / length], now + 2.5));
                        }
                    }
                }
                self.probe = Some((now, position));
            }
            None => self.probe = Some((now, position)),
        }
    }
    /// Rank reachable segments, rather than taking the first partly clipped ray.
    /// Longer clear paths beat dead ends; heading continuity avoids left/right jitter.
    pub fn choose(
        self,
        origin: [f32; 3],
        pursuers: &[[f32; 3]],
        heading: [f32; 2],
        goals: &[[f32; 3]],
    ) -> Option<[f32; 3]> {
        let heading_length = heading[0].hypot(heading[1]);
        let nearest = |p: [f32; 3]| {
            pursuers
                .iter()
                .filter(|a| a.iter().all(|v| v.is_finite()))
                .map(|a| (p[0] - a[0]).hypot(p[1] - a[1]))
                .fold(f32::INFINITY, f32::min)
        };
        let old_distance = nearest(origin);
        let mut best = None;
        let mut best_score = f32::NEG_INFINITY;
        for &goal in goals {
            if !escapes(origin, goal, pursuers) {
                continue;
            }
            let d = [goal[0] - origin[0], goal[1] - origin[1]];
            let length = d[0].hypot(d[1]);
            if length < 0.5 {
                continue;
            }
            let forward = if heading_length > 0.01 && heading_length.is_finite() {
                (d[0] * heading[0] + d[1] * heading[1]) / (length * heading_length)
            } else {
                0.
            };
            let gain = if old_distance.is_finite() {
                (nearest(goal) - old_distance).clamp(0., 10.)
            } else {
                0.
            };
            let penalty = self
                .blocked
                .map(|(blocked, _)| {
                    let alignment = (d[0] * blocked[0] + d[1] * blocked[1]) / length;
                    if alignment > 0.7 {
                        60.
                    } else {
                        0.
                    }
                })
                .unwrap_or(0.);
            let score = length.min(18.) * 2. + gain + forward * 3. - penalty;
            if score > best_score {
                best = Some(goal);
                best_score = score;
            }
        }
        best
    }
}
/// A cached/clipped destination may become unsafe when a pursuer moves. Never
/// approach the nearest living threat; stop if no forward escape remains.
pub fn escapes(origin: [f32; 3], goal: [f32; 3], pursuers: &[[f32; 3]]) -> bool {
    if origin.iter().chain(goal.iter()).any(|v| !v.is_finite()) {
        return false;
    }
    let delta = [goal[0] - origin[0], goal[1] - origin[1]];
    if delta[0].hypot(delta[1]) < 0.01 {
        return false;
    }
    let distance = |a: [f32; 3], b: [f32; 3]| (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2);
    let valid = pursuers
        .iter()
        .copied()
        .filter(|p| p.iter().all(|v| v.is_finite()))
        .collect::<Vec<_>>();
    let Some(closest) = valid
        .iter()
        .copied()
        .min_by(|a, b| distance(origin, *a).total_cmp(&distance(origin, *b)))
    else {
        return true;
    };
    let old = distance(origin, closest);
    if old > 0.01
        && delta[0] * (origin[0] - closest[0]) + delta[1] * (origin[1] - closest[1]) < -0.01
    {
        return false;
    }
    let segment_length2 = delta[0] * delta[0] + delta[1] * delta[1];
    valid.iter().all(|p| {
        let t = (((p[0] - origin[0]) * delta[0] + (p[1] - origin[1]) * delta[1]) / segment_length2)
            .clamp(0., 1.);
        let closest_on_segment = [
            origin[0] + t * delta[0],
            origin[1] + t * delta[1],
            origin[2],
        ];
        distance(goal, *p) > old + 0.01 && distance(closest_on_segment, *p) >= old - 0.01
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encounters_are_optional_and_repeated_goblins_remain_rare_in_endless() {
        let mut successes = 0;
        for seed in 0..1000 {
            let mut d = Director::new(seed);
            assert!(!d.due(10000.));
            if d.encounter(10.) {
                successes += 1;
                assert!(!d.due(29.));
                assert!(d.due(50.));
                d.reserve(50.);
                assert!(!d.due(10000.));
            }
        }
        assert!((80..160).contains(&successes));
        let mut d = Director::new(7);
        assert_eq!(d.chance(), 12);
        d.spawned(1, 0.);
        d.resolve(25., true);
        assert_eq!(d.chance(), 6);
        d.spawned(2, 30.);
        d.resolve(55., true);
        assert_eq!(d.chance(), 3);
        d.spawned(3, 60.);
        d.resolve(85., true);
        assert_eq!(d.chance(), 3);
        let mut misses = 0;
        for i in 0..1000 {
            if !d.encounter(100. + i as f32) {
                misses += 1;
            }
            d.reserve(100. + i as f32);
        }
        assert!(misses > 900);
        assert_eq!(d.chance(), 3);
    }
    #[test]
    fn wall_routes_prefer_a_clear_side_over_a_short_forward_dead_end() {
        let mut nav = FleeNavigation::default();
        let o = [0.; 3];
        let pursuers = [[-5., 0., 0.]];
        let goals = [[2., 0., 0.], [0., 18., 0.], [0., -18., 0.]];
        assert_eq!(nav.choose(o, &pursuers, [1., 0.], &goals), Some(goals[1]));
        assert!(escapes(o, goals[1], &pursuers));
        // An apparently clear forward ray is retried sideways after no displacement.
        nav.observe(1., o, Some([18., 0., 0.]));
        nav.observe(2., o, Some([18., 0., 0.]));
        assert_eq!(
            nav.choose(o, &pursuers, [1., 0.], &[[18., 0., 0.], goals[1]]),
            Some(goals[1])
        );
        // Keep the chosen side instead of alternating as ray order changes.
        assert_eq!(
            nav.choose(o, &pursuers, [0., 1.], &[goals[2], goals[1]]),
            Some(goals[1])
        );
        // Real progress and expiry allow a newly unobstructed direct escape again.
        nav.observe(5., [0., 10., 0.], Some([18., 10., 0.]));
        assert_eq!(
            nav.choose(o, &pursuers, [1., 0.], &[[18., 0., 0.], goals[1]]),
            Some([18., 0., 0.])
        );
    }
    #[test]
    fn lateral_escape_stays_in_arena_and_rejects_a_player_blocking_the_side() {
        let arena = Arena {
            center: [0., 0.],
            radius: 120.,
        };
        let o = [117., 0., 0.];
        let goals = flee_candidates(o, &[[110., 0., 0.]], [1., 0.])
            .into_iter()
            .filter_map(|g| arena.clip(o, g))
            .collect::<Vec<_>>();
        let goal = FleeNavigation::default()
            .choose(o, &[[110., 0., 0.]], [1., 0.], &goals)
            .unwrap();
        assert!(arena.contains(goal));
        assert!(goal[1].abs() > 1.);
        assert!(!escapes(
            [0.; 3],
            [0., 18., 0.],
            &[[-5., 0., 0.], [0., 10., 0.]]
        ));
    }
    #[test]
    fn arena_clips_outbound_segments_and_never_teleports_an_outside_creature() {
        let arena = Arena {
            center: [10., 20.],
            radius: 120.,
        };
        let origin = [120., 20., 3.];
        let goal = [150., 20., 6.];
        let end = arena.clip(origin, goal).unwrap();
        assert!(arena.contains(end));
        assert!(end[0] > origin[0] && end[0] < 128.);
        assert!(escapes(origin, end, &[[100., 20., 3.]]));
        assert_eq!(arena.clip([140., 20., 3.], origin), None);
        assert_eq!(arena.clip(origin, [f32::NAN, 0., 0.]), None);
        assert!(!arena.contains([129., 20., 3.]));
    }
    #[test]
    fn outcome_is_unique_and_native_death_wins_at_expiry() {
        let mut d = Director::new(5);
        d.next = Some(3.);
        assert!(d.due(3.));
        d.reserve(3.);
        assert!(d.spawned(10, 3.));
        assert!(!d.spawned(11, 4.));
        assert_eq!(d.resolve(27., true), None);
        assert_eq!(d.resolve(28., false), Some(Resolution::Killed { id: 10 }));
        assert_eq!(d.resolve(29., false), None);
        assert!(!d.due(30.));
    }
    #[test]
    fn escape_has_no_reward_and_simulation_time_controls_expiry() {
        let mut d = Director::new(1);
        d.spawned(2, 10.);
        for _ in 0..100 {
            assert_eq!(d.resolve(10., true), None);
        }
        assert_eq!(d.resolve(35., true), Some(Resolution::Escaped { id: 2 }));
        assert_eq!(d.resolve(36., false), None);
    }
    #[test]
    fn clipped_or_obsolete_goals_never_turn_toward_a_pursuer() {
        assert!(escapes([0.; 3], [18., 0., 0.], &[[-2., 0., 0.]]));
        assert!(!escapes([0.; 3], [-18., 0., 0.], &[[-2., 0., 0.]]));
        assert!(!escapes([0.; 3], [18., 0., 0.], &[[20., 0., 0.]]));
        assert!(!escapes(
            [0.; 3],
            [18., 0., 0.],
            &[[-2., 0., 0.], [19., 0., 0.]]
        ));
        let far = flee_candidates([100., 0., 0.], &[[0.; 3]], [-1., 0.]);
        assert!(far[0][0] > 100.);
        let mut d = Director::new(1);
        d.spawned(9, 0.);
        d.abandon();
        assert_eq!(d.resolve(50., false), None);
    }
    #[test]
    fn flee_responds_to_the_group_and_rejects_invalid_positions() {
        let goals = flee_candidates([0.; 3], &[[-2., 0., 0.], [0., -2., 0.]], [1., 0.]);
        assert!(goals[0][0] > 0. && goals[0][1] > 0.);
        assert!(goals.iter().all(|p| p.iter().all(|v| v.is_finite())));
        assert!(flee_candidates([f32::NAN, 0., 0.], &[], [0.; 2]).is_empty());
    }
}

#[cfg(test)]
mod mark_reward_tests {
    use super::whole_boss_marks;
    #[test]
    fn native_boss_amount_has_no_minimum_or_fractional_goblin_credit() {
        for (native, expected) in [(0.,0), (0.5,0), (1.,1), (1.25,1), (1.5,1), (2.,2), (7.5,7)] {
            assert_eq!(whole_boss_marks(native), Some(expected));
        }
        for invalid in [-1., f32::NAN, f32::INFINITY, 10_000_001.] {
            assert_eq!(whole_boss_marks(invalid), None);
        }
    }
}
