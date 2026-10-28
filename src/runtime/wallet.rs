//! Host-owned currency policy. Native layouts/calls stay in the selected adapter.
use super::*;
use asx_game::official::{PartyObservation, NATIVE_PLAYERS};
use asx_gameplay::official::{shared_gold_award, SharedWallet};
#[derive(Clone, Copy)]
struct State {
    game: u64,
    clock: f32,
    members: usize,
    wallet: SharedWallet,
    opened: u32,
    cost: u32,
    delivered: [u32; NATIVE_PLAYERS],
    armory: ArmorySnapshot,
}
// Permanent equipment cannot change during a run. Capture at the restored-host
// boundary, not inside native awards: reward dispatch may temporarily own/move
// the private actor vector. GUIDs prevent slot reuse from inheriting old gear.
type ArmorySnapshot = [Option<(u64,Option<f32>)>;NATIVE_PLAYERS];
fn snapshot_armory(previous:ArmorySnapshot,members:impl IntoIterator<Item=(u32,u64)>,mut read:impl FnMut(u32,u64)->Result<Option<f32>,String>)->Result<ArmorySnapshot,String>{
    let mut next=[None;NATIVE_PLAYERS];
    for (slot,guid) in members {
        let index=slot as usize;
        if index>=NATIVE_PLAYERS||guid==0||next[index].is_some(){return Err("Invalid armory membership".into())}
        next[index]=Some(match previous[index].filter(|(id,_)|*id==guid){Some(old)=>old,None=>(guid,read(slot,guid)?)});
    }
    Ok(next)
}
fn cached_armory(snapshot:&ArmorySnapshot,slot:u32,guid:u64)->Result<Option<f32>,String>{
    snapshot.get(slot as usize).copied().flatten().filter(|(id,_)|*id==guid).map(|(_,bonus)|bonus).ok_or("Gold armory membership changed".into())
}
#[cfg(feature="test-hooks")]
pub fn invalidate_fixture_armory(game:u64){if let Ok(mut s)=STATE.lock(){if let Some(s)=s.as_mut().filter(|s|s.game==game){s.armory=[None;NATIVE_PLAYERS];}}}
static STATE: std::sync::Mutex<Option<State>> = std::sync::Mutex::new(None);
static GOLD_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static GOLD_SAMPLES: AtomicU64 = AtomicU64::new(0);
static SWAP_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static CURRENCY_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
thread_local! {static PROJECTING:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}
fn projected<T>(action: impl FnOnce() -> T) -> T {
    struct Guard(bool);
    impl Drop for Guard {
        fn drop(&mut self) {
            PROJECTING.with(|v| v.set(self.0));
        }
    }
    let _guard = Guard(PROJECTING.with(|v| v.replace(true)));
    action()
}
fn state(game: u64) -> Option<State> {
    STATE.lock().ok().and_then(|s| s.filter(|s| s.game == game))
}
fn commit(previous: State, wallet: SharedWallet) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|_| "Wallet state unavailable")?;
    let current = state
        .as_mut()
        .filter(|s| s.game == previous.game && s.wallet == previous.wallet)
        .ok_or("Wallet changed during native projection")?;
    current.wallet = wallet;
    Ok(())
}
pub fn boundary(
    runtime: &Runtime,
    game: u64,
    party: Option<&PartyObservation>,
) -> Result<(), String> {
    // A final-boss report may arrive after native actors enter Victory. Keep
    // the same wallet/armory identities until issued loot is settled, not past teardown.
    let terminal=duplicate_loot::pending(game)&&runtime.adapter.loot_victory(&runtime.local,game).unwrap_or(false);
    let Some(party) = party.filter(|p| {
        terminal || p.actors
            .iter()
            .any(|a| a.phase != asx_gameplay::Phase::Inactive)
    }) else {
        *STATE.lock().map_err(|_| "Wallet state unavailable")? = None;
        return Ok(());
    };
    if party.actors.is_empty() || party.actors.len() > NATIVE_PLAYERS {
        return Err("Unsupported wallet membership".into());
    }
    let old = state(game);
    let fresh = old.is_none_or(|s| party.clock < s.clock);
    let mut next = if fresh {
        let (_, cost, opened) = runtime.adapter.chest_state(&runtime.local, game)?;
        State {
            game,
            clock: party.clock,
            members: party.actors.len(),
            wallet: runtime.adapter.wallet_seed(&runtime.local, game)?,
            cost,
            opened,
            delivered: [opened; NATIVE_PLAYERS],
            armory: [None;NATIVE_PLAYERS],
        }
    } else {
        old.unwrap()
    };
    next.clock = party.clock;
    next.members = party.actors.len();
    next.armory=snapshot_armory(next.armory,party.actors.iter().filter(|a|terminal||a.phase!=asx_gameplay::Phase::Inactive).map(|a|(a.slot,a.guid)),|slot,guid|runtime.adapter.armory_gold_bonus(&runtime.local,game,slot,guid))?;
    {
        *STATE.lock().map_err(|_| "Wallet state unavailable")? = Some(next);
    }
    let io = io::Access {
        local: &runtime.local,
        adapter: runtime.adapter,
    };
    projected(|| runtime.adapter.project_wallet(&io, game, next.wallet))?;
    for actor in &party.actors {
        let index = actor.slot as usize;
        if index >= NATIVE_PLAYERS {
            return Err("Invalid shared chest actor".into());
        }
        let pending = next
            .opened
            .checked_sub(next.delivered[index])
            .ok_or("Shared chest acknowledgement exceeds opened count")?;
        if runtime
            .adapter
            .deliver_chests(&io, game, actor.slot, pending, next.cost, next.opened)?
        {
            if pending > 0 {
                runtime.log.info(&format!(
                    "official chest delivery: slot={} count={pending} opened={}",
                    actor.slot, next.opened
                ));
            }
            next.delivered[index] = next.opened;
            // Persist each successful delivery before visiting another actor;
            // a later rejected row must not duplicate an already committed item.
            *STATE.lock().map_err(|_| "Wallet state unavailable")? = Some(next);
        }
    }
    {
        *STATE.lock().map_err(|_| "Wallet state unavailable")? = Some(next);
    }
    Ok(())
}
crate::context_stub_rw!(gold_entry, 0, gold, GOLD_ORIGINAL);
unsafe extern "system" fn gold(c: &mut crate::stub::Context) {
    let Some(runtime) = RUNTIME.get().filter(|r| r.rules) else {
        return;
    };
    let Some(state) = state(c.rcx) else { return };
    // Once the shared wallet owns this run, never fall back to a second native
    // grant/receipt after rejection. Invalid requests fail closed and are logged.
    c.skip = 1;
    c.result = 0;
    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<(), String> {
            let stack = runtime.local.read(c.rsp_entry as usize, 8)?;
            let site = u64::from_le_bytes(stack.try_into().map_err(|_| "Truncated gold return")?)
                .checked_sub(runtime.local.base as u64)
                .ok_or("Invalid gold caller")?;
            let source = runtime.adapter.gold_source(site);
            let amount = normalized_award(runtime, &state, c.rcx, c.rdx as u32, source)?;
            let wallet = state.wallet.credit(amount)?;
            let io = io::Access {
                local: &runtime.local,
                adapter: runtime.adapter,
            };
            projected(|| runtime.adapter.project_current_wallet(&io, c.rcx, wallet))?;
            commit(state, wallet)?;
            c.result = 1;
            if GOLD_SAMPLES.fetch_add(1, Ordering::Relaxed) < 512
                || matches!(source, asx_gameplay::official::Award::ConvertedReward | asx_gameplay::official::Award::SkippedReward)
            {
                runtime.log.info(&format!(
                    "official gold: source={source:?} raw={} amount={amount} balance={} earned={}",
                    c.rdx as u32, wallet.gold, wallet.earned
                ));
            }
            Ok(())
        }));
    if let Err(error) = result.unwrap_or_else(|_| Err("Gold adapter panicked".into())) {
        runtime
            .log
            .warn(&format!("Official shared gold rejected: {error}"));
    }
}
crate::context_stub!(currency_entry, 0, currency, CURRENCY_ORIGINAL);
unsafe extern "system" fn currency(c: &crate::stub::Context) {
    if PROJECTING.with(|v| v.get()) {
        return;
    }
    let Some(runtime) = RUNTIME.get().filter(|r| r.rules) else {
        return;
    };
    let current = STATE.lock().ok().and_then(|s| *s);
    let Some(state) = current.filter(|s| runtime.adapter.is_currency(s.game, c.rcx, c.rdx as u16))
    else {
        return;
    };
    let result = (|| -> Result<(), String> {
        let (gold, _, _) = runtime.adapter.current_gold(&runtime.local, state.game)?;
        if gold.checked_mul(10000) != Some(c.r8 as u32) {
            return Err("Native coinage does not match its wallet balance".into());
        }
        let wallet = state.wallet.native_debit(gold)?;
        commit(state, wallet)?;
        if wallet.gold != state.wallet.gold {
            runtime.log.info(&format!(
                "official gold spend: amount={} balance={}",
                state.wallet.gold - wallet.gold,
                wallet.gold
            ));
        }
        Ok(())
    })();
    if let Err(error) = result {
        runtime
            .log
            .warn(&format!("Official shared debit rejected: {error}"));
    }
}
type Swap = unsafe extern "system" fn(u64, u64);
unsafe extern "system" fn swap(game: u64, row: u64) {
    let original: Swap = std::mem::transmute(SWAP_ORIGINAL.load(Ordering::Acquire));
    original(game, row);
    let Some(runtime) = RUNTIME.get().filter(|r| r.rules) else {
        return;
    };
    let Some(state) = state(game) else { return };
    let io = io::Access {
        local: &runtime.local,
        adapter: runtime.adapter,
    };
    if let Err(error) = projected(|| {
        runtime
            .adapter
            .project_current_wallet(&io, game, state.wallet)?;
        runtime
            .adapter
            .project_current_chest(&io, game, state.cost, state.opened)
    }) {
        runtime.log.warn(&format!(
            "Official wallet context projection rejected: {error}"
        ));
    }
}
pub fn hooks(adapter: Adapter) -> [crate::install::HookSpec; 4] {
    [
        crate::install::HookSpec {
            name: "official-group-chest",
            target: local_base() + adapter.chest_entry().rva as usize,
            detour: chest as *const () as usize,
            trampoline: &CHEST_ORIGINAL,
        },
        crate::install::HookSpec {
            name: "official-gold",
            target: local_base() + adapter.gold_entry().rva as usize,
            detour: gold_entry as *const () as usize,
            trampoline: &GOLD_ORIGINAL,
        },
        crate::install::HookSpec {
            name: "official-wallet-context",
            target: local_base() + adapter.swap_entry().rva as usize,
            detour: swap as *const () as usize,
            trampoline: &SWAP_ORIGINAL,
        },
        crate::install::HookSpec {
            name: "official-wallet-debit",
            target: local_base() + adapter.currency_entry().rva as usize,
            detour: currency_entry as *const () as usize,
            trampoline: &CURRENCY_ORIGINAL,
        },
    ]
}

