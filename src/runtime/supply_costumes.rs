//! Personal Lost Supplies appearances; only render fields are broadcast.
use super::Runtime;
use azsx_game::official::ActorObservation;
use azsx_gameplay::{Phase,supplies::Costume};

#[derive(Default,Clone,Copy)]
pub struct Look {
    pub effect:Option<Costume>, expires:f32, next:f32,
    display:Option<(u32,u32)>, small:bool,
}
impl Look{
 pub fn observe(&mut self,now:f32,a:&ActorObservation,grant:Option<Costume>)->Option<Self>{
    if let Some(c)=grant{self.effect=Some(c);self.expires=now+Costume::SECONDS;self.next=0.;}
    if self.effect.is_none()&&self.display.is_none()&&!self.small{return None}
    if now>=self.expires||a.hp<=0.||a.phase==Phase::Downed{self.effect=None;}
    if now<self.next{return None}self.next=now+0.25;Some(*self)
 }
 pub fn apply(mut self,r:&Runtime,game:u64,id:(u32,u64))->Result<Self,String>{
    let small=self.effect==Some(Costume::Small);
    let costume=self.effect.filter(|c|*c!=Costume::Small);
    if costume.is_some()||self.display.is_some(){
        let native=r.adapter.supply_display(&r.local,game,id,None)?;
        let shown=if costume.is_some(){r.adapter.supply_display(&r.local,game,id,costume)?}else{native};
        if self.display!=Some((native,shown))||costume.is_none(){
            unsafe{r.adapter.supply_display_packet(&r.local,r.local.base as u64,game,id.1,shown)}?;
            #[cfg(feature="test-hooks")]r.log.info(&format!("Supply costume: slot={} native={native} display={shown}",id.0));
        }
        self.display=costume.map(|_|(native,shown));
    }
    if small!=self.small{
        let scale=if small{0.5}else{super::solo_gadgets::current_scale(game,id.1)};
        unsafe{r.adapter.solo_player_scale(&r.local,r.local.base as u64,game,id.1,scale)}?;
        self.small=small;
    }
    Ok(self)
 }
}
pub(super) fn bonus(_seq:u64,roll:impl FnOnce()->Option<Costume>)->Option<Costume>{
 #[cfg(feature="test-hooks")]if let Ok(value)=std::env::var("AZSX_TEST_SUPPLY_COSTUME"){
  return if matches!(_seq,1|3){match value.as_str(){"skeleton"=>Some(Costume::Skeleton),"small"=>Some(Costume::Small),"pirate"=>Some(Costume::Pirate),"ninja"=>Some(Costume::Ninja),_=>None}}else{None};
 }
 roll()
}
