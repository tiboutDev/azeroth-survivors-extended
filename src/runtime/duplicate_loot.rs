//! Boss-issued entitlements over the existing reliable native coop stream.
//! No polling of collections, extra listener, socket, or per-entity boss scan.
use super::*;
use asx_game::official::SupportCalls;
use asx_gameplay::duplicate_loot::{base_gold, Entitlement, Ledger};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::Mutex,
};
const OFFER: u16 = 0x7a51;
const REPORT: u16 = 0x7a52;
const ACK: u16 = 0x7a53;
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Token {
    epoch: u64,
    ticket: u64,
    encounter: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Report {
    token: Token,
    items: Vec<(u32, bool)>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    token: Token,
    awards: Vec<(u32, u32)>,
    #[serde(default)]
    earned: Option<u32>,
}
#[derive(Default)]
struct Host {
    game: u64,
    guid: u64,
    clock: f32,
    epoch: u64,
    boss: u32,
    issuing: bool,
    next: u64,
    ledger: Ledger,
    peers: BTreeMap<u64, (Option<u64>, (u32, u64))>,
    peers_seen:BTreeSet<u64>,
}
#[derive(Default)]
struct VictoryView{value:Option<serde_json::Value>,earned:Option<u32>,sent:Option<u32>,extras:Vec<serde_json::Value>,restore:bool}
impl VictoryView{
 fn observe(&mut self,value:&serde_json::Value){
  match value["type"].as_str(){
   Some("victory")=>{self.value=Some(value.clone());if self.sent.is_some_and(|gold|value["gold"].as_u64()==Some(u64::from(gold))){self.restore=true;}},
   Some("talentxp"|"unlocked")=>{if let Some(old)=self.extras.iter_mut().find(|old|old["type"]==value["type"]){*old=value.clone();}else{self.extras.push(value.clone());}},
   Some("start"|"resume")=>*self=Self::default(),_=>{}}
 }
 fn take_extras(&mut self)->Vec<serde_json::Value>{if std::mem::take(&mut self.restore){self.extras.clone()}else{vec![]}}
 fn credit(&mut self,earned:u32){self.earned=Some(self.earned.map_or(earned,|old|old.max(earned)));}
 fn update(&mut self)->Option<serde_json::Value>{
  let earned=self.earned?;let value=self.value.as_mut()?;
  if self.sent==Some(earned)||u64::from(earned)<=value["gold"].as_u64()?{return None}
  value["gold"]=earned.into();self.sent=Some(earned);Some(value.clone())
 }
}
#[derive(Default)]
struct State {
    host: Option<Host>,
    guest_epoch:Option<u64>,
    victory:VictoryView,
    offers: VecDeque<Token>,
    waiting: BTreeMap<(u64, u64), Vec<(u32, serde_json::Value)>>,
    local: VecDeque<(Report, Vec<(u32, serde_json::Value)>)>,
    reports: VecDeque<(u64, Report)>,
    receipts: VecDeque<Receipt>,
}
static STATE: Mutex<State> = Mutex::new(State {
    host: None,
    guest_epoch:None,
    victory:VictoryView{value:None,earned:None,sent:None,extras:Vec::new(),restore:false},
    offers: VecDeque::new(),
    waiting: BTreeMap::new(),
    local: VecDeque::new(),
    reports: VecDeque::new(),
    receipts: VecDeque::new(),
});
#[derive(Default)]
struct Batch {
    token: Option<Token>,
    items: Vec<(u32, bool)>,
    ui: Vec<(u32, serde_json::Value)>,
    current: Option<(u32, bool)>,
}
thread_local! {static BATCH:RefCell<Vec<Batch>>=const{RefCell::new(Vec::new())};static INCOMING:RefCell<Option<Token>>=const{RefCell::new(None)};}
static UNLOCK: AtomicUsize = AtomicUsize::new(0);
static RECEIVE: AtomicUsize = AtomicUsize::new(0);
static ITEM: AtomicUsize = AtomicUsize::new(0);
static DROP: AtomicUsize = AtomicUsize::new(0);
static ISSUE: AtomicUsize = AtomicUsize::new(0);
crate::context_stub_rw!(item_entry, 0, item, ITEM);
crate::context_stub_rw!(drop_entry, 0, drop_packet, DROP);
crate::context_stub_rw!(issue_entry, 0, issue, ISSUE);
pub fn reset() {
    if let Ok(mut s) = STATE.lock() {
        *s = State::default();
    }
}
fn warn(r: &Runtime, e: impl std::fmt::Display) {
    r.log.warn(&format!("Duplicate boss loot rejected: {e}"));
}
pub fn pending(game:u64)->bool{
    STATE.lock().ok().is_some_and(|s|s.host.as_ref().is_some_and(|h|h.game==game&&!h.peers.is_empty()))
}
pub fn begin(r: &Runtime, server: u64) {
    if r.adapter.loot_entries().is_empty() {
        return;
    }
    let Some(p) = r.extension_party() else { return };
    let Some(actor) = p.actors.iter().find(|a| a.slot == 0) else {
        return;
    };
    let Ok(game) = r.adapter.game_address(server) else {
        return;
    };
    if let Ok(mut s) = STATE.lock() {
        let fresh = s
            .host
            .as_ref()
            .is_none_or(|h| h.game != game || h.guid != actor.guid || p.clock < h.clock);
        if fresh {
            #[cfg(feature="test-hooks")]
            { static SAMPLES:AtomicUsize=AtomicUsize::new(0);if SAMPLES.fetch_add(1,Ordering::Relaxed)<24{r.log.info(&format!("duplicate loot run: server={server:x} game={game:x} guid={} clock={}",actor.guid,p.clock));} }
            let epoch = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(1, |d| d.as_nanos() as u64)
                ^ game
                ^ actor.guid;
            s.victory=VictoryView::default();
            s.guest_epoch=None;
            s.host = Some(Host {
                game,
                guid: actor.guid,
                epoch: epoch.max(1),
                ..Default::default()
            });
        }
        if let Some(h) = s.host.as_mut() {
            h.clock = p.clock;
        }
    }
}
pub fn encounter(game: u64) {
    #[cfg(feature="test-hooks")]
    if let Some(r)=RUNTIME.get(){if let Ok(s)=STATE.lock(){r.log.info(&format!("duplicate loot encounter: game={game:x} registered={:?}",s.host.as_ref().map(|h|(h.game,h.boss,h.clock))));}}
    if let Ok(mut s) = STATE.lock() {
        if let Some(h) = s.host.as_mut().filter(|h| h.game == game) {
            if let Some(n) = h.boss.checked_add(1) {
                h.boss = n;
                h.issuing = true;
            }
        }
    }
}
fn ticket(
    r: &Runtime,
    game: u64,
    slot: u32,
    peer: Option<u64>,
    items: Vec<u32>,
) -> Result<Option<Token>, String> {
    let Some(actor) = r
        .extension_party()
        .and_then(|p| p.actors.into_iter().find(|a| a.slot == slot))
    else {
        return Ok(None);
    };
    let mut s = STATE.lock().map_err(|_| "Loot state unavailable")?;
    let Some(h) = s
        .host
        .as_mut()
        .filter(|h| h.game == game && h.issuing && h.boss > 0)
    else {
        return Ok(None);
    };
    h.next = h.next.checked_add(1).ok_or("Loot ticket overflow")?;
    let token = Token {
        epoch: h.epoch,
        ticket: h.next,
        encounter: h.boss,
    };
    h.ledger.issue(
        token.ticket,
        Entitlement {
            encounter: h.boss,
            player: (slot, actor.guid),
            items: items.into_iter().collect(),
        },
    )?;
    h.peers.insert(token.ticket, (peer, (slot, actor.guid)));
    if let Some(peer)=peer{h.peers_seen.insert(peer);}
    Ok(Some(token))
}
unsafe extern "system" fn issue(c: &mut crate::stub::Context) {
    #[cfg(test)]
    {tests::FRAME.store(c.rbp,Ordering::SeqCst);tests::VALUE.store(c.r14,Ordering::SeqCst);}
    let Some(r) = RUNTIME.get().filter(|r| r.rules) else {
        return;
    };
    let result = (|| -> Result<(), String> {
        let (server, slot, items) = r.adapter.loot_offer(&r.local, c.rbp, c.r14)?;
        if let Some(token) = ticket(r, server + 16, slot, Some(c.r14), items)? {
            io::Access {
                local: &r.local,
                adapter: r.adapter,
            }
            .loot_send(
                server,
                Some(c.r14),
                OFFER,
                &serde_json::to_vec(&token).map_err(|e| e.to_string())?,
            )?;
            r.log.info(&format!(
                "duplicate loot offer: epoch={} ticket={} boss={} peer={} slot={slot}",
                token.epoch, token.ticket, token.encounter, c.r14
            ));
        }
        Ok(())
    })();
    if let Err(e) = result {
        warn(r, e)
    }
}
type Unlock = unsafe extern "system" fn(u64, u64, u8) -> u64;
unsafe extern "system" fn unlock(server: u64, items: u64, class: u8) -> u64 {
    let original: Unlock = std::mem::transmute(UNLOCK.load(Ordering::Acquire));
    let Some(r) = RUNTIME.get().filter(|r| r.rules) else {
        return original(server, items, class);
    };
    let incoming = INCOMING.with(|t| *t.borrow());
    let active = incoming.is_some()
        || STATE.lock().ok().is_some_and(|s| {
            s.host
                .as_ref()
                .is_some_and(|h| h.game == server + 16 && h.issuing)
        });
    #[cfg(feature="test-hooks")]
    r.log.info(&format!("duplicate loot processor: server={server:x} active={active} incoming={incoming:?} state={:?}",STATE.lock().ok().and_then(|s|s.host.as_ref().map(|h|(h.game,h.boss,h.issuing)))));
    if !active {
        return original(server, items, class);
    }
    BATCH.with(|b| {
        b.borrow_mut().push(Batch {
            token: incoming,
            ..Default::default()
        })
    });
    let value = original(server, items, class);
    if let Some(batch) = BATCH.with(|b| b.borrow_mut().pop()) {
        let result = (|| -> Result<(), String> {
            let token = if let Some(t) = batch.token {
                Some(t)
            } else if batch.items.is_empty() {
                None
            } else {
                ticket(
                    r,
                    server + 16,
                    0,
                    None,
                    batch.items.iter().map(|i| i.0).collect(),
                )?
            };
            if let Some(token) = token {
                let mut s = STATE.lock().map_err(|_| "Loot state unavailable")?;
                if s.local.len() >= 256 {
                    return Err("Loot output queue full".into());
                }
                s.local.push_back((
                    Report {
                        token,
                        items: batch.items,
                    },
                    batch.ui,
                ));
            }
            Ok(())
        })();
        if let Err(e) = result {
            warn(r, e)
        }
    }
    value
}
unsafe extern "system" fn item(c: &mut crate::stub::Context) {
    #[cfg(test)]
    {tests::FRAME.store(c.rbp,Ordering::SeqCst);tests::VALUE.store(c.rdi,Ordering::SeqCst);}
    let Some(r) = RUNTIME.get().filter(|r| r.rules) else {
        return;
    };
    BATCH.with(|b| {
        let mut b = b.borrow_mut();
        let Some(b) = b.last_mut() else { return };
        if let Ok(known) = r.adapter.loot_known(&r.local, c.rbp) {
            #[cfg(feature = "test-hooks")]
            r.log.info(&format!(
                "duplicate loot native item: item={} known={known} epoch={:?}",
                c.rdi as u32, b.token
            ));
            if b.items.len() < 64 {
                let item = (c.rdi as u32, known);
                b.items.push(item);
                b.current = Some(item);
            }
        }
    });
}
unsafe extern "system" fn drop_packet(c: &mut crate::stub::Context) {
    #[cfg(test)]
    {tests::FRAME.store(c.rdx,Ordering::SeqCst);tests::VALUE.store(c.r13,Ordering::SeqCst);}
    let Some(r) = RUNTIME.get().filter(|r| r.rules) else {
        return;
    };
    let current = BATCH.with(|b| b.borrow().last().and_then(|b| b.current));
    let Some((id, true)) = current else { return };
    let io = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    match r.adapter.loot_hide_drop(&io, c.rdx, c.r13) {
        Ok(value) => BATCH.with(|b| {
            if let Some(b) = b.borrow_mut().last_mut() {
                b.ui.push((id, value));
            }
        }),
        Err(e) => warn(r, e),
    }
}
type Receive = unsafe extern "system" fn(u64, u64, f64) -> u64;
unsafe extern "system" fn receive(server: u64, packet: u64, now: f64) -> u64 {
    let original: Receive = std::mem::transmute(RECEIVE.load(Ordering::Acquire));
    let Some(r) = RUNTIME.get().filter(|r| r.rules) else {
        return original(server, packet, now);
    };
    let result = (|| -> Result<Option<Token>, String> {
        if let Some((op, bytes)) = r.adapter.loot_packet(&r.local, packet)? {
            let mut s = STATE.lock().map_err(|_| "Loot state unavailable")?;
            if op == OFFER {
                let token: Token = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                if token.epoch == 0
                    || token.ticket == 0
                    || token.encounter == 0
                    || s.offers.len() >= 256
                {
                    return Err("Invalid loot offer".into());
                }
                if s.guest_epoch.is_some_and(|old|old!=token.epoch){s.victory=VictoryView::default();}
                s.guest_epoch=Some(token.epoch);
                s.offers.push_back(token);
            } else if op == ACK {
                let ack: Receipt = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                if ack.awards.len() > 64 || s.receipts.len() >= 256 {
                    return Err("Invalid loot receipt".into());
                }
                s.receipts.push_back(ack);
            }
        }
        // The next native loot packet immediately follows its ASX offer in the
        // same reliable stream. Keep correlation scoped around native processing.
        let tag = r.local.read(packet as usize, 8)?;
        if tag == 0x800000000000000cu64.to_le_bytes() {
            return Ok(STATE
                .lock()
                .map_err(|_| "Loot state unavailable")?
                .offers
                .pop_front());
        }
        Ok(None)
    })();
    let token = match result {
        Ok(t) => t,
        Err(e) => {
            warn(r, e);
            None
        }
    };
    let previous = INCOMING.with(|t| t.replace(token));
    let value = original(server, packet, now);
    INCOMING.with(|t| {
        t.replace(previous);
    });
    value
}
/// Returns true only for our private command; ordinary native commands are untouched.
pub fn command(r: &Runtime, c: &mut crate::stub::Context) -> bool {
    if c.r9 as u16 != REPORT {
        return false;
    }
    c.skip = 1;
    c.result = 0;
    let result = (|| -> Result<(), String> {
        if c.rdx != 1 {
            return Err("Loot claim is not from a native peer".into());
        }
        let args = r.local.read((c.rsp_entry + 40) as usize, 16)?;
        let ptr = u64::from_le_bytes(args[..8].try_into().unwrap());
        let len = u64::from_le_bytes(args[8..].try_into().unwrap());
        let bytes = r.adapter.loot_bytes(&r.local, ptr, len)?;
        let report: Report = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if report.items.len() > 64 {
            return Err("Invalid loot report".into());
        }
        let mut s = STATE.lock().map_err(|_| "Loot state unavailable")?;
        if s.reports.len() >= 256 {
            return Err("Loot report queue full".into());
        }
        s.reports.push_back((c.r8, report));
        Ok(())
    })();
    if let Err(e) = result {
        warn(r, e)
    }
    true
}
fn settle(r: &Runtime, server: u64, peer: Option<u64>, report: Report) -> Result<Receipt, String> {
    let (game, boss, player, duplicates) = {
        let mut s = STATE.lock().map_err(|_| "Loot state unavailable")?;
        let h = s.host.as_mut().ok_or("Missing loot authority")?;
        if h.epoch != report.token.epoch || h.game != server + 16 {
            return Err("Loot claim belongs to another run".into());
        }
        let (expected, player) = h
            .peers
            .get(&report.token.ticket)
            .copied()
            .ok_or("Unknown loot claimant")?;
        if expected != peer {
            return Err("Wrong native loot peer".into());
        }
        let (boss, items) = h.ledger.claim(report.token.ticket, player, &report.items)?;
        h.peers.remove(&report.token.ticket);
        (h.game, boss, player, items)
    };
    let amounts = wallet::duplicate_credit(r, game, player, base_gold(boss)?, duplicates.len())?;
    r.log.info(&format!("duplicate loot settled: ticket={} boss={boss} peer={peer:?} items={duplicates:?} awards={amounts:?}",report.token.ticket));
    Ok(Receipt {
        token: report.token,
        awards: duplicates.into_iter().zip(amounts).collect(),
        earned:Some(r.adapter.wallet_seed(&r.local,game)?.earned),
    })
}
fn display(
    io: &io::Access<'_>,
    server: u64,
    ui: Vec<(u32, serde_json::Value)>,
    receipt: &Receipt,
) -> Result<(), String> {
    for (id, mut value) in ui {
        let amount = receipt.awards.iter().find(|a| a.0 == id).map_or(0, |a| a.1);
        if amount > 0 {
            let name = value["name"].as_str().unwrap_or("Already collected");
            value["name"] = serde_json::Value::String(format!("{name} — +{amount} gold"));
        }
        io.loot_ui(server, &value)?;
        #[cfg(feature = "test-hooks")]
        if let Some(r) = RUNTIME.get() {
            r.log.info(&format!(
                "duplicate loot native UI: item={id} amount={amount} payload={value}"
            ));
        }
    }
    Ok(())
}
pub fn presentation(value:&serde_json::Value){
 if let Ok(mut s)=STATE.lock(){
  if value["type"]=="start"{s.guest_epoch=None;}
  s.victory.observe(value);
 }
}
fn note_earned(receipt:&Receipt){
 let Some(earned)=receipt.earned else{return};
 if let Ok(mut s)=STATE.lock(){
  let epoch=s.host.as_ref().map(|h|h.epoch).or(s.guest_epoch);
  if epoch==Some(receipt.token.epoch){s.victory.credit(earned);}
 }
}
fn refresh_victory(r:&Runtime,server:u64,io:&io::Access<'_>){
 let pending={
  let Ok(mut s)=STATE.lock()else{return};
  if !s.waiting.is_empty()||s.host.as_ref().is_some_and(|h|h.issuing||!h.peers.is_empty()){return}
  let Some(value)=s.victory.update()else{return};
  let peers=s.host.as_ref().map(|h|(Token{epoch:h.epoch,ticket:0,encounter:h.boss},h.peers_seen.iter().copied().collect::<Vec<_>>()));
  (value,peers)
 };
 let (value,peers)=pending;let earned=value["gold"].as_u64().unwrap_or(0)as u32;
 if let Err(e)=io.loot_ui(server,&value){warn(r,e);return}
 r.log.info(&format!("duplicate loot victory display: gold={earned}"));
 if let Some((token,peers))=peers{
  // Ticket zero is a display-only final total, broadcast once to all native
  // recipients. It has no items and never enters claim/credit policy.
  let receipt=Receipt{token,awards:vec![],earned:Some(earned)};
  if let Ok(bytes)=serde_json::to_vec(&receipt){for peer in peers{if let Err(e)=io.loot_send(server,Some(peer),ACK,&bytes){warn(r,e)}}}
 }
}
pub fn finish(r: &Runtime, server: u64) {
    let io = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    let (local, reports, receipts) = match STATE.lock() {
        Ok(mut s) => {
            if let Some(h) = s.host.as_mut() {
                h.issuing = false;
            }
            (
                std::mem::take(&mut s.local),
                std::mem::take(&mut s.reports),
                std::mem::take(&mut s.receipts),
            )
        }
        Err(_) => return,
    };
    for (report, ui) in local {
        let authority = STATE.lock().ok().is_some_and(|s| {
            s.host
                .as_ref()
                .is_some_and(|h| h.epoch == report.token.epoch)
        });
        if authority {
            match settle(r, server, None, report.clone()) {
                Ok(receipt) => {
                    note_earned(&receipt);
                    if let Err(e) = display(&io, server, ui, &receipt) {
                        warn(r, e)
                    }
                }
                Err(e) => {
                    warn(r, e);
                    let _ = display(
                        &io,
                        server,
                        ui,
                        &Receipt {
                            token: report.token,
                            awards: vec![],
                                earned:None,
                        },
                    );
                }
            }
        } else {
            if let Ok(mut s) = STATE.lock() {
                if s.waiting.len() < 256 {
                    s.waiting
                        .insert((report.token.epoch, report.token.ticket), ui);
                }
            }
            let result = serde_json::to_vec(&report)
                .map_err(|e| e.to_string())
                .and_then(|b| io.loot_send(server, None, REPORT, &b));
            if let Err(e) = result {
                warn(r, e)
            }
        }
    }
    for (peer, report) in reports {
        let receipt = match settle(r, server, Some(peer), report.clone()) {
            Ok(v) => v,
            Err(e) => {
                warn(r, e);
                Receipt {
                    token: report.token,
                    awards: vec![],
                                earned:None,
                }
            }
        };
        note_earned(&receipt);
        let result = serde_json::to_vec(&receipt)
            .map_err(|e| e.to_string())
            .and_then(|b| io.loot_send(server, Some(peer), ACK, &b));
        if let Err(e) = result {
            warn(r, e)
        }
    }
    for receipt in receipts {
        note_earned(&receipt);
        let ui = STATE.lock().ok().and_then(|mut s| {
            s.waiting
                .remove(&(receipt.token.epoch, receipt.token.ticket))
        });
        if let Some(ui) = ui {
            if let Err(e) = display(&io, server, ui, &receipt) {
                warn(r, e)
            }
        }
    }
    refresh_victory(r,server,&io);
    // Native victory resets its supplemental panels. Restore the original UI
    // payloads after the refreshed result was observed, without gameplay calls.
    let extras=STATE.lock().ok().map_or_else(Vec::new,|mut s|s.victory.take_extras());
    for value in extras{if let Err(e)=io.loot_ui(server,&value){warn(r,e)}}
}
pub fn hooks(adapter: Adapter) -> Vec<crate::install::HookSpec> {
    let detours = [
        unlock as *const () as usize,
        receive as *const () as usize,
        item_entry as *const () as usize,
        drop_entry as *const () as usize,
        issue_entry as *const () as usize,
    ];
    let originals = [&UNLOCK, &RECEIVE, &ITEM, &DROP, &ISSUE];
    let names = [
        "duplicate-loot-unlock",
        "duplicate-loot-receive",
        "duplicate-loot-item",
        "duplicate-loot-drop",
        "duplicate-loot-issue",
    ];
    let mut hooks:Vec<_>=adapter
        .loot_entries()
        .iter()
        .enumerate()
        .map(|(i, e)| crate::install::HookSpec {
            name: names[i],
            target: local_base() + e.rva as usize,
            detour: detours[i],
            trampoline: originals[i],
        })
        .collect();
    #[cfg(feature="test-hooks")]
    if let Some(e)=fixture_entry(adapter){hooks.push(crate::install::HookSpec{name:"duplicate-loot-fixture-dispatch",target:local_base()+e.rva as usize,detour:fixture_dispatch_entry as *const ()as usize,trampoline:&FIXTURE_DISPATCH});}
    hooks
}

#[cfg(test)]
mod tests {
    use super::*;
    pub static FRAME:AtomicU64=AtomicU64::new(0);
    pub static VALUE:AtomicU64=AtomicU64::new(0);
    unsafe extern "system" fn continuation(){}
    #[unsafe(naked)]
    unsafe extern "system" fn item_probe(_frame:u64,_id:u64)->u64{
      core::arch::naked_asm!("push rbp","push rdi","sub rsp,40","mov rbp,rcx","mov rdi,rdx","call {entry}","mov rax,rbp","xor rax,rdi","add rsp,40","pop rdi","pop rbp","ret",entry=sym item_entry);
    }
    #[unsafe(naked)]
    unsafe extern "system" fn issue_probe(_frame:u64,_peer:u64)->u64{
      core::arch::naked_asm!("push rbp","push r14","sub rsp,40","mov rbp,rcx","mov r14,rdx","call {entry}","mov rax,rbp","xor rax,r14","add rsp,40","pop r14","pop rbp","ret",entry=sym issue_entry);
    }
    #[test]fn native_item_and_peer_frames_observe_nonvolatile_registers(){
      ITEM.store(continuation as *const ()as usize,Ordering::Release);ISSUE.store(continuation as *const ()as usize,Ordering::Release);
      let frame=0x1122334455667788;let value=0x8877665544332211;
      assert_eq!(unsafe{item_probe(frame,value)},frame^value);assert_eq!(FRAME.load(Ordering::SeqCst),frame);assert_eq!(VALUE.load(Ordering::SeqCst),value);
      assert_eq!(unsafe{issue_probe(frame,value)},frame^value);assert_eq!(FRAME.load(Ordering::SeqCst),frame);assert_eq!(VALUE.load(Ordering::SeqCst),value);
    }

    static ARGS: Mutex<Vec<u64>> = Mutex::new(Vec::new());
    unsafe extern "system" fn native_unlock(server: u64, items: u64, class: u8) -> u64 {
        *ARGS.lock().unwrap() = vec![server, items, u64::from(class)];
        0x8877665544332211
    }
    unsafe extern "system" fn native_receive(server: u64, packet: u64, clock: f64) -> u64 {
        *ARGS.lock().unwrap() = vec![server, packet, clock.to_bits()];
        0x1122334455667788
    }
    #[test]
    fn typed_wrappers_preserve_native_arguments_and_returns() {
        UNLOCK.store(native_unlock as *const () as usize, Ordering::Release);
        RECEIVE.store(native_receive as *const () as usize, Ordering::Release);
        assert_eq!(
            unsafe { unlock(0x123456789, 0x987654321, 0xab) },
            0x8877665544332211
        );
        assert_eq!(*ARGS.lock().unwrap(), [0x123456789, 0x987654321, 0xab]);
        assert_eq!(
            unsafe { receive(0x987654321, 0x123456789, 123.75) },
            0x1122334455667788
        );
        assert_eq!(
            *ARGS.lock().unwrap(),
            [0x987654321, 0x123456789, 123.75f64.to_bits()]
        );
    }
    #[test]
    fn victory_total_refresh_is_display_only_and_clears_on_resume(){
      let mut view=VictoryView::default();let original=serde_json::json!({"type":"victory","gold":100,"kills":17,"class":"Warrior"});
      view.observe(&original);view.credit(150);let updated=view.update().unwrap();
      assert_eq!(updated["gold"],150);assert_eq!(updated["kills"],17);assert_eq!(updated["class"],"Warrior");assert!(view.update().is_none());
      view.observe(&serde_json::json!({"type":"talentxp","xp":410,"total":50410}));
      view.observe(&serde_json::json!({"type":"unlocked","bg":"Duskwood"}));
      view.observe(&updated);let extras=view.take_extras();assert_eq!(extras.len(),2);assert_eq!(extras[0]["total"],50410);assert_eq!(extras[1]["bg"],"Duskwood");assert!(view.take_extras().is_empty());
      view.credit(120);assert!(view.update().is_none());
      view.observe(&serde_json::json!({"type":"resume"}));view.credit(200);assert!(view.update().is_none());
      view.observe(&serde_json::json!({"type":"victory","gold":250}));assert!(view.update().is_none());
    }
    #[test]
    fn payloads_reject_extra_fields_and_oversized_native_extents() {
        assert!(serde_json::from_str::<Token>(
            r#"{"epoch":1,"ticket":2,"encounter":1,"gold":999}"#
        )
        .is_err());
        let report = Report {
            token: Token {
                epoch: 7,
                ticket: 3,
                encounter: 2,
            },
            items: vec![(123, true), (456, false)],
        };
        let decoded: Report =
            serde_json::from_slice(&serde_json::to_vec(&report).unwrap()).unwrap();
        assert_eq!(decoded.items, report.items);
    }
}

