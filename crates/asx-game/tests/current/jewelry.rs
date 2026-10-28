use asx_game::{official::{SelectedAdapter,MutableMemory},official1201 as v,memory::Mem};
use asx_gameplay::enchantments::{Enchantment,Rank,RankedEnchantment};
use std::{cell::{RefCell,Cell},collections::BTreeMap};
const GAME:u64=0x10000;const BASE:u64=0x100000000;
struct Memory {data:RefCell<BTreeMap<u64,Vec<u8>>>,next:Cell<u64>,allocs:Cell<usize>,frees:Cell<usize>}
impl Mem for Memory{fn read(&self,a:u64,n:usize)->Result<Vec<u8>,String>{let d=self.data.borrow();let(b,v)=d.range(..=a).next_back().ok_or("Unmapped")?;v.get((a-b)as usize..(a-b)as usize+n).map(|v|v.to_vec()).ok_or("Outside block".into())}}
impl MutableMemory for Memory{
 fn write(&self,a:u64,b:&[u8])->Result<(),String>{let mut d=self.data.borrow_mut();let(at,v)=d.range_mut(..=a).next_back().ok_or("Unmapped")?;let i=(a-*at)as usize;v.get_mut(i..i+b.len()).ok_or("Outside block")?.copy_from_slice(b);Ok(())}
 fn allocate_native(&self,b:&[u8])->Result<u64,String>{let p=self.next.get();self.next.set(p+0x1000);self.data.borrow_mut().insert(p,b.to_vec());self.allocs.set(self.allocs.get()+1);Ok(p)}
 fn free_native(&self,p:u64,n:usize)->Result<(),String>{let b=self.data.borrow_mut().remove(&p).ok_or("Not owned")?;assert_eq!(b.len(),n);self.frees.set(self.frees.get()+1);Ok(())}
}
fn memory()->Memory{
 let mut root=vec![0;0x2000];root[0x4b0..0x4b8].copy_from_slice(&111u64.to_le_bytes());root[0xd47]=2;
 root[RING_INVENTORY_PTR as usize..RING_INVENTORY_PTR as usize+8].copy_from_slice(&0x30000u64.to_le_bytes());root[RING_INVENTORY_LEN as usize..RING_INVENTORY_LEN as usize+8].copy_from_slice(&1u64.to_le_bytes());
 root[OFFER_PTR as usize..OFFER_PTR as usize+8].copy_from_slice(&0x40000u64.to_le_bytes());root[OFFER_LEN as usize..OFFER_LEN as usize+8].copy_from_slice(&1u64.to_le_bytes());
 root[0x890..0x898].copy_from_slice(&1u64.to_le_bytes());root[0x898..0x8a0].copy_from_slice(&0x50000u64.to_le_bytes());root[0x8a0..0x8a8].copy_from_slice(&1u64.to_le_bytes());
 let mut owned=vec![0;24];owned[..8].copy_from_slice(&(BASE+asx_game::official122::RING_TABLE).to_le_bytes());owned[16..20].copy_from_slice(&0.255f32.to_le_bytes());owned[20..24].copy_from_slice(&1u32.to_le_bytes());
 let mut card=vec![0;0xa0];card[0x78..0x80].copy_from_slice(&0x8000000000000002u64.to_le_bytes());card[0x80..0x88].copy_from_slice(&(BASE+asx_game::official122::RING_TABLE).to_le_bytes());
 let mut defs=vec![0;9*72];let mut keys=vec![0;9*32];for(k,key)in v::RING_KEYS.iter().enumerate(){defs[k*72..k*72+8].copy_from_slice(&(0x60000u64+k as u64*32).to_le_bytes());defs[k*72+8..k*72+16].copy_from_slice(&(key.len()as u64).to_le_bytes());keys[k*32..k*32+key.len()].copy_from_slice(key.as_bytes());}
 Memory{data:RefCell::new([(GAME,root),(0x30000,owned),(0x40000,card),(0x50000,[9u32.to_le_bytes(),0.3f32.to_le_bytes()].concat()),(0x60000,keys),(BASE+asx_game::official122::RING_TABLE,defs)].into()),next:Cell::new(0x80000),allocs:Cell::new(0),frees:Cell::new(0)}
}
fn adapter()->SelectedAdapter{SelectedAdapter::select(asx_game::official122::SHA256).unwrap()}
// Native field positions after the selected 0.122 translator. Production reads
// in these tests go through SelectedAdapter; the constants only seed the fixture.
const RING_INVENTORY_PTR:u64=0x390;const RING_INVENTORY_LEN:u64=0x398;const OFFER_PTR:u64=0x6a8;const OFFER_LEN:u64=0x6b0;const STATS_HEADER:u64=0x890;const RUN_MODE:u64=0xd42;const RUN_PHASE:u64=0xd47;const RUN_ACTOR:u64=0xc30;const RUN_GUID:u64=0x4b0;
#[test]fn acquisition_is_native_and_only_existing_ring_upgrades_are_enchantable(){let m=memory();let a=adapter();let c=a.ring_selected(&m,BASE,GAME,0).unwrap().unwrap();assert_eq!((c.slot,c.guid,c.ring,c.rank),(0,111,0,1));m.write(GAME+RING_INVENTORY_LEN,&0u64.to_le_bytes()).unwrap();assert!(a.ring_selected(&m,BASE,GAME,0).unwrap().is_none());}
#[test]fn trinkets_spell_cards_unreviewed_builds_and_changed_definitions_are_rejected(){let m=memory();let a=adapter();m.write(0x40080,&(BASE+asx_game::official122::RING_TABLE+9*72).to_le_bytes()).unwrap();assert!(a.ring_selected(&m,BASE,GAME,0).unwrap().is_none());m.write(0x40080,&(BASE+asx_game::official122::RING_TABLE).to_le_bytes()).unwrap();m.write(0x40078,&1u64.to_le_bytes()).unwrap();assert!(a.ring_selected(&m,BASE,GAME,0).unwrap().is_none());m.write(0x40078,&0x8000000000000002u64.to_le_bytes()).unwrap();m.write(BASE+asx_game::official122::RING_TABLE+0x45,&[1]).unwrap();assert!(a.ring_selected(&m,BASE,GAME,0).is_err());assert!(a.supports_enchantments());}
#[test]fn native_stats_are_borrowed_once_and_original_permanent_inputs_are_restored(){let m=memory();let a=adapter();let old=m.read(GAME+STATS_HEADER,24).unwrap();let permanent=m.read(0x50000,8).unwrap();for kind in Enchantment::ALL{for rank in Rank::ALL{let e=RankedEnchantment{kind,rank};let loan=a.ring_stats(&m,GAME,&[e]).unwrap().unwrap();let h=m.read(GAME+STATS_HEADER,24).unwrap();let ptr=u64::from_le_bytes(h[8..16].try_into().unwrap());let rows=m.read(ptr,16).unwrap();assert_eq!(&rows[..8],permanent);assert_eq!(u32::from_le_bytes(rows[8..12].try_into().unwrap()),v::RING_STAT_KINDS[e.kind as usize]);assert_eq!(f32::from_le_bytes(rows[12..].try_into().unwrap()),e.value());a.restore_ring_stats(&m,loan).unwrap();assert_eq!(m.read(GAME+STATS_HEADER,24).unwrap(),old);assert_eq!(m.read(0x50000,8).unwrap(),permanent);}}assert_eq!(m.allocs.get(),m.frees.get());}
#[test]fn two_enchantments_have_two_fixed_rows_and_corrupt_native_inputs_allocate_nothing(){let m=memory();let a=adapter();let loan=a.ring_stats(&m,GAME,&[Enchantment::Might.at_ring_rank(2).unwrap(),Enchantment::Might.at_ring_rank(14).unwrap()]).unwrap().unwrap();assert_eq!(m.read(GAME+STATS_HEADER+16,8).unwrap(),3u64.to_le_bytes());a.restore_ring_stats(&m,loan).unwrap();let count=m.allocs.get();m.write(GAME+STATS_HEADER+16,&2u64.to_le_bytes()).unwrap();assert!(a.ring_stats(&m,GAME,&[Enchantment::Might.at_ring_rank(14).unwrap()]).is_err());assert_eq!(m.allocs.get(),count);}
#[test]fn cooperative_current_actor_identity_and_stale_or_invalid_choices_remain_distinct(){let m=memory();let a=adapter();m.write(GAME+RUN_MODE,&[1]).unwrap();m.write(GAME+RUN_ACTOR,&1u64.to_le_bytes()).unwrap();m.write(GAME+RUN_GUID,&222u64.to_le_bytes()).unwrap();let c=a.ring_selected(&m,BASE,GAME,0).unwrap().unwrap();assert_eq!((c.slot,c.guid),(1,222));m.write(GAME+RUN_PHASE,&[1]).unwrap();assert!(a.ring_selected(&m,BASE,GAME,0).unwrap().is_none());m.write(GAME+RUN_PHASE,&[2]).unwrap();assert!(a.ring_selected(&m,BASE,GAME,2).unwrap().is_none());}

