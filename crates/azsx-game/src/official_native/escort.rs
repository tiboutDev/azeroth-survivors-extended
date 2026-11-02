use super::Adapter;
use crate::{current as v, memory::Mem, official::{Entry, MutableMemory}, Result};

type SceneSpline = unsafe extern "system" fn(*mut u64, u64, *const f32, *const f32, u32, u32);
/// All six arguments are used by the native builder, including both stack slots.
unsafe fn invoke_scene_spline(
    call: SceneSpline,
    out: *mut u64,
    guid: u64,
    from: [f32; 3],
    to: [f32; 3],
    sequence: u32,
    millis: u32,
) {
    call(out, guid, from.as_ptr(), to.as_ptr(), sequence, millis);
}
impl Adapter {
    pub fn solo_motion_entries(self) -> Vec<Entry> {
        [v::SOLO_SCENE_SPLINE, v::GOBLIN_CLIP]
            .into_iter()
            .map(|(rva, bytes)| Entry { rva, bytes })
            .collect()
    }
    /// Choose a finite short corridor, not a general pathfinding engine. Every
    /// 1.5-yard segment and the chicken's footprint must pass native clipping
    /// and terrain sampling. Blocked candidates are discarded, never crossed.
    pub unsafe fn escort_route(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
        start: [f32; 3],
        draw: u32,
        segments: usize,
    ) -> Result<Option<Vec<[f32; 3]>>> {
        if !(8..=azsx_gameplay::solo_events::ESCORT_SEGMENTS).contains(&segments)
            || self.event_party(m, game)?.is_none()
        {
            return Ok(None);
        }
        let mut route = vec![start];
        let mut point = start;
        let angle = (draw % 360) as f32 * std::f32::consts::PI / 180.;
        for i in 0..segments {
            // Up to six broad legs form an open curve, keeping the route within
            // an arena-sized footprint without doubling back onto extraction.
            let direction = angle + (i / 20) as f32 * 0.65;
            let xy = [
                point[0] + azsx_gameplay::solo_events::ESCORT_SEGMENT_LENGTH * direction.cos(),
                point[1] + azsx_gameplay::solo_events::ESCORT_SEGMENT_LENGTH * direction.sin(),
            ];
            let Some(site) = self.event_ground_sample(m, base, game, xy, 0.45)? else {
                return Ok(None);
            };
            if site.high - site.low > 1.5 {
                return Ok(None);
            }
            let end = site.center;
            if (end[2] - point[2]).abs() > 1.5 {
                return Ok(None);
            }
            let clipped = self.event_surface(m, base, game, point, end)?;
            if (clipped[0] - end[0]).hypot(clipped[1] - end[1]) > 0.05 {
                return Ok(None);
            }
            // Quarter-yard footprint samples also cover narrow collision cells
            // between endpoints. Height is sampled from native terrain only.
            for step in 1..6 {
                let t = step as f32 / 6.;
                let sample = [
                    point[0] + (end[0] - point[0]) * t,
                    point[1] + (end[1] - point[1]) * t,
                ];
                if self
                    .event_ground_sample(m, base, game, sample, 0.45)?
                    .is_none()
                {
                    return Ok(None);
                }
            }
            point = end;
            route.push(end);
        }
        // The native teleporter needs the same five-yard clearance as the
        // starting chicken scene, including on narrow maps.
        if self.event_ground_sample(m, base, game, [point[0], point[1]], 5.)?.is_none() {
            return Ok(None);
        }
        Ok(Some(route))
    }
    /// An invulnerable presentation unit deliberately outside enemy/pet combat
    /// arrays. It cannot be attacked and cannot consume native summon capacity.
    pub unsafe fn escort_move_chicken(
        self,
        m: &dyn MutableMemory,
        base: u64,
        game: u64,
        object: super::EventObject,
        to: [f32; 3],
        millis: u32,
    ) -> Result<super::EventObject> {
        if self.event_party(m, game)?.is_none()
            || ((object.guid() >> 24) & 0xffffff) != u64::from(v::SOLO_CHICKEN_ENTRY)
            || !valid_scene_step(object.position, to, millis)
        {
            return Err("Invalid owned chicken movement".into());
        }
        let clipped = self.event_surface(m, base, game, object.position, to)?;
        if (clipped[0] - to[0]).hypot(clipped[1] - to[1]) > 0.05 {
            return Err("Chicken segment became blocked".into());
        }
        let at = game + v::SOLO_SCENE_SEQUENCE;
        let sequence = u32::from_le_bytes(
            m.read(at, 4)?
                .try_into()
                .map_err(|_| "Truncated scene movement sequence")?,
        )
        .checked_add(1)
        .ok_or("Scene movement sequence exhausted")?;
        m.write(at, &sequence.to_le_bytes())?;
        let call: SceneSpline = std::mem::transmute((base + v::SOLO_SCENE_SPLINE.0) as usize);
        let mut packet = [0u64; 4];
        invoke_scene_spline(
            call,
            packet.as_mut_ptr(),
            object.guid(),
            object.position,
            to,
            sequence,
            millis,
        );
        self.send_owned_scene_packet(m, base, game, 0xdd, packet)?;
        Ok(super::EventObject {
            position: to,
            ..object
        })
    }
}
fn valid_scene_step(from: [f32; 3], to: [f32; 3], millis: u32) -> bool {
    from.iter().chain(to.iter()).all(|v| v.is_finite())
        && (1..=500).contains(&millis)
        && (from[0] - to[0]).hypot(from[1] - to[1]) <= 2.
        && (from[2] - to[2]).abs() <= 1.5
}
#[cfg(test)]
mod chicken_motion_tests {
    use super::*;
    unsafe extern "system" fn native(
        out: *mut u64,
        guid: u64,
        from: *const f32,
        to: *const f32,
        sequence: u32,
        millis: u32,
    ) {
        assert_eq!(guid, 0xf130a50030000001);
        assert_eq!(std::slice::from_raw_parts(from, 3), &[1., 2., 3.]);
        assert_eq!(std::slice::from_raw_parts(to, 3), &[2., 3., 4.]);
        assert_eq!((sequence, millis), (7654321, 250));
        *out = 0xabcdef;
    }
    #[test]
    fn spline_keeps_guid_borrowed_coordinates_and_both_stack_arguments() {
        let mut out = 0;
        unsafe {
            invoke_scene_spline(
                native,
                &mut out,
                0xf130a50030000001,
                [1., 2., 3.],
                [2., 3., 4.],
                7654321,
                250,
            )
        }
        assert_eq!(out, 0xabcdef);
    }
    #[test]
    fn rejects_nonfinite_teleports_unbounded_time_and_steep_steps() {
        assert!(valid_scene_step([0.; 3], [0.5, 0., 0.2], 250));
        assert!(valid_scene_step([0.; 3], [0.; 3], 1));
        for (to, ms) in [
            ([3., 0., 0.], 250),
            ([0., 0., 2.], 250),
            ([f32::NAN, 0., 0.], 250),
            ([0.; 3], 0),
            ([0.; 3], 501),
        ] {
            assert!(!valid_scene_step([0.; 3], to, ms));
        }
    }
}


