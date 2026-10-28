//! Opt-in development timing; absent from normal builds. Never logs per call.
use super::*;
use std::time::Instant;
const NAMES: [&str; 7] = ["file_filter", "party_wire", "spell_capture", "ui_hud", "ui_event", "hud_capture", "server_observer"];
static COUNTS: [AtomicU64; 7] = [const { AtomicU64::new(0) }; 7];
static NANOS: [AtomicU64; 7] = [const { AtomicU64::new(0) }; 7];
static MAX: [AtomicU64; 7] = [const { AtomicU64::new(0) }; 7];
pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("AZCOOP_TEST_STARTUP_PROFILE").as_deref() == Ok("1"))
}
pub struct Timer(usize, Option<Instant>);
pub fn start(index: usize) -> Timer { Timer(index, enabled().then(Instant::now)) }
impl Drop for Timer {
    fn drop(&mut self) {
        if let Some(start) = self.1 {
            let ns = start.elapsed().as_nanos().min(u64::MAX as u128) as u64;
            COUNTS[self.0].fetch_add(1, Ordering::Relaxed);
            NANOS[self.0].fetch_add(ns, Ordering::Relaxed);
            MAX[self.0].fetch_max(ns, Ordering::Relaxed);
        }
    }
}
pub fn report(log: &Log) {
    if !enabled() { return; }
    let rows: Vec<_> = NAMES.iter().enumerate().map(|(i, name)| serde_json::json!({
        "name": name, "calls": COUNTS[i].load(Ordering::Relaxed),
        "ns": NANOS[i].load(Ordering::Relaxed), "max_ns": MAX[i].load(Ordering::Relaxed)
    })).collect();
    log.info(&format!("ASX startup profile: {}", serde_json::json!(rows)));
}