fn amulet_memory()->Memory{
 let m=memory();m.write(GAME+0x3e8,&u64::MAX.to_le_bytes()).unwrap();
 let mut defs=vec![0;16*80];let mut keys=vec![0;16*64];
 for(n,key)in v::AMULET_KEYS.iter().enumerate(){defs[n*80..n*80+8].copy_from_slice(&(0xa0000u64+n as u64*64).to_le_bytes());defs[n*80+8..n*80+16].copy_from_slice(&(key.len()as u64).to_le_bytes());keys[n*64..n*64+key.len()].copy_from_slice(key.as_bytes());}
 let mut amulet=vec![0;0x70];amulet[8..16].copy_from_slice(&0xa0000u64.to_le_bytes());amulet[16..24].copy_from_slice(&(v::AMULET_KEYS[0].len()as u64).to_le_bytes());
 {let mut d=m.data.borrow_mut();d.insert(BASE+asx_game::official122::AMULET_TABLE,defs);d.insert(0xa0000,keys);d.insert(0x90000,amulet);}
 m.write(0x40078,&v::AMULET_CHOICE_TAG.to_le_bytes()).unwrap();m.write(0x40080,&0x90000u64.to_le_bytes()).unwrap();m
}
#[test]fn first_amulet_and_replacement_decode_native_owned_identity_without_ranks(){let m=amulet_memory();let a=adapter();let c=a.amulet_selected(&m,BASE,GAME,0).unwrap().unwrap();assert_eq!((c.slot,c.guid,c.amulet,c.previous),(0,111,0,None));assert_eq!(a.amulet_selected(&m,BASE,GAME,0).unwrap(),Some(c));let mut owned=m.read(0x90000,0x70).unwrap();owned[8..16].copy_from_slice(&0xa0040u64.to_le_bytes());owned[16..24].copy_from_slice(&(v::AMULET_KEYS[1].len()as u64).to_le_bytes());m.write(GAME+0x3e8,&owned).unwrap();m.write(GAME+0x450,&1u32.to_le_bytes()).unwrap();assert_eq!(a.amulet_selected(&m,BASE,GAME,0).unwrap().unwrap().previous,Some(1));m.write(GAME+0x3e8,&m.read(0x90000,0x70).unwrap()).unwrap();m.write(GAME+0x450,&1u32.to_le_bytes()).unwrap();assert_eq!(a.amulet_selected(&m,BASE,GAME,0).unwrap().unwrap().previous,Some(0));}
#[test]fn amulet_invalid_card_pointer_key_and_client_fail_closed(){let m=amulet_memory();let a=adapter();m.write(0x40080,&0u64.to_le_bytes()).unwrap();assert!(a.amulet_selected(&m,BASE,GAME,0).is_err());m.write(0x40080,&0x90000u64.to_le_bytes()).unwrap();m.write(0x90010,&999u64.to_le_bytes()).unwrap();assert!(a.amulet_selected(&m,BASE,GAME,0).is_err());assert!(SelectedAdapter::select(asx_game::official121::SHA256).is_err());}
#[test]fn gem_and_ring_inputs_coexist_and_restore_without_touching_permanent_rows(){use asx_gameplay::gems::Gem;let m=memory();let a=adapter();let header=m.read(GAME+STATS_HEADER,24).unwrap();let baseline=m.read(0x50000,8).unwrap();for gem in Gem::ALL{let ring=a.ring_stats(&m,GAME,&[Enchantment::Might.at_ring_rank(2).unwrap()]).unwrap().unwrap();let ringheader=m.read(GAME+STATS_HEADER,24).unwrap();let loan=a.gem_stats(&m,GAME,gem).unwrap().unwrap();let h=m.read(GAME+STATS_HEADER,24).unwrap();assert_eq!(u64::from_le_bytes(h[16..].try_into().unwrap()),3);let p=u64::from_le_bytes(h[8..16].try_into().unwrap());let rows=m.read(p,24).unwrap();assert_eq!(&rows[..8],baseline);assert_eq!(u32::from_le_bytes(rows[16..20].try_into().unwrap()),v::RING_STAT_KINDS[gem as usize]);assert_eq!(f32::from_le_bytes(rows[20..24].try_into().unwrap()),gem.value());a.restore_ring_stats(&m,loan).unwrap();assert_eq!(m.read(GAME+STATS_HEADER,24).unwrap(),ringheader);a.restore_ring_stats(&m,ring).unwrap();assert_eq!(m.read(GAME+STATS_HEADER,24).unwrap(),header);assert_eq!(m.read(0x50000,8).unwrap(),baseline);}assert_eq!(m.allocs.get(),m.frees.get());}

