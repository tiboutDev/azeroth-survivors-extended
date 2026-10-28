//! Shared reward-bonus rendering and native widget callbacks.
use super::*;
static UI:AtomicUsize=AtomicUsize::new(0);
static LINE:AtomicUsize=AtomicUsize::new(0);
thread_local!{static DRAWBACK:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};static STATUS_STYLE:std::cell::Cell<Option<bool>>=const{std::cell::Cell::new(None)};static UI_CONTEXT:std::cell::Cell<Option<asx_game::official::BonusUi>>=const{std::cell::Cell::new(None)};static HEADER:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};static TOOLTIP_TITLE:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};static COMPACT:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};static SUPPRESS:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};}
pub(super) fn status_line(text:&str)->bool{text=="Sets your gem for this run"||text=="Gem retained"||text=="Applies enchantment"||text=="Improves enchantment"||text=="Applies glyph"||text=="Keeps glyph"||text.starts_with("Replaces ")||text.starts_with("Removes ")&&!text.starts_with("Removes glyph bonuses")&&!text.starts_with("Removes bonuses")}
struct DrawbackScope(bool);
impl Drop for DrawbackScope{fn drop(&mut self){DRAWBACK.with(|v|v.set(self.0));}}
struct StatusScope(Option<bool>);
impl Drop for StatusScope{fn drop(&mut self){STATUS_STYLE.with(|v|v.set(self.0));}}
struct UiScope(Option<asx_game::official::BonusUi>,bool);
impl Drop for UiScope{fn drop(&mut self){UI_CONTEXT.with(|v|v.set(self.0));SUPPRESS.with(|v|v.set(self.1));}}
struct CompactScope(bool);
impl Drop for CompactScope{fn drop(&mut self){COMPACT.with(|v|v.set(self.0));}}
struct HeaderScope(bool);
impl Drop for HeaderScope{fn drop(&mut self){HEADER.with(|v|v.set(self.0));}}
unsafe extern "system" fn ui(parent:u64,closure:u64)->u64{
 let context=RUNTIME.get().filter(|r|r.rules).and_then(|r|r.adapter.bonus_ui_context(&r.local,closure).ok());
 #[cfg(feature="test-hooks")]
 if bonus_catalog::diagnostics(){static ONCE:AtomicBool=AtomicBool::new(false);if !ONCE.swap(true,Ordering::Relaxed){if let Some(r)=RUNTIME.get(){r.log.info(&format!("glyph fixture native UI context: {}",context.is_some()));}}}
 let _scope=UiScope(UI_CONTEXT.with(|v|v.replace(context)),SUPPRESS.with(|v|v.replace(false)));
 let original:unsafe extern "system" fn(u64,u64)->u64=std::mem::transmute(UI.load(Ordering::Acquire));original(parent,closure)
}
unsafe fn kept_row(r:&Runtime,parent:u64,font:u64,u:asx_game::official::BonusUi,bonus:asx_gameplay::kept_bonus::KeptBonus)->bool{
 let _compact=CompactScope(COMPACT.with(|v|v.replace(true)));
 let(result,attached)=bonus_tooltip::with_row(font,u,bonus,||r.adapter.kept_bonus_ui_draw(r.local.base,parent,font,u,bonus));
 if result.is_err(){return false}
 if !attached{let _normal=CompactScope(COMPACT.with(|v|v.replace(false)));draw_kept_values(parent,font,bonus);}
 let original:unsafe extern "system" fn(u64,u64,u64,u64)=std::mem::transmute(LINE.load(Ordering::Acquire));
 {let _style=StatusScope(STATUS_STYLE.with(|v|v.replace(Some(true))));original(parent,font,b"Active".as_ptr()as u64,6);}
 SUPPRESS.with(|v|v.set(true));
 #[cfg(feature="test-hooks")]if bonus_catalog::diagnostics(){r.log.info(&format!("kept bonus row: category={} name={}",bonus.category(),bonus.name()));}
 true
}
struct TooltipTitleScope(bool);
impl Drop for TooltipTitleScope{fn drop(&mut self){TOOLTIP_TITLE.with(|v|v.set(self.0));}}
unsafe fn draw_kept_values(parent:u64,font:u64,bonus:asx_gameplay::kept_bonus::KeptBonus){
 let line:unsafe extern "system" fn(u64,u64,u64,u64)=std::mem::transmute(LINE.load(Ordering::Acquire));
 let(benefit,drawback)=bonus.lines();line(parent,font,benefit.as_ptr()as u64,benefit.len()as u64);
 if !drawback.is_empty(){let _red=DrawbackScope(DRAWBACK.with(|v|v.replace(true)));line(parent,font,drawback.as_ptr()as u64,drawback.len()as u64);}
}
pub unsafe fn draw_kept_tooltip(_r:&Runtime,parent:u64,font:u64,bonus:asx_gameplay::kept_bonus::KeptBonus){
 let _normal=CompactScope(COMPACT.with(|v|v.replace(false)));
 let _header=HeaderScope(HEADER.with(|v|v.replace(true)));
 let _status=StatusScope(STATUS_STYLE.with(|v|v.replace(None)));
 let line:unsafe extern "system" fn(u64,u64,u64,u64)=std::mem::transmute(LINE.load(Ordering::Acquire));
 let title=bonus.name();{let _gold=TooltipTitleScope(TOOLTIP_TITLE.with(|v|v.replace(true)));line(parent,font,title.as_ptr()as u64,title.len()as u64);}
 draw_kept_values(parent,font,bonus);
 {let _active=StatusScope(STATUS_STYLE.with(|v|v.replace(Some(true))));line(parent,font,b"Active".as_ptr()as u64,6);}
 #[cfg(feature="test-hooks")]if bonus_catalog::diagnostics(){_r.log.info(&format!("native kept tooltip body: category={} name={}",bonus.category(),bonus.name()));}
}
unsafe extern "system" fn line(parent:u64,font:u64,text:u64,len:u64){
 let original:unsafe extern "system" fn(u64,u64,u64,u64)=std::mem::transmute(LINE.load(Ordering::Acquire));
 if HEADER.with(|v|v.get()){original(parent,font,text,len);return}
 let data=RUNTIME.get().filter(|r|r.rules).zip(UI_CONTEXT.with(|v|v.get())).and_then(|(r,u)|{
  if len>8192{return None}let bytes=asx_game::memory::Mem::read(&r.local,text,len as usize).ok()?;let content=String::from_utf8(bytes).ok()?;
  Some((r,u,content))
 });
 if let Some((r,u,content))=data{
  if SUPPRESS.with(|v|v.get()){
   if status_line(&content){
    let _style=StatusScope(STATUS_STYLE.with(|v|v.replace(Some(content.starts_with("Keeps ")||content=="Gem retained"))));
    original(parent,font,text,len);
   }
   return
  }
  if let Some(block)=bonus_catalog::annotation(&content){
   if block.prefix>0{original(parent,font,text,block.prefix as u64);}
   let _header=HeaderScope(HEADER.with(|v|v.replace(true)));
   if block.retained&&r.adapter.supports_bonus_tooltips()&&kept_row(r,parent,font,u,block.bonus){return}
   let title=if block.removed{"Unglyphed".into()}else{block.bonus.name()};
   if r.adapter.bonus_header_draw(r.local.base,parent,font,u,&title,block.bonus.icon()).is_ok(){
    for(value,drawback)in &block.values{let _style=DrawbackScope(DRAWBACK.with(|v|v.replace(*drawback)));original(parent,font,value.as_ptr()as u64,value.len()as u64);}
    if let Some(status)=block.status{let _style=StatusScope(STATUS_STYLE.with(|v|v.replace(Some(block.retained))));original(parent,font,status.as_ptr()as u64,status.len()as u64);}
    SUPPRESS.with(|v|v.set(true));return
   }
  }
 }
 original(parent,font,text,len);
}

