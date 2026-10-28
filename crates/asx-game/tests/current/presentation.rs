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
fn adapter()->SelectedAdapter{SelectedAdapter::select(asx_game::official125::SHA256).unwrap()}

#[test]fn separator_style_is_captured_only_from_the_native_card_callsite(){
 let m=memory();let a=adapter();
 m.data.borrow_mut().insert(0xd0000,vec![0;asx_game::official125::NODE_SIZE]);
 m.data.borrow_mut().insert(0xd1000,vec![0x11;20]);
 assert!(a.bonus_separator_capture(&m,asx_game::official125::BONUS_SEPARATOR_CALLER+1,0xd0000,0xd1000).unwrap().is_none());
 assert!(a.bonus_separator_capture(&m,asx_game::official125::BONUS_SEPARATOR_CALLER,0xd0000,0xd1000).unwrap().is_some());
 assert!(a.bonus_separator_capture(&m,asx_game::official125::BONUS_SEPARATOR_CALLER,0xd0000,0xd2000).is_err());
 m.write(0xd0000,&1u64.to_le_bytes()).unwrap();assert!(a.bonus_separator_capture(&m,asx_game::official125::BONUS_SEPARATOR_CALLER,0xd0000,0xd1000).is_err());
}

#[test]fn healing_text_borrows_native_writer_and_transfers_one_owned_message(){
 unsafe extern "system" fn entity(guid:u64,objects:u64,exclude_self:u64,self_guid:u64,query:*const u8)->u64{
  assert_eq!((guid,objects,exclude_self,self_guid),(11,0xe3000,0,0));assert_eq!(std::slice::from_raw_parts(query,24),&[5;24]);0x100000019
 }
 let m=memory();let a=adapter();let bp=0xe0000;
 for(at,n)in [(bp,0x1000),(0xe1000,0x80),(0xe2000,0x200),(0xe4000,4)]{m.data.borrow_mut().insert(at,vec![0;n]);}
 m.write(bp+0x228,&0xe3000u64.to_le_bytes()).unwrap();m.write(bp+0x368,&[5;24]).unwrap();m.write(bp+0x3b8,&0xe1000u64.to_le_bytes()).unwrap();m.write(bp+0x3c8,&0xe4000u64.to_le_bytes()).unwrap();m.write(bp+0x3d4,&17u32.to_le_bytes()).unwrap();
 for(at,n)in [(0x20,4u64),(0x28,0xe2000),(0x30,1),(0x40,9)]{m.write(0xe1000+at,&n.to_le_bytes()).unwrap();}
 let base=entity as *const() as usize-asx_game::official125::TEXT_ENTITY.0 as usize;
 assert!(unsafe{a.draw_native_heal_text(&m,base,bp,false,asx_game::official_native::HealText{target:11,caster:22,amount:37,critical:true})}.unwrap());
 assert_eq!(m.read(0xe1030,8).unwrap(),2u64.to_le_bytes());assert_eq!(m.read(0xe1040,8).unwrap(),10u64.to_le_bytes());assert_eq!(m.read(0xe4000,4).unwrap(),17u32.to_le_bytes());assert_eq!(m.read(0xe2000,0x38).unwrap(),vec![0;0x38]);
 let row=m.read(0xe2038,0x38).unwrap();let ptr=u64::from_le_bytes(row[8..16].try_into().unwrap());assert_eq!(m.read(ptr,3).unwrap(),b"+37");assert_eq!(m.allocs.get(),1);assert_eq!(row[40],2);assert_eq!(&row[32..40],&0x100000019u64.to_le_bytes());assert_eq!(&row[48..],&9u64.to_le_bytes());
}