fn ordinary_gadget_target(row: &[u8]) -> bool {
    row.len() == v::ENEMY_STRIDE
        && row[v::ENEMY_DEAD] == 0
        && row[v::SOLO_ENEMY_ELITE] == 0
        && row[v::SOLO_ENEMY_ENCOUNTER..v::SOLO_ENEMY_ENCOUNTER + 8] == u64::MAX.to_le_bytes()
        && f32::from_le_bytes(row[v::ENEMY_HP..v::ENEMY_HP + 4].try_into().unwrap()) > 0.
}
type NativeGadgetEffect = unsafe extern "system" fn(u64, u64, *const u8, f32, u32);
unsafe fn invoke_gadget_effect(
    call: NativeGadgetEffect,
    game: u64,
    target: u64,
    tag: u32,
    value: f32,
    spell: u32,
) {
    let mut effect = [0u8; 12];
    effect[..4].copy_from_slice(&tag.to_le_bytes());
    effect[4..8].copy_from_slice(&value.to_le_bytes());
    call(game, target, effect.as_ptr(), 0., spell);
}
impl Adapter {
    // This validates an existing native delivery, including the remainder of a
    // synchronous batch when native choice/pause opens during a callback. The
    // runtime independently requires Playing before starting any new activation.
    /// Read only the currently selected native actor. Cooperative callers swap
    /// into a verified recipient context and restore the host after the effect.
    fn solo_gadget_owner(
        self, m: &dyn Mem, game: u64, owner: u64,
    ) -> Result<super::PartyObservation> {
        let (slot, guid) = self.run_source(m, game)?
            .ok_or("Engineering actor context unavailable")?;
        if guid != owner { return Err("Engineering actor identity changed".into()); }
        let root = self.read_game(m, game)?;
        let actor = Self::actor(slot, &root, root[0xc46])?;
        if actor.guid != owner || actor.hp <= 0. || !actor.phase.can_tank() {
            return Err("Engineering gadget owner is inactive".into());
        }
        let clock = f32::from_le_bytes(root[0xb9c..0xba0].try_into().unwrap());
        if !clock.is_finite() || !(0. ..=1e7).contains(&clock) {
            return Err("Invalid engineering clock".into());
        }
        Ok(super::PartyObservation { clock, actors: vec![actor] })
    }
    /// Resolve a recipient row only from a restored host snapshot. The returned
    /// address is used immediately and never cached across native callbacks.
    pub fn engineering_actor_row(
        self, m: &dyn Mem, game: u64, slot: u32, owner: u64,
    ) -> Result<u64> {
        let party = self.event_party(m, game)?.ok_or("Engineering run retired")?;
        if self.run_source(m, game)? != party.actors.iter().find(|a| a.slot == 0)
            .map(|a| (0, a.guid)) {
            return Err("Engineering requires restored host context".into());
        }
        if !party.actors.iter().any(|a| a.slot == slot && a.guid == owner
            && a.hp > 0. && a.phase == azsx_gameplay::Phase::Playing) {
            return Err("Engineering recipient changed".into());
        }
        let row = self.support_row(m, game, slot)?;
        if m.read(row + self.row_field(row == game, 0x478), 8)? != owner.to_le_bytes() {
            return Err("Engineering row identity changed".into());
        }
        Ok(row)
    }
    /// Execute one recipient effect through the native actor switch and always
    /// restore the host before returning, including after a rejected effect.
    pub fn engineering_with_actor<T>(
        self, io: &dyn crate::official::SupportCalls, game: u64,
        slot: u32, owner: u64, action: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        let row = self.engineering_actor_row(io, game, slot, owner)?;
        if slot == 0 { return action(); }
        let host = self.event_party(io, game)?.and_then(|p| p.actors.into_iter()
            .find(|a| a.slot == 0)).ok_or("Engineering host retired")?;
        io.swap_actor(game, row)?;
        if let Err(e) = io.write(game + self.game_field(0xb40), &u64::from(slot).to_le_bytes()) {
            let host_row = self.support_row(io, game, 0)?;
            let _ = io.swap_actor(game, host_row);
            return Err(e);
        }
        let result = action();
        let flush = io.flush_actor(game);
        let host_row = self.support_row(io, game, 0)?;
        if io.read(host_row + self.row_field(false, 0x478), 8)? != host.guid.to_le_bytes() {
            return Err("Engineering host row identity changed".into());
        }
        io.write(game + self.game_field(0xb40), &0u64.to_le_bytes())?;
        io.swap_actor(game, host_row)?;
        result.and_then(|value| flush.map(|_| value))
    }
    /// Native Entangling Roots effect. Native diminishing returns, aura transport
    /// and motion cancellation remain authoritative; no synthetic root timer.
    pub unsafe fn solo_gadget_net(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
        owner: u64,
    ) -> Result<usize> {
        let p = self.solo_gadget_owner(m, game, owner)?;
        let origin = p.actors[0].position;
        let enemies = self.support_enemies(m, game)?;
        let (ptr, n) = self.goblin_array(m, game)?;
        let rows = m.read(ptr, n * v::ENEMY_STRIDE)?;
        let mut targets = enemies
            .into_iter()
            .filter(|e| {
                (e.position[0] - f64::from(origin[0])).hypot(e.position[1] - f64::from(origin[1]))
                    <= 8.
                    && rows.chunks_exact(v::ENEMY_STRIDE).any(|row| {
                        u64::from_le_bytes(
                            row[v::ENEMY_GUID..v::ENEMY_GUID + 8].try_into().unwrap(),
                        ) == e.guid
                            && ordinary_gadget_target(row)
                    })
            })
            .collect::<Vec<_>>();
        targets.sort_by(|a, b| {
            let d = |e: &crate::enemies::EnemyExact| {
                (e.position[0] - f64::from(origin[0])).hypot(e.position[1] - f64::from(origin[1]))
            };
            d(a).total_cmp(&d(b)).then(a.guid.cmp(&b.guid))
        });
        let call: NativeGadgetEffect =
            std::mem::transmute((base + self.native_hit_effect_entry().rva) as usize);
        let mut rooted = 0;
        for target in targets.into_iter().take(3) {
            // Re-resolve identity and classification after every reentrant call.
            self.solo_gadget_owner(m, game, owner)?;
            let (ptr, n) = self.goblin_array(m, game)?;
            let rows = m.read(ptr, n * v::ENEMY_STRIDE)?;
            if !rows.chunks_exact(v::ENEMY_STRIDE).any(|row| {
                u64::from_le_bytes(row[v::ENEMY_GUID..v::ENEMY_GUID + 8].try_into().unwrap())
                    == target.guid
                    && ordinary_gadget_target(row)
            }) {
                continue;
            }
            invoke_gadget_effect(call, game, target.guid, 2, 2., 339);
            let (ptr, n) = self.goblin_array(m, game)?;
            let rows = m.read(ptr, n * v::ENEMY_STRIDE)?;
            if rows.chunks_exact(v::ENEMY_STRIDE).any(|row| {
                u64::from_le_bytes(row[v::ENEMY_GUID..v::ENEMY_GUID + 8].try_into().unwrap())
                    == target.guid
                    && f32::from_le_bytes(
                        row[v::SOLO_ENEMY_ROOT_UNTIL..v::SOLO_ENEMY_ROOT_UNTIL + 4]
                            .try_into()
                            .unwrap(),
                    ) > p.clock
            }) {
                rooted += 1;
            }
        }
        Ok(rooted)
    }
    /// Applied once after each native stat reconstruction, never incremented in
    /// frame updates. The native movement implementation uses the derived value.
    pub fn solo_gadget_speed(
        self,
        m: &dyn MutableMemory,
        game: u64,
        owner: u64,
        value: f32,
    ) -> Result<()> {
        if !value.is_finite() || !(0.0..=0.55).contains(&value) {
            return Err("Invalid Rocket Boots speed".into());
        }
        self.solo_gadget_owner(m, game, owner)?;
        let at = game + self.field117(v::BLESSING_STAT_FIELDS[4]);
        let base = f32::from_le_bytes(
            m.read(at, 4)?
                .try_into()
                .map_err(|_| "Truncated native movement speed")?,
        );
        if !base.is_finite() || base < 0. {
            return Err("Invalid native movement speed".into());
        }
        m.write(at, &(base + value).to_le_bytes())
    }
}
#[cfg(test)]
mod engineering_tests {
    use super::*;
    #[test]
    fn net_excludes_dead_elite_guardian_and_boss_rows() {
        let mut row = vec![0; v::ENEMY_STRIDE];
        row[v::SOLO_ENEMY_ENCOUNTER..v::SOLO_ENEMY_ENCOUNTER + 8]
            .copy_from_slice(&u64::MAX.to_le_bytes());
        row[v::ENEMY_HP..v::ENEMY_HP + 4].copy_from_slice(&20f32.to_le_bytes());
        assert!(ordinary_gadget_target(&row));
        row[v::SOLO_ENEMY_ELITE] = 1;
        assert!(!ordinary_gadget_target(&row));
        row[v::SOLO_ENEMY_ELITE] = 0;
        row[v::SOLO_ENEMY_ENCOUNTER] = 0;
        assert!(!ordinary_gadget_target(&row));
        row[v::SOLO_ENEMY_ENCOUNTER] = 255;
        row[v::ENEMY_DEAD] = 1;
        assert!(!ordinary_gadget_target(&row));
    }
    unsafe extern "system" fn effect(
        game: u64,
        target: u64,
        bytes: *const u8,
        damage: f32,
        spell: u32,
    ) {
        assert_eq!((game, target, damage, spell), (100, 200, 0., 339));
        let b = std::slice::from_raw_parts(bytes, 12);
        assert_eq!(b, [2, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0]);
    }
    #[test]
    fn engineering_effect_preserves_five_argument_native_abi() {
        unsafe { invoke_gadget_effect(effect, 100, 200, 2, 2., 339) }
    }
}


