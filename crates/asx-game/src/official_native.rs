//! Shared verified native decoder, selected by an exact build descriptor. Never a legacy
//! custom-engine profile: doing so would enable a second simulation coordinator.
use crate::{memory::Mem, Result};
mod combat;
mod difficulty;
mod fatigue;
mod layout;
mod menu_pause;
mod presentation;
pub use fatigue::FatigueAura;
mod menu_icons;
pub use menu_icons::{MenuAssets, MenuBindings};
mod experience;
pub(crate) mod support;
mod wallet;
pub use combat::{BasicFrame, CombatActor};
/// Wire/context extent of this reviewed native build, not a product admission policy.
pub const NATIVE_PLAYERS: usize = 5;
pub const SHA256: &str = "0258b8a43d9ba3170b15e5fc07a646a606eb2972a5dd6aa0e6349a07ec390e2c";
pub const SERVER_UPDATE: u64 = 0x5877810;
pub const SERVER_BYTES: &[u8] = &[
    0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x55, 0x41, 0x54, 0x56, 0x57, 0x53, 0x48, 0x81, 0xec, 0xe8,
    0x0e, 0, 0,
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Guest,
    Inactive,
    Host,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Observation {
    pub role: Role,
    /// Native guest game-stream flag, not lobby connection status.
    pub guest_game_stream: Option<bool>,
    pub remote_connections: u32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ActorObservation {
    /// Native context slot, scoped to this observation; not a durable player ID.
    pub slot: u32,
    pub phase: asx_gameplay::Phase,
    pub gold: u32,
    pub level: u32,
    pub xp: u32,
    pub class_id: u8,
    pub race_id: u8,
    pub xp_multiplier: f32,
    pub gold_multiplier: f32,
    pub hp: f32,
    pub maximum: f32,
    pub position: [f32; 3],
    pub guid: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PartyObservation {
    pub clock: f32,
    pub actors: Vec<ActorObservation>,
}
#[derive(Clone, Copy, Debug)]
pub struct Adapter {
    revision: Revision,
}
impl Adapter {
    pub fn select(hash: &str) -> Result<Self> {
        let revision = match hash {
            crate::official130::SHA256 => Revision::V130,
            _ => return Err("No reviewed official-engine adapter for this executable".into()),
        };
        Ok(Self { revision })
    }
    pub fn game_address(self, server: u64) -> Result<u64> {
        server
            .checked_add(0x10)
            .ok_or("Native server game overflow".into())
    }
    pub fn server_update(self) -> super::official::Entry {
        super::official::Entry {
            rva: self.rva(SERVER_UPDATE),
            bytes: self.expected(SERVER_UPDATE, SERVER_BYTES),
        }
    }
    pub fn shared_xp(self) -> super::official::Entry {
        super::official::Entry {
            rva: self.rva(0x58daa10),
            bytes: self.expected(
                0x58daa10,
                &[0x41, 0x56, 0x56, 0x57, 0x55, 0x53, 0x48, 0x83, 0xec, 0x30],
            ),
        }
    }
    pub fn shared_xp_input(self, mem: &dyn Mem, request: u64, game: u64) -> Result<u32> {
        let word = |address: u64| -> Result<u64> {
            let bytes = mem.read(address, 8)?;
            Ok(u64::from_le_bytes(
                bytes
                    .try_into()
                    .map_err(|_| "Truncated native shared XP pointer")?,
            ))
        };
        let header = word(request)?;
        if header < 0x10000 {
            return Err("Invalid native shared XP owner".into());
        }
        let bytes = mem.read(header, 24)?;
        if bytes.len() != 24 {
            return Err("Truncated native shared XP owner".into());
        }
        let at = |n| u64::from_le_bytes(bytes[n..n + 8].try_into().unwrap());
        let (capacity, pointer, count) = (at(0), at(8), at(16));
        if count > capacity || capacity > 65536 || (count > 0 && pointer < 0x10000) {
            return Err("Invalid native shared XP extent".into());
        }
        let slot = word(
            game.checked_add(self.game_field(0xb40))
                .ok_or("Shared XP actor overflow")?,
        )?;
        if slot as usize >= NATIVE_PLAYERS {
            return Err("Shared XP actor exceeds mod capacity".into());
        }
        if count == 0 {
            return Ok(0);
        }
        pointer
            .checked_add(count * 16)
            .ok_or("Shared XP list overflow")?;
        let rows = mem.read(pointer, (count * 16) as usize)?;
        if rows.len() != (count * 16) as usize {
            return Err("Truncated shared XP receipts".into());
        }
        let mut total = 0u32;
        for row in rows.chunks_exact(16) {
            let source = u64::from_le_bytes(row[..8].try_into().unwrap());
            if source as usize >= NATIVE_PLAYERS {
                return Err("Invalid shared XP source".into());
            }
            if source != slot {
                total = total
                    .checked_add(u32::from_le_bytes(row[8..12].try_into().unwrap()))
                    .ok_or("Shared XP receipt sum overflow")?;
            }
        }
        Ok(total)
    }
    pub fn award_xp(self) -> super::official::Entry {
        super::official::Entry {
            rva: self.rva(0x591db30),
            bytes: self.expected(
                0x591db30,
                &[
                    0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x55, 0x41, 0x54, 0x56, 0x57, 0x53, 0x48,
                    0x81, 0xec, 0xd8, 0x01, 0, 0,
                ],
            ),
        }
    }
    pub fn racial_xp_bonus(self, race_id: u8) -> f32 {
        // Reviewed Human baseline; native 0.115 observation confirms 1.1 XP.
        if race_id == 1 {
            0.1
        } else {
            0.0
        }
    }
    /// Own receipts are rounded once, then stored in the native sharing queue.
    /// The shared path consumes their integer sum without multiplying again.
    /// Bank/revival, language and unknown callers keep their native semantics.
    pub fn normalize_xp(
        self,
        mem: &dyn super::official::MutableMemory,
        game: u64,
        return_rva: u64,
        registers: [u64; 3],
        multiplier: f32,
        shared_guest: Option<u32>,
    ) -> Result<Option<super::official::ExperienceAward>> {
        let (source, raw) = match self.baseline_rva(return_rva) {
            0x595e653 => ("pickup", registers[0] as u32),
            0x5949b3b => ("effect", registers[2] as u32),
            0x59763dd => ("reward", registers[0] as u32),
            0x58fa00b => ("shared", registers[1] as u32),
            0x58dac9f => (
                "shared",
                shared_guest.ok_or("Missing scoped guest XP receipts")?,
            ),
            _ => return Ok(None),
        };
        let field = |offset: u64, n| {
            let data = mem.read(
                game.checked_add(self.game_field(offset))
                    .ok_or("XP field overflow")?,
                n,
            )?;
            if data.len() != n {
                return Err("Truncated XP field".to_string());
            }
            Ok(data)
        };
        if field(0xc41, 1)? != [1] {
            return Ok(None);
        }
        let current = u32::from_le_bytes(field(0x4f4, 4)?.try_into().unwrap());
        let amount = if source == "shared" {
            raw
        } else {
            asx_gameplay::official::shared_xp_award(raw, multiplier)?
        };
        current
            .checked_add(amount)
            .ok_or("Native XP accumulation would overflow")?;
        if source != "shared" {
            let header = field(0x990, 24)?;
            let word = |at| u64::from_le_bytes(header[at..at + 8].try_into().unwrap());
            let (capacity, pointer, count) = (word(0), word(8), word(16));
            if count == 0 || count > capacity || capacity > 65536 || pointer < 0x10000 {
                return Err("Invalid native XP receipt queue".into());
            }
            let address = pointer
                .checked_add((count - 1) * 16)
                .ok_or("XP receipt overflow")?;
            address
                .checked_add(16)
                .ok_or("XP receipt extent overflow")?;
            let receipt = mem.read(address, 16)?;
            if receipt.len() != 16 {
                return Err("Truncated native XP receipt".into());
            }
            let slot = u64::from_le_bytes(field(0xb40, 8)?.try_into().unwrap());
            if slot as usize >= NATIVE_PLAYERS
                || u64::from_le_bytes(receipt[..8].try_into().unwrap()) != slot
                || u32::from_le_bytes(receipt[8..12].try_into().unwrap()) != raw
            {
                return Err("XP receipt does not belong to this native award".into());
            }
            if let Err(error) = mem.write(address + 8, &amount.to_le_bytes()) {
                mem.write(address + 8, &receipt[8..12])?;
                return Err(error);
            }
        }
        Ok(Some(super::official::ExperienceAward {
            source,
            raw,
            amount,
        }))
    }
    pub fn hud(self) -> super::official::Entry {
        super::official::Entry {
            rva: self.rva(0x590f0f0),
            bytes: self.expected(
                0x590f0f0,
                &[
                    0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x55, 0x41, 0x54, 0x56, 0x57, 0x53, 0x48,
                    0x81, 0xec, 0x48, 0x03, 0, 0,
                ],
            ),
        }
    }
    pub fn world_update(self) -> super::official::Entry {
        super::official::Entry {
            rva: self.rva(0x5919240),
            bytes: self.expected(
                0x5919240,
                &[
                    0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x54, 0x56, 0x57, 0x53, 0x48, 0x81, 0xec,
                    0x70, 0x01, 0, 0,
                ],
            ),
        }
    }
    pub fn command(self) -> super::official::Entry {
        super::official::Entry {
            rva: self.rva(0x5873b00),
            bytes: self.expected(
                0x5873b00,
                &[
                    0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x55, 0x41, 0x54, 0x56, 0x57, 0x53, 0x48,
                    0x81, 0xec, 0x88, 0x03, 0, 0,
                ],
            ),
        }
    }
    pub fn ui_hud(self) -> super::official::Entry {
        super::official::Entry {
            rva: self.rva(0x586d120),
            bytes: self.expected(
                0x586d120,
                &[0x55, 0x41, 0x56, 0x56, 0x57, 0x53, 0x48, 0x83, 0xec, 0x40],
            ),
        }
    }
    pub fn ui_event(self) -> super::official::Entry {
        super::official::Entry {
            rva: self.rva(0x586d550),
            bytes: self.expected(
                0x586d550,
                &[
                    0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x55, 0x41, 0x54, 0x56, 0x57, 0x53, 0x48,
                    0x83, 0xec, 0x68,
                ],
            ),
        }
    }
    pub fn heap(self) -> [super::official::Entry; 2] {
        [
            super::official::Entry {
                rva: self.rva(0x21c0),
                bytes: self.expected(0x21c0, &[0xe9, 0xab, 0xbb, 0xcb, 0x05]),
            },
            super::official::Entry {
                rva: self.rva(0x21d0),
                bytes: self.expected(0x21d0, &[0xe9, 0xfb, 0xbb, 0xcb, 0x05]),
            },
        ]
    }
    pub fn pause_addresses(self) -> [usize; 13] {
        [
            0x19d63c, 0x19d66d, 0x2e23d0, 0x2e299c, 0x2e2b79, 0x2e2b19, 0x19d26e, 0x2bc790,
            0x2b72f0, 0x19d661, 0x19cd74, 0x2e2913, 0x2e3253,
        ]
        .map(|v| self.rva(v) as usize)
    }
    pub fn hud_phase(self, value: &serde_json::Value) -> Result<asx_gameplay::Phase> {
        use asx_gameplay::Phase;
        match value["phase"].as_str() {
            Some("ready" | "victory") => Ok(Phase::Inactive),
            Some("playing") => Ok(Phase::Playing),
            Some("choosing") => Ok(Phase::Choosing),
            Some("paused") => Ok(Phase::Paused),
            Some("dead") => Ok(Phase::Downed),
            _ => Err("Invalid official HUD phase".into()),
        }
    }
    pub fn read_ui(self, mem: &dyn Mem, pointer: u64, length: u64) -> Result<serde_json::Value> {
        if pointer < 0x10000
            || length == 0
            || length > 65536
            || pointer.checked_add(length).is_none()
        {
            return Err("Invalid official UI extent".into());
        }
        serde_json::from_slice(&mem.read(pointer, length as usize)?)
            .map_err(|_| "Invalid official UI JSON".into())
    }
    pub fn decorate_pause(
        self,
        value: &mut serde_json::Value,
        view: &asx_gameplay::official::PausePresentation,
    ) {
        value["_azcoop"] = serde_json::json!({"v":1,"pause":{"active":view.active,"visible":view.visible,"local_menu":view.local_menu,"title":view.title,"detail":view.detail}});
    }
    pub fn decode_pause(
        self,
        value: &serde_json::Value,
    ) -> Result<Option<asx_gameplay::official::PausePresentation>> {
        if value["type"] != "hud" || value.get("_azcoop").is_none() {
            return Ok(None);
        }
        if value["_azcoop"]["v"].as_u64() != Some(1) {
            return Err("Unknown mod HUD schema".into());
        }
        let p = &value["_azcoop"]["pause"];
        let flag = |key: &str| p[key].as_bool().ok_or("Invalid mod pause flag".to_string());
        let text = |key: &str, max| {
            p[key]
                .as_str()
                .filter(|s| s.len() <= max && !s.chars().any(|c| c.is_control()))
                .map(str::to_string)
                .ok_or("Invalid mod pause text".to_string())
        };
        let result = asx_gameplay::official::PausePresentation {
            active: flag("active")?,
            visible: flag("visible")?,
            local_menu: flag("local_menu")?,
            title: text("title", 128)?,
            detail: text("detail", 512)?,
        };
        if result.visible && (!result.active || result.local_menu) {
            return Err("Inconsistent mod pause state".into());
        }
        Ok(Some(result))
    }
    /// Transfer an owned string using the game's own allocator. No DLL-owned
    /// allocation can escape into a native Vec/String destructor.
    pub fn replace_hud(
        self,
        mem: &dyn super::official::MutableMemory,
        output: u64,
        value: &serde_json::Value,
    ) -> Result<()> {
        let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
        if bytes.is_empty() || bytes.len() > 65536 {
            return Err("Mod HUD exceeds native frame limit".into());
        }
        self.replace_owned_text(mem, output, &bytes)
    }
    pub(super) fn replace_owned_text(
        self,
        mem: &dyn super::official::MutableMemory,
        output: u64,
        bytes: &[u8],
    ) -> Result<()> {
        if bytes.is_empty() || bytes.len() > 65536 {
            return Err("Native text extension exceeds limit".into());
        }
        let old = mem.read(output, 24)?;
        if old.len() != 24 {
            return Err("Truncated owned native string".into());
        }
        let capacity = u64::from_le_bytes(old[..8].try_into().unwrap());
        let pointer = u64::from_le_bytes(old[8..16].try_into().unwrap());
        let length = u64::from_le_bytes(old[16..].try_into().unwrap());
        if capacity > 1_048_576 || length > capacity || (capacity > 0 && pointer < 0x10000) {
            return Err("Invalid owned native string".into());
        }
        let new = mem.allocate_native(&bytes)?;
        if new < 0x10000 || new.checked_add(bytes.len() as u64).is_none() {
            return Err("Invalid native HUD allocation".into());
        }
        let mut header = Vec::with_capacity(24);
        header.extend((bytes.len() as u64).to_le_bytes());
        header.extend(new.to_le_bytes());
        header.extend((bytes.len() as u64).to_le_bytes());
        if let Err(error) = mem.write(output, &header) {
            // A failed commit must preserve the original owner and contents.
            mem.write(output, &old)?;
            mem.free_native(new, bytes.len())?;
            return Err(error);
        }
        if capacity > 0 {
            mem.free_native(pointer, capacity as usize)?;
        }
        Ok(())
    }
    pub fn ui_receiver(self, mem: &dyn Mem, server: u64, base: u64) -> Result<u64> {
        let pair = mem.read(
            server
                .checked_add(crate::current::SERVER_UI_SINK)
                .ok_or("Official UI owner overflow")?,
            16,
        )?;
        if pair.len() != 16 {
            return Err("Truncated official UI owner".into());
        }
        let receiver = u64::from_le_bytes(pair[..8].try_into().unwrap());
        let vtable = u64::from_le_bytes(pair[8..].try_into().unwrap());
        if receiver < 0x10000
            || vtable
                != base
                    .checked_add(self.rva(0x5ede960))
                    .ok_or("Official UI vtable overflow")?
        {
            return Err("Unknown official UI owner".into());
        }
        Ok(receiver)
    }
    pub fn set_ui_autopick(
        self,
        mem: &dyn super::official::MutableMemory,
        receiver: u64,
        enabled: bool,
    ) -> Result<bool> {
        let address = receiver
            .checked_add(0x19)
            .ok_or("Official UI flag overflow")?;
        let old = mem.read(address, 1)?;
        if !matches!(old.as_slice(), [0] | [1]) {
            return Err("Invalid official autopick flag".into());
        }
        mem.write(address, &[u8::from(enabled)])?;
        Ok(old == [1])
    }
    pub fn fixture_pause_position(self, mem: &dyn Mem, server: u64) -> Result<[f32; 3]> {
        let game = self.game_address(server)?;
        let raw = mem.read(game + self.game_field(0x4e0), 12)?;
        if raw.len() != 12 {
            return Err("Truncated native player position".into());
        }
        let pose =
            std::array::from_fn(|n| f32::from_le_bytes(raw[n * 4..n * 4 + 4].try_into().unwrap()));
        if pose.iter().any(|v| !v.is_finite()) {
            return Err("Invalid native player position".into());
        }
        Ok(pose)
    }
    pub fn command_is_movement(self, opcode: u16) -> bool {
        ((0xb5..0xcc).contains(&opcode) && opcode != 0xc7) || matches!(opcode, 0xda | 0xdb | 0xee)
    }
    pub fn world_party(self, mem: &dyn Mem, game: u64) -> Result<Option<PartyObservation>> {
        self.party(
            mem,
            game.checked_sub(0x10).ok_or("Invalid official game base")?,
        )
    }
    pub fn hud_value(self, mem: &dyn Mem, output: u64) -> Result<serde_json::Value> {
        let header = mem.read(output, 24)?;
        if header.len() != 24 {
            return Err("Truncated official HUD string".into());
        }
        let cap = u64::from_le_bytes(header[..8].try_into().unwrap());
        let pointer = u64::from_le_bytes(header[8..16].try_into().unwrap());
        let length = u64::from_le_bytes(header[16..].try_into().unwrap());
        if length == 0
            || length > 1_048_576
            || length > cap
            || pointer < 0x10000
            || pointer.checked_add(length).is_none()
        {
            return Err("Invalid official HUD extent".into());
        }
        let bytes = mem.read(pointer, length as usize)?;
        serde_json::from_slice(&bytes).map_err(|e| format!("Invalid official HUD JSON: {e}"))
    }
    /// Extension owner in a native solo run or the current cooperative context.
    /// Cooperative-only callers must keep using effect_source.
    pub(super) fn run_actor(self, mem: &dyn Mem, game: u64) -> Result<Option<u32>> {
        let flag = mem.read(game + self.game_field(0xc41), 1)?;
        if flag == [1] {
            return self.hud_actor(mem, game);
        }
        if flag != [0] {
            return Err("Invalid native run mode".into());
        }
        if mem.read(game, 4)? == 2u32.to_le_bytes() {
            return Ok(None);
        }
        Ok(Some(0))
    }
    pub fn run_source(self, mem: &dyn Mem, game: u64) -> Result<Option<(u32, u64)>> {
        let Some(slot) = self.run_actor(mem, game)? else {
            return Ok(None);
        };
        let guid = u64::from_le_bytes(
            mem.read(game + self.game_field(0x478), 8)?
                .try_into()
                .map_err(|_| "Truncated solo identity")?,
        );
        Ok((guid != 0).then_some((slot, guid)))
    }
    /// Solo has no private actor vector. Never inspect its inactive coop payload.
    pub fn solo_run(self, mem: &dyn Mem, game: u64) -> Result<Option<PartyObservation>> {
        if mem.read(game, 4)? == 2u32.to_le_bytes() {
            return Ok(None);
        }
        if mem.read(game + self.game_field(0xc41), 1)? != [0] {
            return Ok(None);
        }
        let root = self.read_game(mem, game)?;
        let actor = Self::actor(0, &root, root[0xc46])?;
        if actor.guid == 0 {
            return Ok(None);
        }
        let clock = f32::from_le_bytes(root[0xb9c..0xba0].try_into().unwrap());
        if !clock.is_finite() || !(0.0..=1e7).contains(&clock) {
            return Err("Invalid solo clock".into());
        }
        Ok(Some(PartyObservation {
            clock,
            actors: vec![actor],
        }))
    }
    /// Current swapped actor identity only; no perk vectors or full context copy.
    pub fn effect_source(self, mem: &dyn Mem, game: u64) -> Result<Option<(u32, u64)>> {
        let Some(slot) = self.hud_actor(mem, game)? else {
            return Ok(None);
        };
        let guid = u64::from_le_bytes(
            mem.read(
                game.checked_add(self.game_field(0x478))
                    .ok_or("Effect identity overflow")?,
                8,
            )?
            .try_into()
            .map_err(|_| "Truncated effect identity")?,
        );
        Ok((guid != 0).then_some((slot, guid)))
    }
    pub fn hud_actor(self, mem: &dyn Mem, game: u64) -> Result<Option<u32>> {
        let field = |offset: u64, n| {
            mem.read(
                game.checked_add(self.game_field(offset))
                    .ok_or("Official HUD address overflow")?,
                n,
            )
        };
        if field(0xc41, 1)? != [1] {
            return Ok(None);
        }
        let id = u64::from_le_bytes(
            field(0xb40, 8)?
                .try_into()
                .map_err(|_| "Truncated official actor ID")?,
        );
        if id > 4 {
            return Err("Official HUD actor exceeds native party capacity".into());
        }
        Ok(Some(id as u32))
    }
    /// Call only at the server-update return boundary, where the host context
    /// has been restored. Never retain context pointers across a native call.
    pub fn party(self, mem: &dyn Mem, server: u64) -> Result<Option<PartyObservation>> {
        if self.observe(mem, server)?.role != Role::Host {
            return Ok(None);
        }
        let game = server
            .checked_add(0x10)
            .ok_or("Official game address overflow")?;
        // The enum discriminant must be read before its inactive union payload.
        let tag = mem.read(game, 4)?;
        if tag.len() != 4 {
            return Err("Truncated official game tag".into());
        }
        if u32::from_le_bytes(tag.try_into().unwrap()) == 2 {
            return Ok(None);
        }
        let root = self.read_game(mem, game)?;
        if root.len() != 0xc48 {
            return Err("Truncated official game snapshot".into());
        }
        if root[0xc41] != 1 {
            return Ok(None);
        }
        let u64_at = |at| u64::from_le_bytes(root[at..at + 8].try_into().unwrap());
        if u64_at(0xb40) != 0 {
            return Err("Official party snapshot requires restored host context".into());
        }
        let pointer = u64_at(0x980);
        let count = u64_at(0x988);
        // Native capacity and product admission policy are separate. Observe a
        // valid native room without pretending its fifth member is corrupt.
        if count > 4 {
            return Err("Official context count exceeds native capacity".into());
        }
        let extent = count
            .checked_mul(self.context_size())
            .ok_or("Official context extent overflow")?;
        if count > 0 && (pointer < 0x10000 || pointer.checked_add(extent).is_none()) {
            return Err("Invalid official context pointer".into());
        }
        let clock = f32::from_le_bytes(root[0xb9c..0xba0].try_into().unwrap());
        if !clock.is_finite() || !(0.0..=1e7).contains(&clock) {
            return Err("Invalid official clock".into());
        }
        let mut actors = Vec::with_capacity(count as usize + 1);
        actors.push(Self::actor(0, &root, root[0xc46])?);
        for index in 0..count {
            // One bounded copy per private context, never saved across calls.
            let row = self.read_context(mem, pointer + index * self.context_size(), false)?;
            if row.len() != 0x980 {
                return Err("Truncated official actor snapshot".into());
            }
            if u32::from_le_bytes(row[..4].try_into().unwrap()) == 2 {
                continue;
            }
            actors.push(Self::actor(index as u32 + 1, &row, row[0x97c])?);
        }
        Ok(Some(PartyObservation { clock, actors }))
    }
    fn actor(slot: u32, row: &[u8], phase: u8) -> Result<ActorObservation> {
        let phase = match phase {
            0 | 5 => asx_gameplay::Phase::Inactive,
            1 => asx_gameplay::Phase::Playing,
            2 => asx_gameplay::Phase::Choosing,
            3 => asx_gameplay::Phase::Paused,
            4 => asx_gameplay::Phase::Downed,
            _ => return Err("Invalid official phase".into()),
        };
        let u32_at = |at| u32::from_le_bytes(row[at..at + 4].try_into().unwrap());
        let multiplier = |at| {
            let value = f32::from_bits(u32_at(at));
            if !value.is_finite() || value < 0.0 {
                return Err("Invalid official resource multiplier".to_string());
            }
            Ok(value)
        };
        Ok(ActorObservation {
            slot,
            phase,
            gold: u32_at(0x500),
            level: u32_at(0x4f0),
            xp: u32_at(0x4f4),
            class_id: row[0x32c],
            race_id: row[0x32d],
            xp_multiplier: multiplier(0x4ac)?,
            gold_multiplier: multiplier(0x4c0)?,
            hp: f32::from_bits(u32_at(0x4f8)),
            maximum: f32::from_bits(u32_at(0x490)),
            position: std::array::from_fn(|n| f32::from_bits(u32_at(0x4e0 + n * 4))),
            guid: u64::from_le_bytes(row[0x478..0x480].try_into().unwrap()),
        })
    }
    pub fn observe(self, mem: &dyn Mem, server: u64) -> Result<Observation> {
        if server == 0 {
            return Err("Missing official server".into());
        }
        let field = |offset: u64, len: usize| {
            mem.read(
                server
                    .checked_add(match offset {
                        0xc58 => crate::current::SERVER_ROLE,
                        0xc98 => crate::current::SERVER_PEERS,
                        0xf49 => crate::current::SERVER_GAME_STREAM,
                        _ => offset,
                    })
                    .ok_or("Official server address overflow")?,
                len,
            )
        };
        let data = field(0xc58, 4)?;
        let role = match data.as_slice() {
            [0, 0, 0, 0] | [1, 0, 0, 0] => Role::Guest,
            [2, 0, 0, 0] => Role::Inactive,
            [3, 0, 0, 0] => Role::Host,
            _ => return Err("Unrecognized official coop state".into()),
        };
        let guest_game_stream = match role {
            Role::Guest => Some(field(0xf49, 1)? == [1]),
            Role::Host | Role::Inactive => None,
        };
        let remote_connections = if role == Role::Host {
            let data = field(0xc98, 8)?;
            let count = u64::from_le_bytes(
                data.try_into()
                    .map_err(|_| "Truncated official peer count")?,
            );
            // Keep the native five-player wire extent; impose no mod admission cap.
            if count > 4 {
                return Err("Official peer count exceeds native capacity".into());
            }
            count as u32
        } else {
            0
        };
        Ok(Observation {
            role,
            guest_game_stream,
            remote_connections,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Memory {
        role: u32,
        count: u64,
        connected: u8,
    }
    impl Mem for Memory {
        fn read(&self, a: u64, n: usize) -> Result<Vec<u8>> {
            let bytes = match a {
                0xc58 => self.role.to_le_bytes().to_vec(),
                0xc98 => self.count.to_le_bytes().to_vec(),
                0xf49 => vec![self.connected],
                0xdb8 => self.role.to_le_bytes().to_vec(),
                0xdf8 => self.count.to_le_bytes().to_vec(),
                0x11f5 => vec![self.connected],
                _ => return Err("Unexpected union read".into()),
            };
            if bytes.len() != n {
                return Err("Wrong read extent".into());
            }
            Ok(bytes)
        }
    }
    struct View(Memory);
    impl Mem for View {
        fn read(&self, a: u64, n: usize) -> Result<Vec<u8>> {
            self.0.read(a.checked_sub(0x1000).ok_or("bad base")?, n)
        }
    }
    #[test]
    fn rejects_other_builds_and_does_not_read_host_union_for_guests() {
        assert!(Adapter::select("0.115.0").is_err());
        let a = Adapter::select(crate::current::SHA256).unwrap();
        let guest = View(Memory {
            role: 1,
            count: u64::MAX,
            connected: 1,
        });
        assert_eq!(
            a.observe(&guest, 0x1000).unwrap(),
            Observation {
                role: Role::Guest,
                guest_game_stream: Some(true),
                remote_connections: 0
            }
        );
        assert!(a.observe(&guest, 0).is_err());
        assert!(a.observe(&guest, u64::MAX).is_err());
        assert!(a
            .observe(
                &View(Memory {
                    role: 99,
                    count: 0,
                    connected: 0
                }),
                0x1000
            )
            .is_err());
        assert!(a
            .observe(
                &View(Memory {
                    role: 3,
                    count: 5,
                    connected: 0
                }),
                0x1000
            )
            .is_err());
        assert_eq!(
            a.observe(
                &View(Memory {
                    role: 3,
                    count: 2,
                    connected: 0
                }),
                0x1000
            )
            .unwrap()
            .remote_connections,
            2
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Revision {
    V130,
}
impl Adapter {
    pub fn name(self) -> &'static str {
        match self.revision {
            Revision::V130 => "official-coop-0.130.0",
        }
    }
    pub fn version(self) -> &'static str {
        match self.revision {
            Revision::V130 => "0.130.0",
        }
    }
    fn rva(self, baseline: u64) -> u64 {
        crate::current::rva(baseline)
    }
    fn baseline_rva(self, native: u64) -> u64 {
        crate::current::baseline(native)
    }
    fn expected(self, baseline: u64, _old: &'static [u8]) -> &'static [u8] {
        crate::current::expected(baseline)
    }
}

mod blessings;

mod glyphs;
pub use glyphs::{GlyphChoice, GlyphOwner};
mod bonus_ui;
pub use bonus_ui::{BonusHeaderColumn, BonusSeparator, BonusUi};

impl Adapter {
    fn field117(self, native: u64) -> u64 {
        crate::current::field117(native)
    }
    fn supports_extensions(self) -> bool {
        true
    }
    fn contact117(self, old: u64) -> u64 {
        crate::current::extension(old)
    }
    fn extension_bytes117(self, old: u64, _bytes: &'static [u8]) -> &'static [u8] {
        crate::current::extension_bytes(old)
    }
}

mod enchantments;
pub use enchantments::{RingChoice, RingStatLoan};

mod immunity_aggro;
pub use immunity_aggro::{AggroEnemy, AggroLoan};

mod duplicate_loot;

mod gems;
pub use gems::AmuletChoice;

mod bonus_tooltip;

impl Adapter {
    pub fn glyphs_enabled(self) -> bool {
        false
    }
    pub fn duplicate_loot_enabled(self) -> bool {
        false
    }
    pub fn native_marks_enabled(self) -> bool {
        true
    }
}

impl Adapter {
    fn field1201(self, n: u64) -> u64 {
        crate::current::field1201(n)
    }
    fn ring_table(self) -> u64 {
        crate::current::RING_TABLE
    }
    fn amulet_table(self) -> u64 {
        crate::current::AMULET_TABLE
    }
}

/// Selected retained pause presentation contacts.
pub mod pause;

mod raid_markers;

pub mod combat_text;
pub use combat_text::HealText;

mod nameplates;

mod treasure_goblins;
pub use treasure_goblins::{GoblinDefinition, GoblinHandle, GoblinSnapshot};

mod goblin_rewards;

mod world_events;
pub use world_events::{EventObject, RitualScene};

mod stat_inputs;
pub use stat_inputs::StatInputLoan;

mod scene_units;

mod event_placement;
pub use event_placement::EventMinimap;

#[cfg(feature="test-hooks")]
mod world_ui_fixture;

#[cfg(feature="test-hooks")]
pub use nameplates::take_plate_witnesses;
