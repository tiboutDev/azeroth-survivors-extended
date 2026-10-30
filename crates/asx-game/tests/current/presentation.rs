use asx_game::{official::{SelectedAdapter,MutableMemory},memory::Mem};
use std::{cell::{RefCell,Cell},collections::BTreeMap};
struct Memory {data:RefCell<BTreeMap<u64,Vec<u8>>>,next:Cell<u64>,allocs:Cell<usize>,frees:Cell<usize>}
impl Mem for Memory{fn read(&self,a:u64,n:usize)->Result<Vec<u8>,String>{let d=self.data.borrow();let(b,v)=d.range(..=a).next_back().ok_or("Unmapped")?;v.get((a-b)as usize..(a-b)as usize+n).map(|v|v.to_vec()).ok_or("Outside block".into())}}
impl MutableMemory for Memory{
 fn write(&self,a:u64,b:&[u8])->Result<(),String>{let mut d=self.data.borrow_mut();let(at,v)=d.range_mut(..=a).next_back().ok_or("Unmapped")?;let i=(a-*at)as usize;v.get_mut(i..i+b.len()).ok_or("Outside block")?.copy_from_slice(b);Ok(())}
 fn allocate_native(&self,b:&[u8])->Result<u64,String>{let p=self.next.get();self.next.set(p+0x1000);self.data.borrow_mut().insert(p,b.to_vec());self.allocs.set(self.allocs.get()+1);Ok(p)}
 fn free_native(&self,p:u64,n:usize)->Result<(),String>{let b=self.data.borrow_mut().remove(&p).ok_or("Not owned")?;assert_eq!(b.len(),n);self.frees.set(self.frees.get()+1);Ok(())}
}
fn memory()->Memory{Memory{data:RefCell::new(BTreeMap::new()),next:Cell::new(0x80000),allocs:Cell::new(0),frees:Cell::new(0)}}
fn adapter()->SelectedAdapter{SelectedAdapter::select(asx_game::current::SHA256).unwrap()}

#[test]fn separator_style_is_captured_only_from_the_native_card_callsite(){
 let m=memory();let a=adapter();
 m.data.borrow_mut().insert(0xd0000,vec![0;asx_game::current::NODE_SIZE]);
 m.data.borrow_mut().insert(0xd1000,vec![0x11;20]);
 assert!(a.bonus_separator_capture(&m,asx_game::current::BONUS_SEPARATOR_CALLER+1,0xd0000,0xd1000).unwrap().is_none());
 assert!(a.bonus_separator_capture(&m,asx_game::current::BONUS_SEPARATOR_CALLER,0xd0000,0xd1000).unwrap().is_some());
 assert!(a.bonus_separator_capture(&m,asx_game::current::BONUS_SEPARATOR_CALLER,0xd0000,0xd2000).is_err());
 m.write(0xd0000,&1u64.to_le_bytes()).unwrap();assert!(a.bonus_separator_capture(&m,asx_game::current::BONUS_SEPARATOR_CALLER,0xd0000,0xd1000).is_err());
}

#[test]fn healing_text_borrows_native_writer_and_transfers_one_owned_message(){
 unsafe extern "system" fn entity(guid:u64,objects:u64,exclude_self:u64,self_guid:u64,query:*const u8)->u64{
  assert_eq!((guid,objects,exclude_self,self_guid),(11,0xe3000,0,0));assert_eq!(std::slice::from_raw_parts(query,24),&[5;24]);0x100000019
 }
 let m=memory();let a=adapter();let bp=0xe0000;
 for(at,n)in [(bp,0x1000),(0xe1000,0x80),(0xe2000,0x200),(0xe4000,4)]{m.data.borrow_mut().insert(at,vec![0;n]);}
 m.write(bp+0x228,&0xe3000u64.to_le_bytes()).unwrap();m.write(bp+0x368,&[5;24]).unwrap();m.write(bp+0x3b8,&0xe1000u64.to_le_bytes()).unwrap();m.write(bp+0x3c8,&0xe4000u64.to_le_bytes()).unwrap();m.write(bp+0x3d4,&17u32.to_le_bytes()).unwrap();
 for(at,n)in [(0x20,4u64),(0x28,0xe2000),(0x30,1),(0x40,9)]{m.write(0xe1000+at,&n.to_le_bytes()).unwrap();}
 let base=entity as *const() as usize-asx_game::current::TEXT_ENTITY.0 as usize;
 assert!(unsafe{a.draw_native_heal_text(&m,base,bp,false,asx_game::official_native::HealText{target:11,caster:22,amount:37,critical:true})}.unwrap());
 assert_eq!(m.read(0xe1030,8).unwrap(),2u64.to_le_bytes());assert_eq!(m.read(0xe1040,8).unwrap(),10u64.to_le_bytes());assert_eq!(m.read(0xe4000,4).unwrap(),17u32.to_le_bytes());assert_eq!(m.read(0xe2000,0x38).unwrap(),vec![0;0x38]);
 let row=m.read(0xe2038,0x38).unwrap();let ptr=u64::from_le_bytes(row[8..16].try_into().unwrap());assert_eq!(m.read(ptr,3).unwrap(),b"+37");assert_eq!(m.allocs.get(),1);assert_eq!(row[40],2);assert_eq!(&row[32..40],&0x100000019u64.to_le_bytes());assert_eq!(&row[48..],&9u64.to_le_bytes());
}