pub fn reset() {
    if let Ok(mut state) = STATE.lock() {
        *state = None;
    }
}

static CHEST_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
type Chest = unsafe extern "system" fn(u64) -> bool;
unsafe extern "system" fn chest(game: u64) -> bool {
    let original: Chest = std::mem::transmute(CHEST_ORIGINAL.load(Ordering::Acquire));
    let result = original(game);
    if !result {
        return false;
    }
    let Some(runtime) = RUNTIME.get().filter(|r| r.rules) else {
        return result;
    };
    let Some(mut state) = state(game) else {
        return result;
    };
    let record = (|| -> Result<(), String> {
        let (slot, cost, _) = runtime.adapter.chest_state(&runtime.local, game)?;
        state.opened = state
            .opened
            .checked_add(1)
            .ok_or("Shared opened chest overflow")?;
        state.delivered[slot] = state.delivered[slot]
            .checked_add(1)
            .ok_or("Shared chest acknowledgement overflow")?;
        state.cost = cost;
        *STATE.lock().map_err(|_| "Wallet state unavailable")? = Some(state);
        runtime.log.info(&format!(
            "official group chest: owner={slot} opened={} next_cost={cost}",
            state.opened
        ));
        Ok(())
    })();
    if let Err(error) = record {
        runtime
            .log
            .warn(&format!("Official chest adaptation rejected: {error}"));
    }
    result
}

