//! Shared native reward-header sizing. Lobby decorations are retired.
use super::*;
static GROUP:AtomicUsize=AtomicUsize::new(0);
crate::context_stub_rw!(group_entry,0,group,GROUP);
unsafe extern "system" fn group(c:&mut crate::stub::Context){
    if let Some(r)=RUNTIME.get(){bonus_ui::size_header(r,c);}
}
pub fn hooks(adapter:Adapter,base:usize)->Vec<crate::install::HookSpec>{
    vec![crate::install::HookSpec{name:"official-reward-header-size",target:base+adapter.menu_entries()[4].rva as usize,detour:group_entry as *const()as usize,trampoline:&GROUP}]
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn only_shared_reward_header_remains_installed(){
        let adapter=Adapter::select(asx_game::current::SHA256).unwrap();let base=0x140000000;let entries=hooks(adapter,base);
        assert_eq!(entries.len(),1);assert_eq!(entries[0].name,"official-reward-header-size");assert_eq!(entries[0].target,base+adapter.menu_entries()[4].rva as usize);
    }
}