static TEXT_NODE:AtomicUsize=AtomicUsize::new(0);
unsafe extern "system" fn text_node(out:u64,parent:u64,bundle:u64,meta:u64)->u64{
 if TOOLTIP_TITLE.with(|v|v.get()){if let Some(r)=RUNTIME.get().filter(|r|r.rules){let io=io::Access{local:&r.local,adapter:r.adapter};let _=r.adapter.kept_tooltip_title(&io,bundle);}}

 if COMPACT.with(|v|v.get())&&STATUS_STYLE.with(|v|v.get()).is_none(){if let Some(r)=RUNTIME.get().filter(|r|r.rules){let io=io::Access{local:&r.local,adapter:r.adapter};let _=r.adapter.bonus_compact_style(&io,bundle);}}
 if DRAWBACK.with(|v|v.get()){if let Some(r)=RUNTIME.get().filter(|r|r.rules){let io=io::Access{local:&r.local,adapter:r.adapter};let _=r.adapter.bonus_ui_drawback(&io,bundle);}}
 if let Some(kept)=STATUS_STYLE.with(|s|s.get()){
  if let Some(r)=RUNTIME.get().filter(|r|r.rules){let io=io::Access{local:&r.local,adapter:r.adapter};let _=r.adapter.bonus_ui_status(&io,bundle,kept);}
 }
 let original:unsafe extern "system" fn(u64,u64,u64,u64)->u64=std::mem::transmute(TEXT_NODE.load(Ordering::Acquire));original(out,parent,bundle,meta)
}
pub fn size_header(r:&Runtime,c:&crate::stub::Context){
 if HEADER.with(|v|v.get()) {if let Some(caller)=(c.return_address() as u64).checked_sub(r.local.base as u64){let io=io::Access{local:&r.local,adapter:r.adapter};let _=r.adapter.bonus_ui_size(&io,caller,c.r8);if COMPACT.with(|v|v.get()){let _=r.adapter.bonus_compact_size(&io,caller,c.r8);}}}
}
pub fn ui_hooks(adapter:Adapter)->Vec<crate::install::HookSpec>{
 let entries=adapter.bonus_ui_entries();if entries.len()!=5{return vec![]}
 vec![crate::install::HookSpec{name:"official-bonus-ui",target:local_base()+entries[0].rva as usize,detour:ui as *const () as usize,trampoline:&UI},crate::install::HookSpec{name:"official-bonus-line",target:local_base()+entries[1].rva as usize,detour:line as *const () as usize,trampoline:&LINE},crate::install::HookSpec{name:"official-bonus-decoration-input",target:local_base()+entries[3].rva as usize,detour:no_action_entry as *const () as usize,trampoline:&NO_ACTION},crate::install::HookSpec{name:"official-bonus-status-text",target:local_base()+entries[4].rva as usize,detour:text_node as *const () as usize,trampoline:&TEXT_NODE}]
}