/// Open the native reward screen for outstanding items, including other members'
/// shared-cache delivery. An existing choice/menu/downed phase is left intact.
pub fn present(runtime:&Runtime,game:u64,party:&PartyObservation)->bool{
    let mut opened=false;
    let io=io::Access{local:&runtime.local,adapter:runtime.adapter};
    for actor in &party.actors {
        match runtime.adapter.present_pending_item(&io,game,actor.slot,actor.guid){
            Ok(true)=>{opened=true;runtime.log.info(&format!("official pending item presented: slot={}",actor.slot));},
            Ok(false)=>{},Err(e)=>runtime.log.warn(&format!("Official item presentation rejected: {e}")),
        }
    }
    opened
}

#[cfg(test)]
mod armory_tests {
    use super::*;
    #[test]
    fn native_reward_can_credit_gold_while_private_contexts_are_temporarily_unavailable(){
        let members=[(0,111),(1,222)];let mut reads=0;
        let snapshot=snapshot_armory([None;NATIVE_PLAYERS],members,|slot,_|{reads+=1;Ok(Some(if slot==0{0.1}else{0.3}))}).unwrap();assert_eq!(reads,2);
        let stable=snapshot_armory(snapshot,members,|_,_|Err("Native actor vector moved during reward".into())).unwrap();
        let own=cached_armory(&stable,0,111).unwrap().unwrap();let gear=members.map(|(s,g)|cached_armory(&stable,s,g).unwrap().unwrap());
        let multiplier=asx_gameplay::official::shared_gold_equipment_multiplier(1.3,own,gear).unwrap();
        assert!((multiplier-1.5).abs()<0.00001);
        let wallet=SharedWallet{gold:0,earned:0}.credit((100.*multiplier).round() as u32).unwrap();assert_eq!(wallet.gold,150);
    }
    #[test]
    fn replacing_or_removing_a_member_cannot_reuse_the_previous_players_equipment(){
        let old=snapshot_armory([None;NATIVE_PLAYERS],[(0,111),(1,222)],|_,_|Ok(Some(0.3))).unwrap();
        let mut reads=vec![];let new=snapshot_armory(old,[(0,111),(1,333)],|s,g|{reads.push((s,g));Ok(Some(0.1))}).unwrap();assert_eq!(reads,vec![(1,333)]);
        assert!(cached_armory(&new,1,222).is_err());assert_eq!(cached_armory(&new,1,333).unwrap(),Some(0.1));
        let solo=snapshot_armory(new,[(0,111)],|_,_|panic!("Unchanged gear must not be read again")).unwrap();assert!(cached_armory(&solo,1,333).is_err());
    }
    #[test]
    fn unsupported_native_equipment_is_cached_without_repeated_reads(){
        let old=snapshot_armory([None;NATIVE_PLAYERS],[(0,111)],|_,_|Ok(None)).unwrap();
        let same=snapshot_armory(old,[(0,111)],|_,_|panic!("Unsupported layout must not be reread")).unwrap();assert_eq!(cached_armory(&same,0,111).unwrap(),None);
    }
}

