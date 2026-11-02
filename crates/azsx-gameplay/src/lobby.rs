//! Product identity on native host submissions; no lobby/network ownership.
pub fn name(value: &str) -> String {
    let mut value=value.trim();
    while let Some(rest)=value.strip_prefix("[AZSX]") { value=rest.trim_start(); }
    if value.is_empty() { value="Azeroth Survivors"; }
    // Native host-name input accepts 32 Unicode scalar values, including prefix.
    format!("[AZSX] {}", value.chars().take(25).collect::<String>())
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn identity_is_idempotent_and_preserves_unicode() {
        assert_eq!(name("Thibaut's run"),"[AZSX] Thibaut's run");
        assert_eq!(name("[AZSX] [AZSX] Équipe"),"[AZSX] Équipe");
        assert_eq!(name(""),"[AZSX] Azeroth Survivors");
        let value=name(&"é".repeat(40));assert_eq!(value.chars().count(),32);assert_eq!(name(&value),value);
    }
}
