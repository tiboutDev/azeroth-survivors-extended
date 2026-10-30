# Glyph status

Glyph gameplay is **disabled for the supported native client**. Spell glyph
choices and spell mutations are not installed by normal ASX activation. Retained
bonus/card/tooltip representations also serve active jewelry features and must
not be removed merely because they were originally shared with glyphs.

ASX-15 remains a stable feature ID. Any future compatibility/design review must
account for all nine retained class catalogs: Warrior, Paladin, Hunter, Rogue,
Priest, Shaman, Mage, Warlock and Druid. Catalogs and policy are in
`crates/asx-gameplay/src/glyphs.rs`; selected native definition checks remain
in `asx-game`. Those sources are the reference for inactive variants, not a
player promise that their benefits are enabled.

Use the `glyphs` profile for retained policy/shared-representation changes.
Re-enabling glyph gameplay requires an explicit design decision and native-set
interaction review; retaining old catalog data is not such authorization.
