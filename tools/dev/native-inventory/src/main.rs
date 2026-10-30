use syn::{spanned::Spanned,Item};
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

fn emit(name:String,span:proc_macro2::Span,source:&str,out:&mut Vec<serde_json::Value>){
 let lines:Vec<_>=source.split_inclusive('\n').collect();let start=span.start();let end=span.end();
 let a=lines.iter().take(start.line-1).map(|l|l.len()).sum::<usize>()+start.column;
 let z=lines.iter().take(end.line-1).map(|l|l.len()).sum::<usize>()+end.column;
 out.push(serde_json::json!({"name":name,"body":&source[a..z]}));
}
fn walk(items:&[Item],source:&str,out:&mut Vec<serde_json::Value>){for item in items {match item {
 Item::Mod(m) if !test_only(&m.attrs)=>{if let Some((_,items))=&m.content{walk(items,source,out)}},
 Item::Impl(i) if !test_only(&i.attrs)=>{for entry in &i.items{if let syn::ImplItem::Fn(f)=entry{if !test_only(&f.attrs){emit(f.sig.ident.to_string(),f.span(),source,out)}}}},
 Item::Fn(f) if !test_only(&f.attrs)=>emit(f.sig.ident.to_string(),f.span(),source,out),
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
}