#[cfg(feature="test-hooks")]
pub fn fixture_loot(r:&Runtime,game:u64){
 if std::env::var("AZCOOP_TEST_DUPLICATE_LOOT").as_deref()!=Ok("1"){return}
 let io=io::Access{local:&r.local,adapter:r.adapter};
 match unsafe{r.adapter.loot_fixture_append(&io,game,r.local.base as u64,0,101100)}{
  Ok(added)=>r.log.info(&format!("duplicate loot fixture native queue: slot=0 item=101100 added={added}")),Err(e)=>warn(r,e)
 }
}
#[cfg(feature="test-hooks")]
static FIXTURE_DISPATCH:AtomicUsize=AtomicUsize::new(0);
#[cfg(feature="test-hooks")]
crate::context_stub_rw!(fixture_dispatch_entry,0,fixture_dispatch,FIXTURE_DISPATCH);
#[cfg(feature="test-hooks")]
unsafe extern "system" fn fixture_dispatch(c:&mut crate::stub::Context){
 let Some(r)=RUNTIME.get().filter(|r|r.rules)else{return};
 if !STATE.lock().ok().is_some_and(|s|s.host.as_ref().is_some_and(|h|h.issuing)){return}
 let io=io::Access{local:&r.local,adapter:r.adapter};
 match r.adapter.loot_fixture_dispatch(&io,c.rbp,r.local.base as u64,c.r14,101100){
  Ok(len)=>{c.rax=len;r.log.info(&format!("duplicate loot fixture guest dispatch: peer={} item=101100 len={len}",c.r14));},Err(e)=>warn(r,e)
 }
}
pub fn fixture_entry(adapter:Adapter)->Option<asx_game::official::Entry>{
 #[cfg(feature="test-hooks")]
 if std::env::var("AZCOOP_TEST_DUPLICATE_LOOT").as_deref()==Ok("1"){return adapter.loot_fixture_entry()}
 let _=adapter;None
}