fn native_extent(m: &dyn Mem, at: u64, stride: usize) -> Result<(u64, usize)> {
    let bytes = m.read(at, 24)?;
    if bytes.len() != 24 {
        return Err("Truncated native ally extent".into());
    }
    let q = |at| u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap());
    let (cap, ptr, n) = (q(0), q(8), q(16));
    if n > cap
        || cap > 1024
        || (n > 0 && (ptr < 0x10000 || ptr.checked_add(n * stride as u64).is_none()))
    {
        return Err("Invalid native ally extent".into());
    }
    Ok((ptr, n as usize))
}

#[derive(Clone, Copy, Debug)]
pub struct SoloMortarLaunch {
    pub target: u64,
    pub point: [f32; 3],
    pub arrives: f32,
}
type NativeProjectile =
    unsafe extern "system" fn(u64, u32, u32, *const u8, u64, f32, f32, *const u8);
unsafe fn invoke_mortar(call: NativeProjectile, game: u64, target: u64, token: u64, delay: f32) {
    // Native effect variant 8 is a no-op. Its otherwise unused payload identifies
    // one owned launch at the existing native hit-effect callback. Zero direct
    // damage prevents double-hitting the selected target before the AoE impact.
    let mut effect = [0u8; 12];
    effect[..4].copy_from_slice(&8u32.to_le_bytes());
    effect[4..12].copy_from_slice(&token.to_le_bytes());
    let source = [0u8; 16];
    // Argument 7 is an additional delay, not projectile speed. Argument 8 is
    // the native optional-position descriptor (tag 0 = native actor origin).
    call(
        game,
        v::SOLO_MORTAR_SPELL,
        2,
        effect.as_ptr(),
        target,
        0.,
        delay,
        source.as_ptr(),
    )
}
impl Adapter {
    pub fn solo_engineering_entries(self) -> Vec<Entry> { [v::SOLO_PROJECTILE_CREATE, v::SOLO_PROJECTILE_DELAY_READ, v::SOLO_PROJECTILE_ORIGIN_READ].into_iter().map(|(rva,bytes)| Entry{rva,bytes}).collect() }
    pub fn solo_mortar_token(self, m: &dyn Mem, spell: u32, effect: u64) -> Result<Option<u64>> {
        if spell != v::SOLO_MORTAR_SPELL {
            return Ok(None);
        }
        let b = m.read(effect, 12)?;
        if b.len() != 12 {
            return Err("Truncated native engineering impact".into());
        }
        if b[..4] != 8u32.to_le_bytes() {
            return Ok(None);
        }
        let token = u64::from_le_bytes(b[4..12].try_into().unwrap());
        Ok(
            ((token & 0xffffffff00000000) == v::SOLO_MORTAR_TOKEN && (token as u32) != 0)
                .then_some(token),
        )
    }
    pub fn solo_new_mortar_token(self, serial: u32) -> Option<u64> {
        (serial != 0).then_some(v::SOLO_MORTAR_TOKEN | u64::from(serial))
    }
    pub unsafe fn solo_launch_mortar(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
        owner: u64,
        draw: u32,
        token: u64,
    ) -> Result<Option<SoloMortarLaunch>> {
        if token & 0xffffffff00000000 != v::SOLO_MORTAR_TOKEN || token as u32 == 0 {
            return Err("Invalid engineering launch identity".into());
        }
        let p = self.solo_gadget_owner(m, game, owner)?;
        let origin = p.actors[0].position;
        let (ptr,n)=self.goblin_array(m,game)?;
        let rows=m.read(ptr,n*v::ENEMY_STRIDE)?;
        let mut targets = self
            .support_enemies(m, game)?
            .into_iter()
            .filter(|e| {
                e.hp > 0.
                    && (e.position[0] - f64::from(origin[0]))
                        .hypot(e.position[1] - f64::from(origin[1]))
                        <= 30.
                    && rows.chunks_exact(v::ENEMY_STRIDE).any(|row|
                        u64::from_le_bytes(row[v::ENEMY_GUID..v::ENEMY_GUID+8].try_into().unwrap())==e.guid
                        && ordinary_gadget_target(row))
            })
            .collect::<Vec<_>>();
        targets.sort_by_key(|e| e.guid);
        if targets.is_empty() {
            return Ok(None);
        }
        let target = &targets[draw as usize % targets.len()];
        let point = target.position.map(|v| v as f32);
        if self
            .event_ground_sample(m, base, game, [point[0], point[1]], 0.5)?
            .is_none()
        {
            return Ok(None);
        }
        let (_, before) = native_extent(
            m,
            game + v::SOLO_PROJECTILE_VECTOR,
            v::SOLO_PROJECTILE_STRIDE,
        )?;
        let call: NativeProjectile =
            std::mem::transmute((base + v::SOLO_PROJECTILE_CREATE.0) as usize);
        let flight =
            mortar_flight_delay(origin, point).ok_or("Invalid native Mortar flight distance")?;
        invoke_mortar(call, game, target.guid, token, flight);
        self.solo_gadget_owner(m, game, owner)?;
        let (ptr, n) = native_extent(
            m,
            game + v::SOLO_PROJECTILE_VECTOR,
            v::SOLO_PROJECTILE_STRIDE,
        )?;
        if n != before + 1 {
            return Err("Uncertain native Mortar launch; not retried".into());
        }
        let row = m.read(
            ptr + before as u64 * v::SOLO_PROJECTILE_STRIDE as u64,
            v::SOLO_PROJECTILE_STRIDE,
        )?;
        if row[112..116] != 8u32.to_le_bytes()
            || row[116..124] != token.to_le_bytes()
            || row[128..132] != v::SOLO_MORTAR_SPELL.to_le_bytes()
        {
            return Err("Native Mortar receipt changed".into());
        }
        let arrives = f32::from_le_bytes(row[124..128].try_into().unwrap());
        if !arrives.is_finite() || arrives < p.clock || arrives > p.clock + 10. {
            return Err("Invalid native Mortar flight deadline".into());
        }
        Ok(Some(SoloMortarLaunch {
            target: target.guid,
            point,
            arrives,
        }))
    }
    /// Bounded native control deliveries. The caller invokes the grenade only after its
    /// identified original native projectile impact, never on launch or a timer.
    pub unsafe fn solo_grenade_stun(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
        owner: u64,
        point: [f32; 3],
    ) -> Result<usize> {
        if point.iter().any(|v| !v.is_finite()) {
            return Err("Invalid engineering explosion".into());
        }
        self.solo_gadget_owner(m, game, owner)?;
        let radius = 3.;
        let (ptr,n)=self.goblin_array(m,game)?;
        let rows=m.read(ptr,n*v::ENEMY_STRIDE)?;
        let mut targets = self
            .support_enemies(m, game)?
            .into_iter()
            .filter(|e| {
                e.hp > 0.
                    && (e.position[0] - f64::from(point[0]))
                        .hypot(e.position[1] - f64::from(point[1]))
                        <= radius
                    && rows.chunks_exact(v::ENEMY_STRIDE).any(|row|
                        u64::from_le_bytes(row[v::ENEMY_GUID..v::ENEMY_GUID+8].try_into().unwrap())==e.guid
                        && ordinary_gadget_target(row))
            })
            .collect::<Vec<_>>();
        targets.sort_by(|a,b| {
            let distance = |e: &crate::enemies::EnemyExact| (e.position[0]-f64::from(point[0])).hypot(e.position[1]-f64::from(point[1]));
            distance(a).total_cmp(&distance(b)).then(a.guid.cmp(&b.guid))
        });
        let call: NativeGadgetEffect = std::mem::transmute((base + self.native_hit_effect_entry().rva) as usize);
        let mut hits = 0;
        for target in targets.into_iter().take(3) {
            self.solo_gadget_owner(m, game, owner)?;
            let current = self.support_enemies(m, game)?.into_iter().find(|e| {
                e.guid == target.guid
                    && e.hp > 0.
                    && (e.position[0] - f64::from(point[0]))
                        .hypot(e.position[1] - f64::from(point[1]))
                        <= radius
            });
            let Some(_current) = current else { continue };
            {
                let (ptr, n) = self.goblin_array(m, game)?;
                let rows = m.read(ptr, n * v::ENEMY_STRIDE)?;
                if !rows.chunks_exact(v::ENEMY_STRIDE).any(|row| {
                    u64::from_le_bytes(row[v::ENEMY_GUID..v::ENEMY_GUID + 8].try_into().unwrap())
                        == target.guid
                        && ordinary_gadget_target(row)
                }) {
                    continue;
                }
            }
            // Native control tags 2/3 share the exact reviewed crowd-control path.
            // Zero damage; native immunity and diminishing returns remain authoritative.
            invoke_gadget_effect(call, game, target.guid, 3, 2., v::SOLO_MORTAR_SPELL);
            let clock=self.solo_gadget_owner(m,game,owner)?.clock;
            let (ptr,n)=self.goblin_array(m,game)?;
            let rows=m.read(ptr,n*v::ENEMY_STRIDE)?;
            if rows.chunks_exact(v::ENEMY_STRIDE).any(|row|
                u64::from_le_bytes(row[v::ENEMY_GUID..v::ENEMY_GUID+8].try_into().unwrap())==target.guid
                && f32::from_le_bytes(row[v::SOLO_ENEMY_ROOT_UNTIL..v::SOLO_ENEMY_ROOT_UNTIL+4].try_into().unwrap())>clock) {hits+=1;}

        }
        Ok(hits)
    }

}
#[cfg(test)]
mod mortar_abi_tests {
    use super::*;
    unsafe extern "system" fn native(
        game: u64,
        spell: u32,
        school: u32,
        effect: *const u8,
        target: u64,
        damage: f32,
        delay: f32,
        source: *const u8,
    ) {
        assert_eq!(
            (game, spell, school, target, damage, delay),
            (0x123456789, 13237, 2, 0xf130000000000123, 0., 0.75)
        );
        assert_eq!(std::slice::from_raw_parts(source, 16), [0; 16]);
        let b = std::slice::from_raw_parts(effect, 12);
        assert_eq!(&b[..4], &8u32.to_le_bytes());
        assert_eq!(&b[4..], &0x415a535800000007u64.to_le_bytes());
    }
    #[test]
    fn native_mortar_preserves_borrowed_effect_guid_and_delay_and_eighth_borrowed_source_argument()
    {
        unsafe {
            invoke_mortar(
                native,
                0x123456789,
                0xf130000000000123,
                0x415a535800000007,
                0.75,
            )
        }
    }
}