#[cfg(feature="test-hooks")]
pub fn fixture_enabled()->bool{std::env::var("AZCOOP_TEST_REWARD_WALLET").as_deref()==Ok("1")}
#[cfg(feature="test-hooks")]
pub fn fixture_choice(r:&Runtime,game:u64)->bool{
 if !fixture_enabled(){return false}
 static STAGES:std::sync::Mutex<[[u8;2];5]>=std::sync::Mutex::new([[0;2];5]);
 let result=(||->Result<bool,String>{
  let Some((slot,guid))=r.adapter.run_source(&r.local,game)?else{return Ok(false)};
  let Some(kind)=r.adapter.reward_choice_kind(&r.local,game)?else{return Ok(false)};
  let action={let mut s=STAGES.lock().map_err(|_|"Reward fixture stage")?;let a=&mut s[slot as usize][kind as usize];if *a>=3{return Ok(false)}let n=*a;*a+=1;n};
  let before=r.adapter.wallet_seed(&r.local,game)?;
  let shared_before=STATE.lock().ok().and_then(|s|s.map(|s|(s.game,s.wallet)));
  let io=io::Access{local:&r.local,adapter:r.adapter};
  unsafe{r.adapter.fixture_reward_action(&io,r.local.base,game,action)?;}
  let after=r.adapter.wallet_seed(&r.local,game)?;
  let shared_after=STATE.lock().ok().and_then(|s|s.map(|s|(s.game,s.wallet)));
  r.log.info(&format!("reward wallet fixture: slot={slot} guid={guid} kind={kind} action={action} game={game} before={before:?} after={after:?} shared_before={shared_before:?} shared_after={shared_after:?}"));
  Ok(true)
 })();match result{Ok(v)=>v,Err(e)=>{r.log.warn(&format!("Reward wallet fixture rejected: {e}"));false}}
}
#[cfg(feature="test-hooks")]
pub fn fixture_tick(r:&Runtime,server:u64){
 static OPENED:AtomicUsize=AtomicUsize::new(0);
 let result=(||->Result<(),String>{
  let Some(p)=r.adapter.party(&r.local,server)?else{return Ok(())};
  if p.actors.len()!=2||p.clock<2.||p.actors.iter().any(|a|a.phase!=asx_gameplay::Phase::Playing){return Ok(())}
  if OPENED.load(Ordering::Acquire)>=3{return Ok(())}
  OPENED.fetch_add(1,Ordering::AcqRel);let game=r.adapter.game_address(server)?;
  type Gold=unsafe extern "system" fn(u64,u32)->usize;type Chest=unsafe extern "system" fn(u64)->bool;
  let gold:Gold=unsafe{std::mem::transmute(r.local.base+r.adapter.gold_entry().rva as usize)};
  let chest:Chest=unsafe{std::mem::transmute(r.local.base+r.adapter.chest_entry().rva as usize)};
  unsafe{gold(game,1000);chest(game);}
  Ok(())
 })();if let Err(e)=result{r.log.warn(&format!("Reward wallet fixture setup rejected: {e}"));}
}

