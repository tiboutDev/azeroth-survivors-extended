//! Diagnostic-only aggregate of actual native movement, without per-frame logging.
use super::super::event_services::Services;
use std::{collections::BTreeMap, sync::Mutex};
#[derive(Default)]
struct Track {
    previous: Option<(f32, [f32; 3], Vec<[f32; 3]>)>,
    samples: usize,
    min_projection: f32,
    max_gap: f32,
}
impl Track {
    fn observe(&mut self, now: f32, position: [f32; 3], pursuers: Vec<[f32; 3]>) {
        if let Some((clock, before, threats)) = &self.previous {
            let gap = now - clock;
            if gap > 0. {
                self.max_gap = self.max_gap.max(gap);
                if let Some(nearest) = threats.iter().min_by(|a, b| {
                    let distance = |p: &[f32; 3]| (before[0] - p[0]).hypot(before[1] - p[1]);
                    distance(a).total_cmp(&distance(b))
                }) {
                    let away = [before[0] - nearest[0], before[1] - nearest[1]];
                    let length = away[0].hypot(away[1]);
                    if length > 0.1 {
                        let projection = ((position[0] - before[0]) * away[0]
                            + (position[1] - before[1]) * away[1])
                            / length;
                        self.min_projection = self.min_projection.min(projection);
                        self.samples += 1;
                    }
                }
            }
        }
        self.previous = Some((now, position, pursuers));
    }
}
static TRACKS: Mutex<BTreeMap<u64, Track>> = Mutex::new(BTreeMap::new());
pub(super) fn observe(
    guid: u64,
    now: f32,
    position: [f32; 3],
    party: &asx_game::official::PartyObservation,
) {
    let threats = party
        .actors
        .iter()
        .filter(|a| a.hp > 0. && a.phase != asx_gameplay::Phase::Inactive)
        .map(|a| a.position)
        .collect();
    if let Ok(mut tracks) = TRACKS.lock() {
        if tracks.len() < 4 || tracks.contains_key(&guid) {
            tracks
                .entry(guid)
                .or_default()
                .observe(now, position, threats);
        }
    }
}
pub(super) fn report(r: &dyn Services, guid: u64) {
    let track = TRACKS
        .lock()
        .ok()
        .and_then(|mut tracks| tracks.remove(&guid));
    if let Some(t) = track {
        r.events_info(&format!(
            "treasure goblin fine motion: guid={guid} samples={} min_projection={} max_gap={}",
            t.samples, t.min_projection, t.max_gap
        ));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn outward_curve_and_actual_reversal_are_distinguished() {
        let mut arc = Track::default();
        for n in 0..=200 {
            let t = n as f32 / 200.;
            let angle = t * 2.;
            let radius = 8. + t * 4.;
            arc.observe(
                t * 2.,
                [radius * angle.cos(), radius * angle.sin(), 0.],
                vec![[0.; 3]],
            );
        }
        assert_eq!(arc.samples, 200);
        assert!(arc.min_projection >= -0.01);
        // Its sparse chord nevertheless has a negative initial away projection.
        assert!(12. * 2_f32.cos() - 8. < 0.);
        arc.observe(2.01, [0.; 3], vec![[0.; 3]]);
        assert!(arc.min_projection < -1.);
    }
}

pub(super) fn ready(guid:u64)->bool{TRACKS.lock().ok().is_some_and(|tracks|tracks.get(&guid).is_some_and(|t|t.samples>120))}