#[cfg(test)]mod ui_tests {
 use super::*;
 static ARGS:std::sync::Mutex<Vec<u64>>=std::sync::Mutex::new(Vec::new());
 unsafe extern "system" fn native_ui(p:u64,c:u64)->u64{*ARGS.lock().unwrap()=vec![p,c];p^c}
 unsafe extern "system" fn native_line(p:u64,f:u64,t:u64,n:u64){*ARGS.lock().unwrap()=vec![p,f,t,n];}
 #[test]fn native_glyph_ui_wrappers_preserve_abi_and_scope(){UI.store(native_ui as *const () as usize,Ordering::SeqCst);LINE.store(native_line as *const () as usize,Ordering::SeqCst);let a=0x1122334455667788;let b=0x8877665544332211;unsafe{assert_eq!(ui(a,b),a^b);assert_eq!(*ARGS.lock().unwrap(),vec![a,b]);line(a,b,88,9);assert_eq!(*ARGS.lock().unwrap(),vec![a,b,88,9]);}assert!(UI_CONTEXT.with(|v|v.get()).is_none());assert!(!HEADER.with(|v|v.get()));{let _a=HeaderScope(HEADER.with(|v|v.replace(true)));assert!(HEADER.with(|v|v.get()));}assert!(!HEADER.with(|v|v.get()));}
}


static NO_ACTION:AtomicUsize=AtomicUsize::new(0);
crate::context_stub_flags!(no_action_entry,no_action,NO_ACTION);
unsafe extern "system" fn no_action(c:&mut crate::stub::Context){
 if !HEADER.with(|v|v.get()){return}
 #[cfg(test)]{c.rflags &= !0x40;}
 if let Some(r)=RUNTIME.get().filter(|r|r.rules){c.rflags=r.adapter.bonus_ui_no_action(c.rflags);}
}
#[cfg(test)]mod action_tests {
 use super::*;
 #[unsafe(naked)]unsafe extern "C" fn continuation(){core::arch::naked_asm!("ret");}
 #[unsafe(naked)]unsafe extern "system" fn probe()->u64{core::arch::naked_asm!("sub rsp,40","xor eax,eax","cmp eax,eax","stc","call {entry}","pushfq","pop rax","add rsp,40","ret",entry=sym no_action_entry);}
 #[test]fn glyph_decoration_preserves_flags_except_the_reviewed_action_branch(){NO_ACTION.store(continuation as *const () as usize,Ordering::SeqCst);assert_eq!(unsafe{probe()}&0x41,0x41);{let _scope=HeaderScope(HEADER.with(|v|v.replace(true)));assert_eq!(unsafe{probe()}&0x41,1);}assert_eq!(unsafe{probe()}&0x41,0x41);}
}

#[cfg(test)]mod status_tests{use super::*;
 unsafe extern "system" fn native_text(o:u64,p:u64,b:u64,m:u64)->u64{o^p^b^m}
 #[test]fn status_wrapper_preserves_abi_and_scoped_style(){
 TEXT_NODE.store(native_text as *const () as usize,Ordering::SeqCst);
 let a=0x1122334455667788;let b=0x8877665544332211;
 assert_eq!(unsafe{text_node(a,b,77,91)},a^b^77^91);
 {let _s=StatusScope(STATUS_STYLE.with(|v|v.replace(Some(true))));assert_eq!(unsafe{text_node(a,b,77,91)},a^b^77^91);}
 assert!(STATUS_STYLE.with(|v|v.get()).is_none());
 }
}