#[test]fn native_spell_specific_legendary_amulets_are_supported_without_rewriting_them(){let m=amulet_memory();let a=adapter();let key=b"legendary:plague:rend";m.data.borrow_mut().insert(0xb0000,key.to_vec());m.write(0x90008,&0xb0000u64.to_le_bytes()).unwrap();m.write(0x90010,&(key.len()as u64).to_le_bytes()).unwrap();let c=a.amulet_selected(&m,BASE,GAME,0).unwrap().unwrap();assert!(c.amulet>=1<<63);let permanent=m.read(0x90000,0x70).unwrap();assert_eq!(a.amulet_selected(&m,BASE,GAME,0).unwrap(),Some(c));assert_eq!(m.read(0x90000,0x70).unwrap(),permanent);m.data.borrow_mut().insert(0xb0000,b"legendary:bad key!!!!".to_vec());assert!(a.amulet_selected(&m,BASE,GAME,0).is_err());}

#[test]fn native_kept_tooltip_descriptor_cannot_collide_with_an_official_item(){
 use asx_gameplay::kept_bonus::KeptBonus;use asx_gameplay::gems::Gem;
 let m=memory();let a=adapter();let bonus=KeptBonus::Gem(Gem::Ruby);
 let desc=a.kept_tooltip_descriptor(bonus).unwrap();m.data.borrow_mut().insert(0xc0000,desc.into_iter().flat_map(u64::to_le_bytes).collect());
 m.data.borrow_mut().insert(0xc1000,[0x20000u64,0x30000,0xc0000].into_iter().flat_map(u64::to_le_bytes).collect());
 assert_eq!(a.kept_tooltip_body(&m,0xc1000).unwrap(),Some((0x20000,bonus)));
 m.write(0xc0048,&3u64.to_le_bytes()).unwrap();assert_eq!(a.kept_tooltip_body(&m,0xc1000).unwrap(),None);
 m.write(0xc0048,&v::KEPT_TOOLTIP_MARKER.to_le_bytes()).unwrap();m.write(0xc0050,&65535u64.to_le_bytes()).unwrap();assert!(a.kept_tooltip_body(&m,0xc1000).is_err());
 assert!(adapter().kept_tooltip_descriptor(bonus).is_some());
}
#[test]fn native_tooltip_command_owns_only_the_entity_and_native_component_flags(){
 let m=memory();let a=adapter();
 m.data.borrow_mut().insert(0xc0000,[0x1234567800000011u64,0,0,0xc1000,0,0].into_iter().flat_map(u64::to_le_bytes).collect());
 m.data.borrow_mut().insert(0xc1000,[64u64,0xc2000,8].into_iter().flat_map(u64::to_le_bytes).collect());m.data.borrow_mut().insert(0xc2000,vec![0xaa;64]);
 unsafe{a.kept_tooltip_target(&m,BASE as usize,0xc0000).unwrap()};
 assert_eq!(m.read(0xc1010,8).unwrap(),32u64.to_le_bytes());let command=m.read(0xc2008,24).unwrap();assert_eq!(&command[..8],&(BASE+asx_game::official122::KEPT_TOOLTIP_BINDINGS[3].0).to_le_bytes());assert_eq!(&command[8..16],&0x1234567800000011u64.to_le_bytes());assert_eq!(&command[16..19],&[0,2,1]);assert_eq!(m.read(0xc2000,8).unwrap(),vec![0xaa;8]);
 m.write(0xc1010,&65u64.to_le_bytes()).unwrap();assert!(unsafe{a.kept_tooltip_target(&m,BASE as usize,0xc0000)}.is_err());
}

