//! Composition of feature descriptions for shared UI; no native widgets or hooks.
use super::*;
use asx_gameplay::kept_bonus::KeptBonus;
pub struct Block{pub bonus:KeptBonus,pub prefix:usize,pub retained:bool,pub removed:bool,pub values:Vec<(String,bool)>,pub status:Option<String>}
pub fn annotation(content:&str)->Option<Block>{
 if let Some((gem,prefix))=gems::annotation(content){return Some(Block{bonus:KeptBonus::Gem(gem),prefix,retained:content[prefix..].trim_start().starts_with("Active gem:"),removed:false,values:vec![(gem.text().into(),false)],status:content.lines().last().filter(|s|matches!(*s,"Sets your gem for this run"|"Gem retained")).map(str::to_owned)})}
 if let Some((e,prefix))=enchantments::annotation(content){return Some(Block{bonus:KeptBonus::Enchantment(e),prefix,retained:content[prefix..].trim_start().starts_with("Active enchantment:"),removed:false,values:vec![(content[prefix..].trim_start().lines().nth(1).map(str::to_owned).unwrap_or_else(||e.text()),false)],status:content.lines().last().filter(|s|matches!(*s,"Applies enchantment"|"Improves enchantment")).map(str::to_owned)})}
 if !glyphs::enabled(){return None}
 let(g,removed,prefix)=glyphs::annotation(content)?;
 Some(Block{bonus:KeptBonus::Glyph(g),prefix,retained:content[prefix..].trim_start().starts_with("Keeps glyph:"),removed,values:if removed{vec![("Restores unglyphed behavior".into(),false)]}else{g.lines().iter().enumerate().map(|(i,v)|(v.to_string(),i==1)).collect()},status:content.lines().last().filter(|s|super::bonus_ui::status_line(s)).map(str::to_owned)})
}
#[cfg(feature="test-hooks")]pub fn diagnostics()->bool{fixture::compat121()||bonus_tooltip::fixture_enabled()||glyphs::fixture_enabled()}