#[test]fn native_plate_inputs_hide_only_normal_enemy_labels_before_the_first_draw(){
 let m=memory();let a=adapter();let mut rows=vec![0;0x58*2];
 for i in 0..2{let at=i*0x58;rows[at+0x10..at+0x18].copy_from_slice(&5u64.to_le_bytes());rows[at+0x18..at+0x1c].copy_from_slice(&1u32.to_le_bytes());rows[at+0x52]=1;rows[at+0x54]=1;rows[at+0x38+i*4..at+0x3c+i*4].copy_from_slice(&1f32.to_le_bytes());}
 let script=0x100000;let style=0x130000;
 m.data.borrow_mut().insert(script,vec![0;0x7000]);
 let config=[110f32,14.,2.,2.,100.,10.,3.,1.,12.,16.,12.,10.,2.].into_iter().flat_map(f32::to_le_bytes).collect::<Vec<_>>();m.data.borrow_mut().insert(style,config);
 m.data.borrow_mut().insert(0xf0000,rows.clone());let modified=unsafe{a.nameplate_rows(&m,0,script,0xf0000,2)}.unwrap().unwrap();assert_eq!(m.read(0xf0000,rows.len()).unwrap(),rows);
 assert_eq!(&modified[0x10..0x1c],&[0;12]);assert_eq!((modified[0x52],modified[0x54]),(0,1));assert_eq!(&modified[0x58..],&rows[0x58..]);assert_eq!(&modified[0x30..0x52],&rows[0x30..0x52]);
 assert_eq!(&modified[0x28..0x30],&rows[0x28..0x30]);
 assert!(unsafe{a.nameplate_rows(&m,0,script,0xf0000,257)}.is_err());
}
#[test]fn native_participants_use_current_native_aliases_without_pets_or_duplicates(){
 let m=memory();let a=adapter();let script=0x100000;let aliases=script+asx_game::official125::UI_ALIASES;
 m.data.borrow_mut().insert(script,vec![0;0x7000]);
 m.write(aliases+asx_game::official125::UI_SELF_GUID,&11u64.to_le_bytes()).unwrap();
 for(i,id)in [22u64,11,0,33].into_iter().enumerate(){m.write(aliases+asx_game::official125::UI_PARTY_GUIDS+i as u64*8,&id.to_le_bytes()).unwrap();}
 assert_eq!(a.native_ui_participants(&m,script).unwrap(),vec![11,22,33]);
 m.write(aliases+asx_game::official125::UI_SELF_GUID,&0u64.to_le_bytes()).unwrap();assert!(a.native_ui_participants(&m,script).unwrap().is_empty());
}
#[test]fn native_unit_rank_uses_the_guid_key_and_checks_native_rank(){
 unsafe extern "system" fn hash(_:u64,guid:*const u64)->u64{assert_eq!(*guid,77);0}
 let m=memory();let a=adapter();let script=0x100000;let ctrl=0x120000;let map=script+asx_game::official125::UI_UNIT_MAP;
 m.data.borrow_mut().insert(script,vec![0;0x7000]);m.data.borrow_mut().insert(ctrl-16*asx_game::official125::UI_UNIT_STRIDE,vec![0;16*asx_game::official125::UI_UNIT_STRIDE as usize+32]);
 m.write(map,&ctrl.to_le_bytes()).unwrap();m.write(map+8,&15u64.to_le_bytes()).unwrap();m.write(map+24,&1u64.to_le_bytes()).unwrap();m.write(ctrl,&[0xff;32]).unwrap();m.write(ctrl,&[0]).unwrap();
 let slot=ctrl-asx_game::official125::UI_UNIT_STRIDE;m.write(slot,&77u64.to_le_bytes()).unwrap();m.write(slot+8+asx_game::official125::UI_UNIT_RANK,&3u32.to_le_bytes()).unwrap();
 let base=hash as *const()as usize-asx_game::official125::UI_UNIT_HASH.0 as usize;
 assert_eq!(unsafe{a.native_unit_rank(&m,base,script,77)}.unwrap(),Some(3));
 m.write(slot,&88u64.to_le_bytes()).unwrap();assert_eq!(unsafe{a.native_unit_rank(&m,base,script,77)}.unwrap(),None);
}
#[test]fn native_plate_geometry_uses_a_fresh_settings_copy_and_keeps_its_cache_baseline(){
 let m=memory();let a=adapter();let(script,cache,bp,style)=(0x100000u64,0x110000u64,0x120000u64,0x130000u64);
 for(at,n)in [(script,0x5000),(cache,0xe0),(bp,0x400),(style,52)]{m.data.borrow_mut().insert(at,vec![0;n]);}
 m.write(script+asx_game::official125::PLATE_CACHE_PTR,&cache.to_le_bytes()).unwrap();m.write(script+asx_game::official125::PLATE_CACHE_LEN,&1u64.to_le_bytes()).unwrap();m.write(bp+0x290,&cache.to_le_bytes()).unwrap();m.write(cache+0x38,&1f32.to_le_bytes()).unwrap();
 let original=[110f32,14.,2.,2.,106.,10.,3.,1.,12.,16.,12.,10.,2.].into_iter().flat_map(f32::to_le_bytes).collect::<Vec<_>>();m.write(style,&original).unwrap();
 for _ in 0..3{let(scaled,dest,saved)=a.nameplate_style(&m,asx_game::official125::PLATE_STYLE_CALLERS[0],bp,script,0,style).unwrap().unwrap();assert_eq!(scaled[0],110.*0.70);assert_eq!(scaled[10],12.);assert_eq!(dest,cache+0x5c);assert_eq!(saved.as_slice(),original);assert_eq!(m.read(style,52).unwrap(),original);a.restore_nameplate_style_cache(&m,dest,&saved).unwrap();}
 assert!(a.nameplate_style(&m,0,bp,script,0,style).unwrap().is_none());
}