/// Exact native values-update schema: one block, no movement block, and only
/// OBJECT_FIELD_SCALE_X (4). Gameplay radius/reach/position/stats are absent.
fn player_scale_packet(guid: u64, scale: f32) -> Vec<u8> {
    let mut packet = vec![1, 0, 0, 0, 0, 0];
    let bytes = guid.to_le_bytes();
    let mut mask = 0u8;
    for (i, byte) in bytes.iter().enumerate() {
        if *byte != 0 {
            mask |= 1 << i;
        }
    }
    packet.push(mask);
    for byte in bytes {
        if byte != 0 {
            packet.push(byte);
        }
    }
    packet.push(1);
    packet.extend_from_slice(&16u32.to_le_bytes());
    packet.extend_from_slice(&scale.to_le_bytes());
    packet
}
impl Adapter {
    pub fn solo_visual_entries(self) -> Vec<Entry> { Vec::new() }
    pub fn solo_player_scale_base(self) -> f32 {
        v::SOLO_PLAYER_SCALE_BASE
    }
    pub unsafe fn solo_player_scale(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
        owner: u64,
        scale: f32,
    ) -> Result<()> {
        if !scale.is_finite() || !(0.5..=1.5).contains(&scale) {
            return Err("Invalid cosmetic engineering scale".into());
        }
        let p = self.event_party(m, game)?.ok_or("Cosmetic engineering run retired")?;
        if !p.actors.iter().any(|a| a.guid == owner)
            || self.run_source(m, game)?.is_none_or(|(slot, _)| slot != 0)
        {
            return Err("Cosmetic engineering owner changed".into());
        }
        self.event_broadcast(m, base, game, 0xa9, &player_scale_packet(owner, scale))
    }
}
#[cfg(test)]
mod cosmetic_scale_tests {
    use super::*;
    #[test]
    fn scale_delta_updates_only_the_render_field_and_roundtrips_packed_guid() {
        for guid in [1, 0x1122334455667788, 0xf130000000000001] {
            let packet = player_scale_packet(guid, 0.7);
            assert_eq!(&packet[..6], &[1, 0, 0, 0, 0, 0]);
            let mask = packet[6];
            let mut bytes = [0u8; 8];
            let mut at = 7;
            for (i, b) in bytes.iter_mut().enumerate() {
                if mask & (1 << i) != 0 {
                    *b = packet[at];
                    at += 1;
                }
            }
            assert_eq!(u64::from_le_bytes(bytes), guid);
            assert_eq!(packet[at], 1);
            at += 1;
            assert_eq!(&packet[at..at + 4], &16u32.to_le_bytes());
            at += 4;
            assert_eq!(&packet[at..], &0.7f32.to_le_bytes());
        }
    }
}