#[test]fn native_participants_use_current_native_aliases_without_pets_or_duplicates(){
 let m=memory();let a=adapter();let script=0x100000;let aliases=script+asx_game::current::UI_ALIASES;
 m.data.borrow_mut().insert(script,vec![0;0x7000]);
 m.write(aliases+asx_game::current::UI_SELF_GUID,&11u64.to_le_bytes()).unwrap();
 for(i,id)in [22u64,11,0,33].into_iter().enumerate(){m.write(aliases+asx_game::current::UI_PARTY_GUIDS+i as u64*8,&id.to_le_bytes()).unwrap();}
 assert_eq!(a.native_ui_participants(&m,script).unwrap(),vec![11,22,33]);
 m.write(aliases+asx_game::current::UI_SELF_GUID,&0u64.to_le_bytes()).unwrap();assert!(a.native_ui_participants(&m,script).unwrap().is_empty());
}
#[test]fn bonus_title_column_consumes_a_fresh_native_node_and_reparents_only_its_title(){
 unsafe extern "system" fn spawn(out:*mut u64,commands:u64,bundle:*mut u64,meta:u64)->u64{
  assert_eq!((commands,meta),(0xe10000,0x20000));assert_eq!(*bundle,0x100000019);
  let node=std::slice::from_raw_parts(bundle.add(1).cast::<u8>(),0x180);
  assert_eq!(&node[0x88..0x98],&[0;16]);assert_eq!(node[0x176],1);assert_eq!(node[asx_game::current::BONUS_NODE_ALIGN_ITEMS],1);assert_eq!(&node[0x140..0x148],&[1,0,0,0,0,0,0,0]);
  std::ptr::copy_nonoverlapping([0x200000019,10,20,30,40,50].as_ptr(),out,6);out as u64
 }
 let m=memory();let a=adapter();m.data.borrow_mut().insert(0xd0000,vec![0;0x188]);m.data.borrow_mut().insert(0xe0000,vec![0x55;0x100]);
 m.write(0xe0000,&0x100000019u64.to_le_bytes()).unwrap();let original=m.read(0xe0000,0x100).unwrap();
 assert!(a.bonus_column_capture(&m,0,0xd0000,0x20000).unwrap().is_none());
 let style=a.bonus_column_capture(&m,asx_game::current::BONUS_HEADER_CALLER,0xd0000,0x20000).unwrap().unwrap();
 let base=spawn as *const()as usize-asx_game::current::BONUS_COLUMN_SPAWN.0 as usize;
 let child=unsafe{a.bonus_column_draw(&m,base,0xe10000,0xe0000,style)}.unwrap();assert_eq!(child,[0x200000019,10,20,30,40,50]);
 let mut expected=original;expected[..8].copy_from_slice(&child[0].to_le_bytes());expected[0x50..0x54].copy_from_slice(&13f32.to_le_bytes());assert_eq!(m.read(0xe0000,0x100).unwrap(),expected);
 m.write(0xd0008,&1u64.to_le_bytes()).unwrap();assert!(a.bonus_column_capture(&m,asx_game::current::BONUS_HEADER_CALLER,0xd0000,0x20000).is_err());
}

#[test]fn deferred_bonus_caption_reborrows_live_commands_without_retaining_a_parent_pointer(){
 let m=memory();let a=adapter();let original=[11u64,22,33,44,55,66];m.data.borrow_mut().insert(0xd0000,original.into_iter().flat_map(u64::to_le_bytes).collect());
 assert_eq!(a.bonus_column_spawner(&m,0xd0000,99).unwrap(),[99,22,33,44,55,66]);assert_eq!(m.read(0xd0000,48).unwrap(),original.into_iter().flat_map(u64::to_le_bytes).collect::<Vec<_>>());
}

