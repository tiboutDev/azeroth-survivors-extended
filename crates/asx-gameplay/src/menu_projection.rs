//! Cosmetic cache key extension; no native layout or gameplay authority.
#[derive(Default)]
pub struct MenuProjection {
    wanted: Vec<(u64, u8)>,
    rendered: Vec<(u64, u8)>,
    attempted: Option<Vec<(u64, u8)>>,
    caption_refreshes: u8,
}
impl MenuProjection {
    pub fn update(&mut self, members: Vec<(u64, u8)>) {
        if self.wanted != members {
            self.wanted = members;
            self.attempted = None;
        }
    }
    /// At most one native cache miss per changed projection, including failure.
    pub fn request(&mut self) -> bool {
        if self.wanted == self.rendered || self.attempted.as_ref() == Some(&self.wanted) {
            return false;
        }
        self.attempted = Some(self.wanted.clone());
        true
    }
    /// Two bounded status passes cover native deferred command application.
    pub fn refresh_caption(&mut self) -> bool {
        if self.caption_refreshes == 0 {
            return false;
        }
        self.caption_refreshes -= 1;
        true
    }
    pub fn rendered(&mut self, members: Vec<(u64, u8)>) {
        self.rendered = members;
        self.caption_refreshes = 2;
        if self.rendered == self.wanted {
            self.attempted = None;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn joins_classes_and_leave_invalidate_once_not_every_frame() {
        let mut s = MenuProjection::default();
        assert!(!s.request());
        for rows in [vec![(1, 1), (2, 2)], vec![(1, 1), (2, 11)], vec![]] {
            s.update(rows.clone());
            assert!(s.request());
            for _ in 0..100 {
                assert!(!s.request());
            }
            s.rendered(rows);
            assert!(!s.request());
            assert!(s.refresh_caption());
            assert!(s.refresh_caption());
            assert!(!s.refresh_caption());
        }
    }
    #[test]
    fn stale_or_failed_native_render_cannot_loop_or_ack_a_newer_roster() {
        let mut s = MenuProjection::default();
        s.update(vec![(1, 1)]);
        assert!(s.request());
        assert!(!s.request());
        s.update(vec![(1, 1), (2, 2)]);
        s.rendered(vec![(1, 1)]);
        assert!(s.request());
        assert!(!s.request());
        s.rendered(vec![(1, 1), (2, 2)]);
        assert!(!s.request());
    }
    #[test]
    fn multiple_updates_coalesce_and_ordinary_native_rebuilds_ack_them() {
        let mut s = MenuProjection::default();
        s.update(vec![(1, 1)]);
        s.update(vec![(1, 1), (2, 2)]);
        s.rendered(vec![(1, 1), (2, 2)]);
        assert!(!s.request());
    }
}