#[test]fn native_special_enemy_names_are_preserved_while_levels_are_hidden(){
 unsafe extern "system" fn hash(_:u64,_:*const u64)->u64{0}
 let m=memory();let a=adapter();let script=0x100000;let ctrl=0x120000;let style=0x130000;let map=script+asx_game::official125::UI_UNIT_MAP;
 m.data.borrow_mut().insert(script,vec![0;0x7000]);m.data.borrow_mut().insert(ctrl-16*asx_game::official125::UI_UNIT_STRIDE,vec![0;16*asx_game::official125::UI_UNIT_STRIDE as usize+32]);
 m.write(map,&ctrl.to_le_bytes()).unwrap();m.write(map+8,&15u64.to_le_bytes()).unwrap();m.write(map+24,&1u64.to_le_bytes()).unwrap();m.write(ctrl,&[0xff;32]).unwrap();m.write(ctrl,&[0]).unwrap();
 let slot=ctrl-asx_game::official125::UI_UNIT_STRIDE;m.write(slot,&77u64.to_le_bytes()).unwrap();
 let mut row=vec![0;0x58];row[0x10..0x18].copy_from_slice(&5u64.to_le_bytes());row[0x20..0x28].copy_from_slice(&77u64.to_le_bytes());row[0x38..0x3c].copy_from_slice(&1f32.to_le_bytes());row[0x18]=1;row[0x52]=1;row[0x54]=1;m.data.borrow_mut().insert(0xf0000,row.clone());
 m.data.borrow_mut().insert(style,[110f32,14.,2.,2.,100.,10.,3.,1.,12.,16.,12.,10.,2.].into_iter().flat_map(f32::to_le_bytes).collect());
 let base=hash as *const()as usize-asx_game::official125::UI_UNIT_HASH.0 as usize;
 for rank in 1u32..=4{m.write(slot+8+asx_game::official125::UI_UNIT_RANK,&rank.to_le_bytes()).unwrap();let changed=unsafe{a.nameplate_rows(&m,base,script,0xf0000,1)}.unwrap().unwrap();assert_eq!(&changed[..0x18],&row[..0x18]);assert_eq!(changed[0x18],0);assert_eq!(changed[0x52],0);assert_eq!(changed[0x54],1);}
}

#[test]fn bonus_title_column_consumes_a_fresh_native_node_and_reparents_only_its_title(){
 unsafe extern "system" fn spawn(out:*mut u64,commands:u64,bundle:*mut u64,meta:u64)->u64{
  assert_eq!((commands,meta),(0xe10000,0x20000));assert_eq!(*bundle,0x100000019);
  let node=std::slice::from_raw_parts(bundle.add(1).cast::<u8>(),0x180);
  assert_eq!(&node[0x88..0x98],&[0;16]);assert_eq!(node[0x176],1);assert_eq!(node[asx_game::official125::BONUS_NODE_ALIGN_ITEMS],1);assert_eq!(&node[0x140..0x148],&[1,0,0,0,0,0,0,0]);
  std::ptr::copy_nonoverlapping([0x200000019,10,20,30,40,50].as_ptr(),out,6);out as u64
 }
 let m=memory();let a=adapter();m.data.borrow_mut().insert(0xd0000,vec![0;0x188]);m.data.borrow_mut().insert(0xe0000,vec![0x55;0x100]);
 m.write(0xe0000,&0x100000019u64.to_le_bytes()).unwrap();let original=m.read(0xe0000,0x100).unwrap();
 assert!(a.bonus_column_capture(&m,0,0xd0000,0x20000).unwrap().is_none());
 let style=a.bonus_column_capture(&m,asx_game::official125::BONUS_HEADER_CALLER,0xd0000,0x20000).unwrap().unwrap();
 let base=spawn as *const()as usize-asx_game::official125::BONUS_COLUMN_SPAWN.0 as usize;
 let child=unsafe{a.bonus_column_draw(&m,base,0xe10000,0xe0000,style)}.unwrap();assert_eq!(child,[0x200000019,10,20,30,40,50]);
 let mut expected=original;expected[..8].copy_from_slice(&child[0].to_le_bytes());expected[0x50..0x54].copy_from_slice(&13f32.to_le_bytes());assert_eq!(m.read(0xe0000,0x100).unwrap(),expected);
 m.write(0xd0008,&1u64.to_le_bytes()).unwrap();assert!(a.bonus_column_capture(&m,asx_game::official125::BONUS_HEADER_CALLER,0xd0000,0x20000).is_err());
}

#[test]fn deferred_bonus_caption_reborrows_live_commands_without_retaining_a_parent_pointer(){
 let m=memory();let a=adapter();let original=[11u64,22,33,44,55,66];m.data.borrow_mut().insert(0xd0000,original.into_iter().flat_map(u64::to_le_bytes).collect());
 assert_eq!(a.bonus_column_spawner(&m,0xd0000,99).unwrap(),[99,22,33,44,55,66]);assert_eq!(m.read(0xd0000,48).unwrap(),original.into_iter().flat_map(u64::to_le_bytes).collect::<Vec<_>>());
}
