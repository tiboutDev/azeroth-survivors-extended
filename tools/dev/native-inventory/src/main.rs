use syn::{spanned::Spanned,visit::Visit,Item};
fn cfg_value(meta:&syn::Meta)->Option<bool>{match meta{
 syn::Meta::Path(p) if p.is_ident("test")=>Some(false),
 syn::Meta::List(l)=>{
 let args=l.parse_args_with(syn::punctuated::Punctuated::<syn::Meta,syn::Token![,]>::parse_terminated).ok()?;
 let values:Vec<_>=args.iter().map(cfg_value).collect();
 if l.path.is_ident("all"){if values.contains(&Some(false)){Some(false)}else if values.iter().all(|x|*x==Some(true)){Some(true)}else{None}}
 else if l.path.is_ident("any"){if values.contains(&Some(true)){Some(true)}else if values.iter().all(|x|*x==Some(false)){Some(false)}else{None}}
 else if l.path.is_ident("not")&&values.len()==1{values[0].map(|v|!v)}else{None}
 },_=>None}}
fn test_only(attrs:&[syn::Attribute])->bool {attrs.iter().any(|a|a.path().is_ident("test") || (a.path().is_ident("cfg")&&a.parse_args::<syn::Meta>().ok().is_some_and(|m|cfg_value(&m)==Some(false))))}