#[test]fn native_tooltip_reserve_preserves_abi_and_uses_relocated_queue_storage(){
 thread_local!{static MEMORY:Cell<*const Memory>=const{Cell::new(std::ptr::null())};static ARGS:RefCell<Vec<u64>>=const{RefCell::new(Vec::new())};}
 unsafe extern "system" fn reserve(queue:u64,len:u64,additional:u64,size:u64,align:u64){
  ARGS.with(|v|*v.borrow_mut()=vec![queue,len,additional,size,align]);
  MEMORY.with(|v|{let m=&*v.get();m.write(queue,&96u64.to_le_bytes()).unwrap();m.write(queue+8,&0xc3000u64.to_le_bytes()).unwrap();});
 }
 let m=memory();let a=adapter();MEMORY.with(|v|v.set(&m));
 m.data.borrow_mut().insert(0xc0000,[42u64,0,0,0,0xc1000,0].into_iter().flat_map(u64::to_le_bytes).collect());
 m.data.borrow_mut().insert(0xc1000,[16u64,0xc2000,8].into_iter().flat_map(u64::to_le_bytes).collect());m.data.borrow_mut().insert(0xc2000,vec![0xaa;16]);m.data.borrow_mut().insert(0xc3000,vec![0xbb;96]);
 let base=(reserve as *const ()as usize).wrapping_sub(asx_game::official122::KEPT_TOOLTIP_BINDINGS[4].0 as usize);
 unsafe{a.kept_tooltip_target(&m,base,0xc0000).unwrap()};
 assert_eq!(ARGS.with(|v|v.borrow().clone()),[0xc1000,8,24,1,1]);assert_eq!(m.read(0xc1010,8).unwrap(),32u64.to_le_bytes());assert_eq!(&m.read(0xc3008,24).unwrap()[8..16],&42u64.to_le_bytes());assert_eq!(m.read(0xc2000,16).unwrap(),vec![0xaa;16]);MEMORY.with(|v|v.set(std::ptr::null()));
}