fn normalized_award(runtime:&Runtime,state:&State,game:u64,raw:u32,source:asx_gameplay::official::Award)->Result<u32,String>{
    let (_,multiplier,race)=runtime.adapter.current_gold(&runtime.local,game)?;
    let collector=runtime.adapter.effect_source(&runtime.local,game)?.ok_or("Missing gold collector identity")?;
    normalized_for_actor(runtime,state,raw,source,(multiplier,race,collector,false))
}
fn normalized_for_actor(runtime:&Runtime,state:&State,raw:u32,source:asx_gameplay::official::Award,origin:(f32,u8,(u32,u64),bool))->Result<u32,String>{
    let (multiplier,race,collector,terminal)=origin;
    let actors=runtime.party.lock().ok().and_then(|p|p.as_ref().map(|p|p.actors.iter().map(|a|(a.slot,a.guid,a.race_id,a.phase)).collect::<Vec<_>>())).ok_or("Missing gold membership")?;
    let best=actors.iter().map(|a|runtime.adapter.racial_gold_bonus(a.2)).fold(0.0f32,f32::max);
    let mut equipment=vec![];let mut collector_equipment=None;
    for &(slot,guid,_,phase) in &actors{
        if phase==asx_gameplay::Phase::Inactive&&!terminal{continue}
        if let Some(bonus)=cached_armory(&state.armory,slot,guid)?{
            equipment.push(bonus);
            if (slot,guid)==collector{collector_equipment=Some(bonus);}
        }
    }
    let multiplier=if let Some(own)=collector_equipment{
        asx_gameplay::official::shared_gold_equipment_multiplier(multiplier,own,equipment)?
    }else if equipment.is_empty(){multiplier}else{return Err("Gold collector absent from armory membership".into())};
    let multiplier=multiplier-blessings::personal(collector.0,collector.1,2)+blessings::strongest(2);
    shared_gold_award(raw,multiplier,runtime.adapter.racial_gold_bonus(race),best,state.members,source)
}
/// Boss compensation is already a per-participant reward: never divide it.
/// Apply the same current gold provenance once, then credit one transaction.
pub fn duplicate_credit(runtime:&Runtime,game:u64,collector:(u32,u64),base:u32,count:usize)->Result<Vec<u32>,String>{
    if count>64{return Err("Invalid duplicate reward count".into())}
    if count==0{return Ok(vec![])}
    let current=state(game);
    let amount=if let Some(s)=current {
        let (multiplier,race)=runtime.adapter.loot_actor_gold(&runtime.local,game,collector.0,collector.1)?;
        #[cfg(feature="test-hooks")]
        runtime.log.info(&format!("duplicate loot gold provenance: slot={} base={base} multiplier={multiplier} personal_fortune={} shared_fortune={}",collector.0,blessings::personal(collector.0,collector.1,2),blessings::strongest(2)));
        normalized_for_actor(runtime,&s,base,asx_gameplay::official::Award::World,(multiplier,race,collector,runtime.adapter.loot_victory(&runtime.local,game)?))?
    }else{
        if runtime.adapter.solo_run(&runtime.local,game)?.is_none(){return Err("Missing duplicate-loot wallet authority".into())}
        let (_,multiplier,_)=runtime.adapter.current_gold(&runtime.local,game)?;
        asx_gameplay::duplicate_loot::award(base,multiplier)?
    };
    let total=amount.checked_mul(count as u32).ok_or("Duplicate reward total overflow")?;
    let before=if let Some(s)=current{s.wallet}else{runtime.adapter.wallet_seed(&runtime.local,game)?};
    let wallet=before.credit(total)?;
    let io=io::Access{local:&runtime.local,adapter:runtime.adapter};
    if current.is_some(){projected(||runtime.adapter.project_wallet(&io,game,wallet))?;}else{projected(||runtime.adapter.project_current_wallet(&io,game,wallet))?;}
    if let Some(s)=current{commit(s,wallet)?;}
    runtime.log.info(&format!("duplicate loot gold: base={base} count={count} amount={amount} total={total} balance={} earned={}",wallet.gold,wallet.earned));
    Ok(vec![amount;count])
}