#[test]fn fractional_heal_publication_transfers_one_packet_without_mutating_gameplay(){
 unsafe extern "system" fn packet(out:*mut u8,target:u64,caster:u64,spell:u32,amount:u32,critical:bool)->u64{
  assert_eq!((target,caster,spell,amount,critical),(22,11,139,9,false));
  let value=[32u64,0xe2000,25,0x150];std::ptr::copy_nonoverlapping(value.as_ptr().cast::<u8>(),out,32);out as u64
 }
 let m=memory();let a=adapter();let game=0xe0000;let queue=game+asx_game::current::HEAL_LOG_QUEUE;
 m.data.borrow_mut().insert(game,vec![0;0x800]);m.data.borrow_mut().insert(0xe1000,vec![0;64]);m.data.borrow_mut().insert(0xe2000,vec![0;32]);
 m.write(queue,&2u64.to_le_bytes()).unwrap();m.write(queue+8,&0xe1000u64.to_le_bytes()).unwrap();
 let before=m.read(game,0x800).unwrap();let base=packet as *const()as usize-asx_game::current::HEAL_LOG_PACKET.0 as usize;
 unsafe{a.emit_fractional_heal_text(&m,base,game,asx_game::official_native::HealText{caster:11,target:22,amount:9,critical:false},139)}.unwrap();
 let mut expected=before;let length=(asx_game::current::HEAL_LOG_QUEUE+16)as usize;expected[length..length+8].copy_from_slice(&1u64.to_le_bytes());
 assert_eq!(m.read(game,0x800).unwrap(),expected);assert_eq!(m.read(0xe1000,32).unwrap(),[32u64,0xe2000,25,0x150].into_iter().flat_map(u64::to_le_bytes).collect::<Vec<_>>());assert_eq!(m.frees.get(),0);
 assert!(unsafe{a.emit_fractional_heal_text(&m,base,game,asx_game::official_native::HealText{caster:11,target:22,amount:0,critical:false},139)}.is_err());
}

#[test]
fn addon_extension_uses_asx_and_rejects_duplicate_fields_without_allocation() {
    let m = memory();
    let a = adapter();
    let output = 0xd0000;
    m.data.borrow_mut().insert(output, vec![0; 100]);
    m.data.borrow_mut().insert(0xd1000, b"AZSRUN".to_vec());
    m.data.borrow_mut().insert(0xd2000, b"t=42".to_vec());
    for (offset, value) in [(40, 0xd1000u64), (48, 6), (56, 4), (64, 0xd2000), (72, 4)] {
        m.write(output + offset, &value.to_le_bytes()).unwrap();
    }
    a.decorate_party_addon(&m, output, "0200").unwrap();
    let header = m.read(output + 56, 24).unwrap();
    let ptr = u64::from_le_bytes(header[8..16].try_into().unwrap());
    let len = u64::from_le_bytes(header[16..24].try_into().unwrap());
    assert_eq!(m.read(ptr, len as usize).unwrap(), b"t=42;asx=0200");
    let allocations = m.allocs.get();
    assert!(a.decorate_party_addon(&m, output, "0201").is_err());
    assert_eq!(m.allocs.get(), allocations);
    assert_eq!(m.read(output + 56, 24).unwrap(), header);
}

#[test]
fn minimap_projection_needs_no_readable_native_zoom_fields() {
    let m=memory();let a=adapter();let script=0xd0000;
    let aliases=script+asx_game::current::UI_ALIASES;
    m.data.borrow_mut().insert(aliases+asx_game::current::UI_SELF_GUID,11u64.to_le_bytes().to_vec());
    m.data.borrow_mut().insert(aliases+asx_game::current::UI_PARTY_GUIDS,vec![0;32]);
    m.data.borrow_mut().insert(script+0x6640,[3f32,4.].into_iter().flat_map(f32::to_le_bytes).collect());
    m.data.borrow_mut().insert(script+0x6648,vec![1]);
    m.data.borrow_mut().insert(script+0x664c,[10f32,20.].into_iter().flat_map(f32::to_le_bytes).collect());
    m.data.borrow_mut().insert(script+0x665c,vec![0]);
    // Both legacy zoom fields deliberately remain unmapped.
    assert!(m.read(script+0x6654,4).is_err());assert!(m.read(script+0x6658,4).is_err());
    let outside=a.event_minimap(&m,script,11).unwrap().unwrap();
    assert_eq!(outside.center,[10.,20.]);assert_eq!(outside.ping,[3.,4.]);
    assert!((outside.radii[0]-233.33333).abs()<0.001);
    m.write(script+0x665c,&[1]).unwrap();
    let inside=a.event_minimap(&m,script,11).unwrap().unwrap();
    assert_eq!(inside.radii,[150.,120.,90.,60.,40.,25.]);
}
