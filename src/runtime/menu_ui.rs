//! Native main-menu title and shared reward-header sizing. Lobby decorations are retired.
use super::*;
static GROUP:AtomicUsize=AtomicUsize::new(0);
crate::context_stub_rw!(group_entry,0,group,GROUP);
unsafe extern "system" fn group(c:&mut crate::stub::Context){
    if let Some(r)=RUNTIME.get(){bonus_ui::size_header(r,c);}
}
const EXTENDED_TITLE: &str = "Azeroth Survivors Extended";
static TITLE: AtomicUsize = AtomicUsize::new(0);
crate::context_stub_rw!(title_entry, 0, title, TITLE);
fn set_title(c: &mut crate::stub::Context, enabled: bool) {
    if enabled {
        c.rax = EXTENDED_TITLE.as_ptr() as u64;
        c.rdx = EXTENDED_TITLE.len() as u64;
    }
}
unsafe extern "system" fn title(c: &mut crate::stub::Context) {
    set_title(c, RUNTIME.get().is_some_and(|r| r.rules));
}
pub fn hooks(adapter: Adapter, base: usize) -> Vec<crate::install::HookSpec> {
    let title = adapter.menu_title_result_entry();
    vec![
        crate::install::HookSpec {
            name: "official-menu-extended-title",
            target: base + title.rva as usize,
            detour: title_entry as *const () as usize,
            trampoline: &TITLE,
        },
        crate::install::HookSpec {
            name: "official-reward-header-size",
            target: base + adapter.menu_entries()[4].rva as usize,
            detour: group_entry as *const () as usize,
            trampoline: &GROUP,
        },
    ]
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn title_result_changes_only_when_enabled(){
        let mut context: crate::stub::Context = unsafe { std::mem::zeroed() };
        context.rax = 0x1234;
        context.rdx = 17;
        set_title(&mut context, false);
        assert_eq!((context.rax, context.rdx), (0x1234, 17));
        set_title(&mut context, true);
        assert_eq!(context.rax, EXTENDED_TITLE.as_ptr() as u64);
        assert_eq!(context.rdx, EXTENDED_TITLE.len() as u64);
    }
    #[test]fn title_and_shared_reward_header_are_installed(){
        let adapter=Adapter::select(azsx_game::current::SHA256).unwrap();let base=0x140000000;let entries=hooks(adapter,base);
        assert_eq!(entries.len(),2);
        assert_eq!(entries[0].name,"official-menu-extended-title");
        assert_eq!(entries[0].target,base+adapter.menu_title_result_entry().rva as usize);
        assert_eq!(entries[1].name,"official-reward-header-size");
        assert_eq!(entries[1].target,base+adapter.menu_entries()[4].rva as usize);
    }
}