impl Adapter {
    pub fn solo_gadget_observation(
        self,
        m: &dyn Mem,
        game: u64,
        owner: u64,
    ) -> Result<(f32, f32, f32, f32)> {
        let p = self.solo_gadget_owner(m, game, owner)?;
        let a = &p.actors[0];
        let read = |at| -> Result<f32> {
            let v = f32::from_le_bytes(
                m.read(at, 4)?
                    .try_into()
                    .map_err(|_| "Truncated engineering observation")?,
            );
            if !v.is_finite() {
                return Err("Invalid engineering observation".into());
            }
            Ok(v)
        };
        Ok((
            a.hp,
            a.maximum,
            read(game + v::SOLO_NATIVE_SHIELD)?,
            read(game + self.field117(v::BLESSING_STAT_FIELDS[4]))?,
        ))
    }
}

#[cfg(test)]
mod mortar_receipt_tests {
    use super::*;
    use std::cell::Cell;
    struct Memory {
        body: Vec<u8>,
        reads: Cell<usize>,
    }
    impl Mem for Memory {
        fn read(&self, address: u64, size: usize) -> Result<Vec<u8>> {
            self.reads.set(self.reads.get() + 1);
            if address != 0x10000 || size != 12 {
                return Err("Unexpected impact pointer read".into());
            }
            Ok(self.body.clone())
        }
    }
    #[test]
    fn impact_receipts_reject_foreign_spells_tags_namespaces_and_truncation() {
        let a = Adapter::select(v::SHA256).unwrap();
        let token = a.solo_new_mortar_token(7).unwrap();
        let mut body = 8u32.to_le_bytes().to_vec();
        body.extend(token.to_le_bytes());
        let mut m = Memory {
            body,
            reads: Cell::new(0),
        };
        assert_eq!(a.solo_mortar_token(&m, 133, 0).unwrap(), None);
        assert_eq!(m.reads.get(), 0);
        assert_eq!(
            a.solo_mortar_token(&m, 13237, 0x10000).unwrap(),
            Some(token)
        );
        m.body[0] = 7;
        assert_eq!(a.solo_mortar_token(&m, 13237, 0x10000).unwrap(), None);
        m.body[0] = 8;
        m.body[11] = 0;
        assert_eq!(a.solo_mortar_token(&m, 13237, 0x10000).unwrap(), None);
        m.body.truncate(11);
        assert!(a.solo_mortar_token(&m, 13237, 0x10000).is_err());
        assert_eq!(a.solo_new_mortar_token(0), None);
    }
}


fn mortar_flight_delay(origin: [f32; 3], target: [f32; 3]) -> Option<f32> {
    if origin.iter().chain(target.iter()).any(|v| !v.is_finite()) {
        return None;
    }
    let distance = (origin[0] - target[0]).hypot(origin[1] - target[1]);
    (distance <= 30.).then_some((distance / 14.).max(0.05))
}
#[cfg(test)]
mod mortar_flight_tests {
    use super::*;
    #[test]
    fn flight_follows_verified_native_speed_with_no_arbitrary_fixed_deadline() {
        assert_eq!(mortar_flight_delay([0.; 3], [14., 0., 0.]), Some(1.));
        assert_eq!(mortar_flight_delay([0.; 3], [0.; 3]), Some(0.05));
        assert_eq!(mortar_flight_delay([0.; 3], [28., 0., 0.]), Some(2.));
        assert!(mortar_flight_delay([0.; 3], [31., 0., 0.]).is_none());
        assert!(mortar_flight_delay([f32::NAN, 0., 0.], [0.; 3]).is_none());
    }
}