fn source_text(source: &str, span: proc_macro2::Span) -> String {
    let lines: Vec<_> = source.split_inclusive('\n').collect();
    let start = span.start();
    let end = span.end();
    let a = lines.iter().take(start.line - 1).map(|l| l.len()).sum::<usize>() + start.column;
    let z = lines.iter().take(end.line - 1).map(|l| l.len()).sum::<usize>() + end.column;
    source[a..z].to_owned()
}
struct Accesses<'a> {
    source: &'a str,
    edges: Vec<serde_json::Value>,
    bindings: Vec<serde_json::Value>,
}
impl<'ast> Visit<'ast> for Accesses<'_> {
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let target = call.method.to_string();
        let kind = match target.as_str() {
            "game_field" | "actor_field" | "row_field" | "field117" | "field1201" => "field",
            "rva" | "contact117" | "expected" | "extension_bytes117" => "contact",
            "read" | "write" => "memory",
            _ => "method",
        };
        self.edges.push(serde_json::json!({
            "kind": kind, "target": target,
            "receiver": source_text(self.source, call.receiver.span()),
            "arguments": call.args.iter().map(|a| source_text(self.source,a.span())).collect::<Vec<_>>(),
            "line": call.span().start().line,
        }));
        syn::visit::visit_expr_method_call(self, call);
    }
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        let callee = source_text(self.source, call.func.span());
        let leaf = if let syn::Expr::Path(path) = &*call.func {
            path.path.segments.last().map(|s| s.ident.to_string()).unwrap_or_default()
        } else { String::new() };
        let field = matches!(leaf.as_str(), "game_field" | "actor_field" | "row_field" | "field117" | "field1201");
        self.edges.push(serde_json::json!({
            "kind": if field { "field" } else { "call" },
            "target": if field { leaf } else { callee.clone() }, "callee": callee,
            "arguments": call.args.iter().map(|a| source_text(self.source,a.span())).collect::<Vec<_>>(),
            "line": call.span().start().line,
        }));
        syn::visit::visit_expr_call(self,call);
    }
    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let Some(init) = &local.init {
            self.bindings.push(serde_json::json!({
                "pattern": source_text(self.source,local.pat.span()),
                "value": source_text(self.source,init.expr.span()),
                "line": local.span().start().line,
            }));
        }
        syn::visit::visit_local(self,local);
    }
    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        self.bindings.push(serde_json::json!({
            "pattern": source_text(self.source,expression.pat.span()),
            "value": source_text(self.source,expression.expr.span()),
            "line": expression.span().start().line,
            "loop": true,
        }));
        syn::visit::visit_expr_for_loop(self,expression);
    }
}
fn emit(name:String,span:proc_macro2::Span,block:&syn::Block,source:&str,out:&mut Vec<serde_json::Value>){
    let mut accesses = Accesses { source, edges: Vec::new(), bindings: Vec::new() };
    accesses.visit_block(block);
    out.push(serde_json::json!({"name":name,"body":source_text(source,span),
        "accesses":accesses.edges,"bindings":accesses.bindings}));
}
fn walk(items:&[Item],source:&str,out:&mut Vec<serde_json::Value>){for item in items {match item {
 Item::Mod(m) if !test_only(&m.attrs)=>{if let Some((_,items))=&m.content{walk(items,source,out)}},
 Item::Impl(i) if !test_only(&i.attrs)=>{for entry in &i.items{if let syn::ImplItem::Fn(f)=entry{if !test_only(&f.attrs){emit(f.sig.ident.to_string(),f.span(),&f.block,source,out)}}}},
 Item::Fn(f) if !test_only(&f.attrs)=>emit(f.sig.ident.to_string(),f.span(),&f.block,source,out),
 _=>{}
}}}
fn main(){let mut result=serde_json::Map::new();for name in std::env::args().skip(1){let source=std::fs::read_to_string(&name).expect("source");let file=syn::parse_file(&source).expect("Rust syntax");let mut out=vec![];walk(&file.items,&source,&mut out);result.insert(name,serde_json::json!(out));}println!("{}",serde_json::Value::Object(result));}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn preserves_production_after_tests_and_ignores_strings_and_test_items(){
 let source=r##"fn first(){} #[cfg(test)] mod tests{fn hidden(){}} impl Adapter{pub unsafe fn last(){let s="pub fn phantom() {}";}} #[cfg(all(test,windows))] fn also_hidden(){} #[cfg(not(test))] fn production(){}"##;
 let parsed=syn::parse_file(source).unwrap();let mut out=vec![];walk(&parsed.items,source,&mut out);
 let names:Vec<_>=out.iter().map(|r|r["name"].as_str().unwrap()).collect();assert_eq!(names,vec!["first","last","production"]);
 }
 #[test]fn includes_multiline_generic_and_private_functions(){let source="impl Adapter{fn private<T>(&self, value:T)->T{value}}";let parsed=syn::parse_file(source).unwrap();let mut out=vec![];walk(&parsed.items,source,&mut out);assert_eq!(out[0]["name"],"private");assert!(out[0]["body"].as_str().unwrap().contains("value:T"));}
 #[test]
 fn inventories_indirect_field_arguments_closures_and_raw_reads_without_string_matches() {
     let source = r#"impl Adapter {
       fn dynamic(&self,m:&Mem,key:u64) {
         let read = |offset| m.read(self.game_field(offset),4);
         read(0xbf4);
         crate::current::actor_field(key);
         for offset in [0x7b0,0xbb4] { self.row_field(true,offset); }
         m.read(row + 0x178,60);
         let text = "self.game_field(0x123); m.read(other,4)";
       }
     }"#;
     let parsed = syn::parse_file(source).unwrap(); let mut out = vec![];
     walk(&parsed.items,source,&mut out);
     let edges=out[0]["accesses"].as_array().unwrap();
     assert_eq!(edges.iter().filter(|e|e["kind"]=="field").count(),3);
     assert!(edges.iter().any(|e|e["kind"]=="field" && e["arguments"][0]=="offset"));
     assert!(edges.iter().any(|e|e["kind"]=="call" && e["target"]=="read" && e["arguments"][0]=="0xbf4"));
     assert!(edges.iter().any(|e|e["kind"]=="memory" && e["arguments"][0]=="row + 0x178"));
     assert!(edges.iter().any(|e|e["kind"]=="field" && e["callee"]=="crate::current::actor_field"));
     let bindings=out[0]["bindings"].as_array().unwrap();
     assert!(bindings.iter().any(|b|b["pattern"]=="offset" && b["loop"]==true && b["value"]=="[0x7b0,0xbb4]"));
 }

}
